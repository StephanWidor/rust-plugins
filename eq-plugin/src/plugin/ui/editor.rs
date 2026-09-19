use crate::plugin::*;
use nice_plug::{context::gui::GuiContext, editor::*};
use std::sync::{self, atomic};

pub const MIN_WINDOW_SIZE: dpi::LogicalSize<f32> = dpi::LogicalSize::new(1000.0, 700.0);
pub const RESIZE_HINT: ResizeHint = ResizeHint::resizable().with_min_logical_size(MIN_WINDOW_SIZE);
pub const ZOOM_FACTOR: f32 = 1.0;

pub fn create<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>(
    params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    presets: sync::Arc<sync::Mutex<Presets<NUM_BANDS>>>,
    ui_settings: ui::Settings,
    repaint_notifier: nice_plug_egui::RepaintNotifier,
) -> Option<nice_plug_egui::EguiEditor<Editor<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>> {
    nice_plug_egui::create_egui_editor(
        nice_plug_egui::EguiEditorState::from_size(MIN_WINDOW_SIZE, ZOOM_FACTOR),
        repaint_notifier,
        nice_plug_egui::EguiNiceSettings::new().with_resize_hint(RESIZE_HINT),
        Editor::new(params, presets, ui_settings),
    )
}

pub struct Editor<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
{
    nice_gui_context: Option<nice_plug::context::gui::GuiContext>,
    ui_state: ui::State<NUM_BANDS>,
    ui_settings: ui::Settings,
    params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
    presets: sync::Arc<sync::Mutex<Presets<NUM_BANDS>>>,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    Editor<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(
        params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>>,
        presets: sync::Arc<sync::Mutex<Presets<NUM_BANDS>>>,
        ui_settings: ui::Settings,
    ) -> Self {
        Self {
            nice_gui_context: None,
            ui_state: ui::State {
                show_options: params.show_params.load_options(),
                multiband_eq: params.to_multiband_eq(),
                sample_rate: params.sample_rate.load(atomic::Ordering::Relaxed),
                drag_eq_index: usize::MAX,
                preset_selection: presets::Selection::None,
                new_preset_name: None,
            },
            ui_settings,
            params: params,
            presets: presets,
        }
    }
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    nice_plug_egui::NiceEguiApp for Editor<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    fn build(
        &mut self,
        _egui_context: egui::Context,
        nice_gui_context: GuiContext,
        frame: &mut nice_plug_egui::Frame,
    ) -> Result<(), nice_plug_egui::baseview::HandlerError> {
        self.nice_gui_context = Some(nice_gui_context);
        //frame.set_clear_color(self.settings.color_palette.background.into()); // TODO: doesn't work somehow
        let background_color = self.ui_settings.color_palette.background;
        frame.set_clear_color(egui::ecolor::Rgba::from_rgb(
            background_color.r() as f32 / 255.0,
            background_color.g() as f32 / 255.0,
            background_color.b() as f32 / 255.0,
        ));

        self.params
            .editor_open
            .store(true, atomic::Ordering::Relaxed);
        Ok(())
    }

    fn editor_closed(&mut self) {
        self.nice_gui_context = None;
        self.params
            .editor_open
            .store(false, atomic::Ordering::Relaxed);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut nice_plug_egui::Frame) {
        let Some(nice_gui_context) = self.nice_gui_context.as_mut() else {
            return;
        };

        egui::Frame::new()
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                let mut presets = self.presets.lock().unwrap();
                let backup_eqs = self.params.to_multiband_eq();
                if let presets::Selection::Selected(_, preset) = &self.ui_state.preset_selection {
                    if backup_eqs != *preset {
                        self.ui_state.preset_selection.mark_as_changed();
                    }
                }
                self.ui_state.multiband_eq = backup_eqs.clone();
                self.ui_state.sample_rate = self.params.sample_rate.load(atomic::Ordering::Relaxed);
                self.ui_state.show_options = self.params.show_params.load_options();
                let spectrum_gains = self
                    .params
                    .analyzer_data
                    .linear_gains
                    .consumer
                    .pull_and_read();
                let spectrum_data = Some(ui::SpectrumData {
                    frequency_bins: &self.params.analyzer_data.frequency_bins.read().unwrap(),
                    linear_gains: &spectrum_gains,
                });
                crate::ui::draw(
                    ui,
                    &mut self.ui_state,
                    &mut presets,
                    &self.ui_settings,
                    &spectrum_data,
                );

                let setter = nice_gui_context.param_setter();
                if self.ui_state.multiband_eq.processing_type != backup_eqs.processing_type {
                    self.params
                        .set_multiband_type(self.ui_state.multiband_eq.processing_type, &setter);
                }
                for ((new_eq, old_eq), band_params) in self
                    .ui_state
                    .multiband_eq
                    .eqs
                    .iter()
                    .zip(backup_eqs.eqs)
                    .zip(self.params.eq_params.as_ref())
                {
                    if new_eq.gain.db() != old_eq.gain.db() {
                        band_params.set_gain_db(new_eq.gain.db(), &setter);
                    }
                    if new_eq.frequency.log_hz() != old_eq.frequency.log_hz() {
                        band_params.set_log_frequency(new_eq.frequency.log_hz(), &setter);
                    }
                    if new_eq.q != old_eq.q {
                        band_params.set_q(new_eq.q, &setter);
                    }
                    if new_eq.eq_type != old_eq.eq_type {
                        band_params.set_eq_type(new_eq.eq_type, &setter);
                    }
                    if new_eq.makeup_gain != old_eq.makeup_gain {
                        band_params.set_makeup_gain_db(new_eq.makeup_gain.db(), &setter);
                    }
                }

                self.params
                    .show_params
                    .store_options(&self.ui_state.show_options);
            });
    }
}
