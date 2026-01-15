//! Feedback comb filter implementation.

/// Feedback comb filter.
///
/// A comb filter creates resonances at harmonics of 1/delay_time by feeding
/// the output back into a delay line. Creates metallic/resonant coloration.
#[derive(Debug, Clone)]
pub struct CombFilter {
    delay_line: Vec<f64>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f64,
    wet: f64,
}

impl CombFilter {
    /// Creates a new comb filter.
    ///
    /// # Arguments
    /// * `delay_ms` - Delay time in milliseconds
    /// * `feedback` - Feedback amount (clamped to 0.0..0.99 for stability)
    /// * `wet` - Wet/dry mix (clamped to 0.0..1.0)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(delay_ms: f64, feedback: f64, wet: f64, sample_rate: f64) -> Self {
        // Clamp feedback for stability (must be < 1.0 to prevent runaway)
        let feedback = feedback.clamp(0.0, 0.99);
        let wet = wet.clamp(0.0, 1.0);

        // Calculate delay in samples (minimum 1 sample)
        let delay_samples = ((delay_ms / 1000.0) * sample_rate).round() as usize;
        let delay_samples = delay_samples.max(1);

        Self {
            delay_line: vec![0.0; delay_samples],
            write_pos: 0,
            delay_samples,
            feedback,
            wet,
        }
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        self.delay_line.fill(0.0);
        self.write_pos = 0;
    }

    /// Processes a single sample through the comb filter.
    ///
    /// Implements feedback comb: output[n] = input[n] + feedback * delay_buffer[n - delay_samples]
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        // Read from delay line
        let delayed = self.delay_line[self.write_pos];

        // Compute output: input + feedback * delayed
        let output = input + self.feedback * delayed;

        // Write output to delay line
        self.delay_line[self.write_pos] = output;

        // Advance write position (circular buffer)
        self.write_pos = (self.write_pos + 1) % self.delay_samples;

        // Apply wet/dry mix
        input * (1.0 - self.wet) + output * self.wet
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
    fn test_comb_filter() {
        // 10ms delay at 44100 Hz = 441 samples
        let mut filter = CombFilter::new(10.0, 0.5, 1.0, 44100.0);

        // Process an impulse
        let mut output = Vec::new();
        output.push(filter.process(1.0));
        for _ in 1..1000 {
            output.push(filter.process(0.0));
        }

        // First sample should be the impulse
        assert!((output[0] - 1.0).abs() < 0.01);

        // After ~441 samples, we should see the first echo at feedback level
        // delay_samples = round(10.0 / 1000.0 * 44100.0) = 441
        assert!(output[441].abs() > 0.4);

        // Echo should decay over time
        assert!(output[882].abs() < output[441].abs());
    }

    #[test]
    fn test_comb_filter_feedback_clamp() {
        // Test that feedback > 0.99 gets clamped
        let mut filter = CombFilter::new(10.0, 1.5, 1.0, 44100.0);

        // Process an impulse and verify it doesn't explode
        let mut output = Vec::new();
        output.push(filter.process(1.0));
        for _ in 1..10000 {
            output.push(filter.process(0.0));
        }

        // Should not explode (all values should be finite and bounded)
        for &s in &output {
            assert!(s.is_finite());
            assert!(s.abs() < 100.0);
        }
    }

    #[test]
    fn test_comb_filter_wet_dry() {
        // Test wet/dry mix
        let mut filter_dry = CombFilter::new(10.0, 0.5, 0.0, 44100.0);
        let mut filter_wet = CombFilter::new(10.0, 0.5, 1.0, 44100.0);

        // Dry should pass input unchanged
        let dry_out = filter_dry.process(1.0);
        assert!((dry_out - 1.0).abs() < 0.01);

        // Wet should include comb processing
        let wet_out = filter_wet.process(1.0);
        assert!((wet_out - 1.0).abs() < 0.01); // First sample is still ~1.0
    }

    #[test]
    fn test_comb_filter_determinism() {
        let mut filter1 = CombFilter::new(5.0, 0.7, 0.8, 44100.0);
        let mut filter2 = CombFilter::new(5.0, 0.7, 0.8, 44100.0);

        // Process same input
        let mut out1 = Vec::new();
        let mut out2 = Vec::new();
        for i in 0..100 {
            let input = (i as f64 * 0.1).sin();
            out1.push(filter1.process(input));
            out2.push(filter2.process(input));
        }

        // Should be identical
        assert_eq!(out1, out2);
    }
}
