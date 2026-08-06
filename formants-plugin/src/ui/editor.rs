use super::*;
use std::sync;

pub fn create(params: sync::Arc<params::PluginParams>) -> Option<Box<dyn nice::Editor>> {
    let editor_state = params.editor_state.clone();
    let min_size = {
        let state_size = params.editor_state.size();
        egui::Vec2::new(state_size.0 as f32, state_size.1 as f32)
    };

    let ui_state = state::State::default_from_frequency_transform(&params.frequency_transform);

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

                                let plot_size =
                                    ui.available_width().min(0.9 * ui.available_height());
                                control_plot::add(ui, plot_size, &params, setter, ui_state);

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
