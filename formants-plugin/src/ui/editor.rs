use super::*;
use egui::ecolor;
use nice_plug::{context::gui::GuiContext, editor::*};
use std::sync;

pub const INITIAL_WINDOW_SIZE: dpi::LogicalSize<f32> = dpi::LogicalSize::new(500.0, 600.0);
pub const MIN_WINDOW_SIZE: dpi::LogicalSize<f32> = dpi::LogicalSize::new(300.0, 300.0);
pub const RESIZE_HINT: ResizeHint = ResizeHint::resizable().with_min_logical_size(MIN_WINDOW_SIZE);
pub const ZOOM_FACTOR: f32 = 1.0;
pub const BACKGROUND_COLOR: ecolor::Rgba =
    ecolor::Rgba::from_rgb(32.0 / 255.0, 35.0 / 255.0, 38.0 / 255.0);

pub fn create(
    params: sync::Arc<params::PluginParams>,
    repaint_notifier: nice_plug_egui::RepaintNotifier,
) -> Option<nice_plug_egui::EguiEditor<Editor>> {
    nice_plug_egui::create_egui_editor(
        nice_plug_egui::EguiEditorState::from_size(INITIAL_WINDOW_SIZE, ZOOM_FACTOR),
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
        //ui.ctx().set_debug_on_hover(true);
        ui.visuals_mut().handle_shape = egui::style::HandleShape::Circle;
        let param_setter = &nice_gui_context.param_setter();
        let corner_radius = 5;
        let margin_width = 10;
        egui::Frame::new()
            .inner_margin(0)
            .outer_margin(0)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    add_path_ui(
                        ui,
                        corner_radius,
                        margin_width,
                        self.params.as_ref(),
                        param_setter,
                    );

                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let plot_height = ui.available_height()
                            - estimated_dry_gain_and_limiter_ui_height(ui, margin_width)
                            - ui.spacing().item_spacing.y;
                        let plot_side_length = ui.available_width().min(plot_height);
                        self.control_plot.draw(
                            ui,
                            (plot_side_length, plot_side_length),
                            margin_width,
                            corner_radius,
                            &self.params,
                            &param_setter,
                        );
                    });

                    add_dry_gain_and_limiter_ui(
                        ui,
                        corner_radius,
                        margin_width,
                        self.params.as_ref(),
                        param_setter,
                    );
                });
            });
    }
}

fn add_path_ui(
    ui: &mut egui::Ui,
    corner_radius: u8,
    margin_width: i8,
    params: &params::PluginParams,
    param_setter: &nice::ParamSetter<'_>,
) {
    egui::Frame::group(ui.style())
        .corner_radius(corner_radius)
        .inner_margin(margin_width)
        .outer_margin(margin_width)
        .show(ui, |ui| {
            let available_size = ui.available_size();
            ui.set_min_width(available_size.x);
            ui.horizontal(|ui| {
                let mut path_enabled = false;

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        path_enabled = nice_utils::egui::param_checkbox(
                            ui,
                            &params.path.enabled,
                            param_setter,
                            "",
                        )
                        .inner;
                        ui.add(egui::Label::new(format!("Path")));
                    });

                    if path_enabled {
                        nice_utils::egui::param_slider(
                            ui,
                            &params.path.t,
                            param_setter,
                            Some(params.path.current_t()),
                            "",
                            "",
                        );
                    }
                });
                if path_enabled {
                    ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                        let t_modulation = &params.path.t_modulation;
                        ui.add(egui::Label::new(format!("Modulation")));
                        ui.horizontal(|ui| {
                            nice_utils::egui::param_slider(
                                ui,
                                &t_modulation.amplitude,
                                param_setter,
                                None,
                                "",
                                "",
                            );
                            ui.add(egui::Label::new(format!("Amplitude ")));
                        });
                        ui.horizontal(|ui| {
                            match t_modulation.length_unit.value() {
                                params::modulation::LengthUnit::Note => {
                                    nice_utils::egui::param_ratio_control(
                                        ui,
                                        &t_modulation.length_in_notes.numerator,
                                        &t_modulation.length_in_notes.denominator,
                                        param_setter,
                                        0.1,
                                    );
                                    ui.add_space(47_f32);
                                }
                                params::modulation::LengthUnit::Seconds => {
                                    nice_utils::egui::param_slider(
                                        ui,
                                        &t_modulation.length_in_seconds,
                                        param_setter,
                                        None,
                                        "",
                                        "s",
                                    );
                                }
                            }
                            nice_utils::egui::param_combo_box(
                                ui,
                                50_f32,
                                &t_modulation.length_unit,
                                param_setter,
                            );
                        });
                    });
                }
            });
        });
}

fn estimated_dry_gain_and_limiter_ui_height(ui: &egui::Ui, margin_width: i8) -> f32 {
    2_f32 * egui_utils::custom::Slider::thickness(ui)
        + 4_f32 * margin_width as f32
        + 2_f32
        + ui.spacing().item_spacing.y
}

fn add_dry_gain_and_limiter_ui(
    ui: &mut egui::Ui,
    corner_radius: u8,
    margin_width: i8,
    params: &params::PluginParams,
    param_setter: &nice::ParamSetter<'_>,
) {
    egui::Frame::group(ui.style())
        .corner_radius(corner_radius)
        .inner_margin(margin_width)
        .outer_margin(margin_width)
        .show(ui, |ui| {
            let available_size = ui.available_size();
            ui.set_min_width(available_size.x);
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add(egui::Label::new(format!("Dry mix")));
                    nice_utils::egui::param_slider(
                        ui,
                        &params.dry_gain_db,
                        param_setter,
                        None,
                        "",
                        "dB",
                    );
                });
                ui.with_layout(egui::Layout::top_down(egui::Align::RIGHT), |ui| {
                    ui.horizontal(|ui| {
                        nice_utils::egui::param_checkbox(
                            ui,
                            &params.limiter_enabled,
                            param_setter,
                            "",
                        );
                        ui.add(egui::Label::new(format!("Limiter")));
                    });
                });
            });
        });
}
