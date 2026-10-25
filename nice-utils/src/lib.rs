#![allow(dead_code)]

pub mod egui;

use nice_plug::prelude::*;

pub const fn make_audio_layouts<const NUM_CHANNELS: usize>() -> [AudioIOLayout; NUM_CHANNELS] {
    // seems like std::array::from_fn doesn't work as const :-(
    let mut layouts: [AudioIOLayout; NUM_CHANNELS] = [AudioIOLayout::const_default(); NUM_CHANNELS];
    let mut i = 0;
    while i < NUM_CHANNELS {
        let num_channels = (NUM_CHANNELS - i) as u32;
        layouts[i].main_input_channels = NonZeroU32::new(num_channels);
        layouts[i].main_output_channels = NonZeroU32::new(num_channels);
        i += 1;
    }
    layouts
}

pub fn set_param<P: Param, V: Into<P::Plain>>(param: &P, value: V, setter: &ParamSetter<'_>) {
    setter.begin_set_parameter(param);
    setter.set_parameter(param, value.into());
    setter.end_set_parameter(param);
}

pub fn set_params<P: Param, V: Into<P::Plain>, const NUM: usize>(
    params: [&P; NUM],
    values: [V; NUM],
    setter: &ParamSetter<'_>,
) {
    for param in params {
        setter.begin_set_parameter(param);
    }
    for (param, value) in params.iter().zip(values) {
        setter.set_parameter(*param, value.into());
    }
    for param in params {
        setter.end_set_parameter(param);
    }
}

pub fn to_range_inclusive(float_range: &FloatRange) -> std::ops::RangeInclusive<f32> {
    float_range.unnormalize(0_f32)..=float_range.unnormalize(1_f32)
}

pub fn to_range_inclusive_int(int_range: &IntRange) -> std::ops::RangeInclusive<i32> {
    int_range.unnormalize(0_f32)..=int_range.unnormalize(1_f32)
}
