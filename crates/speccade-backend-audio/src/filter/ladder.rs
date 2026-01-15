//! Moog-style ladder filter implementation.
//!
//! A 4-pole (24 dB/octave) lowpass filter with resonance feedback.
//! Uses tanh saturation for stability at high resonance.

use std::f64::consts::PI;

/// Moog-style ladder filter.
///
/// Implements a classic 4-pole lowpass filter using cascaded one-pole stages
/// with resonance feedback. Uses tanh saturation to prevent instability at
/// high resonance values.
#[derive(Debug, Clone)]
pub struct LadderFilter {
    /// Filter coefficient (calculated from cutoff and sample rate).
    g: f64,
    /// Resonance amount (0.0-4.0 internal, from 0.0-1.0 input).
    resonance: f64,
    /// Sample rate in Hz.
    sample_rate: f64,
    /// Stage 1 state.
    stage1: f64,
    /// Stage 2 state.
    stage2: f64,
    /// Stage 3 state.
    stage3: f64,
    /// Stage 4 state (output).
    stage4: f64,
}

impl LadderFilter {
    /// Creates a new ladder filter.
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `resonance` - Resonance amount (0.0-1.0, clamped)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(cutoff: f64, resonance: f64, sample_rate: f64) -> Self {
        let resonance = resonance.clamp(0.0, 1.0);
        let g = Self::calculate_g(cutoff, sample_rate);

        Self {
            g,
            resonance: resonance * 4.0, // Map 0-1 to 0-4 for internal feedback
            sample_rate,
            stage1: 0.0,
            stage2: 0.0,
            stage3: 0.0,
            stage4: 0.0,
        }
    }

    /// Calculates the filter coefficient from cutoff frequency.
    fn calculate_g(cutoff: f64, sample_rate: f64) -> f64 {
        // g = 2 * PI * cutoff / sample_rate, clamped to [0, 0.5] for stability
        let g = 2.0 * PI * cutoff / sample_rate;
        g.clamp(0.0, 0.5)
    }

    /// Updates the cutoff frequency.
    pub fn set_cutoff(&mut self, cutoff: f64) {
        self.g = Self::calculate_g(cutoff, self.sample_rate);
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        self.stage1 = 0.0;
        self.stage2 = 0.0;
        self.stage3 = 0.0;
        self.stage4 = 0.0;
    }

    /// Processes a single sample through the ladder filter.
    ///
    /// Algorithm:
    /// 1. Apply resonance feedback from output to input
    /// 2. Use tanh saturation to prevent instability
    /// 3. Cascade through 4 one-pole lowpass stages
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        // Apply resonance feedback with tanh saturation for stability
        let feedback = self.resonance * self.stage4;
        let input_with_fb = (input - feedback).tanh();

        // Cascade through 4 one-pole lowpass stages
        self.stage1 += self.g * (input_with_fb - self.stage1);
        self.stage2 += self.g * (self.stage1 - self.stage2);
        self.stage3 += self.g * (self.stage2 - self.stage3);
        self.stage4 += self.g * (self.stage3 - self.stage4);

        self.stage4
    }

    /// Processes a buffer of samples in place.
    pub fn process_buffer(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ladder_filter_basic() {
        let mut filter = LadderFilter::new(1000.0, 0.0, 44100.0);

        // Process DC input - should converge toward tanh(1.0) ~ 0.76 due to saturation
        let mut output = Vec::new();
        for _ in 0..1000 {
            output.push(filter.process(1.0));
        }

        // Ladder filter with tanh saturation converges to tanh(1.0) for DC input
        // tanh(1.0) ~ 0.7616
        let expected = 1.0_f64.tanh();
        assert!(
            (output[999] - expected).abs() < 0.1,
            "Expected ~{}, got {}",
            expected,
            output[999]
        );
    }

    #[test]
    fn test_ladder_filter_resonance() {
        let mut filter_low_res = LadderFilter::new(1000.0, 0.1, 44100.0);
        let mut filter_high_res = LadderFilter::new(1000.0, 0.9, 44100.0);

        // Process an impulse
        let mut out_low = Vec::new();
        let mut out_high = Vec::new();

        out_low.push(filter_low_res.process(1.0));
        out_high.push(filter_high_res.process(1.0));

        for _ in 1..500 {
            out_low.push(filter_low_res.process(0.0));
            out_high.push(filter_high_res.process(0.0));
        }

        // High resonance should produce more sustained output (ringing)
        // Check that high res has more energy after initial transient
        let late_energy_low: f64 = out_low[200..400].iter().map(|x| x.abs()).sum();
        let late_energy_high: f64 = out_high[200..400].iter().map(|x| x.abs()).sum();

        assert!(
            late_energy_high > late_energy_low,
            "High resonance should produce more ringing"
        );
    }

    #[test]
    fn test_ladder_filter_stability() {
        // Test that filter remains stable at maximum resonance
        let mut filter = LadderFilter::new(2000.0, 1.0, 44100.0);

        let mut output = Vec::new();
        output.push(filter.process(1.0));
        for _ in 1..10000 {
            output.push(filter.process(0.0));
        }

        // All values should be finite and bounded
        for &s in &output {
            assert!(s.is_finite(), "Output should be finite");
            assert!(s.abs() < 10.0, "Output should be bounded");
        }
    }

    #[test]
    fn test_ladder_filter_determinism() {
        let mut filter1 = LadderFilter::new(800.0, 0.7, 44100.0);
        let mut filter2 = LadderFilter::new(800.0, 0.7, 44100.0);

        let mut out1 = Vec::new();
        let mut out2 = Vec::new();

        for i in 0..100 {
            let input = (i as f64 * 0.1).sin();
            out1.push(filter1.process(input));
            out2.push(filter2.process(input));
        }

        assert_eq!(out1, out2, "Filter output should be deterministic");
    }

    #[test]
    fn test_ladder_filter_cutoff_update() {
        let mut filter = LadderFilter::new(1000.0, 0.5, 44100.0);

        // Process some samples
        for _ in 0..100 {
            filter.process(1.0);
        }

        // Update cutoff
        filter.set_cutoff(2000.0);

        // Should still work correctly
        let output = filter.process(1.0);
        assert!(output.is_finite());
    }

    #[test]
    fn test_ladder_filter_resonance_clamp() {
        // Test that resonance > 1.0 is clamped
        let mut filter = LadderFilter::new(1000.0, 2.0, 44100.0);

        let mut output = Vec::new();
        output.push(filter.process(1.0));
        for _ in 1..1000 {
            output.push(filter.process(0.0));
        }

        // Should be stable (clamped to 1.0)
        for &s in &output {
            assert!(s.is_finite());
            assert!(s.abs() < 10.0);
        }
    }

    #[test]
    fn test_ladder_filter_cutoff_clamp() {
        // Test cutoff clamping for stability
        let mut filter = LadderFilter::new(30000.0, 0.5, 44100.0); // Above Nyquist

        let mut output = Vec::new();
        output.push(filter.process(1.0));
        for _ in 1..100 {
            output.push(filter.process(0.0));
        }

        // Should still be stable
        for &s in &output {
            assert!(s.is_finite());
        }
    }
}
