pub mod params;
pub mod plugin;
pub mod processing;
pub mod ui;

use super::*;
use config::{Config, ConfigTrait};
use nice_plug::prelude as nice;

pub type Settings<const NUM_BANDS: usize> = settings::Settings<f32, NUM_BANDS>;
pub type Presets<const NUM_BANDS: usize> = presets::Presets<f32, NUM_BANDS>;

pub type Plugin =
    plugin::Plugin<{ Config::NUM_BANDS }, { Config::NUM_CHANNELS }, { Config::ANALYZER_NUM_BINS }>;

nice_plug::nice_export_clap!(Plugin);
nice_plug::nice_export_vst3!(Plugin);
