//! Centralized budget definitions for validation and generation.
//!
//! Budget limits enforce resource bounds at validation stage to prevent
//! expensive operations from being attempted during generation.
//!
//! This module provides:
//! - Budget structs for each asset type (audio, texture, music, mesh)
//! - A unified `BudgetProfile` that combines all budgets
//! - Pre-defined profiles: default, strict, zx-8bit
//! - Error types for budget validation failures

use serde::{Deserialize, Serialize};
use std::fmt;

/// Audio generation budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AudioBudget {
    /// Maximum audio duration in seconds.
    pub max_duration_seconds: f64,
    /// Maximum number of audio layers.
    pub max_layers: usize,
    /// Maximum number of samples (computed from duration * max_sample_rate).
    pub max_samples: usize,
    /// Allowed sample rates.
    pub allowed_sample_rates: Vec<u32>,
}

impl Default for AudioBudget {
    fn default() -> Self {
        Self {
            max_duration_seconds: 30.0,
            max_layers: 32,
            max_samples: 30 * 48_000, // 30 seconds at 48kHz
            allowed_sample_rates: vec![22050, 44100, 48000],
        }
    }
}

impl AudioBudget {
    /// Returns the default maximum audio duration in seconds.
    pub const DEFAULT_MAX_DURATION_SECONDS: f64 = 30.0;

    /// Returns the default maximum number of audio layers.
    pub const DEFAULT_MAX_LAYERS: usize = 32;

    /// Returns the default allowed sample rates.
    pub const DEFAULT_ALLOWED_SAMPLE_RATES: &'static [u32] = &[22050, 44100, 48000];
}

/// Texture generation budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TextureBudget {
    /// Maximum texture dimension (width or height).
    pub max_dimension: u32,
    /// Maximum total pixels (width * height).
    pub max_pixels: u64,
    /// Maximum number of graph nodes (for procedural textures).
    pub max_graph_nodes: usize,
    /// Maximum graph evaluation depth.
    pub max_graph_depth: usize,
}

impl Default for TextureBudget {
    fn default() -> Self {
        Self {
            max_dimension: 4096,
            max_pixels: 4096 * 4096,
            max_graph_nodes: 256,
            max_graph_depth: 64,
        }
    }
}

impl TextureBudget {
    /// Returns the default maximum texture dimension.
    pub const DEFAULT_MAX_DIMENSION: u32 = 4096;

    /// Returns the default maximum total pixels.
    pub const DEFAULT_MAX_PIXELS: u64 = 4096 * 4096;

    /// Returns the default maximum number of graph nodes.
    pub const DEFAULT_MAX_GRAPH_NODES: usize = 256;

    /// Returns the default maximum graph evaluation depth.
    pub const DEFAULT_MAX_GRAPH_DEPTH: usize = 64;
}

/// Music/tracker generation budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MusicBudget {
    /// Maximum channels for XM format.
    pub xm_max_channels: u8,
    /// Maximum patterns for XM format.
    pub xm_max_patterns: u16,
    /// Maximum instruments for XM format.
    pub xm_max_instruments: u16,
    /// Maximum pattern rows for XM format.
    pub xm_max_pattern_rows: u16,
    /// Maximum channels for IT format.
    pub it_max_channels: u8,
    /// Maximum patterns for IT format.
    pub it_max_patterns: u16,
    /// Maximum instruments for IT format.
    pub it_max_instruments: u16,
    /// Maximum samples for IT format.
    pub it_max_samples: u16,
    /// Maximum recursion depth for compose expansion.
    pub max_compose_recursion: usize,
    /// Maximum cells per pattern for compose.
    pub max_cells_per_pattern: usize,
}

impl Default for MusicBudget {
    fn default() -> Self {
        Self {
            xm_max_channels: 32,
            xm_max_patterns: 256,
            xm_max_instruments: 128,
            xm_max_pattern_rows: 256,
            it_max_channels: 64,
            it_max_patterns: 200,
            it_max_instruments: 99,
            it_max_samples: 99,
            max_compose_recursion: 64,
            max_cells_per_pattern: 50_000,
        }
    }
}

impl MusicBudget {
    /// Returns the default maximum channels for XM format.
    pub const DEFAULT_XM_MAX_CHANNELS: u8 = 32;

    /// Returns the default maximum channels for IT format.
    pub const DEFAULT_IT_MAX_CHANNELS: u8 = 64;

    /// Returns the default maximum compose recursion depth.
    pub const DEFAULT_MAX_COMPOSE_RECURSION: usize = 64;

    /// Returns the default maximum cells per pattern.
    pub const DEFAULT_MAX_CELLS_PER_PATTERN: usize = 50_000;

    /// Returns the maximum channels for a given tracker format.
    pub fn max_channels_for_format(&self, format: &str) -> u8 {
        match format {
            "xm" => self.xm_max_channels,
            "it" => self.it_max_channels,
            _ => self.xm_max_channels, // Default to XM
        }
    }
}

/// Mesh generation budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshBudget {
    /// Maximum vertex count.
    pub max_vertices: usize,
    /// Maximum face/triangle count.
    pub max_faces: usize,
    /// Maximum bone count for skeletal meshes.
    pub max_bones: usize,
}

impl Default for MeshBudget {
    fn default() -> Self {
        Self {
            max_vertices: 100_000,
            max_faces: 100_000,
            max_bones: 256,
        }
    }
}

impl MeshBudget {
    /// Returns the default maximum vertex count.
    pub const DEFAULT_MAX_VERTICES: usize = 100_000;

    /// Returns the default maximum face count.
    pub const DEFAULT_MAX_FACES: usize = 100_000;

    /// Returns the default maximum bone count.
    pub const DEFAULT_MAX_BONES: usize = 256;
}

/// General pipeline budget limits.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralBudget {
    /// Maximum Starlark evaluation timeout in seconds.
    pub starlark_timeout_seconds: u64,
    /// Maximum spec JSON size in bytes.
    pub max_spec_size_bytes: usize,
}

impl Default for GeneralBudget {
    fn default() -> Self {
        Self {
            starlark_timeout_seconds: 30,
            max_spec_size_bytes: 10 * 1024 * 1024, // 10 MB
        }
    }
}

impl GeneralBudget {
    /// Returns the default Starlark evaluation timeout in seconds.
    pub const DEFAULT_STARLARK_TIMEOUT_SECONDS: u64 = 30;

    /// Returns the default maximum spec JSON size in bytes.
    pub const DEFAULT_MAX_SPEC_SIZE_BYTES: usize = 10 * 1024 * 1024; // 10 MB
}

/// A complete budget profile for validation and generation.
///
/// Budget profiles can be used to enforce different limits for different
/// target platforms (e.g., stricter limits for retro consoles).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BudgetProfile {
    /// Profile identifier (e.g., "default", "zx-8bit", "strict").
    pub name: String,
    /// Audio budget limits.
    pub audio: AudioBudget,
    /// Texture budget limits.
    pub texture: TextureBudget,
    /// Music budget limits.
    pub music: MusicBudget,
    /// Mesh budget limits.
    pub mesh: MeshBudget,
    /// General pipeline limits.
    pub general: GeneralBudget,
}

impl Default for BudgetProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            audio: AudioBudget::default(),
            texture: TextureBudget::default(),
            music: MusicBudget::default(),
            mesh: MeshBudget::default(),
            general: GeneralBudget::default(),
        }
    }
}

impl BudgetProfile {
    /// Creates a new budget profile with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Returns the strict profile with reduced limits.
    pub fn strict() -> Self {
        Self {
            name: "strict".to_string(),
            audio: AudioBudget {
                max_duration_seconds: 10.0,
                max_layers: 16,
                max_samples: 10 * 48_000,
                allowed_sample_rates: vec![22050, 44100],
            },
            texture: TextureBudget {
                max_dimension: 2048,
                max_pixels: 2048 * 2048,
                max_graph_nodes: 128,
                max_graph_depth: 32,
            },
            music: MusicBudget::default(),
            mesh: MeshBudget {
                max_vertices: 50_000,
                max_faces: 50_000,
                max_bones: 128,
            },
            general: GeneralBudget {
                starlark_timeout_seconds: 15,
                max_spec_size_bytes: 5 * 1024 * 1024,
            },
        }
    }

    /// Returns the ZX-8bit profile optimized for retro targets.
    pub fn zx_8bit() -> Self {
        Self {
            name: "zx-8bit".to_string(),
            audio: AudioBudget {
                max_duration_seconds: 5.0,
                max_layers: 8,
                max_samples: 5 * 22050,
                allowed_sample_rates: vec![22050],
            },
            texture: TextureBudget {
                max_dimension: 256,
                max_pixels: 256 * 256,
                max_graph_nodes: 32,
                max_graph_depth: 16,
            },
            music: MusicBudget {
                xm_max_channels: 8,
                xm_max_patterns: 64,
                xm_max_instruments: 32,
                xm_max_pattern_rows: 64,
                it_max_channels: 8,
                it_max_patterns: 64,
                it_max_instruments: 32,
                it_max_samples: 32,
                max_compose_recursion: 16,
                max_cells_per_pattern: 10_000,
            },
            mesh: MeshBudget {
                max_vertices: 10_000,
                max_faces: 10_000,
                max_bones: 32,
            },
            general: GeneralBudget::default(),
        }
    }

    /// Returns the Nethercore profile optimized for modern sprite-based games.
    pub fn nethercore() -> Self {
        Self {
            name: "nethercore".to_string(),
            audio: AudioBudget {
                max_duration_seconds: 30.0,
                max_layers: 16,
                max_samples: 30 * 22050,
                allowed_sample_rates: vec![22050],
            },
            texture: TextureBudget {
                max_dimension: 1024,
                max_pixels: 1024 * 1024,
                max_graph_nodes: 128,
                max_graph_depth: 32,
            },
            music: MusicBudget {
                xm_max_channels: 16,
                xm_max_patterns: 128,
                xm_max_instruments: 64,
                xm_max_pattern_rows: 128,
                it_max_channels: 16,
                it_max_patterns: 128,
                it_max_instruments: 64,
                it_max_samples: 64,
                max_compose_recursion: 32,
                max_cells_per_pattern: 25_000,
            },
            mesh: MeshBudget {
                max_vertices: 25_000,
                max_faces: 25_000,
                max_bones: 128,
            },
            general: GeneralBudget::default(),
        }
    }

    /// Looks up a profile by name.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::default()),
            "strict" => Some(Self::strict()),
            "zx-8bit" => Some(Self::zx_8bit()),
            "nethercore" => Some(Self::nethercore()),
            _ => None,
        }
    }

    /// Returns the maximum channels for a tracker format using this profile.
    pub fn max_channels_for_format(&self, format: &str) -> u8 {
        self.music.max_channels_for_format(format)
    }
}

/// Error type for budget validation failures.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetError {
    /// Which budget category was exceeded.
    pub category: BudgetCategory,
    /// Limit name that was exceeded.
    pub limit: String,
    /// The actual value that exceeded the limit.
    pub actual: String,
    /// The maximum allowed value.
    pub maximum: String,
}

impl fmt::Display for BudgetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} budget exceeded: {} is {}, maximum is {}",
            self.category, self.limit, self.actual, self.maximum
        )
    }
}

impl std::error::Error for BudgetError {}

impl BudgetError {
    /// Creates a new budget error.
    pub fn new(
        category: BudgetCategory,
        limit: impl Into<String>,
        actual: impl Into<String>,
        maximum: impl Into<String>,
    ) -> Self {
        Self {
            category,
            limit: limit.into(),
            actual: actual.into(),
            maximum: maximum.into(),
        }
    }
}

/// Budget category for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCategory {
    Audio,
    Texture,
    Music,
    Mesh,
    General,
}

impl fmt::Display for BudgetCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Audio => write!(f, "audio"),
            Self::Texture => write!(f, "texture"),
            Self::Music => write!(f, "music"),
            Self::Mesh => write!(f, "mesh"),
            Self::General => write!(f, "general"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_budget_profile_default() {
        let profile = BudgetProfile::default();
        assert_eq!(profile.name, "default");
        assert_eq!(profile.audio.max_duration_seconds, 30.0);
        assert_eq!(profile.audio.max_layers, 32);
        assert_eq!(profile.texture.max_dimension, 4096);
    }

    #[test]
    fn test_budget_profile_strict() {
        let profile = BudgetProfile::strict();
        assert_eq!(profile.name, "strict");
        assert_eq!(profile.audio.max_duration_seconds, 10.0);
        assert_eq!(profile.audio.max_layers, 16);
        assert_eq!(profile.texture.max_dimension, 2048);
    }

    #[test]
    fn test_budget_profile_zx_8bit() {
        let profile = BudgetProfile::zx_8bit();
        assert_eq!(profile.name, "zx-8bit");
        assert_eq!(profile.texture.max_dimension, 256);
        assert_eq!(profile.music.xm_max_channels, 8);
        assert_eq!(profile.audio.max_duration_seconds, 5.0);
    }

    #[test]
    fn test_budget_profile_nethercore() {
        let profile = BudgetProfile::nethercore();
        assert_eq!(profile.name, "nethercore");
        assert_eq!(profile.audio.max_duration_seconds, 30.0);
        assert_eq!(profile.audio.max_layers, 16);
        assert_eq!(profile.audio.allowed_sample_rates, vec![22050]);
        assert_eq!(profile.texture.max_dimension, 1024);
        assert_eq!(profile.texture.max_pixels, 1024 * 1024);
        assert_eq!(profile.music.xm_max_channels, 16);
        assert_eq!(profile.music.it_max_channels, 16);
        assert_eq!(profile.mesh.max_vertices, 25_000);
        assert_eq!(profile.mesh.max_faces, 25_000);
    }

    #[test]
    fn test_budget_profile_by_name() {
        assert!(BudgetProfile::by_name("default").is_some());
        assert!(BudgetProfile::by_name("strict").is_some());
        assert!(BudgetProfile::by_name("zx-8bit").is_some());
        assert!(BudgetProfile::by_name("nethercore").is_some());
        assert!(BudgetProfile::by_name("nonexistent").is_none());
    }

    #[test]
    fn test_max_channels_for_format() {
        let default = BudgetProfile::default();
        assert_eq!(default.max_channels_for_format("xm"), 32);
        assert_eq!(default.max_channels_for_format("it"), 64);
        assert_eq!(default.max_channels_for_format("unknown"), 32); // Default to XM

        let zx = BudgetProfile::zx_8bit();
        assert_eq!(zx.max_channels_for_format("xm"), 8);
        assert_eq!(zx.max_channels_for_format("it"), 8);
    }

    #[test]
    fn test_budget_error_display() {
        let err = BudgetError::new(BudgetCategory::Audio, "duration_seconds", "60.0", "30.0");
        assert_eq!(
            err.to_string(),
            "audio budget exceeded: duration_seconds is 60.0, maximum is 30.0"
        );
    }

    #[test]
    fn test_budget_category_display() {
        assert_eq!(BudgetCategory::Audio.to_string(), "audio");
        assert_eq!(BudgetCategory::Texture.to_string(), "texture");
        assert_eq!(BudgetCategory::Music.to_string(), "music");
        assert_eq!(BudgetCategory::Mesh.to_string(), "mesh");
        assert_eq!(BudgetCategory::General.to_string(), "general");
    }

    #[test]
    fn test_audio_budget_constants() {
        assert_eq!(AudioBudget::DEFAULT_MAX_DURATION_SECONDS, 30.0);
        assert_eq!(AudioBudget::DEFAULT_MAX_LAYERS, 32);
        assert_eq!(
            AudioBudget::DEFAULT_ALLOWED_SAMPLE_RATES,
            &[22050, 44100, 48000]
        );
    }

    #[test]
    fn test_texture_budget_constants() {
        assert_eq!(TextureBudget::DEFAULT_MAX_DIMENSION, 4096);
        assert_eq!(TextureBudget::DEFAULT_MAX_PIXELS, 4096 * 4096);
        assert_eq!(TextureBudget::DEFAULT_MAX_GRAPH_NODES, 256);
        assert_eq!(TextureBudget::DEFAULT_MAX_GRAPH_DEPTH, 64);
    }

    #[test]
    fn test_music_budget_constants() {
        assert_eq!(MusicBudget::DEFAULT_XM_MAX_CHANNELS, 32);
        assert_eq!(MusicBudget::DEFAULT_IT_MAX_CHANNELS, 64);
        assert_eq!(MusicBudget::DEFAULT_MAX_COMPOSE_RECURSION, 64);
        assert_eq!(MusicBudget::DEFAULT_MAX_CELLS_PER_PATTERN, 50_000);
    }

    #[test]
    fn test_mesh_budget_constants() {
        assert_eq!(MeshBudget::DEFAULT_MAX_VERTICES, 100_000);
        assert_eq!(MeshBudget::DEFAULT_MAX_FACES, 100_000);
        assert_eq!(MeshBudget::DEFAULT_MAX_BONES, 256);
    }

    #[test]
    fn test_general_budget_constants() {
        assert_eq!(GeneralBudget::DEFAULT_STARLARK_TIMEOUT_SECONDS, 30);
        assert_eq!(GeneralBudget::DEFAULT_MAX_SPEC_SIZE_BYTES, 10 * 1024 * 1024);
    }

    #[test]
    fn test_budget_profile_new() {
        let profile = BudgetProfile::new("custom");
        assert_eq!(profile.name, "custom");
        // Should use default values for all other fields
        assert_eq!(profile.audio.max_duration_seconds, 30.0);
    }
}
