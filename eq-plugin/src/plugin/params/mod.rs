use super::*;
use audio_lib::*;
use std::sync::{self, atomic};

pub mod eq_params;
pub mod eq_type;
pub mod multiband_type;
pub mod show_params;

pub use eq_params::EqParams;
pub use show_params::ShowParams;

// hm, can we somehow get rid of this without destroying the nice::Enum and nice::Params derive?
use nice_plug::params::Params;

#[derive(nice::Params)]
pub struct PluginParams<
    const NUM_BANDS: usize,
    const NUM_CHANNELS: usize,
    const ANALYZER_NUM_BINS: usize,
> {
    #[nested(array, group = "eq_params")]
    pub eq_params: [EqParams; NUM_BANDS],

    #[id = "multiband_type"]
    pub multiband_type: multiband_type::Param,

    pub sample_rate: nice::AtomicF32,

    #[nested(group = "show_params")]
    pub show_params: ShowParams,

    pub editor_open: atomic::AtomicBool,

    pub analyzer_data:
        fft::signal_analyzer::SharedData<f32, { ANALYZER_NUM_BINS }, { NUM_CHANNELS }>,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const ANALYZER_NUM_BINS: usize>
    PluginParams<NUM_BANDS, NUM_CHANNELS, ANALYZER_NUM_BINS>
{
    pub fn new(settings: &Settings<NUM_BANDS>) -> Self {
        let eq_ranges = settings.ui.eq_ranges.clone();
        Self {
            eq_params: std::array::from_fn(|index| {
                EqParams::from_eq(
                    format!(" [{}]", index + 1).as_str(),
                    &settings.init_eq.eqs[index],
                    &eq_ranges.log_frequency_range,
                    &eq_ranges.db_range,
                    &eq_ranges.q_range,
                    settings.parameter_smoothing_length_ms,
                )
            }),
            multiband_type: multiband_type::Param::new(
                format!("Multiband Processing Type"),
                multiband_type::Wrapper::from(settings.init_eq.processing_type),
            ),
            sample_rate: nice::AtomicF32::new(settings.init_sample_rate),
            show_params: ShowParams::from_options(&settings.ui.init_show_options),
            editor_open: atomic::AtomicBool::new(false),
            analyzer_data: fft::signal_analyzer::SharedData::new(settings.init_sample_rate),
        }
    }

    pub fn to_multiband_eq<F: utils::Float>(&self) -> eq::MultibandEq<F, NUM_BANDS> {
        eq::MultibandEq {
            eqs: std::array::from_fn(|index| self.eq_params[index].to_eq()),
            processing_type: self.multiband_type.value().into(),
        }
    }

    pub fn set_multiband_type(
        &self,
        multiband_type: eq::MultibandType,
        setter: &nice::ParamSetter<'_>,
    ) {
        setter.begin_set_parameter(&self.multiband_type);
        setter.set_parameter(&self.multiband_type, multiband_type.into());
        setter.end_set_parameter(&self.multiband_type);
    }
}
