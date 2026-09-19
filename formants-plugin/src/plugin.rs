use crate::params::PluginParams;

use super::*;
use audio_lib::*;
use nice::Plugin as NicePlugin;
use std::sync::{self, atomic};

const NUM_CHANNELS: usize = 2;
type Filter = biquad::filter::Filter<f32>;
type ThreeBandFilter = [Filter; 3];
type Limiter = limiter::SimpleLimiter<limiter::gain::SoftKnee<f32>, 500, f32>;

pub struct Plugin {
    params: sync::Arc<params::PluginParams>,
    repaint_notifier: nice_plug_egui::RepaintNotifier,
    filters: [ThreeBandFilter; NUM_CHANNELS],
    limiters: [Limiter; NUM_CHANNELS],
}

impl Plugin {
    pub fn new() -> Self {
        Self {
            params: sync::Arc::new(params::PluginParams::new()),
            repaint_notifier: nice_plug_egui::RepaintNotifier::new(),
            filters: std::array::from_fn(|_| {
                std::array::from_fn(|_| Filter::new(biquad::coefficients::Coefficients::muted()))
            }),
            limiters: std::array::from_fn(|_| Limiter::new_passthrough()),
        }
    }

    const AUDIO_IO_LAYOUTS_INSTANCE: [nice::AudioIOLayout; NUM_CHANNELS] =
        Self::make_audio_layouts();

    const fn make_audio_layouts() -> [nice::AudioIOLayout; NUM_CHANNELS] {
        // seems like std::array::from_fn doesn't work as const :-(
        let mut layouts: [nice::AudioIOLayout; NUM_CHANNELS] =
            [nice::AudioIOLayout::const_default(); NUM_CHANNELS];
        let mut i = 0;
        while i < NUM_CHANNELS {
            let num_channels = (NUM_CHANNELS - i) as u32;
            layouts[i].main_input_channels = nice::NonZeroU32::new(num_channels);
            layouts[i].main_output_channels = nice::NonZeroU32::new(num_channels);
            i += 1;
        }
        layouts
    }
}

impl Default for Plugin {
    fn default() -> Self {
        Self::new()
    }
}

impl nice::Plugin for Plugin {
    const NAME: &'static str = "FormantsPlugin";
    const VENDOR: &'static str = "Stephan Widor";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "stephan@widor.online";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [nice::AudioIOLayout] = &Self::AUDIO_IO_LAYOUTS_INSTANCE;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type Editor = nice_plug_egui::EguiEditor<ui::editor::Editor>;
    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> sync::Arc<dyn nice::Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: nice::AsyncExecutor<Self>) -> Option<Self::Editor> {
        ui::editor::create(self.params.clone(), self.repaint_notifier.clone())
    }

    fn activate(
        &mut self,
        _audio_io_layout: &nice::AudioIOLayout,
        buffer_config: &nice::BufferConfig,
        context: &mut impl nice::ActivateContext<Self>,
    ) -> bool {
        let sample_rate = buffer_config.sample_rate as f32;
        self.params
            .sample_rate
            .store(sample_rate, atomic::Ordering::Relaxed);
        for filters in self.filters.iter_mut() {
            for filter in filters.iter_mut() {
                filter.reset_state();
            }
        }
        for limiter in self.limiters.iter_mut() {
            limiter.set(PluginParams::LIMITER, sample_rate);
        }
        context.set_latency_samples(self.limiters[0].get_lookahead_samples() as u32);
        true
    }

    fn process(
        &mut self,
        buffer: &mut nice::Buffer,
        _aux: &mut nice::AuxiliaryBuffers,
        _context: &mut impl nice::ProcessContext<Self>,
    ) -> nice::ProcessStatus {
        assert!(buffer.channels() <= NUM_CHANNELS);
        let buffer_slice = buffer.as_slice();
        let coefficients = self.params.get_multiband_coefficients();
        let limiter_enabled = self.params.limiter_enabled.value();
        for channel in 0..buffer_slice.len() {
            let channel_samples = buffer_slice.get_mut(channel).unwrap();
            let filters = &mut self.filters[channel];
            for (filter, c) in filters.iter_mut().zip(coefficients.iter()) {
                filter.set_coefficients(c.clone(), false);
            }
            for sample in (*channel_samples).iter_mut() {
                *sample = biquad::multiband::parallel_sum::process(filters.iter_mut(), *sample);
            }

            let limiter = &mut self.limiters[channel];
            limiter.set_enabled(limiter_enabled);
            for sample in (*channel_samples).iter_mut() {
                *sample = limiter.process(*sample);
            }
        }
        nice::ProcessStatus::Normal
    }
}

impl nice::ClapPlugin for Plugin {
    const CLAP_ID: &'static str = "com.stephanwidor.FormantsPlugin";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("This is a simple Formants Plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [nice::ClapFeature] = &[
        nice::ClapFeature::AudioEffect,
        nice::ClapFeature::Mono,
        nice::ClapFeature::Stereo,
    ];
}

impl nice::Vst3Plugin for Plugin {
    const VST3_CLASS_ID: [u8; 16] = *b"widor.FmtsPlugin";
    const VST3_SUBCATEGORIES: &'static [nice::Vst3SubCategory] = &[nice::Vst3SubCategory::Fx];
}
