use super::*;
use audio_lib::eq;

pub fn add_multiband_type_control(
    ui: &mut egui::Ui,
    outer_margin: f32,
    multiband_type: &mut eq::MultibandType,
) -> bool {
    let mut type_has_changed = false;
    egui::Frame::group(ui.style())
        .corner_radius(5)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                type_has_changed |= ui
                    .selectable_value(multiband_type, eq::MultibandType::Sequential, "Sequential")
                    .changed();
                type_has_changed |= ui
                    .selectable_value(multiband_type, eq::MultibandType::ParallelSum, "Parallel")
                    .changed();
            });
        });
    type_has_changed
}

/// Return true if the eq has changed
pub fn add_slider_controls<F: audio_utils::Float + egui::emath::Numeric>(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    color: egui::Color32,
    eq_ranges: &crate::ui::settings::EqRanges<F>,
    eq: &mut eq::Eq<F>,
) -> bool {
    let mut gain_db = eq.gain.db();
    let mut log_frequency = eq.frequency.log_hz();
    let mut makeup_gain_db = eq.makeup_gain.db();
    let mut eq_has_changed = false;
    egui::Frame::group(ui.style())
        .fill(color)
        .multiply_with_opacity(0.2_f32)
        .corner_radius(5)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                egui::ComboBox::from_id_salt(ui.next_auto_id())
                    .selected_text(eq.eq_type.to_string())
                    .width(width)
                    .show_ui(ui, |ui| {
                        for eq_type in eq::EqType::ALL.iter() {
                            let response =
                                ui.selectable_value(&mut eq.eq_type, *eq_type, eq_type.to_string());
                            eq_has_changed |= response.changed();
                        }
                    });

                if eq.eq_type.has_frequency() {
                    let response = ui.add(
                        egui::Slider::new(
                            &mut log_frequency,
                            eq_ranges.log_frequency_range.clone(),
                        )
                        .custom_formatter(|log_frequency, _| {
                            utils::log_frequency_to_string(log_frequency)
                        })
                        .custom_parser(utils::string_to_log_frequency)
                        .prefix("frequency: ")
                        .suffix("Hz"),
                    );
                    if response.changed() {
                        eq.frequency = eq::Frequency::LogHz(log_frequency);
                        eq_has_changed = true;
                    }
                }

                if eq.eq_type.has_gain_db() {
                    let response = ui.add(
                        egui::Slider::new(&mut gain_db, eq_ranges.db_range.clone())
                            .prefix("gain: ")
                            .suffix("dB"),
                    );
                    if response.changed() {
                        eq.gain = eq::Gain::Db(gain_db);
                        eq_has_changed = true;
                    }
                }

                if eq.eq_type.has_q() {
                    let response = ui
                        .add(egui::Slider::new(&mut eq.q, eq_ranges.q_range.clone()).prefix("Q: "));
                    if response.changed() {
                        eq_has_changed = true;
                    }
                }

                if eq.eq_type != eq::EqType::Bypassed {
                    let response = ui.add(
                        egui::Slider::new(&mut makeup_gain_db, eq_ranges.db_range.clone())
                            .prefix("makeup: ")
                            .suffix("dB"),
                    );
                    if response.changed() {
                        eq.makeup_gain = eq::Gain::Db(makeup_gain_db);
                        eq_has_changed = true;
                    }
                }
            });
        });
    eq_has_changed
}

pub fn add_preset_controls<F: audio_utils::Float, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    width: f32,
    outer_margin: f32,
    params: &mut State<F, NUM_BANDS>,
    presets: &mut presets::Presets<F, NUM_BANDS>,
) {
    egui::Frame::group(ui.style())
        .corner_radius(5)
        .outer_margin(outer_margin)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                let no_preset_text = || String::from("No preset chosen");
                let mut selected_preset_name = match params.preset_selection.clone() {
                    presets::Selection::None => no_preset_text(),
                    presets::Selection::Selected(name, _) => {
                        if presets.contains(&name) {
                            name.clone()
                        } else {
                            params.preset_selection = presets::Selection::None;
                            no_preset_text()
                        }
                    }
                    presets::Selection::SelectedAndChanged(name, _) => {
                        if presets.contains(&name) {
                            name.clone() + "*"
                        } else {
                            params.preset_selection = presets::Selection::None;
                            no_preset_text()
                        }
                    }
                };

                let mut selection_changed = false;
                egui::ComboBox::from_id_salt("preset_selector")
                    .selected_text(selected_preset_name.clone())
                    .width(width)
                    .show_ui(ui, |ui| {
                        for preset_name in presets.names_iter() {
                            let response = ui.selectable_value(
                                &mut selected_preset_name,
                                preset_name.clone(),
                                preset_name.as_str(),
                            );
                            selection_changed |= response.changed();
                        }
                    });
                if selection_changed {
                    if let Some(preset) = presets.get(&selected_preset_name) {
                        params.multiband_eq = preset.clone();
                        params.preset_selection =
                            presets::Selection::Selected(selected_preset_name, preset.clone());
                    }
                }

                match params.preset_selection.clone() {
                    presets::Selection::None => {
                        if params.new_preset_name.is_some() {
                            add_new_preset_popup(ui, params, presets);
                        } else if ui.button("Add").clicked() {
                            params.new_preset_name = Some(String::from(""));
                        }
                    }
                    presets::Selection::Selected(name, _) => {
                        if ui.button("Delete").clicked() {
                            presets.remove(&name);
                            params.preset_selection = presets::Selection::None;
                        }
                    }
                    presets::Selection::SelectedAndChanged(name, _) => {
                        ui.horizontal(|ui| {
                            if params.new_preset_name.is_some() {
                                add_new_preset_popup(ui, params, presets);
                            } else {
                                if ui.button("Save").clicked() {
                                    presets.force_add(name.clone(), params.multiband_eq.clone());
                                    params.preset_selection = presets::Selection::Selected(
                                        name.clone(),
                                        params.multiband_eq.clone(),
                                    );
                                } else if ui.button("Add").clicked() {
                                    params.new_preset_name = Some(String::from(""));
                                }
                            }
                        });
                    }
                };
            });
        });
}

fn add_new_preset_popup<F: audio_utils::Float, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    params: &mut State<F, NUM_BANDS>,
    presets: &mut presets::Presets<F, NUM_BANDS>,
) {
    if let Some(new_preset_name) = params.new_preset_name.as_mut() {
        let mut should_close = false;
        egui::containers::modal::Modal::new(egui::Id::new("add_new_preset_modal")).show(
            ui.ctx(),
            |ui| {
                ui.vertical_centered(|ui| {
                    let name_is_valid =
                        !new_preset_name.is_empty() && !presets.contains(new_preset_name);
                    ui.label("New preset name:");
                    ui.add_space(10.0);
                    let text_edit_response = ui.text_edit_singleline(new_preset_name);
                    ui.ctx()
                        .memory_mut(|mem| mem.request_focus(text_edit_response.id));
                    ui.add_space(10.0);
                    ui.horizontal_centered(|ui| {
                        if ui
                            .add_enabled(name_is_valid, egui::Button::new("Ok"))
                            .clicked()
                        {
                            if presets.add(new_preset_name.clone(), params.multiband_eq.clone()) {
                                params.preset_selection = presets::Selection::Selected(
                                    new_preset_name.clone(),
                                    params.multiband_eq.clone(),
                                );
                                should_close = true;
                            }
                        }
                        if ui.button("Cancel").clicked() {
                            should_close = true;
                        }
                    });
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) && name_is_valid {
                        if presets.add(new_preset_name.clone(), params.multiband_eq.clone()) {
                            params.preset_selection = presets::Selection::Selected(
                                new_preset_name.clone(),
                                params.multiband_eq.clone(),
                            );
                        }
                        should_close = true;
                    } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        should_close = true;
                    }
                });
            },
        );
        if should_close {
            params.new_preset_name = None;
        }
    }
}
