use super::*;

/// Processor for modulations. Modulation ticks will use this to get values for phase in [0,1]
pub trait Processor<F: utils::Float> {
    fn new() -> Self;
    fn value(&self, phase: F) -> F;
}

/// Modulation using Processor<F> for getting modulation values. Each tick advances the phase by step_length in [0,1]-ring.
pub struct Modulation<F: utils::Float, P: Processor<F>> {
    wave_length: f64,
    step_length: f64,
    current_phase: f64,
    pub processor: P,
    phantom: std::marker::PhantomData<F>,
}

impl<F: utils::Float, P: Processor<F>> Modulation<F, P> {
    pub fn new() -> Self {
        Self {
            wave_length: 1.0,
            step_length: 1.0,
            current_phase: 0.0,
            processor: P::new(),
            phantom: std::marker::PhantomData {},
        }
    }

    pub fn tick(&mut self) -> F {
        self.current_phase += self.step_length;
        if self.current_phase > 1.0 {
            self.current_phase = self.current_phase - self.current_phase.floor();
        }
        self.current_value()
    }

    pub fn set_wavelength(&mut self, wave_length: std::time::Duration, sample_rate: F) {
        assert!(sample_rate > F::ZERO);
        self.wave_length = wave_length.as_secs_f64();
        assert!(self.wave_length > 0.0);
        self.step_length = 1.0 / (self.wave_length * sample_rate.cast::<f64>());
    }

    pub fn reset_current_phase(&mut self, time_offset: std::time::Duration) {
        let phase = time_offset.as_secs_f64() / self.wave_length;
        self.current_phase = phase - phase.floor();
    }

    pub fn current_value(&self) -> F {
        self.processor.value(F::from_float(self.current_phase))
    }
}

pub struct SineProcessor<F: utils::Float> {
    pub amplitude: F,
}

impl<F: utils::Float> Processor<F> for SineProcessor<F> {
    fn new() -> Self {
        Self { amplitude: F::ZERO }
    }
    fn value(&self, phase: F) -> F {
        self.amplitude * (phase * F::TWO_PI).sin()
    }
}

pub struct HalfSineProcessor<F: utils::Float> {
    pub amplitude: F,
}

impl<F: utils::Float> Processor<F> for HalfSineProcessor<F> {
    fn new() -> Self {
        Self { amplitude: F::ZERO }
    }
    fn value(&self, phase: F) -> F {
        self.amplitude * (phase * F::PI()).sin()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn sine_processor_ticks() {
        let mut modulation = Modulation::<f32, SineProcessor<f32>>::new();
        let sample_rate = 48000;
        let amplitude = 0.8_f32;
        modulation.set_wavelength(
            std::time::Duration::from_secs_f32(4_f32),
            sample_rate as f32,
        );
        modulation.processor.amplitude = amplitude;
        let eps = 1e-5_f32;
        let mut out = 0_f32;
        for _ in 0..sample_rate {
            out = modulation.tick();
        }
        assert_approx_eq!(out, amplitude, eps);
        for _ in 0..sample_rate {
            out = modulation.tick();
        }
        assert_approx_eq!(out, 0_f32, eps);
        for _ in 0..sample_rate {
            out = modulation.tick();
        }
        assert_approx_eq!(out, -amplitude, eps);
        for _ in 0..sample_rate {
            out = modulation.tick();
        }
        assert_approx_eq!(out, 0_f32, eps);
    }

    #[test]
    fn half_sine_processor_ticks() {
        let mut modulation = Modulation::<f32, HalfSineProcessor<f32>>::new();
        let amplitude = 1_f32;
        modulation.set_wavelength(std::time::Duration::from_secs(1), 48000_f32);
        modulation.processor.amplitude = amplitude;
        let eps = 1e-5_f32;
        let mut out = 0_f32;
        for _ in 0..8000 {
            out = modulation.tick();
        }
        assert_approx_eq!(out, amplitude / 2_f32, eps);
        for _ in 8000..12000 {
            out = modulation.tick();
        }
        assert_approx_eq!(out, 0.5_f32 * amplitude * 2_f32.sqrt(), eps);
        for _ in 12000..16000 {
            out = modulation.tick();
        }
        assert_approx_eq!(out, 0.5_f32 * amplitude * 3_f32.sqrt(), eps);
        for _ in 16000..24000 {
            out = modulation.tick();
        }
        assert_approx_eq!(out, amplitude, eps);
    }

    #[test]
    fn reset_current_phase() {
        let mut modulation = Modulation::<f64, HalfSineProcessor<f64>>::new();
        let wave_length = std::time::Duration::from_secs_f64(1.5);
        modulation.set_wavelength(wave_length, 44100.0);
        modulation.processor.amplitude = 1.0;
        modulation.reset_current_phase(std::time::Duration::from_secs_f64(0.5));
        let expected = 0.5 * 3.0_f64.sqrt();
        assert_approx_eq!(modulation.current_value(), expected, 1e-7);
    }
}
