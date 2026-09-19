use crate::ui::settings::*;

use super::*;

pub mod eqs;

pub fn add<F: audio_utils::Float + egui::emath::Numeric, const NUM_BANDS: usize>(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    params: &mut State<F, NUM_BANDS>,
    presets: &mut presets::Presets<F, NUM_BANDS>,
    settings: &Settings<F>,
    spectrum_available: bool,
) {
    let control_outer_margin = size.x / 25_f32;
    let control_width = size.x - 2_f32 * control_outer_margin;
    egui::Frame::group(ui.style()).show(ui, |ui| {
        egui::ScrollArea::vertical()
            .min_scrolled_height(size.y)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    add_show_options_controls(ui, &mut params.show_options, spectrum_available);
                    eqs::add_preset_controls(
                        ui,
                        control_width,
                        control_outer_margin,
                        params,
                        presets,
                    );
                    let mut eq_has_changed = false;
                    eq_has_changed |= eqs::add_multiband_type_control(
                        ui,
                        control_outer_margin,
                        &mut params.multiband_eq.processing_type,
                    );
                    let eq_colors = &settings.color_palette.eq_stroke;
                    let eq_ranges = &settings.eq_ranges;
                    for (index, eq) in params.multiband_eq.eqs.iter_mut().enumerate() {
                        eq_has_changed |= eqs::add_slider_controls(
                            ui,
                            control_width,
                            control_outer_margin,
                            eq_colors[index % eq_colors.len()],
                            eq_ranges,
                            eq,
                        );
                    }
                    if eq_has_changed {
                        params.preset_selection.mark_as_changed();
                    }
                });
            });
    });
}

fn add_show_options_controls(
    ui: &mut egui::Ui,
    show_options: &mut ShowOptions,
    spectrum_available: bool,
) {
    egui::CollapsingHeader::new("Show").show(ui, |ui| {
        if show_options.gain {
            ui.horizontal(|ui| {
                ui.checkbox(&mut show_options.gain, "Gain");
                if spectrum_available {
                    ui.checkbox(&mut show_options.signal_gain_spectrum, "Analyze Signal");
                }
            });
        } else {
            ui.checkbox(&mut show_options.gain, "Gain");
        }
        ui.checkbox(&mut show_options.phase, "Phase");
        ui.checkbox(&mut show_options.impulse_response, "Impulse Response");
        ui.checkbox(&mut show_options.poles_and_zeros, "Poles And Zeros");
    });
}
