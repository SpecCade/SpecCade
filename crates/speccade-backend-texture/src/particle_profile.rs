//! VFX particle profile generation (metadata-only output).
//!
//! This module generates metadata describing particle rendering profiles.
//! It is a Tier 1 metadata-only backend - no actual texture generation, just
//! rendering hints for game engines.

use speccade_spec::recipe::vfx::{VfxParticleProfileMetadata, VfxParticleProfileV1Params};
use thiserror::Error;

/// Errors that can occur during particle profile generation.
#[derive(Debug, Error)]
pub enum ParticleProfileError {
    /// Invalid intensity value (must be non-negative).
    #[error("intensity must be non-negative, got {0}")]
    InvalidIntensity(f64),

    /// Invalid distortion strength (must be in [0.0, 1.0]).
    #[error("distortion_strength must be in [0.0, 1.0], got {0}")]
    InvalidDistortionStrength(f64),

    /// Invalid color tint component (must be in [0.0, 1.0]).
    #[error("color_tint[{0}] must be in [0.0, 1.0], got {1}")]
    InvalidColorTint(usize, f64),
}

/// Result of particle profile generation.
#[derive(Debug)]
pub struct ParticleProfileResult {
    /// Metadata structure (serialization handled by caller).
    pub metadata: VfxParticleProfileMetadata,
}

/// Generate particle profile metadata from parameters.
///
/// This is a metadata-only generator that produces rendering profile metadata.
/// The seed is included for API consistency but does not affect the deterministic
/// output (no random elements).
///
/// # Arguments
/// * `params` - Particle profile parameters
/// * `_seed` - Seed (unused, for API consistency with other backends)
///
/// # Returns
/// A `ParticleProfileResult` containing the metadata structure.
pub fn generate_particle_profile(
    params: &VfxParticleProfileV1Params,
    _seed: u32,
) -> Result<ParticleProfileResult, ParticleProfileError> {
    // Validate intensity if provided
    if let Some(intensity) = params.intensity {
        if intensity < 0.0 {
            return Err(ParticleProfileError::InvalidIntensity(intensity));
        }
    }

    // Validate distortion_strength if provided
    if let Some(strength) = params.distortion_strength {
        if !(0.0..=1.0).contains(&strength) {
            return Err(ParticleProfileError::InvalidDistortionStrength(strength));
        }
    }

    // Validate color_tint if provided
    if let Some(tint) = params.color_tint {
        for (i, &c) in tint.iter().enumerate() {
            if !(0.0..=1.0).contains(&c) {
                return Err(ParticleProfileError::InvalidColorTint(i, c));
            }
        }
    }

    // Generate metadata from params
    let metadata = params.to_metadata();

    Ok(ParticleProfileResult { metadata })
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::vfx::ParticleProfileType;

    #[test]
    fn test_generate_additive_profile() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Additive);
        let result = generate_particle_profile(&params, 42).unwrap();

        assert_eq!(result.metadata.profile, ParticleProfileType::Additive);
        assert_eq!(result.metadata.blend_mode, "additive");
    }

    #[test]
    fn test_generate_with_tint() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Soft)
            .with_color_tint(0.8, 0.6, 0.4);
        let result = generate_particle_profile(&params, 42).unwrap();

        assert_eq!(result.metadata.tint, [0.8, 0.6, 0.4]);
    }

    #[test]
    fn test_generate_with_intensity() {
        let params =
            VfxParticleProfileV1Params::new(ParticleProfileType::Screen).with_intensity(1.5);
        let result = generate_particle_profile(&params, 42).unwrap();

        assert_eq!(result.metadata.intensity, 1.5);
    }

    #[test]
    fn test_generate_distort_with_strength() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort)
            .with_distortion_strength(0.7);
        let result = generate_particle_profile(&params, 42).unwrap();

        assert_eq!(result.metadata.distortion_strength, 0.7);
        assert!(result.metadata.shader_hints.distortion_pass);
    }

    #[test]
    fn test_determinism() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Multiply)
            .with_color_tint(0.5, 0.5, 0.5)
            .with_intensity(0.8);

        let result1 = generate_particle_profile(&params, 42).unwrap();
        let result2 = generate_particle_profile(&params, 42).unwrap();

        // Metadata should be identical
        assert_eq!(result1.metadata.profile, result2.metadata.profile);
        assert_eq!(result1.metadata.tint, result2.metadata.tint);
        assert_eq!(result1.metadata.intensity, result2.metadata.intensity);
    }

    #[test]
    fn test_seed_independence() {
        // Metadata-only output should be identical regardless of seed
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Normal);

        let result1 = generate_particle_profile(&params, 1).unwrap();
        let result2 = generate_particle_profile(&params, 999).unwrap();

        assert_eq!(result1.metadata.profile, result2.metadata.profile);
        assert_eq!(result1.metadata.blend_mode, result2.metadata.blend_mode);
    }

    #[test]
    fn test_invalid_intensity() {
        let params =
            VfxParticleProfileV1Params::new(ParticleProfileType::Additive).with_intensity(-0.5);
        let err = generate_particle_profile(&params, 42).unwrap_err();

        assert!(matches!(err, ParticleProfileError::InvalidIntensity(_)));
    }

    #[test]
    fn test_invalid_distortion_strength_over() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort)
            .with_distortion_strength(1.5);
        let err = generate_particle_profile(&params, 42).unwrap_err();

        assert!(matches!(
            err,
            ParticleProfileError::InvalidDistortionStrength(_)
        ));
    }

    #[test]
    fn test_invalid_distortion_strength_under() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort)
            .with_distortion_strength(-0.1);
        let err = generate_particle_profile(&params, 42).unwrap_err();

        assert!(matches!(
            err,
            ParticleProfileError::InvalidDistortionStrength(_)
        ));
    }

    #[test]
    fn test_invalid_color_tint() {
        let mut params = VfxParticleProfileV1Params::new(ParticleProfileType::Soft);
        params.color_tint = Some([1.5, 0.5, 0.5]); // r > 1.0

        let err = generate_particle_profile(&params, 42).unwrap_err();
        assert!(matches!(err, ParticleProfileError::InvalidColorTint(0, _)));
    }

    #[test]
    fn test_all_profile_types() {
        let profiles = vec![
            ParticleProfileType::Additive,
            ParticleProfileType::Soft,
            ParticleProfileType::Distort,
            ParticleProfileType::Multiply,
            ParticleProfileType::Screen,
            ParticleProfileType::Normal,
        ];

        for profile in profiles {
            let params = VfxParticleProfileV1Params::new(profile);
            let result = generate_particle_profile(&params, 42);
            assert!(result.is_ok(), "Profile {:?} failed", profile);
        }
    }

    #[test]
    fn test_shader_hints_by_profile() {
        // Test specific shader hints for different profiles
        let soft_params = VfxParticleProfileV1Params::new(ParticleProfileType::Soft);
        let soft_result = generate_particle_profile(&soft_params, 42).unwrap();
        assert!(soft_result.metadata.shader_hints.soft_particles);
        assert!(!soft_result.metadata.shader_hints.distortion_pass);

        let distort_params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort);
        let distort_result = generate_particle_profile(&distort_params, 42).unwrap();
        assert!(distort_result.metadata.shader_hints.distortion_pass);
        assert!(!distort_result.metadata.shader_hints.soft_particles);
    }
}
