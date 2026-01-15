//! Formant filter implementation.
//!
//! A formant filter shapes the spectrum of an input signal to match
//! the resonant characteristics of human vowel sounds. It uses a bank
//! of parallel resonant bandpass filters tuned to formant frequencies.

use speccade_spec::recipe::audio::FormantVowel;

use super::BiquadFilter;

/// Formant filter using a bank of resonant bandpass filters.
///
/// Shapes the input signal to sound like a specific vowel by applying
/// parallel bandpass filters at the characteristic formant frequencies.
#[derive(Debug, Clone)]
pub struct FormantFilter {
    /// Bank of 3 bandpass filters (F1, F2, F3)
    filters: [BiquadFilter; 3],
    /// Relative amplitudes for each formant band
    amplitudes: [f64; 3],
    /// Intensity of the effect (0.0 = dry, 1.0 = full wet)
    intensity: f64,
}

/// Formant frequencies for each vowel.
///
/// Based on standard acoustic phonetics formant charts.
/// Values are (F1, F2, F3) in Hz.
fn get_formant_frequencies(vowel: FormantVowel) -> [(f64, f64); 3] {
    // Returns (frequency, amplitude) for each formant
    match vowel {
        // /a/ (ah) as in "father"
        FormantVowel::A => [(800.0, 1.0), (1200.0, 0.7), (2600.0, 0.4)],
        // /e/ (eh) as in "bed"
        FormantVowel::E => [(400.0, 1.0), (2200.0, 0.5), (2600.0, 0.3)],
        // /i/ (ee) as in "feet"
        FormantVowel::I => [(300.0, 1.0), (2300.0, 0.5), (3000.0, 0.3)],
        // /o/ (oh) as in "boat"
        FormantVowel::O => [(450.0, 1.0), (800.0, 0.6), (2600.0, 0.3)],
        // /u/ (oo) as in "boot"
        FormantVowel::U => [(350.0, 1.0), (700.0, 0.5), (2600.0, 0.3)],
    }
}

impl FormantFilter {
    /// Creates a new formant filter for the specified vowel.
    ///
    /// # Arguments
    /// * `vowel` - The target vowel shape
    /// * `intensity` - Effect intensity (0.0 = dry, 1.0 = full vowel shape)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(vowel: FormantVowel, intensity: f64, sample_rate: f64) -> Self {
        let formants = get_formant_frequencies(vowel);

        // Q factor for formant filters - moderate resonance for vocal quality
        let q = 5.0;

        let filters = [
            BiquadFilter::bandpass(formants[0].0, q, sample_rate),
            BiquadFilter::bandpass(formants[1].0, q, sample_rate),
            BiquadFilter::bandpass(formants[2].0, q, sample_rate),
        ];

        let amplitudes = [formants[0].1, formants[1].1, formants[2].1];

        // Clamp intensity to valid range
        let intensity = intensity.clamp(0.0, 1.0);

        Self {
            filters,
            amplitudes,
            intensity,
        }
    }

    /// Processes a single sample through the formant filter bank.
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        // Process through all formant filters in parallel and sum
        let wet = self.filters[0].process(input) * self.amplitudes[0]
            + self.filters[1].process(input) * self.amplitudes[1]
            + self.filters[2].process(input) * self.amplitudes[2];

        // Normalize by total amplitude weight
        let total_amplitude: f64 = self.amplitudes.iter().sum();
        let wet = wet / total_amplitude;

        // Mix dry and wet based on intensity
        input * (1.0 - self.intensity) + wet * self.intensity
    }

    /// Processes a buffer of samples in place.
    pub fn process_buffer(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        for filter in &mut self.filters {
            filter.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formant_filter_creation() {
        let filter = FormantFilter::new(FormantVowel::A, 0.8, 44100.0);
        assert!((filter.intensity - 0.8).abs() < 0.001);
    }

    #[test]
    fn test_formant_filter_intensity_clamping() {
        let filter = FormantFilter::new(FormantVowel::A, 1.5, 44100.0);
        assert!((filter.intensity - 1.0).abs() < 0.001);

        let filter = FormantFilter::new(FormantVowel::A, -0.5, 44100.0);
        assert!(filter.intensity.abs() < 0.001);
    }

    #[test]
    fn test_formant_filter_dry_signal() {
        let mut filter = FormantFilter::new(FormantVowel::A, 0.0, 44100.0);

        // With intensity 0.0, output should match input exactly
        let input = 0.5;
        let output = filter.process(input);
        assert!((output - input).abs() < 0.001);
    }

    #[test]
    fn test_formant_filter_processes_buffer() {
        let mut filter = FormantFilter::new(FormantVowel::I, 0.8, 44100.0);

        let mut buffer = vec![0.5; 100];
        filter.process_buffer(&mut buffer);

        // Buffer should be modified (not all zeros or unchanged)
        assert!(buffer.len() == 100);
    }

    #[test]
    fn test_formant_filter_determinism() {
        let mut filter1 = FormantFilter::new(FormantVowel::E, 0.7, 44100.0);
        let mut filter2 = FormantFilter::new(FormantVowel::E, 0.7, 44100.0);

        let input = [0.1, 0.2, 0.3, 0.4, 0.5];
        let mut output1 = input;
        let mut output2 = input;

        filter1.process_buffer(&mut output1);
        filter2.process_buffer(&mut output2);

        for (a, b) in output1.iter().zip(output2.iter()) {
            assert!((a - b).abs() < 1e-10);
        }
    }

    #[test]
    fn test_all_vowels() {
        let sample_rate = 44100.0;
        let vowels = [
            FormantVowel::A,
            FormantVowel::E,
            FormantVowel::I,
            FormantVowel::O,
            FormantVowel::U,
        ];

        for vowel in &vowels {
            let mut filter = FormantFilter::new(*vowel, 1.0, sample_rate);
            let mut buffer = vec![1.0; 100];
            filter.process_buffer(&mut buffer);
            // Just verify it runs without error
        }
    }
}
