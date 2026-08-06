use super::*;
use audio_lib::biquad;
use audio_lib::utils;

pub fn add(
    ui: &mut egui::Ui,
    plot_size: f32,
    params: &params::PluginParams,
    setter: &nice::ParamSetter,
    ui_state: &mut state::State,
) {
    egui::Frame::group(ui.style())
        .corner_radius(5)
        .show(ui, |ui| {
            let plot = egui_plot::Plot::new("Formant Space")
                .allow_zoom(false)
                .allow_drag(false)
                .allow_scroll(false)
                .width(plot_size)
                .height(plot_size)
                .show_background(false)
                .show_axes(false)
                .show_grid(false)
                .label_formatter(|hover_position| match hover_position {
                    egui_plot::HoverPosition::NearDataPoint {
                        plot_name: _,
                        position: point,
                        index: _,
                    } => {
                        let frequencies = params
                            .frequency_transform
                            .coordinates_to_frequencies(point.x as f32, point.y as f32);
                        Some(format!(
                            "f0: {:.0}\nf1: {:.0}",
                            frequencies[0], frequencies[1]
                        ))
                    }
                    _ => None,
                });

            let control_point_ids: [egui::Id; params::path::NUM_CONTROL_POINTS] =
                std::array::from_fn(|i| ui.make_persistent_id(format!("control_point_{}", i)));
            let item_id_to_control_point_index = |id: &Option<egui::Id>| match id {
                None => None,
                Some(id) => control_point_ids
                    .iter()
                    .position(|point_id| point_id.value() == id.value()),
            };
            let plot_response = plot.show(ui, |plot_ui| {
                plot_ui.set_plot_bounds(egui_plot::PlotBounds::from_min_max(
                    [-0.1, -0.1],
                    [1.1, 1.1],
                ));

                for (vowel, xy) in ui_state.vowels_map.as_slice() {
                    if let Some([x, y]) = xy {
                        plot_ui.text(egui_plot::Text::new(
                            vowel,
                            egui_plot::PlotPoint::new(*x, *y),
                            egui::RichText::new(vowel).size(16.0),
                        ));
                    }
                }

                let gain_points = make_gain_response_points(
                    params.get_multiband_coefficients(),
                    params
                        .sample_rate
                        .load(std::sync::atomic::Ordering::Relaxed),
                );
                plot_ui.line(
                    egui_plot::Line::new("gain response", gain_points)
                        .color(egui::Color32::from_rgba_unmultiplied(255, 165, 0, 32)),
                );

                if params.path.enabled.value() {
                    let path_points = make_path_points(&params.path);
                    plot_ui.line(
                        egui_plot::Line::new("path", path_points)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 0, 32)),
                    );
                }

                let [x, y] = params.get_xy::<f64>();
                plot_ui.points(
                    egui_plot::Points::new("F", egui_plot::PlotPoints::from([x, y]))
                        .shape(egui_plot::MarkerShape::Circle)
                        .color(egui::Color32::ORANGE)
                        .filled(true)
                        .radius(5.0)
                        .allow_hover(!params.path.enabled.value()),
                );

                if params.path.enabled.value() {
                    for i in 0..params::path::NUM_CONTROL_POINTS {
                        plot_ui.points(
                            egui_plot::Points::new(
                                "Path Start",
                                egui_plot::PlotPoints::from(params.path.control_points[i].get()),
                            )
                            .id(control_point_ids[i])
                            .shape(egui_plot::MarkerShape::Circle)
                            .color(egui::Color32::from_rgba_unmultiplied(255, 255, 0, 32))
                            .filled(true)
                            .radius(4.0),
                        );
                    }
                }

                plot_ui.pointer_coordinate()
            });
            if plot_response.response.is_pointer_button_down_on() {
                if ui_state.hovered_control_point_index == usize::MAX {
                    if let Some(hovered_control_point_index) =
                        item_id_to_control_point_index(&plot_response.hovered_plot_item)
                    {
                        ui_state.hovered_control_point_index = hovered_control_point_index;
                    }
                }
                if params.path.enabled.value() {
                    if ui_state.hovered_control_point_index < usize::MAX
                        && let Some(drag_position) = plot_response.inner
                    {
                        params.path.control_points[ui_state.hovered_control_point_index].set(
                            drag_position.x as f32,
                            drag_position.y as f32,
                            setter,
                        );
                    }
                } else {
                    if let Some(drag_position) = plot_response.inner {
                        params.set_x(drag_position.x as f32, setter);
                        params.set_y(drag_position.y as f32, setter);
                    }
                }
            } else {
                ui_state.hovered_control_point_index = usize::MAX
            }
        });
}

fn make_gain_response_points<'a>(
    coefficients: [biquad::coefficients::Coefficients<f32>; 3],
    sample_rate: f32,
) -> egui_plot::PlotPoints<'a> {
    let gain_response =
        utils::make_gain_db_response(biquad::multiband::parallel_sum::make_frequency_response(
            coefficients.into_iter(),
            sample_rate,
        ));
    let log_frequency_range = utils::frequency_to_log(20.0)..=utils::frequency_to_log(20000.0);
    let log_frequency_length = range::get_length(&log_frequency_range);
    let x_to_log_frequency = move |x| log_frequency_range.start() + x as f32 * log_frequency_length;
    let x_to_frequency = move |x| utils::log_to_frequency(x_to_log_frequency(x));
    let gain_db_range = -60_f32..=60_f32;
    let gain_db_length = range::get_length(&gain_db_range);
    let gain_to_normalized = move |gain_db: f32| {
        let gain_clamped = gain_db.clamp(*gain_db_range.start(), *gain_db_range.end());
        ((gain_clamped - gain_db_range.start()) / gain_db_length) as f64
    };
    egui_plot::PlotPoints::from_explicit_callback(
        move |x| gain_to_normalized(gain_response(x_to_frequency(x))),
        0.0..=1.0,
        1000,
    )
}

fn make_path_points<'a>(path: &params::Path) -> egui_plot::PlotPoints<'a> {
    let c = path.coefficients_matrix();
    egui_plot::PlotPoints::from_parametric_callback(
        move |t| {
            let [x, y] = params::Path::point_from_coefficients_and_t(&c, t as f32);
            (x, y)
        },
        0.0..=1.0,
        100,
    )
}
