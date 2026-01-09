//! Layer mixing with volume and stereo panning.
//!
//! This module combines multiple audio layers with independent volume and pan
//! controls, producing either mono or stereo output.

/// A single audio layer with mixing parameters.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Audio samples.
    pub samples: Vec<f64>,
    /// Volume level (0.0 to 1.0).
    pub volume: f64,
    /// Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f64,
    /// Delay in samples before this layer starts.
    pub delay_samples: usize,
}

impl Layer {
    /// Creates a new layer.
    pub fn new(samples: Vec<f64>, volume: f64, pan: f64) -> Self {
        Self {
            samples,
            volume: volume.clamp(0.0, 1.0),
            pan: pan.clamp(-1.0, 1.0),
            delay_samples: 0,
        }
    }

    /// Creates a centered (mono) layer.
    pub fn centered(samples: Vec<f64>, volume: f64) -> Self {
        Self::new(samples, volume, 0.0)
    }

    /// Sets a delay for the layer.
    pub fn with_delay(mut self, delay_samples: usize) -> Self {
        self.delay_samples = delay_samples;
        self
    }

    /// Sets a delay in seconds.
    pub fn with_delay_seconds(mut self, delay_seconds: f64, sample_rate: f64) -> Self {
        self.delay_samples = (delay_seconds * sample_rate).round() as usize;
        self
    }
}

/// Stereo audio output.
#[derive(Debug, Clone)]
pub struct StereoOutput {
    /// Left channel samples.
    pub left: Vec<f64>,
    /// Right channel samples.
    pub right: Vec<f64>,
}

impl StereoOutput {
    /// Creates a new stereo output with the given number of samples.
    pub fn new(num_samples: usize) -> Self {
        Self {
            left: vec![0.0; num_samples],
            right: vec![0.0; num_samples],
        }
    }

    /// Creates a stereo output from mono samples.
    pub fn from_mono(mono: Vec<f64>) -> Self {
        Self {
            left: mono.clone(),
            right: mono,
        }
    }

    /// Creates interleaved stereo samples.
    pub fn interleave(&self) -> Vec<f64> {
        let mut output = Vec::with_capacity(self.left.len() * 2);
        for (l, r) in self.left.iter().zip(self.right.iter()) {
            output.push(*l);
            output.push(*r);
        }
        output
    }

    /// Returns true if left and right channels are identical (mono-compatible).
    pub fn is_mono(&self) -> bool {
        self.left == self.right
    }

    /// Converts to mono by averaging channels.
    pub fn to_mono(&self) -> Vec<f64> {
        self.left
            .iter()
            .zip(self.right.iter())
            .map(|(l, r)| (l + r) * 0.5)
            .collect()
    }

    /// Gets the number of samples per channel.
    pub fn len(&self) -> usize {
        self.left.len()
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.left.is_empty()
    }
}

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

/// Output from the mixer (mono or stereo).
#[derive(Debug, Clone)]
pub enum MixerOutput {
    /// Mono output.
    Mono(Vec<f64>),
    /// Stereo output.
    Stereo(StereoOutput),
}

impl MixerOutput {
    /// Returns true if this is stereo output.
    pub fn is_stereo(&self) -> bool {
        matches!(self, MixerOutput::Stereo(_))
    }

    /// Converts to mono (averaging if stereo).
    pub fn to_mono(&self) -> Vec<f64> {
        match self {
            MixerOutput::Mono(samples) => samples.clone(),
            MixerOutput::Stereo(stereo) => stereo.to_mono(),
        }
    }

    /// Converts to stereo (duplicating if mono).
    pub fn to_stereo(&self) -> StereoOutput {
        match self {
            MixerOutput::Mono(samples) => StereoOutput::from_mono(samples.clone()),
            MixerOutput::Stereo(stereo) => stereo.clone(),
        }
    }

    /// Gets the number of samples per channel.
    pub fn len(&self) -> usize {
        match self {
            MixerOutput::Mono(samples) => samples.len(),
            MixerOutput::Stereo(stereo) => stereo.len(),
        }
    }

    /// Returns true if empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Normalizes audio to prevent clipping.
///
/// # Arguments
/// * `samples` - Audio samples to normalize
/// * `headroom_db` - Headroom in dB below 0 dBFS (e.g., -3.0 for -3dB headroom)
pub fn normalize(samples: &mut [f64], headroom_db: f64) {
    let target_peak = 10.0_f64.powf(headroom_db / 20.0);
    let current_peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));

    if current_peak > 0.0 {
        let gain = target_peak / current_peak;
        for sample in samples.iter_mut() {
            *sample *= gain;
        }
    }
}

/// Normalizes stereo audio.
pub fn normalize_stereo(stereo: &mut StereoOutput, headroom_db: f64) {
    let target_peak = 10.0_f64.powf(headroom_db / 20.0);

    let left_peak = stereo.left.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
    let right_peak = stereo.right.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
    let current_peak = left_peak.max(right_peak);

    if current_peak > 0.0 {
        let gain = target_peak / current_peak;
        for sample in stereo.left.iter_mut() {
            *sample *= gain;
        }
        for sample in stereo.right.iter_mut() {
            *sample *= gain;
        }
    }
}

/// Applies soft clipping to prevent harsh digital distortion.
///
/// # Arguments
/// * `sample` - Input sample
/// * `threshold` - Threshold above which soft clipping begins (0.0 to 1.0)
///
/// # Returns
/// Soft-clipped sample
#[inline]
pub fn soft_clip(sample: f64, threshold: f64) -> f64 {
    let abs = sample.abs();
    if abs <= threshold {
        sample
    } else {
        let sign = sample.signum();
        let excess = abs - threshold;
        let compressed = threshold + (1.0 - threshold) * (1.0 - (-excess * 3.0).exp());
        sign * compressed
    }
}

/// Applies soft clipping to a buffer.
pub fn soft_clip_buffer(samples: &mut [f64], threshold: f64) {
    for sample in samples.iter_mut() {
        *sample = soft_clip(*sample, threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mono_mixing() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 50], 1.0);
        mixer.add_mono(vec![0.3; 50], 1.0);

        let output = mixer.mix_mono();
        assert_eq!(output.len(), 100);
        // First 50 samples should have both layers
        assert!((output[25] - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_stereo_panning() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left
        mixer.add_panned(vec![1.0; 100], 1.0, 1.0); // Hard right

        let output = mixer.mix_stereo();

        // Left channel should be louder from left-panned signal
        // Right channel should be louder from right-panned signal
        assert!(output.left[50] > 0.5);
        assert!(output.right[50] > 0.5);
    }

    #[test]
    fn test_center_pan_equal_power() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, 0.0); // Center

        let output = mixer.mix_stereo();

        // Center pan should have equal power in both channels
        // At center, each channel gets cos(45deg) = ~0.707 of the signal
        let expected = std::f64::consts::FRAC_PI_4.cos();
        assert!((output.left[50] - expected).abs() < 0.01);
        assert!((output.right[50] - expected).abs() < 0.01);
    }

    #[test]
    fn test_layer_delay() {
        let mut mixer = Mixer::new(100, 44100.0);
        let layer = Layer::centered(vec![1.0; 20], 1.0).with_delay(50);
        mixer.add_layer(layer);

        let output = mixer.mix_mono();

        // First 50 samples should be silent
        assert!(output[49].abs() < 0.001);
        // Sample 50 onwards should have signal
        assert!((output[50] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalization() {
        let mut samples = vec![0.5, -0.3, 0.8, -0.2];
        normalize(&mut samples, -3.0);

        // Peak should be at -3dB
        let target = 10.0_f64.powf(-3.0 / 20.0);
        let peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        assert!((peak - target).abs() < 0.01);
    }

    #[test]
    fn test_soft_clip() {
        // Below threshold: unchanged
        assert!((soft_clip(0.5, 0.8) - 0.5).abs() < 0.001);

        // Above threshold: compressed
        let clipped = soft_clip(2.0, 0.8);
        assert!(clipped < 2.0);
        assert!(clipped > 0.8);

        // Very high values should approach but not exceed 1.0
        let hard = soft_clip(10.0, 0.8);
        assert!(hard < 1.0);
    }

    #[test]
    fn test_mixer_output() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 100], 1.0);

        let output = mixer.mix();
        assert!(!output.is_stereo()); // Should be mono since no panning

        let mut mixer_stereo = Mixer::new(100, 44100.0);
        mixer_stereo.add_panned(vec![0.5; 100], 1.0, 0.5);

        let output_stereo = mixer_stereo.mix();
        assert!(output_stereo.is_stereo()); // Should be stereo with panning
    }

    #[test]
    fn test_stereo_interleave() {
        let stereo = StereoOutput {
            left: vec![1.0, 2.0, 3.0],
            right: vec![4.0, 5.0, 6.0],
        };

        let interleaved = stereo.interleave();
        assert_eq!(interleaved, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }
}
