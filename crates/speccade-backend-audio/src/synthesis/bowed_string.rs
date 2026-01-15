//! Bowed string synthesis for violin/cello-like sounds.
//!
//! Implements a physical model of a bowed string using a bidirectional
//! delay-line waveguide with continuous bow excitation via stick-slip friction.
//! Key differences from plucked strings (Karplus-Strong):
//! - Continuous excitation during the entire duration (not just initial)
//! - Bow position affects where excitation enters the delay lines
//! - Bow pressure controls the stick-slip friction nonlinearity
//! - Two delay lines traveling in opposite directions (bidirectional waveguide)

use rand_pcg::Pcg32;

use super::Synthesizer;

/// Bowed string synthesis parameters.
#[derive(Debug, Clone)]
pub struct BowedStringSynth {
    /// Base frequency in Hz.
    pub frequency: f64,
    /// Bow pressure / force on string (0.0-1.0).
    pub bow_pressure: f64,
    /// Bow position along string (0.0 = bridge, 1.0 = nut).
    pub bow_position: f64,
    /// String damping / high-frequency absorption (0.0-1.0).
    pub damping: f64,
}

impl BowedStringSynth {
    /// Creates a new bowed string synthesizer.
    ///
    /// # Arguments
    /// * `frequency` - Base frequency in Hz
    /// * `bow_pressure` - Bow pressure (0.0-1.0)
    /// * `bow_position` - Bow position (0.0 = bridge, 1.0 = nut)
    /// * `damping` - String damping (0.0-1.0)
    pub fn new(frequency: f64, bow_pressure: f64, bow_position: f64, damping: f64) -> Self {
        Self {
            frequency,
            bow_pressure: bow_pressure.clamp(0.0, 1.0),
            bow_position: bow_position.clamp(0.01, 0.99),
            damping: damping.clamp(0.0, 1.0),
        }
    }

    /// Creates a violin-like preset.
    pub fn violin(frequency: f64) -> Self {
        Self::new(frequency, 0.5, 0.12, 0.3)
    }

    /// Creates a viola-like preset.
    pub fn viola(frequency: f64) -> Self {
        Self::new(frequency, 0.55, 0.15, 0.35)
    }

    /// Creates a cello-like preset.
    pub fn cello(frequency: f64) -> Self {
        Self::new(frequency, 0.6, 0.18, 0.4)
    }

    /// Creates a double bass-like preset.
    pub fn double_bass(frequency: f64) -> Self {
        Self::new(frequency, 0.7, 0.2, 0.5)
    }
}

/// Stick-slip friction model using a waveshaping nonlinearity.
///
/// This implements a simplified friction curve that produces
/// stick-slip behavior characteristic of bowed strings.
/// The curve has a positive slope at low velocities (stick)
/// and a negative slope at high velocities (slip).
fn bow_friction(velocity_diff: f64, bow_pressure: f64) -> f64 {
    // Scale the input by bow pressure
    let v = velocity_diff * (1.0 + 3.0 * bow_pressure);

    // Friction curve: combination of hyperbolic tangent and a negative slope region
    // This creates the stick-slip characteristic
    let friction = v * (1.0 - v.abs()).max(0.0);

    // Apply pressure-dependent gain
    friction * (0.3 + 0.7 * bow_pressure)
}

impl Synthesizer for BowedStringSynth {
    fn synthesize(&self, num_samples: usize, sample_rate: f64, _rng: &mut Pcg32) -> Vec<f64> {
        // Handle zero/invalid frequency
        if self.frequency <= 0.0 || !self.frequency.is_finite() {
            return vec![0.0; num_samples];
        }

        // Total delay line length for the full string
        let total_delay = (sample_rate / self.frequency).round() as usize;
        if total_delay < 2 {
            return vec![0.0; num_samples];
        }

        // Split delay line based on bow position
        // bow_position = 0.0 means bow is at bridge (short delay to bridge)
        // bow_position = 1.0 means bow is at nut (long delay to bridge)
        let bow_point = (self.bow_position * total_delay as f64).round() as usize;
        let delay_to_bridge = bow_point.max(1);
        let delay_to_nut = (total_delay - bow_point).max(1);

        // Two delay lines: bridge side and nut side
        let mut delay_bridge = vec![0.0; delay_to_bridge];
        let mut delay_nut = vec![0.0; delay_to_nut];

        let mut output = Vec::with_capacity(num_samples);

        // Indices for circular buffer access
        let mut bridge_write = 0;
        let mut nut_write = 0;

        // Low-pass filter state for damping (one for each delay line)
        let mut filter_bridge = 0.0;
        let mut filter_nut = 0.0;

        // Damping coefficient: higher damping = more high-frequency absorption
        let damping_coeff = 0.1 + self.damping * 0.8;

        // Bow velocity (normalized, constant for sustained bow)
        let bow_velocity = 0.2;

        // Attack ramp to avoid clicks
        let attack_samples = (0.01 * sample_rate) as usize;

        for i in 0..num_samples {
            // Read from delay lines at bow point
            // These represent waves arriving at the bow from each direction
            let bridge_read = (bridge_write + 1) % delay_to_bridge;
            let nut_read = (nut_write + 1) % delay_to_nut;

            let v_from_bridge = delay_bridge[bridge_read];
            let v_from_nut = delay_nut[nut_read];

            // String velocity at bow point is sum of incoming waves
            let string_velocity = v_from_bridge + v_from_nut;

            // Velocity difference between bow and string
            let velocity_diff = bow_velocity - string_velocity;

            // Apply friction model to get excitation force
            let friction_force = bow_friction(velocity_diff, self.bow_pressure);

            // Apply attack ramp
            let envelope = if i < attack_samples {
                i as f64 / attack_samples as f64
            } else {
                1.0
            };
            let scaled_friction = friction_force * envelope;

            // Waves traveling away from bow point
            // Each outgoing wave = incoming wave from opposite direction + friction/2
            let to_bridge = v_from_nut + scaled_friction * 0.5;
            let to_nut = v_from_bridge + scaled_friction * 0.5;

            // Apply damping filter to bridge-side delay
            // (simulates string losses and bridge reflection)
            filter_bridge = (1.0 - damping_coeff) * to_bridge + damping_coeff * filter_bridge;

            // Apply damping filter to nut-side delay
            filter_nut = (1.0 - damping_coeff) * to_nut + damping_coeff * filter_nut;

            // Reflection at bridge (inverted, represents fixed end)
            // The wave returning from bridge is inverted and slightly attenuated
            let bridge_return = -filter_bridge * 0.98;

            // Reflection at nut (inverted, represents fixed end)
            let nut_return = -filter_nut * 0.99;

            // Write to delay lines
            delay_bridge[bridge_write] = bridge_return;
            delay_nut[nut_write] = nut_return;

            // Output is the velocity at the bridge (where the sound radiates)
            // We take the wave arriving at the bridge
            output.push(filter_bridge);

            // Advance write positions
            bridge_write = (bridge_write + 1) % delay_to_bridge;
            nut_write = (nut_write + 1) % delay_to_nut;
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rng::create_rng;

    #[test]
    fn test_bowed_string_basic() {
        let synth = BowedStringSynth::new(440.0, 0.5, 0.12, 0.3);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        assert_eq!(samples.len(), 44100);
        // Should produce non-zero output
        let energy: f64 = samples.iter().map(|s| s.powi(2)).sum();
        assert!(energy > 0.0, "Bowed string should produce output");
    }

    #[test]
    fn test_bowed_string_presets() {
        let mut rng = create_rng(42);

        let violin = BowedStringSynth::violin(440.0);
        let violin_samples = violin.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(violin_samples.len(), 1000);

        let viola = BowedStringSynth::viola(330.0);
        let viola_samples = viola.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(viola_samples.len(), 1000);

        let cello = BowedStringSynth::cello(220.0);
        let cello_samples = cello.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(cello_samples.len(), 1000);

        let bass = BowedStringSynth::double_bass(110.0);
        let bass_samples = bass.synthesize(1000, 44100.0, &mut rng);
        assert_eq!(bass_samples.len(), 1000);
    }

    #[test]
    fn test_bowed_string_determinism() {
        let synth = BowedStringSynth::new(440.0, 0.5, 0.12, 0.3);

        let mut rng1 = create_rng(42);
        let mut rng2 = create_rng(42);

        let samples1 = synth.synthesize(1000, 44100.0, &mut rng1);
        let samples2 = synth.synthesize(1000, 44100.0, &mut rng2);

        assert_eq!(samples1, samples2, "Output should be deterministic");
    }

    #[test]
    fn test_bowed_string_parameter_clamping() {
        // Test that out-of-range parameters are clamped
        let synth = BowedStringSynth::new(440.0, 2.0, -0.5, 1.5);
        assert_eq!(synth.bow_pressure, 1.0);
        assert_eq!(synth.bow_position, 0.01);
        assert_eq!(synth.damping, 1.0);

        let synth2 = BowedStringSynth::new(440.0, -0.5, 1.5, -0.5);
        assert_eq!(synth2.bow_pressure, 0.0);
        assert_eq!(synth2.bow_position, 0.99);
        assert_eq!(synth2.damping, 0.0);
    }

    #[test]
    fn test_bowed_string_zero_frequency_handling() {
        let synth = BowedStringSynth::new(0.0, 0.5, 0.12, 0.3);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(100, 44100.0, &mut rng);

        // Should return silence for zero frequency
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn test_bowed_string_stability() {
        // Ensure high bow pressure doesn't cause instability
        let synth = BowedStringSynth::new(440.0, 1.0, 0.5, 0.1);
        let mut rng = create_rng(42);
        let samples = synth.synthesize(44100, 44100.0, &mut rng);

        // All samples should be finite
        assert!(samples.iter().all(|s| s.is_finite()));
        // Should not explode
        assert!(samples.iter().all(|s| s.abs() < 10.0));
    }

    #[test]
    fn test_bowed_string_different_bow_positions() {
        let mut rng = create_rng(42);

        // Near bridge (brighter, more harmonics)
        let near_bridge = BowedStringSynth::new(220.0, 0.5, 0.05, 0.3);
        let bridge_samples = near_bridge.synthesize(4410, 44100.0, &mut rng);

        // Middle (mellower)
        let middle = BowedStringSynth::new(220.0, 0.5, 0.5, 0.3);
        let middle_samples = middle.synthesize(4410, 44100.0, &mut rng);

        // Both should produce output
        let bridge_energy: f64 = bridge_samples.iter().map(|s| s.powi(2)).sum();
        let middle_energy: f64 = middle_samples.iter().map(|s| s.powi(2)).sum();

        assert!(bridge_energy > 0.0);
        assert!(middle_energy > 0.0);
    }
}
