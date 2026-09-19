pub struct State<F: audio_lib::utils::Float, const NUM_BANDS: usize> {
    pub show_options: crate::ui::settings::ShowOptions,
    pub multiband_eq: audio_lib::eq::MultibandEq<F, NUM_BANDS>,
    pub sample_rate: F,
    pub drag_eq_index: usize,
    pub preset_selection: crate::presets::Selection<F, NUM_BANDS>,
    pub new_preset_name: Option<String>,
}
