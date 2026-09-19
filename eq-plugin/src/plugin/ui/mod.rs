pub mod editor;

pub type Settings = crate::ui::settings::Settings<f32>;
pub type State<const NUM_BANDS: usize> = crate::ui::State<f32, NUM_BANDS>;
pub use crate::ui::SpectrumData;
pub use crate::ui::settings::ShowOptions;
