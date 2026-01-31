//! Audio mixer for combining multiple layers.

use super::types::{Layer, LayerSamples, MixerOutput, StereoOutput};

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
        if layer.pan.abs() > 1e-6 || layer.pan_curve.is_some() || layer.samples.is_stereo() {
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

            match &layer.samples {
                LayerSamples::Mono(samples) => {
                    for (i, &sample) in samples.iter().enumerate() {
                        let output_idx = start + i;
                        if output_idx < self.num_samples {
                            output[output_idx] += sample * layer.volume;
                        }
                    }
                }
                LayerSamples::Stereo { left, right } => {
                    // Mix stereo down to mono by averaging
                    for (i, (&l, &r)) in left.iter().zip(right.iter()).enumerate() {
                        let output_idx = start + i;
                        if output_idx < self.num_samples {
                            output[output_idx] += (l + r) * 0.5 * layer.volume;
                        }
                    }
                }
            }
        }

        output
    }

    /// Mixes all layers to stereo output using equal power panning.
    pub fn mix_stereo(&self) -> StereoOutput {
        let mut output = StereoOutput::new(self.num_samples);

        for layer in &self.layers {
            let start = layer.delay_samples;

            match &layer.samples {
                LayerSamples::Mono(samples) => {
                    // For mono sources: apply equal-power panning
                    self.mix_mono_layer_to_stereo(
                        samples,
                        layer.volume,
                        layer.pan,
                        layer.pan_curve.as_ref(),
                        start,
                        &mut output,
                    );
                }
                LayerSamples::Stereo {
                    left: src_left,
                    right: src_right,
                } => {
                    // For stereo sources: apply stereo image positioning
                    self.mix_stereo_layer_to_stereo(
                        src_left,
                        src_right,
                        layer.volume,
                        layer.pan,
                        start,
                        &mut output,
                    );
                }
            }
        }

        output
    }

    /// Mix a mono layer into stereo output using equal power panning.
    fn mix_mono_layer_to_stereo(
        &self,
        samples: &[f64],
        volume: f64,
        pan: f64,
        pan_curve: Option<&Vec<f64>>,
        start: usize,
        output: &mut StereoOutput,
    ) {
        match pan_curve {
            Some(curve) => {
                for (i, &sample) in samples.iter().enumerate() {
                    let output_idx = start + i;
                    if output_idx < self.num_samples {
                        let p = curve.get(i).copied().unwrap_or(pan);
                        let pan_angle = (p + 1.0) * std::f64::consts::FRAC_PI_4;
                        let left_gain = pan_angle.cos() * volume;
                        let right_gain = pan_angle.sin() * volume;

                        output.left[output_idx] += sample * left_gain;
                        output.right[output_idx] += sample * right_gain;
                    }
                }
            }
            None => {
                let pan_angle = (pan + 1.0) * std::f64::consts::FRAC_PI_4;
                let left_gain = pan_angle.cos() * volume;
                let right_gain = pan_angle.sin() * volume;

                for (i, &sample) in samples.iter().enumerate() {
                    let output_idx = start + i;
                    if output_idx < self.num_samples {
                        output.left[output_idx] += sample * left_gain;
                        output.right[output_idx] += sample * right_gain;
                    }
                }
            }
        }
    }

    /// Mix a stereo layer into stereo output.
    ///
    /// For stereo layers, `pan` controls stereo image positioning:
    /// - pan = 0: stereo image unchanged (L -> L, R -> R)
    /// - pan = -1: stereo collapses to left (both L and R go to L only)
    /// - pan = 1: stereo collapses to right (both L and R go to R only)
    fn mix_stereo_layer_to_stereo(
        &self,
        src_left: &[f64],
        src_right: &[f64],
        volume: f64,
        pan: f64,
        start: usize,
        output: &mut StereoOutput,
    ) {
        // Pan controls how the stereo image is shifted:
        // At pan=0: L goes fully to left output, R goes fully to right output
        // At pan=-1: Both L and R go fully to left output
        // At pan=+1: Both L and R go fully to right output
        //
        // We use a crossfade approach:
        // - left_to_left and right_to_right decrease as |pan| increases
        // - left_to_right increases with positive pan
        // - right_to_left increases with negative pan

        let pan_normalized = (pan + 1.0) / 2.0; // 0.0 (hard left) to 1.0 (hard right)

        // At center (pan=0, normalized=0.5): L->L=1, R->R=1, cross=0
        // At hard left (pan=-1, normalized=0): L->L=1, R->L=1, L->R=0, R->R=0
        // At hard right (pan=1, normalized=1): L->R=1, R->R=1, L->L=0, R->L=0

        let left_to_left = (1.0 - pan_normalized) * volume;
        let left_to_right = pan_normalized * volume;
        let right_to_left = (1.0 - pan_normalized) * volume;
        let right_to_right = pan_normalized * volume;

        for (i, (&l, &r)) in src_left.iter().zip(src_right.iter()).enumerate() {
            let output_idx = start + i;
            if output_idx < self.num_samples {
                output.left[output_idx] += l * left_to_left + r * right_to_left;
                output.right[output_idx] += l * left_to_right + r * right_to_right;
            }
        }
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
