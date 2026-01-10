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

    // ============================================================================
    // Layer Construction Tests
    // ============================================================================

    #[test]
    fn test_layer_new() {
        let samples = vec![0.1, 0.2, 0.3];
        let layer = Layer::new(samples.clone(), 0.8, 0.5);
        assert_eq!(layer.samples, samples);
        assert_eq!(layer.volume, 0.8);
        assert_eq!(layer.pan, 0.5);
        assert_eq!(layer.delay_samples, 0);
    }

    #[test]
    fn test_layer_new_clamps_volume() {
        // Volume above 1.0 should be clamped
        let layer_high = Layer::new(vec![0.5], 1.5, 0.0);
        assert_eq!(layer_high.volume, 1.0);

        // Volume below 0.0 should be clamped
        let layer_low = Layer::new(vec![0.5], -0.5, 0.0);
        assert_eq!(layer_low.volume, 0.0);
    }

    #[test]
    fn test_layer_new_clamps_pan() {
        // Pan above 1.0 should be clamped
        let layer_high = Layer::new(vec![0.5], 1.0, 2.0);
        assert_eq!(layer_high.pan, 1.0);

        // Pan below -1.0 should be clamped
        let layer_low = Layer::new(vec![0.5], 1.0, -2.0);
        assert_eq!(layer_low.pan, -1.0);
    }

    #[test]
    fn test_layer_centered() {
        let samples = vec![0.1, 0.2, 0.3];
        let layer = Layer::centered(samples.clone(), 0.75);
        assert_eq!(layer.samples, samples);
        assert_eq!(layer.volume, 0.75);
        assert_eq!(layer.pan, 0.0); // Centered means pan = 0.0
        assert_eq!(layer.delay_samples, 0);
    }

    #[test]
    fn test_layer_with_delay() {
        let samples = vec![1.0, 2.0, 3.0];
        let layer = Layer::new(samples, 1.0, 0.0).with_delay(100);
        assert_eq!(layer.delay_samples, 100);
    }

    #[test]
    fn test_layer_with_delay_chainable() {
        let samples = vec![1.0, 2.0];
        let layer = Layer::new(samples, 0.5, 0.25)
            .with_delay(50)
            .with_delay(100); // Second call should override
        assert_eq!(layer.delay_samples, 100);
        assert_eq!(layer.volume, 0.5);
        assert_eq!(layer.pan, 0.25);
    }

    #[test]
    fn test_layer_with_delay_seconds() {
        let samples = vec![1.0, 2.0, 3.0];
        let sample_rate = 44100.0;
        let delay_seconds = 0.5; // 500ms

        let layer = Layer::new(samples, 1.0, 0.0).with_delay_seconds(delay_seconds, sample_rate);

        // 0.5 seconds at 44100 Hz = 22050 samples
        let expected_delay = (delay_seconds * sample_rate).round() as usize;
        assert_eq!(layer.delay_samples, expected_delay);
        assert_eq!(layer.delay_samples, 22050);
    }

    #[test]
    fn test_layer_with_delay_seconds_fractional() {
        let samples = vec![1.0];
        let sample_rate = 48000.0;
        let delay_seconds = 0.001; // 1ms

        let layer = Layer::new(samples, 1.0, 0.0).with_delay_seconds(delay_seconds, sample_rate);

        // 0.001 seconds at 48000 Hz = 48 samples
        assert_eq!(layer.delay_samples, 48);
    }

    // ============================================================================
    // StereoOutput Tests
    // ============================================================================

    #[test]
    fn test_stereo_output_new() {
        let stereo = StereoOutput::new(100);
        assert_eq!(stereo.left.len(), 100);
        assert_eq!(stereo.right.len(), 100);
        assert!(stereo.left.iter().all(|&s| s == 0.0));
        assert!(stereo.right.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_stereo_from_mono() {
        let mono = vec![1.0, 0.5, 0.0, -0.5, -1.0];
        let stereo = StereoOutput::from_mono(mono.clone());
        assert_eq!(stereo.left, mono);
        assert_eq!(stereo.right, mono);
        assert_eq!(stereo.left, stereo.right);
    }

    #[test]
    fn test_stereo_from_mono_empty() {
        let mono: Vec<f64> = vec![];
        let stereo = StereoOutput::from_mono(mono);
        assert!(stereo.left.is_empty());
        assert!(stereo.right.is_empty());
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

    #[test]
    fn test_stereo_interleave_empty() {
        let stereo = StereoOutput::new(0);
        let interleaved = stereo.interleave();
        assert!(interleaved.is_empty());
    }

    #[test]
    fn test_stereo_interleave_single_sample() {
        let stereo = StereoOutput {
            left: vec![0.5],
            right: vec![-0.5],
        };
        let interleaved = stereo.interleave();
        assert_eq!(interleaved, vec![0.5, -0.5]);
    }

    #[test]
    fn test_stereo_is_mono_true() {
        let stereo = StereoOutput::from_mono(vec![1.0, 0.5, 0.0]);
        assert!(stereo.is_mono());
    }

    #[test]
    fn test_stereo_is_mono_false() {
        let stereo = StereoOutput {
            left: vec![1.0, 0.5, 0.0],
            right: vec![0.5, 0.25, 0.0],
        };
        assert!(!stereo.is_mono());
    }

    #[test]
    fn test_stereo_to_mono() {
        let stereo = StereoOutput {
            left: vec![1.0, 0.5, 0.0],
            right: vec![0.0, 0.5, 1.0],
        };
        let mono = stereo.to_mono();
        assert_eq!(mono, vec![0.5, 0.5, 0.5]); // Average of each pair
    }

    #[test]
    fn test_stereo_to_mono_preserves_mono_content() {
        let original = vec![1.0, 0.5, -0.5];
        let stereo = StereoOutput::from_mono(original.clone());
        let mono = stereo.to_mono();
        assert_eq!(mono, original);
    }

    #[test]
    fn test_stereo_len() {
        let stereo = StereoOutput::new(50);
        assert_eq!(stereo.len(), 50);
    }

    #[test]
    fn test_stereo_is_empty() {
        let empty = StereoOutput::new(0);
        assert!(empty.is_empty());

        let non_empty = StereoOutput::new(1);
        assert!(!non_empty.is_empty());
    }

    // ============================================================================
    // Mixer Basic Tests
    // ============================================================================

    #[test]
    fn test_mixer_new() {
        let mixer = Mixer::new(1000, 48000.0);
        assert_eq!(mixer.num_samples(), 1000);
        assert_eq!(mixer.sample_rate(), 48000.0);
        assert!(!mixer.is_stereo());
    }

    #[test]
    fn test_mixer_add_layer_mono() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_layer(Layer::centered(vec![0.5; 50], 1.0));
        assert!(!mixer.is_stereo()); // Centered layer doesn't create stereo content
    }

    #[test]
    fn test_mixer_add_layer_stereo() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_layer(Layer::new(vec![0.5; 50], 1.0, 0.5)); // Non-zero pan
        assert!(mixer.is_stereo());
    }

    #[test]
    fn test_mixer_add_mono() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 50], 0.8);
        assert!(!mixer.is_stereo());
    }

    #[test]
    fn test_mixer_add_panned() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![0.5; 50], 0.8, -0.5);
        assert!(mixer.is_stereo());
    }

    // ============================================================================
    // Mono Mixing Tests
    // ============================================================================

    #[test]
    fn test_mix_mono_single_layer() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 100], 1.0);

        let output = mixer.mix_mono();
        assert_eq!(output.len(), 100);
        assert!(output.iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_mix_mono_single_layer_with_volume() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![1.0; 100], 0.5);

        let output = mixer.mix_mono();
        assert!(output.iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_mix_mono_multiple_layers() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.3; 100], 1.0);
        mixer.add_mono(vec![0.2; 100], 1.0);
        mixer.add_mono(vec![0.1; 100], 1.0);

        let output = mixer.mix_mono();
        // Sum should be 0.3 + 0.2 + 0.1 = 0.6
        assert!(output.iter().all(|&s| (s - 0.6).abs() < 0.001));
    }

    #[test]
    fn test_mix_mono_multiple_layers_with_volumes() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![1.0; 100], 0.5); // 1.0 * 0.5 = 0.5
        mixer.add_mono(vec![1.0; 100], 0.25); // 1.0 * 0.25 = 0.25

        let output = mixer.mix_mono();
        assert!(output.iter().all(|&s| (s - 0.75).abs() < 0.001));
    }

    #[test]
    fn test_mix_mono_partial_overlap() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 50], 1.0); // Samples 0-49
        mixer.add_layer(Layer::centered(vec![0.3; 50], 1.0).with_delay(25)); // Samples 25-74

        let output = mixer.mix_mono();

        // Samples 0-24: only first layer (0.5)
        assert!((output[10] - 0.5).abs() < 0.001);
        // Samples 25-49: both layers overlap (0.5 + 0.3 = 0.8)
        assert!((output[30] - 0.8).abs() < 0.001);
        // Samples 50-74: only second layer (0.3)
        assert!((output[60] - 0.3).abs() < 0.001);
        // Samples 75-99: silent
        assert!((output[80] - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_mix_mono_empty_layers() {
        let mixer = Mixer::new(100, 44100.0);
        let output = mixer.mix_mono();
        assert!(output.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_mix_mono_layer_exceeds_output_length() {
        let mut mixer = Mixer::new(50, 44100.0);
        mixer.add_mono(vec![1.0; 100], 1.0); // Layer is longer than output

        let output = mixer.mix_mono();
        assert_eq!(output.len(), 50);
        assert!(output.iter().all(|&s| (s - 1.0).abs() < 0.001));
    }

    // ============================================================================
    // Stereo Panning Tests
    // ============================================================================

    #[test]
    fn test_mix_stereo_panning_left() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left (pan = -1.0)

        let output = mixer.mix_stereo();

        // At hard left, pan_angle = 0, so cos(0) = 1.0 and sin(0) = 0.0
        assert!((output.left[50] - 1.0).abs() < 0.001);
        assert!(output.right[50].abs() < 0.001);
    }

    #[test]
    fn test_mix_stereo_panning_right() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, 1.0); // Hard right (pan = 1.0)

        let output = mixer.mix_stereo();

        // At hard right, pan_angle = PI/2, so cos(PI/2) = 0.0 and sin(PI/2) = 1.0
        assert!(output.left[50].abs() < 0.001);
        assert!((output.right[50] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_mix_stereo_panning_center() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, 0.0); // Center (pan = 0.0)

        let output = mixer.mix_stereo();

        // At center, pan_angle = PI/4, so cos(PI/4) = sin(PI/4) = ~0.707
        let expected = std::f64::consts::FRAC_PI_4.cos();
        assert!((output.left[50] - expected).abs() < 0.001);
        assert!((output.right[50] - expected).abs() < 0.001);
        assert!((output.left[50] - output.right[50]).abs() < 0.001);
    }

    #[test]
    fn test_mix_stereo_equal_power_preservation() {
        // Equal power panning should preserve total power across all pan positions
        let sample_value = 1.0;
        let num_samples = 100;

        // Test multiple pan positions
        for pan in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            let mut mixer = Mixer::new(num_samples, 44100.0);
            mixer.add_panned(vec![sample_value; num_samples], 1.0, pan);

            let output = mixer.mix_stereo();

            // Calculate power (L^2 + R^2)
            let power = output.left[50].powi(2) + output.right[50].powi(2);

            // Power should be constant (approximately 1.0 for unit input)
            assert!(
                (power - 1.0).abs() < 0.01,
                "Power at pan={} is {}, expected ~1.0",
                pan,
                power
            );
        }
    }

    #[test]
    fn test_mix_stereo_multiple_layers() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![0.5; 100], 1.0, -1.0); // Hard left
        mixer.add_panned(vec![0.5; 100], 1.0, 1.0); // Hard right

        let output = mixer.mix_stereo();

        // Left channel gets the left-panned signal
        assert!((output.left[50] - 0.5).abs() < 0.001);
        // Right channel gets the right-panned signal
        assert!((output.right[50] - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mix_stereo_with_volume() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 0.5, -1.0); // Hard left with 50% volume

        let output = mixer.mix_stereo();

        assert!((output.left[50] - 0.5).abs() < 0.001);
        assert!(output.right[50].abs() < 0.001);
    }

    #[test]
    fn test_mix_stereo_with_delay() {
        let mut mixer = Mixer::new(100, 44100.0);
        let layer = Layer::new(vec![1.0; 20], 1.0, 0.5).with_delay(30);
        mixer.add_layer(layer);

        let output = mixer.mix_stereo();

        // Before delay: silent
        assert!(output.left[20].abs() < 0.001);
        assert!(output.right[20].abs() < 0.001);

        // After delay: has signal
        assert!(output.left[35].abs() > 0.1);
        assert!(output.right[35].abs() > 0.1);
    }

    // ============================================================================
    // MixerOutput Tests
    // ============================================================================

    #[test]
    fn test_mixer_output_mono() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 100], 1.0);

        let output = mixer.mix();
        assert!(!output.is_stereo());
        assert_eq!(output.len(), 100);
    }

    #[test]
    fn test_mixer_output_stereo() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![0.5; 100], 1.0, 0.5);

        let output = mixer.mix();
        assert!(output.is_stereo());
        assert_eq!(output.len(), 100);
    }

    #[test]
    fn test_mixer_output_to_mono_from_mono() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 100], 1.0);

        let output = mixer.mix();
        let mono = output.to_mono();
        assert_eq!(mono.len(), 100);
        assert!(mono.iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_mixer_output_to_mono_from_stereo() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, -1.0); // Hard left

        let output = mixer.mix();
        let mono = output.to_mono();

        // Hard left has L=1.0, R=0.0, so mono = (1.0 + 0.0) / 2 = 0.5
        assert!(mono.iter().all(|&s| (s - 0.5).abs() < 0.001));
    }

    #[test]
    fn test_mixer_output_to_stereo_from_mono() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![0.5; 100], 1.0);

        let output = mixer.mix();
        let stereo = output.to_stereo();

        assert_eq!(stereo.left.len(), 100);
        assert_eq!(stereo.right.len(), 100);
        assert_eq!(stereo.left, stereo.right);
    }

    #[test]
    fn test_mixer_output_to_stereo_from_stereo() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_panned(vec![1.0; 100], 1.0, 0.5);

        let output = mixer.mix();
        let stereo = output.to_stereo();

        assert_eq!(stereo.left.len(), 100);
        assert_eq!(stereo.right.len(), 100);
    }

    #[test]
    fn test_mixer_output_is_empty() {
        let mixer = Mixer::new(0, 44100.0);
        let output = mixer.mix();
        assert!(output.is_empty());
    }

    // ============================================================================
    // Normalization Tests
    // ============================================================================

    #[test]
    fn test_normalize_basic() {
        let mut samples = vec![0.5, -0.3, 0.8, -0.2];
        normalize(&mut samples, -3.0);

        // Peak should be at -3dB
        let target = 10.0_f64.powf(-3.0 / 20.0);
        let peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        assert!((peak - target).abs() < 0.01);
    }

    #[test]
    fn test_normalize_silent_audio() {
        let mut samples = vec![0.0, 0.0, 0.0, 0.0];
        normalize(&mut samples, -3.0);

        // Silent audio should remain silent (no division by zero)
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_normalize_loud_audio() {
        let mut samples = vec![2.0, -1.5, 3.0, -2.5];
        normalize(&mut samples, 0.0); // Normalize to 0dB (peak = 1.0)

        let peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        assert!((peak - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_quiet_audio() {
        let mut samples = vec![0.01, -0.005, 0.008, -0.003];
        normalize(&mut samples, 0.0); // Normalize to 0dB

        let peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
        assert!((peak - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_peak_db() {
        // Test various headroom values
        for headroom_db in [-6.0, -3.0, 0.0, -12.0] {
            let mut samples = vec![1.0, -0.5, 0.75, -0.25];
            normalize(&mut samples, headroom_db);

            let target = 10.0_f64.powf(headroom_db / 20.0);
            let peak = samples.iter().map(|s| s.abs()).fold(0.0_f64, |a, b| a.max(b));
            assert!(
                (peak - target).abs() < 0.001,
                "For {}dB, expected peak {}, got {}",
                headroom_db,
                target,
                peak
            );
        }
    }

    #[test]
    fn test_normalize_preserves_relative_amplitudes() {
        let mut samples = vec![1.0, 0.5, 0.25];
        normalize(&mut samples, 0.0);

        // Ratios should be preserved
        assert!((samples[1] / samples[0] - 0.5).abs() < 0.001);
        assert!((samples[2] / samples[0] - 0.25).abs() < 0.001);
    }

    #[test]
    fn test_normalize_stereo_basic() {
        let mut stereo = StereoOutput {
            left: vec![0.5, -0.3],
            right: vec![0.2, -0.1],
        };
        normalize_stereo(&mut stereo, 0.0);

        let peak = stereo
            .left
            .iter()
            .chain(stereo.right.iter())
            .map(|s| s.abs())
            .fold(0.0_f64, |a, b| a.max(b));
        assert!((peak - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_normalize_stereo_silent() {
        let mut stereo = StereoOutput {
            left: vec![0.0, 0.0],
            right: vec![0.0, 0.0],
        };
        normalize_stereo(&mut stereo, 0.0);

        // Silent audio should remain silent
        assert!(stereo.left.iter().all(|&s| s == 0.0));
        assert!(stereo.right.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_normalize_stereo_uses_global_peak() {
        let mut stereo = StereoOutput {
            left: vec![0.5],
            right: vec![1.0], // Right has higher peak
        };
        normalize_stereo(&mut stereo, 0.0);

        // Right should be at 1.0, left should be at 0.5
        assert!((stereo.right[0] - 1.0).abs() < 0.001);
        assert!((stereo.left[0] - 0.5).abs() < 0.001);
    }

    // ============================================================================
    // Delay Tests
    // ============================================================================

    #[test]
    fn test_layer_delay_samples() {
        let mut mixer = Mixer::new(100, 44100.0);
        let layer = Layer::centered(vec![1.0; 20], 1.0).with_delay(50);
        mixer.add_layer(layer);

        let output = mixer.mix_mono();

        // First 50 samples should be silent
        for i in 0..50 {
            assert!(
                output[i].abs() < 0.001,
                "Sample {} should be silent but is {}",
                i,
                output[i]
            );
        }
        // Samples 50-69 should have signal
        for i in 50..70 {
            assert!(
                (output[i] - 1.0).abs() < 0.001,
                "Sample {} should be 1.0 but is {}",
                i,
                output[i]
            );
        }
        // Samples 70-99 should be silent
        for i in 70..100 {
            assert!(
                output[i].abs() < 0.001,
                "Sample {} should be silent but is {}",
                i,
                output[i]
            );
        }
    }

    #[test]
    fn test_layer_delay_seconds() {
        let sample_rate = 44100.0;
        let mut mixer = Mixer::new(88200, sample_rate); // 2 seconds
        let layer =
            Layer::centered(vec![1.0; 4410], 1.0).with_delay_seconds(0.5, sample_rate); // 500ms delay
        mixer.add_layer(layer);

        let output = mixer.mix_mono();

        // First 22050 samples (0.5 seconds) should be silent
        assert!(output[22049].abs() < 0.001);
        // Sample 22050 onwards should have signal
        assert!((output[22050] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_layer_delay_truncates_at_output_boundary() {
        let mut mixer = Mixer::new(100, 44100.0);
        let layer = Layer::centered(vec![1.0; 50], 1.0).with_delay(80);
        mixer.add_layer(layer);

        let output = mixer.mix_mono();

        // Only 20 samples should fit (80 + 20 = 100)
        assert!(output[79].abs() < 0.001);
        assert!((output[80] - 1.0).abs() < 0.001);
        assert!((output[99] - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_layer_delay_beyond_output() {
        let mut mixer = Mixer::new(100, 44100.0);
        let layer = Layer::centered(vec![1.0; 50], 1.0).with_delay(150); // Beyond output length
        mixer.add_layer(layer);

        let output = mixer.mix_mono();

        // All samples should be silent
        assert!(output.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_multiple_layers_with_different_delays() {
        let mut mixer = Mixer::new(100, 44100.0);
        // Layer 1: samples 0-29 (delay 0, length 30)
        mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(0));
        // Layer 2: samples 20-49 (delay 20, length 30)
        mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(20));
        // Layer 3: samples 40-69 (delay 40, length 30)
        mixer.add_layer(Layer::centered(vec![0.3; 30], 1.0).with_delay(40));

        let output = mixer.mix_mono();

        // 0-19: one layer (layer 1 only) -> 0.3
        assert!((output[10] - 0.3).abs() < 0.001);
        // 20-29: two layers (layer 1 + layer 2) -> 0.6
        assert!((output[25] - 0.6).abs() < 0.001);
        // 30-39: one layer (layer 2 only, layer 1 ended, layer 3 not started) -> 0.3
        assert!((output[35] - 0.3).abs() < 0.001);
        // 40-49: two layers (layer 2 + layer 3) -> 0.6
        assert!((output[45] - 0.6).abs() < 0.001);
        // 50-69: one layer (layer 3 only) -> 0.3
        assert!((output[55] - 0.3).abs() < 0.001);
        // 70-99: silent (all layers ended)
        assert!((output[80] - 0.0).abs() < 0.001);
    }

    // ============================================================================
    // Soft Clipping Tests
    // ============================================================================

    #[test]
    fn test_soft_clip_below_threshold() {
        assert!((soft_clip(0.5, 0.8) - 0.5).abs() < 0.001);
        assert!((soft_clip(-0.5, 0.8) - (-0.5)).abs() < 0.001);
        assert!((soft_clip(0.0, 0.8) - 0.0).abs() < 0.001);
        assert!((soft_clip(0.79, 0.8) - 0.79).abs() < 0.001);
    }

    #[test]
    fn test_soft_clip_at_threshold() {
        let result = soft_clip(0.8, 0.8);
        assert!((result - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_soft_clip_above_threshold() {
        let clipped = soft_clip(2.0, 0.8);
        assert!(clipped < 2.0, "Should be compressed");
        assert!(clipped > 0.8, "Should be above threshold");
        assert!(clipped < 1.0, "Should approach but not exceed 1.0");
    }

    #[test]
    fn test_soft_clip_very_high_values() {
        let hard = soft_clip(10.0, 0.8);
        assert!(hard < 1.0);
        assert!(hard > 0.95); // Should be very close to 1.0
    }

    #[test]
    fn test_soft_clip_preserves_sign() {
        let positive = soft_clip(2.0, 0.8);
        let negative = soft_clip(-2.0, 0.8);
        assert!(positive > 0.0);
        assert!(negative < 0.0);
        assert!((positive + negative).abs() < 0.001); // Symmetric
    }

    #[test]
    fn test_soft_clip_buffer() {
        let mut samples = vec![0.5, 1.5, -0.3, 2.0, -1.0];
        soft_clip_buffer(&mut samples, 0.8);

        assert!((samples[0] - 0.5).abs() < 0.001); // Below threshold
        assert!(samples[1] > 0.8 && samples[1] < 1.0); // Compressed
        assert!((samples[2] - (-0.3)).abs() < 0.001); // Below threshold
        assert!(samples[3] > 0.8 && samples[3] < 1.0); // Compressed
        assert!(samples[4] > -1.0 && samples[4] < -0.8); // Compressed (negative)
    }

    // ============================================================================
    // Edge Cases and Integration Tests
    // ============================================================================

    #[test]
    fn test_empty_mixer() {
        let mixer = Mixer::new(0, 44100.0);
        let output = mixer.mix_mono();
        assert!(output.is_empty());
    }

    #[test]
    fn test_zero_volume_layer() {
        let mut mixer = Mixer::new(100, 44100.0);
        mixer.add_mono(vec![1.0; 100], 0.0);

        let output = mixer.mix_mono();
        assert!(output.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_very_small_pan() {
        let mut mixer = Mixer::new(100, 44100.0);
        // Pan value just above the stereo detection threshold
        mixer.add_panned(vec![1.0; 100], 1.0, 0.0000001);

        // Should not be detected as stereo (threshold is 1e-6)
        assert!(!mixer.is_stereo());
    }

    #[test]
    fn test_pan_at_stereo_threshold() {
        let mut mixer = Mixer::new(100, 44100.0);
        // Pan value just above the stereo detection threshold
        mixer.add_panned(vec![1.0; 100], 1.0, 0.00001);

        assert!(mixer.is_stereo());
    }

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
    fn test_stereo_interleave_original() {
        let stereo = StereoOutput {
            left: vec![1.0, 2.0, 3.0],
            right: vec![4.0, 5.0, 6.0],
        };

        let interleaved = stereo.interleave();
        assert_eq!(interleaved, vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0]);
    }
}
