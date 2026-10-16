pub mod frequency_transform;
pub mod path;
pub mod plugin_params;

use super::*;
use nice::Params;

pub use frequency_transform::*;
pub use path::*;
pub use plugin_params::*;

const SMOOTHING_LENGTH: f32 = 20_f32;
