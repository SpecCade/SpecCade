//! Audio mixer for combining multiple layers.

use super::types::{Layer, MixerOutput, StereoOutput};

/// Audio mixer for combining multiple layers.
#[derive(Debug)]
pub struct Mixer {
    /// Output sample length.
    num_samples: usize,
    /// Sample rate.
    sample_rate: f64,
    /// Accumulated layers.
    layers: Vec<Layer>,
    /// Whether any layer has non-zero pan.
    has_stereo_content: bool,
}

impl Mixer {
    /// Creates a new mixer.
    ///
    /// # Arguments
    /// * `num_samples` - Number of output samples
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(num_samples: usize, sample_rate: f64) -> Self {
        Self {
            num_samples,
            sample_rate,
            layers: Vec::new(),
            has_stereo_content: false,
        }
    }

    /// Adds a layer to the mix.
    pub fn add_layer(&mut self, layer: Layer) {
        if layer.pan.abs() > 1e-6 {
            self.has_stereo_content = true;
        }
        self.layers.push(layer);
    }

    /// Adds samples as a centered layer.
    pub fn add_mono(&mut self, samples: Vec<f64>, volume: f64) {
        self.add_layer(Layer::centered(samples, volume));
    }

    /// Adds samples with pan.
    pub fn add_panned(&mut self, samples: Vec<f64>, volume: f64, pan: f64) {
        self.add_layer(Layer::new(samples, volume, pan));
    }

    /// Returns whether the mix has stereo content.
    pub fn is_stereo(&self) -> bool {
        self.has_stereo_content
    }

    /// Mixes all layers to mono output.
    pub fn mix_mono(&self) -> Vec<f64> {
        let mut output = vec![0.0; self.num_samples];

        for layer in &self.layers {
            let start = layer.delay_samples;

            for (i, &sample) in layer.samples.iter().enumerate() {
                let output_idx = start + i;
                if output_idx < self.num_samples {
                    output[output_idx] += sample * layer.volume;
                }
            }
        }

        output
    }

    /// Mixes all layers to stereo output using equal power panning.
    pub fn mix_stereo(&self) -> StereoOutput {
        let mut output = StereoOutput::new(self.num_samples);

        for layer in &self.layers {
            // Equal power panning
            let pan_angle = (layer.pan + 1.0) * std::f64::consts::FRAC_PI_4; // 0 to PI/2
            let left_gain = pan_angle.cos() * layer.volume;
            let right_gain = pan_angle.sin() * layer.volume;

            let start = layer.delay_samples;

            for (i, &sample) in layer.samples.iter().enumerate() {
                let output_idx = start + i;
                if output_idx < self.num_samples {
                    output.left[output_idx] += sample * left_gain;
                    output.right[output_idx] += sample * right_gain;
                }
            }
        }

        output
    }

    /// Mixes all layers, automatically choosing mono or stereo based on content.
    pub fn mix(&self) -> MixerOutput {
        if self.has_stereo_content {
            MixerOutput::Stereo(self.mix_stereo())
        } else {
            MixerOutput::Mono(self.mix_mono())
        }
    }

    /// Returns the sample rate.
    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    /// Returns the number of output samples.
    pub fn num_samples(&self) -> usize {
        self.num_samples
    }
}
