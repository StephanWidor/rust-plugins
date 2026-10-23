use super::*;
use egui::ecolor;
use nice_plug::{context::gui::GuiContext, editor::*};
use std::sync;

pub const MIN_WINDOW_SIZE: dpi::LogicalSize<f32> = dpi::LogicalSize::new(450.0, 500.0);
pub const RESIZE_HINT: ResizeHint = ResizeHint::resizable().with_min_logical_size(MIN_WINDOW_SIZE);
pub const ZOOM_FACTOR: f32 = 1.0;
pub const BACKGROUND_COLOR: ecolor::Rgba =
    ecolor::Rgba::from_rgb(32.0 / 255.0, 35.0 / 255.0, 38.0 / 255.0);

pub fn create(
    params: sync::Arc<params::PluginParams>,
    repaint_notifier: nice_plug_egui::RepaintNotifier,
) -> Option<nice_plug_egui::EguiEditor<Editor>> {
    nice_plug_egui::create_egui_editor(
        nice_plug_egui::EguiEditorState::from_size(MIN_WINDOW_SIZE, ZOOM_FACTOR),
        repaint_notifier,
        nice_plug_egui::EguiNiceSettings::new().with_resize_hint(RESIZE_HINT),
        Editor::new(params),
    )
}

pub struct Editor {
    nice_gui_context: Option<nice_plug::context::gui::GuiContext>,
    params: sync::Arc<params::PluginParams>,
    control_plot: control_plot::ControlPlot,
}

impl Editor {
    pub fn new(params: sync::Arc<params::PluginParams>) -> Self {
        Self {
            nice_gui_context: None,
            params: params.clone(),
            control_plot: control_plot::ControlPlot::new(control_plot::make_vowels_map(
                &params.frequency_transform,
            )),
        }
    }
}

impl nice_plug_egui::NiceEguiApp for Editor {
    fn build(
        &mut self,
        _egui_context: egui::Context,
        nice_gui_context: GuiContext,
        frame: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        self.nice_gui_context = Some(nice_gui_context);
        frame.set_clear_color(BACKGROUND_COLOR);
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.nice_gui_context = None;
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut nice_plug_egui::Frame) {
        let Some(nice_gui_context) = self.nice_gui_context.as_mut() else {
            return;
        };
        ui.visuals_mut().handle_shape = egui::style::HandleShape::Circle;
        let param_setter = &nice_gui_context.param_setter();
        egui::Frame::new()
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    egui::Frame::group(ui.style())
                        .corner_radius(5)
                        .show(ui, |ui| {
                            egui::Grid::new("path_control_grid").show(ui, |ui| {
                                let path_enabled = nice_utils::egui::param_checkbox(
                                    ui,
                                    &self.params.path.enabled,
                                    param_setter,
                                    "Path mode",
                                );
                                if path_enabled {
                                    nice_utils::egui::param_slider(
                                        ui,
                                        &self.params.path.t,
                                        param_setter,
                                        Some(self.params.path.current_t()),
                                        "",
                                        "",
                                    );
                                    ui.end_row();

                                    ui.add(egui::Label::new(format!("Modulation")));

                                    ui.vertical(|ui| {
                                        nice_utils::egui::param_slider(
                                            ui,
                                            &self.params.path.t_modulation.wave_length,
                                            param_setter,
                                            None,
                                            "Length: ",
                                            "s",
                                        );
                                        nice_utils::egui::param_slider(
                                            ui,
                                            &self.params.path.t_modulation.amplitude,
                                            param_setter,
                                            None,
                                            " Amplitude: ",
                                            "",
                                        );
                                    });
                                }
                                ui.end_row();
                            });
                        });

                    let plot_size = ui.available_width().min(0.9 * ui.available_height());
                    self.control_plot
                        .draw(ui, plot_size, &self.params, &param_setter);

                    ui.horizontal(|ui| {
                        egui::Frame::group(ui.style())
                            .corner_radius(5)
                            .show(ui, |ui| {
                                ui.add(egui::Label::new(format!("Dry mix")));
                                nice_utils::egui::param_slider(
                                    ui,
                                    &self.params.dry_gain_db,
                                    param_setter,
                                    None,
                                    "",
                                    "dB",
                                );
                            });
                        egui::Frame::group(ui.style())
                            .corner_radius(5)
                            .show(ui, |ui| {
                                nice_utils::egui::param_checkbox(
                                    ui,
                                    &self.params.limiter_enabled,
                                    param_setter,
                                    "Limiter",
                                );
                            });
                    });
                });
            });
    }
}
