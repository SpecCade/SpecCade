//! Reusable delay line buffer for delay-based effects.

/// A ring buffer for implementing delay lines with interpolation support.
#[derive(Debug, Clone)]
pub struct DelayLine {
    buffer: Vec<f64>,
    write_pos: usize,
}

impl DelayLine {
    /// Creates a new delay line with the given maximum size in samples.
    pub fn new(max_samples: usize) -> Self {
        Self {
            buffer: vec![0.0; max_samples.max(4)],
            write_pos: 0,
        }
    }

    /// Writes a sample to the delay line and advances the write position.
    pub fn write(&mut self, sample: f64) {
        self.buffer[self.write_pos] = sample;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
    }

    /// Reads a sample at the given integer delay in samples.
    pub fn read(&self, delay_samples: usize) -> f64 {
        let read_pos = (self.write_pos + self.buffer.len() - delay_samples) % self.buffer.len();
        self.buffer[read_pos]
    }

    /// Reads a sample at a fractional delay using linear interpolation.
    pub fn read_interpolated(&self, delay_samples: f64) -> f64 {
        let delay_int = delay_samples.floor() as usize;
        let delay_frac = delay_samples - delay_int as f64;

        let sample1 = self.read(delay_int);
        let sample2 = self.read(delay_int + 1);

        // Linear interpolation
        sample1 * (1.0 - delay_frac) + sample2 * delay_frac
    }

    /// Reads from the delay line at the given delay and writes a new sample,
    /// returning the delayed sample.
    pub fn read_and_write(&mut self, input: f64, delay_samples: f64) -> f64 {
        let delayed = self.read_interpolated(delay_samples);
        self.write(input);
        delayed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_line_basic() {
        let mut dl = DelayLine::new(10);

        // Write some samples
        for i in 0..5 {
            dl.write(i as f64);
        }

        // Read with 1 sample delay - should get the second-to-last written sample
        assert!((dl.read(1) - 4.0).abs() < 1e-10);

        // Read with 5 sample delay - should get the first written sample
        assert!((dl.read(5) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_delay_line_interpolation() {
        let mut dl = DelayLine::new(10);

        // Write known values
        dl.write(0.0);
        dl.write(1.0);
        dl.write(2.0);

        // Read at fractional delay - should interpolate
        let val = dl.read_interpolated(1.5);
        // At delay 1.5: interpolate between delay=1 (2.0) and delay=2 (1.0)
        // = 2.0 * 0.5 + 1.0 * 0.5 = 1.5
        assert!((val - 1.5).abs() < 1e-10);
    }

    #[test]
    fn test_delay_line_read_and_write() {
        let mut dl = DelayLine::new(10);

        // Pre-fill with zeros (already done by new)
        // Write 1.0 and read with 3 sample delay
        let delayed = dl.read_and_write(1.0, 3.0);
        assert!((delayed - 0.0).abs() < 1e-10);

        // Continue writing
        dl.write(2.0);
        dl.write(3.0);

        // Now read with 3 sample delay - should get 1.0
        let delayed = dl.read_and_write(4.0, 3.0);
        assert!((delayed - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_delay_line_wrap_around() {
        let mut dl = DelayLine::new(4);

        // Write more samples than buffer size to test wrap
        for i in 0..10 {
            dl.write(i as f64);
        }

        // The last 4 samples written are 6, 7, 8, 9
        // Delay of 1 should give 9
        assert!((dl.read(1) - 9.0).abs() < 1e-10);

        // Delay of 4 should give 6
        assert!((dl.read(4) - 6.0).abs() < 1e-10);
    }
}
