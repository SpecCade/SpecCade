//! Noise synthesis module.
//!
//! Generates various colors of noise (white, pink, brown) with optional filtering.

use rand_pcg::Pcg32;
use speccade_spec::recipe::audio::FormantVowel;

use crate::filter::{BiquadCoeffs, BiquadFilter, CombFilter, FormantFilter, LadderFilter};
use crate::oscillator;

use super::Synthesizer;

/// Noise color/type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseColor {
    /// White noise - equal energy at all frequencies.
    White,
    /// Pink noise - 1/f spectrum, equal energy per octave.
    Pink,
    /// Brown/Brownian noise - 1/f^2 spectrum, very bass-heavy.
    Brown,
}

/// Filter type for noise shaping.
#[derive(Debug, Clone, Copy)]
pub enum NoiseFilter {
    /// No filter.
    None,
    /// Lowpass filter.
    Lowpass { cutoff: f64, resonance: f64 },
    /// Highpass filter.
    Highpass { cutoff: f64, resonance: f64 },
    /// Bandpass filter.
    Bandpass { center: f64, resonance: f64 },
    /// Notch (band-reject) filter.
    Notch { center: f64, resonance: f64 },
    /// Allpass filter (phase shifting).
    Allpass { frequency: f64, resonance: f64 },
    /// Comb filter (delay-based resonant filter).
    Comb {
        delay_ms: f64,
        feedback: f64,
        wet: f64,
    },
    /// Formant filter (vowel-shaping resonant filter bank).
    Formant { vowel: FormantVowel, intensity: f64 },
    /// Ladder filter (Moog-style 4-pole lowpass with resonance).
    Ladder { cutoff: f64, resonance: f64 },
    /// Low shelf filter (bass boost/cut).
    ShelfLow { frequency: f64, gain_db: f64 },
    /// High shelf filter (treble boost/cut).
    ShelfHigh { frequency: f64, gain_db: f64 },
}

/// Noise burst synthesizer.
#[derive(Debug, Clone)]
pub struct NoiseSynth {
    /// Type of noise.
    pub color: NoiseColor,
    /// Optional filter.
    pub filter: NoiseFilter,
}

impl NoiseSynth {
    /// Creates white noise synthesizer.
    pub fn white() -> Self {
        Self {
            color: NoiseColor::White,
            filter: NoiseFilter::None,
        }
    }

    /// Creates pink noise synthesizer.
    pub fn pink() -> Self {
        Self {
            color: NoiseColor::Pink,
            filter: NoiseFilter::None,
        }
    }

    /// Creates brown noise synthesizer.
    pub fn brown() -> Self {
        Self {
            color: NoiseColor::Brown,
            filter: NoiseFilter::None,
        }
    }

    /// Creates a new noise synthesizer with specified color.
    pub fn new(color: NoiseColor) -> Self {
        Self {
            color,
            filter: NoiseFilter::None,
        }
    }

    /// Adds a lowpass filter.
    pub fn with_lowpass(mut self, cutoff: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Lowpass { cutoff, resonance };
        self
    }

    /// Adds a highpass filter.
    pub fn with_highpass(mut self, cutoff: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Highpass { cutoff, resonance };
        self
    }

    /// Adds a bandpass filter.
    pub fn with_bandpass(mut self, center: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Bandpass { center, resonance };
        self
    }

    /// Adds a notch (band-reject) filter.
    pub fn with_notch(mut self, center: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Notch { center, resonance };
        self
    }

    /// Adds an allpass filter (phase shifting).
    pub fn with_allpass(mut self, frequency: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Allpass {
            frequency,
            resonance,
        };
        self
    }

    /// Adds a comb filter (delay-based resonant filter).
    pub fn with_comb(mut self, delay_ms: f64, feedback: f64, wet: f64) -> Self {
        self.filter = NoiseFilter::Comb {
            delay_ms,
            feedback,
            wet,
        };
        self
    }

    /// Adds a formant filter (vowel-shaping resonant filter bank).
    pub fn with_formant(mut self, vowel: FormantVowel, intensity: f64) -> Self {
        self.filter = NoiseFilter::Formant { vowel, intensity };
        self
    }

    /// Adds a ladder filter (Moog-style 4-pole lowpass with resonance).
    pub fn with_ladder(mut self, cutoff: f64, resonance: f64) -> Self {
        self.filter = NoiseFilter::Ladder { cutoff, resonance };
        self
    }

    /// Adds a low shelf filter (bass boost/cut).
    pub fn with_shelf_low(mut self, frequency: f64, gain_db: f64) -> Self {
        self.filter = NoiseFilter::ShelfLow { frequency, gain_db };
        self
    }

    /// Adds a high shelf filter (treble boost/cut).
    pub fn with_shelf_high(mut self, frequency: f64, gain_db: f64) -> Self {
        self.filter = NoiseFilter::ShelfHigh { frequency, gain_db };
        self
    }
}

impl Synthesizer for NoiseSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, rng: &mut Pcg32) -> Vec<f64> {
        // Generate raw noise
        let mut samples = match self.color {
            NoiseColor::White => oscillator::white_noise(rng, num_samples),
            NoiseColor::Pink => oscillator::pink_noise(rng, num_samples),
            NoiseColor::Brown => oscillator::brown_noise(rng, num_samples),
        };

        // Apply filter if specified
        match self.filter {
            NoiseFilter::None => {}
            NoiseFilter::Lowpass { cutoff, resonance } => {
                let mut filter = BiquadFilter::lowpass(cutoff, resonance, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Highpass { cutoff, resonance } => {
                let mut filter = BiquadFilter::highpass(cutoff, resonance, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Bandpass { center, resonance } => {
                let mut filter = BiquadFilter::bandpass(center, resonance, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Notch { center, resonance } => {
                let mut filter = BiquadFilter::notch(center, resonance, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Allpass {
                frequency,
                resonance,
            } => {
                let mut filter =
                    BiquadFilter::new(BiquadCoeffs::allpass(frequency, resonance, sample_rate));
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Comb {
                delay_ms,
                feedback,
                wet,
            } => {
                let mut filter = CombFilter::new(delay_ms, feedback, wet, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Formant { vowel, intensity } => {
                let mut filter = FormantFilter::new(vowel, intensity, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::Ladder { cutoff, resonance } => {
                let mut filter = LadderFilter::new(cutoff, resonance, sample_rate);
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::ShelfLow { frequency, gain_db } => {
                let mut filter =
                    BiquadFilter::new(BiquadCoeffs::low_shelf(frequency, gain_db, sample_rate));
                filter.process_buffer(&mut samples);
            }
            NoiseFilter::ShelfHigh { frequency, gain_db } => {
                let mut filter =
                    BiquadFilter::new(BiquadCoeffs::high_shelf(frequency, gain_db, sample_rate));
                filter.process_buffer(&mut samples);
            }
        }

        // Normalize to prevent clipping (especially for pink/brown noise)
        normalize_samples(&mut samples);

        samples
    }
}

/// Normalizes samples to peak at 1.0.
fn normalize_samples(samples: &mut [f64]) {
    let max = samples
        .iter()
        .map(|s| s.abs())
        .fold(0.0_f64, |a, b| a.max(b));

    if max > 0.0 {
        let scale = 1.0 / max;
        for s in samples.iter_mut() {
            *s *= scale;
        }
    }
}

/// Generates a noise burst with attack-release envelope.
///
/// # Arguments
/// * `color` - Noise color
/// * `num_samples` - Number of samples
/// * `sample_rate` - Audio sample rate
/// * `attack_samples` - Number of samples for attack
/// * `release_samples` - Number of samples for release
/// * `rng` - Deterministic RNG
///
/// # Returns
/// Enveloped noise samples
pub fn noise_burst(
    color: NoiseColor,
    num_samples: usize,
    sample_rate: f64,
    attack_samples: usize,
    release_samples: usize,
    rng: &mut Pcg32,
) -> Vec<f64> {
    let synth = NoiseSynth::new(color);
    let mut samples = synth.synthesize(num_samples, sample_rate, rng);

    // Apply simple AR envelope
    for (i, sample) in samples.iter_mut().enumerate() {
        let env = if i < attack_samples {
            // Attack phase
            i as f64 / attack_samples as f64
        } else if i >= num_samples - release_samples {
            // Release phase
            (num_samples - i) as f64 / release_samples as f64
        } else {
            // Sustain
            1.0
        };
        *sample *= env;
    }

    samples
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_white_noise() {
        let synth = NoiseSynth::white();
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Check normalized range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_pink_noise() {
        let synth = NoiseSynth::pink();
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_brown_noise() {
        let synth = NoiseSynth::brown();
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_filtered_noise() {
        let synth = NoiseSynth::white().with_lowpass(1000.0, 0.707);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
    }

    #[test]
    fn test_noise_determinism() {
        let synth = NoiseSynth::white();

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_noise_burst() {
        let mut rng = create_rng(42);
        let samples = noise_burst(NoiseColor::White, 1000, 44100.0, 100, 200, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Attack should start near zero
        assert!(samples[0].abs() < 0.1);
        // Release should end near zero
        assert!(samples[999].abs() < 0.1);
    }

    #[test]
    fn test_notch_filtered_noise() {
        let synth = NoiseSynth::white().with_notch(1000.0, 2.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_notch_filter_determinism() {
        let synth = NoiseSynth::white().with_notch(800.0, 1.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_allpass_filtered_noise() {
        let synth = NoiseSynth::white().with_allpass(1000.0, 2.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_allpass_filter_determinism() {
        let synth = NoiseSynth::white().with_allpass(800.0, 1.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_formant_filtered_noise() {
        let synth = NoiseSynth::white().with_formant(FormantVowel::A, 0.8);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_formant_filter_determinism() {
        let synth = NoiseSynth::white().with_formant(FormantVowel::I, 0.7);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_ladder_filtered_noise() {
        let synth = NoiseSynth::white().with_ladder(1000.0, 0.7);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_ladder_filter_determinism() {
        let synth = NoiseSynth::white().with_ladder(800.0, 0.5);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_shelf_low_filtered_noise() {
        let synth = NoiseSynth::white().with_shelf_low(200.0, 6.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_shelf_low_filter_determinism() {
        let synth = NoiseSynth::white().with_shelf_low(200.0, 6.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }

    #[test]
    fn test_shelf_high_filtered_noise() {
        let synth = NoiseSynth::white().with_shelf_high(4000.0, -3.0);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(1000, 44100.0, &mut rng);

        assert_eq!(samples.len(), 1000);
        // Verify normalized output range
        for &s in &samples {
            assert!((-1.0..=1.0).contains(&s));
        }
    }

    #[test]
    fn test_shelf_high_filter_determinism() {
        let synth = NoiseSynth::white().with_shelf_high(4000.0, -3.0);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(100, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(100, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2);
    }
}
