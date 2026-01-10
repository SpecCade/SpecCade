//! Noise synthesis types.

use serde::{Deserialize, Serialize};

/// Noise type for noise-based synthesis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoiseType {
    /// White noise (equal energy per frequency).
    White,
    /// Pink noise (1/f spectrum).
    Pink,
    /// Brown noise (1/f^2 spectrum).
    Brown,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // NoiseType Tests
    // ========================================================================

    #[test]
    fn test_noise_type_serde() {
        let noise_types = vec![NoiseType::White, NoiseType::Pink, NoiseType::Brown];

        for noise_type in noise_types {
            let json = serde_json::to_string(&noise_type).unwrap();
            let parsed: NoiseType = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed, noise_type);
        }
    }
}
