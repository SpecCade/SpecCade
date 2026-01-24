//! Particle profile recipe types for `vfx.particle_profile_v1`.
//!
//! Defines parameters for VFX particle rendering profiles. This is a metadata-only
//! recipe that outputs JSON describing blend modes, color grading, and distortion
//! properties for particle effects.

use serde::{Deserialize, Serialize};

/// Parameters for the `vfx.particle_profile_v1` recipe.
///
/// Generates metadata describing a particle rendering profile for VFX systems.
/// This is a metadata-only output (JSON) that provides rendering hints for game
/// engines, not actual texture generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VfxParticleProfileV1Params {
    /// The particle profile type (determines blend mode and rendering behavior).
    pub profile: ParticleProfileType,

    /// Optional RGB tint color (each component in [0.0, 1.0]).
    /// Applied as a color multiplier to particles.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color_tint: Option<[f64; 3]>,

    /// Optional intensity multiplier (default 1.0).
    /// Scales the overall brightness/opacity of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intensity: Option<f64>,

    /// Optional distortion strength for the Distort profile (in [0.0, 1.0]).
    /// Only meaningful when profile is Distort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub distortion_strength: Option<f64>,
}

/// Particle profile type determining blend mode and rendering behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParticleProfileType {
    /// Additive blending (bright, glowing effects like fire, sparks, magic).
    Additive,

    /// Soft/premultiplied alpha (smoke, fog, soft particles).
    Soft,

    /// Distortion/refraction effect (heat haze, shockwaves, underwater).
    Distort,

    /// Multiply blending (shadows, darkening effects).
    Multiply,

    /// Screen blending (bright overlay, lightning, lens flares).
    Screen,

    /// Normal alpha blending (standard transparent particles).
    Normal,
}

/// Metadata output for a particle profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VfxParticleProfileMetadata {
    /// The profile type.
    pub profile: ParticleProfileType,

    /// Human-readable blend mode description.
    pub blend_mode: String,

    /// RGB tint color ([r, g, b] in [0.0, 1.0], default [1.0, 1.0, 1.0]).
    pub tint: [f64; 3],

    /// Intensity multiplier (default 1.0).
    pub intensity: f64,

    /// Distortion strength (0.0-1.0, only relevant for Distort profile).
    pub distortion_strength: f64,

    /// Recommended shader hints for the game engine.
    pub shader_hints: ShaderHints,
}

/// Shader configuration hints for game engines.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShaderHints {
    /// Whether depth writing should be disabled.
    pub depth_write: bool,

    /// Whether the effect is considered transparent.
    pub transparent: bool,

    /// Whether soft particle depth fading is recommended.
    pub soft_particles: bool,

    /// Whether distortion/refraction pass is needed.
    pub distortion_pass: bool,
}

impl VfxParticleProfileV1Params {
    /// Creates a new particle profile with the given profile type.
    pub fn new(profile: ParticleProfileType) -> Self {
        Self {
            profile,
            color_tint: None,
            intensity: None,
            distortion_strength: None,
        }
    }

    /// Sets the color tint.
    pub fn with_color_tint(mut self, r: f64, g: f64, b: f64) -> Self {
        self.color_tint = Some([r, g, b]);
        self
    }

    /// Sets the intensity multiplier.
    pub fn with_intensity(mut self, intensity: f64) -> Self {
        self.intensity = Some(intensity);
        self
    }

    /// Sets the distortion strength (only meaningful for Distort profile).
    pub fn with_distortion_strength(mut self, strength: f64) -> Self {
        self.distortion_strength = Some(strength);
        self
    }

    /// Converts parameters to metadata output.
    pub fn to_metadata(&self) -> VfxParticleProfileMetadata {
        let blend_mode = match self.profile {
            ParticleProfileType::Additive => "additive".to_string(),
            ParticleProfileType::Soft => "soft_additive".to_string(),
            ParticleProfileType::Distort => "distortion".to_string(),
            ParticleProfileType::Multiply => "multiply".to_string(),
            ParticleProfileType::Screen => "screen".to_string(),
            ParticleProfileType::Normal => "alpha_blend".to_string(),
        };

        let tint = self.color_tint.unwrap_or([1.0, 1.0, 1.0]);
        let intensity = self.intensity.unwrap_or(1.0);
        let distortion_strength = self.distortion_strength.unwrap_or(0.0);

        let shader_hints = match self.profile {
            ParticleProfileType::Additive => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: false,
                distortion_pass: false,
            },
            ParticleProfileType::Soft => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: true,
                distortion_pass: false,
            },
            ParticleProfileType::Distort => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: false,
                distortion_pass: true,
            },
            ParticleProfileType::Multiply => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: false,
                distortion_pass: false,
            },
            ParticleProfileType::Screen => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: false,
                distortion_pass: false,
            },
            ParticleProfileType::Normal => ShaderHints {
                depth_write: false,
                transparent: true,
                soft_particles: true,
                distortion_pass: false,
            },
        };

        VfxParticleProfileMetadata {
            profile: self.profile,
            blend_mode,
            tint,
            intensity,
            distortion_strength,
            shader_hints,
        }
    }
}

impl std::fmt::Display for ParticleProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParticleProfileType::Additive => write!(f, "additive"),
            ParticleProfileType::Soft => write!(f, "soft"),
            ParticleProfileType::Distort => write!(f, "distort"),
            ParticleProfileType::Multiply => write!(f, "multiply"),
            ParticleProfileType::Screen => write!(f, "screen"),
            ParticleProfileType::Normal => write!(f, "normal"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_new() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Additive);
        assert_eq!(params.profile, ParticleProfileType::Additive);
        assert!(params.color_tint.is_none());
        assert!(params.intensity.is_none());
        assert!(params.distortion_strength.is_none());
    }

    #[test]
    fn test_params_builder() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort)
            .with_color_tint(1.0, 0.5, 0.0)
            .with_intensity(1.5)
            .with_distortion_strength(0.8);

        assert_eq!(params.profile, ParticleProfileType::Distort);
        assert_eq!(params.color_tint, Some([1.0, 0.5, 0.0]));
        assert_eq!(params.intensity, Some(1.5));
        assert_eq!(params.distortion_strength, Some(0.8));
    }

    #[test]
    fn test_to_metadata_defaults() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Additive);
        let metadata = params.to_metadata();

        assert_eq!(metadata.profile, ParticleProfileType::Additive);
        assert_eq!(metadata.blend_mode, "additive");
        assert_eq!(metadata.tint, [1.0, 1.0, 1.0]);
        assert_eq!(metadata.intensity, 1.0);
        assert_eq!(metadata.distortion_strength, 0.0);
        assert!(!metadata.shader_hints.depth_write);
        assert!(metadata.shader_hints.transparent);
    }

    #[test]
    fn test_to_metadata_with_options() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Soft)
            .with_color_tint(0.8, 0.6, 0.4)
            .with_intensity(0.75);
        let metadata = params.to_metadata();

        assert_eq!(metadata.blend_mode, "soft_additive");
        assert_eq!(metadata.tint, [0.8, 0.6, 0.4]);
        assert_eq!(metadata.intensity, 0.75);
        assert!(metadata.shader_hints.soft_particles);
    }

    #[test]
    fn test_distort_profile() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Distort)
            .with_distortion_strength(0.5);
        let metadata = params.to_metadata();

        assert_eq!(metadata.blend_mode, "distortion");
        assert_eq!(metadata.distortion_strength, 0.5);
        assert!(metadata.shader_hints.distortion_pass);
    }

    #[test]
    fn test_profile_type_serde() {
        let profile = ParticleProfileType::Screen;
        let json = serde_json::to_string(&profile).unwrap();
        assert_eq!(json, "\"screen\"");

        let parsed: ParticleProfileType = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, profile);
    }

    #[test]
    fn test_params_serde_roundtrip() {
        let params = VfxParticleProfileV1Params::new(ParticleProfileType::Multiply)
            .with_color_tint(0.2, 0.3, 0.4)
            .with_intensity(1.2);

        let json = serde_json::to_string_pretty(&params).unwrap();
        let parsed: VfxParticleProfileV1Params = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed, params);
    }

    #[test]
    fn test_profile_type_display() {
        assert_eq!(ParticleProfileType::Additive.to_string(), "additive");
        assert_eq!(ParticleProfileType::Soft.to_string(), "soft");
        assert_eq!(ParticleProfileType::Distort.to_string(), "distort");
        assert_eq!(ParticleProfileType::Multiply.to_string(), "multiply");
        assert_eq!(ParticleProfileType::Screen.to_string(), "screen");
        assert_eq!(ParticleProfileType::Normal.to_string(), "normal");
    }

    #[test]
    fn test_all_blend_modes() {
        let profiles = vec![
            (ParticleProfileType::Additive, "additive"),
            (ParticleProfileType::Soft, "soft_additive"),
            (ParticleProfileType::Distort, "distortion"),
            (ParticleProfileType::Multiply, "multiply"),
            (ParticleProfileType::Screen, "screen"),
            (ParticleProfileType::Normal, "alpha_blend"),
        ];

        for (profile, expected_blend_mode) in profiles {
            let params = VfxParticleProfileV1Params::new(profile);
            let metadata = params.to_metadata();
            assert_eq!(
                metadata.blend_mode, expected_blend_mode,
                "Profile {:?} should have blend mode {}",
                profile, expected_blend_mode
            );
        }
    }
}
