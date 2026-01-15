//! Core types for audio mixing.

/// A single audio layer with mixing parameters.
#[derive(Debug, Clone)]
pub struct Layer {
    /// Audio samples.
    pub samples: Vec<f64>,
    /// Volume level (0.0 to 1.0).
    pub volume: f64,
    /// Stereo pan (-1.0 = left, 0.0 = center, 1.0 = right).
    pub pan: f64,
    /// Optional per-sample pan curve (-1.0 to 1.0).
    ///
    /// When present, this overrides `pan` for stereo mixing.
    pub pan_curve: Option<Vec<f64>>,
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
            pan_curve: None,
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

    /// Sets a per-sample pan curve for the layer.
    pub fn with_pan_curve(mut self, pan_curve: Vec<f64>) -> Self {
        self.pan_curve = Some(pan_curve);
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
