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
        egui::Frame::new()
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        let mut path_enabled = self.params.path.enabled.value();
                        if ui.checkbox(&mut path_enabled, "Path Mode").clicked() {
                            self.params
                                .path
                                .set_enabled(path_enabled, &nice_gui_context.param_setter());
                        }
                        if path_enabled {
                            let mut path_t = self.params.path.t.value();
                            let t_response = ui.add(egui::Slider::new(
                                &mut path_t,
                                range::to_range_inclusive(&self.params.path.t.range()),
                            ));
                            if t_response.changed() {
                                self.params
                                    .path
                                    .set_t(path_t, &nice_gui_context.param_setter());
                            }
                        }
                    });

                    let plot_size = ui.available_width().min(0.9 * ui.available_height());
                    self.control_plot.add(
                        ui,
                        plot_size,
                        &self.params,
                        &nice_gui_context.param_setter(),
                    );

                    ui.horizontal(|ui| {
                        ui.add(egui::Label::new(format!("Dry Mix")));
                        let mut dry_gain = self.params.dry_gain_db.value();
                        let gain_response = ui.add(
                            egui::Slider::new(
                                &mut dry_gain,
                                range::to_range_inclusive(&self.params.dry_gain_db.range()),
                            )
                            .suffix("dB"),
                        );
                        if gain_response.changed() {
                            self.params
                                .set_dry_gain_db(dry_gain, &nice_gui_context.param_setter());
                        }
                        let mut limiter_enabled = self.params.limiter_enabled.value();
                        if ui.checkbox(&mut limiter_enabled, "Limiter").clicked() {
                            self.params.set_limiter_enabled(
                                limiter_enabled,
                                &nice_gui_context.param_setter(),
                            );
                        }
                    })
                });
            });
        //});
    }
}
