use crate::plugin::*;
use audio_lib::*;
use std::sync::{self, atomic};

pub type Coefficients = fft::signal_analyzer::Coefficients<f32>;

pub struct Analyzer<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const NUM_BINS: usize> {
    plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, NUM_BINS>>,
    analyzer: fft::SignalAnalyzer<f32, { NUM_BINS }, { NUM_CHANNELS }>,
}

impl<const NUM_BANDS: usize, const NUM_CHANNELS: usize, const NUM_BINS: usize>
    Analyzer<NUM_BANDS, NUM_CHANNELS, NUM_BINS>
{
    pub fn new(
        plugin_params: sync::Arc<params::PluginParams<NUM_BANDS, NUM_CHANNELS, NUM_BINS>>,
        coefficients: &Coefficients,
    ) -> Self {
        Self {
            plugin_params,
            analyzer: fft::SignalAnalyzer::new(coefficients),
        }
    }

    pub fn initialize(&mut self) -> bool {
        let sample_rate = self
            .plugin_params
            .sample_rate
            .load(atomic::Ordering::Relaxed);
        self.plugin_params.analyzer_data.reset(sample_rate);
        self.analyzer.reset_sample_rate(sample_rate);
        true
    }

    pub fn process(&mut self, buffer: &nice::Buffer) {
        let analyzer_data = &self.plugin_params.analyzer_data;

        if self
            .plugin_params
            .show_params
            .signal_gain_spectrum
            .load(atomic::Ordering::Relaxed)
        {
            self.analyzer.push(
                buffer.as_slice_immutable(),
                &analyzer_data.frequency_bins.read().unwrap(),
                &analyzer_data.linear_gains.producer,
            );
        } else {
            self.analyzer.push_mute_signal(
                buffer.samples(),
                &analyzer_data.frequency_bins.read().unwrap(),
                &analyzer_data.linear_gains.producer,
            );
        }
    }
}
