mod params;
mod plugin;
mod ui;

use nice_plug::prelude as nice;
pub use plugin::Plugin;

nice_plug::nice_export_clap!(Plugin);
nice_plug::nice_export_vst3!(Plugin);
