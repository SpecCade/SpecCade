//! Biquad filter implementations.
//!
//! This module provides lowpass, highpass, and bandpass filters using
//! the standard biquad filter topology. Coefficients are calculated
//! using the Audio EQ Cookbook formulas.

use std::f64::consts::PI;

/// Biquad filter coefficients.
#[derive(Debug, Clone, Copy)]
pub struct BiquadCoeffs {
    pub b0: f64,
    pub b1: f64,
    pub b2: f64,
    pub a1: f64,
    pub a2: f64,
}

impl BiquadCoeffs {
    /// Creates lowpass filter coefficients.
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `q` - Q factor (resonance), typical values 0.5-10, 0.707 is Butterworth
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn lowpass(cutoff: f64, q: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = (1.0 - cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates highpass filter coefficients.
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `q` - Q factor (resonance)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn highpass(cutoff: f64, q: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let omega = 2.0 * PI * cutoff / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 + cos_omega) / 2.0;
        let b1 = -(1.0 + cos_omega);
        let b2 = (1.0 + cos_omega) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates bandpass filter coefficients (constant skirt gain).
    ///
    /// # Arguments
    /// * `center` - Center frequency in Hz
    /// * `q` - Q factor (bandwidth = center / Q)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn bandpass(center: f64, q: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let omega = 2.0 * PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = alpha;
        let b1 = 0.0;
        let b2 = -alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates a notch (band-reject) filter.
    ///
    /// # Arguments
    /// * `center` - Center frequency in Hz
    /// * `q` - Q factor
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn notch(center: f64, q: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let omega = 2.0 * PI * center / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates an allpass filter.
    ///
    /// # Arguments
    /// * `frequency` - Center frequency in Hz
    /// * `q` - Q factor
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn allpass(frequency: f64, q: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let omega = 2.0 * PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 - alpha;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 + alpha;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates a peak EQ filter.
    ///
    /// # Arguments
    /// * `frequency` - Center frequency in Hz
    /// * `q` - Q factor
    /// * `db_gain` - Gain in dB (positive for boost, negative for cut)
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn peaking_eq(frequency: f64, q: f64, db_gain: f64, sample_rate: f64) -> Self {
        // Clamp Q to minimum safe value to prevent division by zero
        let q = q.max(0.5);
        let a = 10.0_f64.powf(db_gain / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_omega;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates a low-shelf filter.
    ///
    /// # Arguments
    /// * `frequency` - Shelf frequency in Hz
    /// * `db_gain` - Gain in dB
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn low_shelf(frequency: f64, db_gain: f64, sample_rate: f64) -> Self {
        let a = 10.0_f64.powf(db_gain / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.9 - 1.0) + 2.0).sqrt();

        let b0 = a * ((a + 1.0) - (a - 1.0) * cos_omega + 2.0 * a.sqrt() * alpha);
        let b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) - (a - 1.0) * cos_omega - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) + (a - 1.0) * cos_omega + 2.0 * a.sqrt() * alpha;
        let a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) + (a - 1.0) * cos_omega - 2.0 * a.sqrt() * alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }

    /// Creates a high-shelf filter.
    ///
    /// # Arguments
    /// * `frequency` - Shelf frequency in Hz
    /// * `db_gain` - Gain in dB
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn high_shelf(frequency: f64, db_gain: f64, sample_rate: f64) -> Self {
        let a = 10.0_f64.powf(db_gain / 40.0);
        let omega = 2.0 * PI * frequency / sample_rate;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / 2.0 * ((a + 1.0 / a) * (1.0 / 0.9 - 1.0) + 2.0).sqrt();

        let b0 = a * ((a + 1.0) + (a - 1.0) * cos_omega + 2.0 * a.sqrt() * alpha);
        let b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cos_omega);
        let b2 = a * ((a + 1.0) + (a - 1.0) * cos_omega - 2.0 * a.sqrt() * alpha);
        let a0 = (a + 1.0) - (a - 1.0) * cos_omega + 2.0 * a.sqrt() * alpha;
        let a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cos_omega);
        let a2 = (a + 1.0) - (a - 1.0) * cos_omega - 2.0 * a.sqrt() * alpha;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
        }
    }
}

/// Biquad filter state.
#[derive(Debug, Clone)]
pub struct BiquadFilter {
    coeffs: BiquadCoeffs,
    // Delay line for input samples
    x1: f64,
    x2: f64,
    // Delay line for output samples
    y1: f64,
    y2: f64,
}

impl BiquadFilter {
    /// Creates a new biquad filter with the given coefficients.
    pub fn new(coeffs: BiquadCoeffs) -> Self {
        Self {
            coeffs,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Creates a lowpass filter.
    pub fn lowpass(cutoff: f64, q: f64, sample_rate: f64) -> Self {
        Self::new(BiquadCoeffs::lowpass(cutoff, q, sample_rate))
    }

    /// Creates a highpass filter.
    pub fn highpass(cutoff: f64, q: f64, sample_rate: f64) -> Self {
        Self::new(BiquadCoeffs::highpass(cutoff, q, sample_rate))
    }

    /// Creates a bandpass filter.
    pub fn bandpass(center: f64, q: f64, sample_rate: f64) -> Self {
        Self::new(BiquadCoeffs::bandpass(center, q, sample_rate))
    }

    /// Updates the filter coefficients.
    pub fn set_coeffs(&mut self, coeffs: BiquadCoeffs) {
        self.coeffs = coeffs;
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }

    /// Processes a single sample through the filter.
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        let output = self.coeffs.b0 * input + self.coeffs.b1 * self.x1 + self.coeffs.b2 * self.x2
            - self.coeffs.a1 * self.y1
            - self.coeffs.a2 * self.y2;

        // Update delay lines
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;

        output
    }

    /// Processes a buffer of samples in place.
    pub fn process_buffer(&mut self, buffer: &mut [f64]) {
        for sample in buffer.iter_mut() {
            *sample = self.process(*sample);
        }
    }

    /// Processes a buffer of samples, returning a new buffer.
    pub fn process_buffer_copy(&mut self, input: &[f64]) -> Vec<f64> {
        input.iter().map(|&s| self.process(s)).collect()
    }
}

/// One-pole lowpass filter (simple RC filter).
///
/// This is a computationally cheap filter useful for smoothing parameters
/// or simple filtering applications.
#[derive(Debug, Clone)]
pub struct OnePoleFilter {
    a0: f64,
    b1: f64,
    y1: f64,
}

impl OnePoleFilter {
    /// Creates a new one-pole lowpass filter.
    ///
    /// # Arguments
    /// * `cutoff` - Cutoff frequency in Hz
    /// * `sample_rate` - Audio sample rate in Hz
    pub fn new(cutoff: f64, sample_rate: f64) -> Self {
        let b1 = (-2.0 * PI * cutoff / sample_rate).exp();
        Self {
            a0: 1.0 - b1,
            b1,
            y1: 0.0,
        }
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        self.y1 = 0.0;
    }

    /// Processes a single sample.
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        self.y1 = self.a0 * input + self.b1 * self.y1;
        self.y1
    }
}

/// DC blocker filter.
///
/// Removes DC offset from a signal using a highpass filter with very low cutoff.
#[derive(Debug, Clone)]
pub struct DcBlocker {
    x1: f64,
    y1: f64,
    r: f64,
}

impl DcBlocker {
    /// Creates a new DC blocker.
    ///
    /// # Arguments
    /// * `r` - Pole radius, typically 0.995 to 0.999. Higher values = lower cutoff.
    pub fn new(r: f64) -> Self {
        Self {
            x1: 0.0,
            y1: 0.0,
            r,
        }
    }

    /// Creates a DC blocker with a default pole radius.
    pub fn default_r() -> Self {
        Self::new(0.995)
    }

    /// Resets the filter state.
    pub fn reset(&mut self) {
        self.x1 = 0.0;
        self.y1 = 0.0;
    }

    /// Processes a single sample.
    #[inline]
    pub fn process(&mut self, input: f64) -> f64 {
        let output = input - self.x1 + self.r * self.y1;
        self.x1 = input;
        self.y1 = output;
        output
    }
}

/// Filter sweep mode for time-varying cutoff.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SweepMode {
    /// Linear interpolation between start and end cutoff.
    Linear,
    /// Exponential interpolation (perceptually linear).
    Exponential,
}

/// Generates a filter cutoff sweep.
///
/// # Arguments
/// * `start_cutoff` - Starting cutoff frequency in Hz
/// * `end_cutoff` - Ending cutoff frequency in Hz
/// * `num_samples` - Number of samples in the sweep
/// * `mode` - Interpolation mode
///
/// # Returns
/// Vector of cutoff frequencies
pub fn generate_cutoff_sweep(
    start_cutoff: f64,
    end_cutoff: f64,
    num_samples: usize,
    mode: SweepMode,
) -> Vec<f64> {
    let mut cutoffs = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f64 / num_samples as f64;
        let cutoff = match mode {
            SweepMode::Linear => start_cutoff + (end_cutoff - start_cutoff) * t,
            SweepMode::Exponential => start_cutoff * (end_cutoff / start_cutoff).powf(t),
        };
        cutoffs.push(cutoff);
    }

    cutoffs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lowpass_filter() {
        let mut filter = BiquadFilter::lowpass(1000.0, 0.707, 44100.0);

        // Process some samples
        let mut output = Vec::new();
        for _ in 0..100 {
            output.push(filter.process(1.0));
        }

        // Should converge towards 1.0 for DC input (lowpass passes DC)
        assert!((output[99] - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_highpass_filter() {
        let mut filter = BiquadFilter::highpass(1000.0, 0.707, 44100.0);

        // Process DC input (constant 1.0)
        let mut output = Vec::new();
        for _ in 0..1000 {
            output.push(filter.process(1.0));
        }

        // Should converge towards 0.0 for DC input (highpass blocks DC)
        assert!(output[999].abs() < 0.1);
    }

    #[test]
    fn test_dc_blocker() {
        let mut blocker = DcBlocker::default_r();

        // Add DC offset to a signal
        let mut output = Vec::new();
        for _ in 0..10000 {
            output.push(blocker.process(0.5)); // 0.5 DC offset
        }

        // After settling, should approach 0
        assert!(output[9999].abs() < 0.01);
    }

    #[test]
    fn test_cutoff_sweep_linear() {
        let sweep = generate_cutoff_sweep(100.0, 1000.0, 10, SweepMode::Linear);

        assert_eq!(sweep.len(), 10);
        assert!((sweep[0] - 100.0).abs() < 0.01);
        assert!((sweep[4] - 460.0).abs() < 1.0); // Midpoint should be ~460
    }

    #[test]
    fn test_cutoff_sweep_exponential() {
        let sweep = generate_cutoff_sweep(100.0, 1000.0, 10, SweepMode::Exponential);

        assert_eq!(sweep.len(), 10);
        assert!((sweep[0] - 100.0).abs() < 0.01);
        // Exponential midpoint should be geometric mean: sqrt(100 * 1000) = ~316
        assert!((sweep[5] - 316.0).abs() < 10.0);
    }

    #[test]
    fn test_one_pole_filter() {
        let mut filter = OnePoleFilter::new(100.0, 44100.0);

        // Process step input
        let mut output = Vec::new();
        for _ in 0..1000 {
            output.push(filter.process(1.0));
        }

        // Should approach 1.0 (passes DC)
        assert!((output[999] - 1.0).abs() < 0.01);
    }
}
