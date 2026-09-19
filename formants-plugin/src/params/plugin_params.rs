use super::*;
use audio_lib::*;
use std::sync::atomic;

#[derive(nice::Params)]
pub struct PluginParams {
    #[id = "x"]
    pub x: nice::FloatParam,

    #[id = "y"]
    pub y: nice::FloatParam,

    #[nested(group = "Path")]
    pub path: Path,

    #[id = "Dry Gain (dB)"]
    pub dry_gain_db: nice::FloatParam,

    #[id = "Limiter Enabled"]
    pub limiter_enabled: nice::BoolParam,

    pub sample_rate: nice::AtomicF32,

    pub frequency_transform: FrequencyTransform,
}

impl PluginParams {
    pub fn new() -> Self {
        Self {
            x: nice::FloatParam::new(
                "x",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            y: nice::FloatParam::new(
                "y",
                0.5_f32,
                nice::FloatRange::Linear {
                    min: 0_f32,
                    max: 1_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            path: Path::new(),
            dry_gain_db: nice::FloatParam::new(
                format!("Mix"),
                -36_f32,
                nice::FloatRange::Linear {
                    min: -60_f32,
                    max: 0_f32,
                },
            )
            .with_smoother(nice::SmoothingStyle::Linear(SMOOTHING_LENGTH)),
            limiter_enabled: nice::BoolParam::new("Limiter Enabled", true),
            sample_rate: nice::AtomicF32::new(48000_f32),
            frequency_transform: FrequencyTransform::default(),
        }
    }

    pub fn set_x(&self, x: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.x);
        setter.set_parameter(&self.x, x);
        setter.end_set_parameter(&self.x);
    }

    pub fn set_y(&self, y: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.y);
        setter.set_parameter(&self.y, y);
        setter.end_set_parameter(&self.y);
    }

    pub fn set_dry_gain_db(&self, gain_db: f32, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.dry_gain_db);
        setter.set_parameter(&self.dry_gain_db, gain_db);
        setter.end_set_parameter(&self.dry_gain_db);
    }

    pub fn set_limiter_enabled(&self, enabled: bool, setter: &nice::ParamSetter<'_>) {
        setter.begin_set_parameter(&self.limiter_enabled);
        setter.set_parameter(&self.limiter_enabled, enabled);
        setter.end_set_parameter(&self.limiter_enabled);
    }

    pub fn get_multiband_coefficients(&self) -> [biquad::coefficients::Coefficients<f32>; 3] {
        use biquad::coefficients::Coefficients;
        let frequencies = self.get_frequencies();
        let sample_rate = self.sample_rate.load(atomic::Ordering::Relaxed);
        [
            Self::get_single_band_coefficients(
                frequencies[0],
                self.frequency_transform.q_for_frequency(frequencies[0], 0),
                self.frequency_transform
                    .gain_db_for_frequency(frequencies[0], 0),
                sample_rate,
            ),
            Self::get_single_band_coefficients(
                frequencies[1],
                self.frequency_transform.q_for_frequency(frequencies[1], 1),
                self.frequency_transform
                    .gain_db_for_frequency(frequencies[1], 1),
                sample_rate,
            ),
            Coefficients::from_volume_db(self.dry_gain_db.value()),
        ]
    }

    pub fn get_xy<F: audio_lib::utils::Float>(&self) -> [F; 2] {
        if self.path.enabled.value() {
            self.path.get_point::<F>()
        } else {
            [F::from_float(self.x.value()), F::from_float(self.y.value())]
        }
    }

    fn get_frequencies(&self) -> [f32; 2] {
        if self.path.enabled.value() {
            let [x, y] = self.path.get_point::<f32>();
            self.frequency_transform.coordinates_to_frequencies(x, y)
        } else {
            self.frequency_transform
                .coordinates_to_frequencies(self.x.value(), self.y.value())
        }
    }

    fn get_single_band_coefficients(
        frequency: f32,
        q: f32,
        gain_db: f32,
        sample_rate: f32,
    ) -> biquad::coefficients::Coefficients<f32> {
        let mut c = biquad::coefficients::Coefficients::from_bandpass(frequency, q, sample_rate);
        c.add_makeup_gain(eq::Gain::Db(gain_db));
        c
    }

    pub const LIMITER: limiter::simple_limiter::Params<limiter::gain::SoftKnee<f32>, f32> =
        limiter::simple_limiter::Params {
            knee: limiter::gain::SoftKneeParams {
                threshold_linear: 0.9,
                compression_factor: 0.0,
                knee_ratio: 1.1,
            },
            attack_time: 0.0,
            release_time: 0.1,
            lookahead_time: 0.002,
        };
}
