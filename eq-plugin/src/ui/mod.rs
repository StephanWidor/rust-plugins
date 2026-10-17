pub mod colors;
pub mod control;
pub mod plotter;
pub mod settings;
pub mod state;
pub mod utils;

pub use state::State;

use crate::presets;
use audio_lib::fft;
use audio_lib::utils as audio_utils;

pub struct SpectrumData<'a, F: audio_utils::Float, const NUM_BINS: usize, const NUM_CHANNELS: usize>
{
    pub frequency_bins: &'a fft::LogFrequencyRangeBins<F, NUM_BINS>,
    pub linear_gains: &'a [[F; NUM_BINS]; NUM_CHANNELS],
}

pub fn draw<
    F: audio_utils::Float + egui::emath::Numeric,
    const NUM_BANDS: usize,
    const NUM_SPECTRUM_BINS: usize,
    const NUM_SPECTRUM_CHANNELS: usize,
>(
    ui: &mut egui::Ui,
    ui_state: &mut State<F, NUM_BANDS>,
    presets: &mut presets::Presets<F, NUM_BANDS>,
    settings: &crate::ui::settings::Settings<F>,
    spectrum_data: &Option<SpectrumData<F, NUM_SPECTRUM_BINS, NUM_SPECTRUM_CHANNELS>>,
) {
    let ui_size = ui.available_size();

    ui.horizontal(|ui| {
        let control_width = 250_f32;
        control::add(
            ui,
            egui::Vec2::new(control_width, ui_size.y),
            ui_state,
            presets,
            settings,
            spectrum_data.is_some(),
        );

        if !(ui_state.show_options.gain
            || ui_state.show_options.phase
            || ui_state.show_options.impulse_response
            || ui_state.show_options.poles_and_zeros)
        {
            return;
        }
        let available_size = egui::Vec2::new(0.96_f32 * (ui_size.x - control_width), ui_size.y);
        plotter::add_plots(ui, &available_size, ui_state, settings, spectrum_data);
    });
}
