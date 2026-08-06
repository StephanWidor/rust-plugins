use crate::*;
use audio_lib::biquad;
use audio_lib::utils;
use std::sync::{self, atomic};

struct UiState {
    vowels_map: Vec<(String, Option<[f32; 2]>)>,
    hovered_control_point_index: usize,
}

impl UiState {
    fn default_from_frequency_transform(t: &params::FrequencyTransform) -> Self {
        Self {
            vowels_map: vec![
                (String::from("U"), t.xy_from_frequencies([320_f32, 800_f32])),
                (
                    String::from("O"),
                    t.xy_from_frequencies([500_f32, 1000_f32]),
                ),
                (
                    String::from("A"),
                    t.xy_from_frequencies([1000_f32, 1400_f32]),
                ),
                (
                    String::from("Ö"),
                    t.xy_from_frequencies([500_f32, 1500_f32]),
                ),
                (
                    String::from("Ü"),
                    t.xy_from_frequencies([320_f32, 1650_f32]),
                ),
                (
                    String::from("Ä"),
                    t.xy_from_frequencies([800_f32, 2000_f32]),
                ),
                (
                    String::from("E"),
                    t.xy_from_frequencies([470_f32, 2300_f32]),
                ),
                (
                    String::from("I"),
                    t.xy_from_frequencies([260_f32, 3200_f32]),
                ),
            ],
            hovered_control_point_index: usize::MAX,
        }
    }
}

pub fn create_editor(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nice::Editor>> {
    let editor_state = params.editor_state.clone();
    let min_size = {
        let state_size = params.editor_state.size();
        egui::Vec2::new(state_size.0 as f32, state_size.1 as f32)
    };

    let ui_state = UiState::default_from_frequency_transform(&params.frequency_transform);

    nice_plug_egui::create_egui_editor(
        params.editor_state.clone(),
        ui_state,
        nice_plug_egui::EguiSettings::default(),
        |egui_ctx, _, _| {
            egui_ctx.set_theme(egui::Theme::Dark);
        },
        move |ui, setter, _, ui_state| {
            if !editor_state.is_open() {
                return;
            }
            nice_plug_egui::resizable_window::ResizableWindow::new("plugin-window")
                .min_size(min_size)
                .show(ui, editor_state.as_ref(), |ui| {
                    // ResizableWindow already has a CentralPanel, so this is a bit weird. But I couldn't find out a better way
                    // to set the global background color.
                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::default()
                                .inner_margin(20)
                                .fill(egui::Color32::from_rgb(32, 35, 38)),
                        )
                        .show(ui, |ui| {
                            let plot_size = ui.available_width().min(0.9 * ui.available_height());
                            let [x, y] = params.get_xy::<f64>();
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let mut path_enabled = params.path.enabled.value();
                                    if ui.checkbox(&mut path_enabled, "Path Mode").clicked() {
                                        params.path.set_enabled(path_enabled, setter);
                                    }
                                    if path_enabled {
                                        let mut path_t = params.path.t.value();
                                        let t_response = ui.add(egui::Slider::new(
                                            &mut path_t,
                                            range::to_range_inclusive(&params.path.t.range()),
                                        ));
                                        if t_response.changed() {
                                            params.path.set_t(path_t, setter);
                                        }
                                    }
                                });
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
                                            .label_formatter(
                                                |hover_position| match hover_position {
                                                    egui_plot::HoverPosition::NearDataPoint {
                                                        plot_name: _,
                                                        position: point,
                                                        index: _,
                                                    } => {
                                                        let frequencies = params
                                                            .frequency_transform
                                                            .coordinates_to_frequencies(
                                                                point.x as f32,
                                                                point.y as f32,
                                                            );
                                                        Some(format!(
                                                            "f0: {:.0}\nf1: {:.0}",
                                                            frequencies[0], frequencies[1]
                                                        ))
                                                    }
                                                    _ => None,
                                                },
                                            );

                                        let control_point_ids: [egui::Id;
                                            params::path::NUM_CONTROL_POINTS] =
                                            std::array::from_fn(|i| {
                                                ui.make_persistent_id(format!(
                                                    "control_point_{}",
                                                    i
                                                ))
                                            });
                                        let item_id_to_control_point_index =
                                            |id: &Option<egui::Id>| match id {
                                                None => None,
                                                Some(id) => {
                                                    control_point_ids.iter().position(|point_id| {
                                                        point_id.value() == id.value()
                                                    })
                                                }
                                            };
                                        let plot_response = plot.show(ui, |plot_ui| {
                                            plot_ui.set_plot_bounds(
                                                egui_plot::PlotBounds::from_min_max(
                                                    [-0.1, -0.1],
                                                    [1.1, 1.1],
                                                ),
                                            );

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
                                                params.sample_rate.load(atomic::Ordering::Relaxed),
                                            );
                                            plot_ui.line(
                                                egui_plot::Line::new("gain response", gain_points)
                                                    .color(egui::Color32::from_rgba_unmultiplied(
                                                        255, 165, 0, 32,
                                                    )),
                                            );

                                            if params.path.enabled.value() {
                                                let path_points = make_path_points(&params.path);
                                                plot_ui.line(
                                                    egui_plot::Line::new("path", path_points)
                                                        .color(
                                                            egui::Color32::from_rgba_unmultiplied(
                                                                255, 255, 0, 32,
                                                            ),
                                                        ),
                                                );
                                            }

                                            plot_ui.points(
                                                egui_plot::Points::new(
                                                    "F",
                                                    egui_plot::PlotPoints::from([x, y]),
                                                )
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
                                                            egui_plot::PlotPoints::from(
                                                                params.path.control_points[i].get(),
                                                            ),
                                                        )
                                                        .id(control_point_ids[i])
                                                        .shape(egui_plot::MarkerShape::Circle)
                                                        .color(
                                                            egui::Color32::from_rgba_unmultiplied(
                                                                255, 255, 0, 32,
                                                            ),
                                                        )
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
                                                    item_id_to_control_point_index(
                                                        &plot_response.hovered_plot_item,
                                                    )
                                                {
                                                    ui_state.hovered_control_point_index =
                                                        hovered_control_point_index;
                                                }
                                            }
                                            if params.path.enabled.value() {
                                                if ui_state.hovered_control_point_index < usize::MAX
                                                    && let Some(drag_position) = plot_response.inner
                                                {
                                                    params.path.control_points
                                                        [ui_state.hovered_control_point_index]
                                                        .set(
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
                                ui.horizontal(|ui| {
                                    ui.add(egui::Label::new(format!("Dry Mix")));
                                    let mut dry_gain = params.dry_gain_db.value();
                                    let gain_response = ui.add(
                                        egui::Slider::new(
                                            &mut dry_gain,
                                            range::to_range_inclusive(&params.dry_gain_db.range()),
                                        )
                                        .suffix("dB"),
                                    );
                                    if gain_response.changed() {
                                        params.set_dry_gain_db(dry_gain, setter);
                                    }
                                    let mut limiter_enabled = params.limiter_enabled.value();
                                    if ui.checkbox(&mut limiter_enabled, "Limiter").clicked() {
                                        params.set_limiter_enabled(limiter_enabled, setter);
                                    }
                                })
                            });
                        });
                });
        },
    )
}

pub fn make_gain_response_points<'a>(
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

pub fn make_path_points<'a>(path: &params::Path) -> egui_plot::PlotPoints<'a> {
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
