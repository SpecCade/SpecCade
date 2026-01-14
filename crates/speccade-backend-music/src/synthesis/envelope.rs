//! ADSR envelope processing for amplitude shaping.

use speccade_spec::recipe::audio::Envelope;

/// Apply ADSR envelope to samples.
pub(super) fn apply_envelope(samples: &[f64], envelope: &Envelope, sample_rate: u32) -> Vec<f64> {
    let attack_samples = (envelope.attack * sample_rate as f64) as usize;
    let decay_samples = (envelope.decay * sample_rate as f64) as usize;
    let release_samples = (envelope.release * sample_rate as f64) as usize;

    // For tracker instruments, we typically want the full sample to play
    // with just attack and decay (no sustain phase for short samples)
    let sustain_end = samples.len().saturating_sub(release_samples);

    samples
        .iter()
        .enumerate()
        .map(|(i, &sample)| {
            let env_value = if attack_samples == 0 {
                // No attack phase, start at full volume
                if i < decay_samples {
                    let decay_progress = i as f64 / decay_samples.max(1) as f64;
                    1.0 - (1.0 - envelope.sustain) * decay_progress
                } else if i < sustain_end {
                    envelope.sustain
                } else {
                    let release_progress = (i - sustain_end) as f64 / release_samples.max(1) as f64;
                    envelope.sustain * (1.0 - release_progress).max(0.0)
                }
            } else if i < attack_samples {
                // Attack phase: ramp from 0 to 1
                i as f64 / attack_samples as f64
            } else if decay_samples > 0 && i < attack_samples + decay_samples {
                // Decay phase: ramp from 1 to sustain level
                let decay_progress = (i - attack_samples) as f64 / decay_samples as f64;
                1.0 - (1.0 - envelope.sustain) * decay_progress
            } else if i < sustain_end {
                // Sustain phase
                envelope.sustain
            } else if release_samples > 0 {
                // Release phase: ramp from sustain to 0
                let release_progress = (i - sustain_end) as f64 / release_samples as f64;
                envelope.sustain * (1.0 - release_progress).max(0.0)
            } else {
                // No release phase, stay at sustain
                envelope.sustain
            };

            sample * env_value
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_envelope() {
        let samples: Vec<f64> = vec![1.0; 1000];
        let envelope = Envelope {
            attack: 0.01,
            decay: 0.01,
            sustain: 0.5,
            release: 0.01,
        };
        let result = apply_envelope(&samples, &envelope, 22050);
        assert_eq!(result.len(), 1000);

        // First sample should be near 0 (attack start)
        assert!(result[0] < 0.1);
    }
}
