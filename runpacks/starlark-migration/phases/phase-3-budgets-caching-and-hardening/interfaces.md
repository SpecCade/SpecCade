# Phase 3 Interfaces: Budgets, Caching, and Hardening

**Date**: 2026-01-17
**Phase**: Phase 3 - Budgets, Caching, and Hardening

---

## New Types and Structures

### Budget System (`speccade-spec/src/validation/budgets.rs`)

```rust
//! Centralized budget definitions for validation and generation.
//!
//! Budget limits enforce resource bounds at validation stage to prevent
//! expensive operations from being attempted during generation.

use serde::{Deserialize, Serialize};

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

    /// Looks up a profile by name.
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "default" => Some(Self::default()),
            "strict" => Some(Self::strict()),
            "zx-8bit" => Some(Self::zx_8bit()),
            _ => None,
        }
    }

    /// Returns the maximum channels for a tracker format.
    pub fn max_channels_for_format(&self, format: &str) -> u8 {
        match format {
            "xm" => self.music.xm_max_channels,
            "it" => self.music.it_max_channels,
            _ => self.music.xm_max_channels, // Default to XM
        }
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

impl std::fmt::Display for BudgetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} budget exceeded: {} is {}, maximum is {}",
            self.category, self.limit, self.actual, self.maximum
        )
    }
}

impl std::error::Error for BudgetError {}

/// Budget category for error reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BudgetCategory {
    Audio,
    Texture,
    Music,
    Mesh,
    General,
}

impl std::fmt::Display for BudgetCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Audio => write!(f, "audio"),
            Self::Texture => write!(f, "texture"),
            Self::Music => write!(f, "music"),
            Self::Mesh => write!(f, "mesh"),
            Self::General => write!(f, "general"),
        }
    }
}
```

---

### Cache System (`speccade-cli/src/cache/`)

#### Cache Key (`cache/key.rs`)

```rust
//! Cache key computation for generation outputs.

use speccade_spec::hash::blake3_hash;

/// Components that make up a cache key.
#[derive(Debug, Clone)]
pub struct CacheKeyComponents {
    /// BLAKE3 hash of the recipe (kind + params).
    pub recipe_hash: String,
    /// Backend version string.
    pub backend_version: String,
    /// Starlark stdlib version (if applicable).
    pub stdlib_version: Option<String>,
    /// Budget profile name.
    pub budget_profile: String,
}

impl CacheKeyComponents {
    /// Computes the full cache key as a BLAKE3 hash.
    pub fn compute(&self) -> String {
        let mut input = String::new();
        input.push_str(&self.recipe_hash);
        input.push('|');
        input.push_str(&self.backend_version);
        input.push('|');
        if let Some(ref stdlib) = self.stdlib_version {
            input.push_str(stdlib);
        }
        input.push('|');
        input.push_str(&self.budget_profile);

        blake3_hash(input.as_bytes())
    }

    /// Returns the first 16 characters of the cache key (for directory naming).
    pub fn short_key(&self) -> String {
        self.compute()[..16].to_string()
    }
}

/// A computed cache key with metadata.
#[derive(Debug, Clone)]
pub struct CacheKey {
    /// Full 64-character BLAKE3 hash.
    pub full: String,
    /// Short prefix for directory naming.
    pub prefix: String,
    /// Original components used to compute the key.
    pub components: CacheKeyComponents,
}

impl CacheKey {
    /// Creates a new cache key from components.
    pub fn new(components: CacheKeyComponents) -> Self {
        let full = components.compute();
        let prefix = full[..16].to_string();
        Self {
            full,
            prefix,
            components,
        }
    }

    /// Creates a cache key from a report (for validation).
    pub fn from_report(report: &speccade_spec::Report, budget_profile: &str) -> Option<Self> {
        let recipe_hash = report.recipe_hash.as_ref()?;
        let components = CacheKeyComponents {
            recipe_hash: recipe_hash.clone(),
            backend_version: report.backend_version.clone(),
            stdlib_version: report.stdlib_version.clone(),
            budget_profile: budget_profile.to_string(),
        };
        Some(Self::new(components))
    }
}
```

#### Cache Storage (`cache/storage.rs`)

```rust
//! File-based cache storage for generation outputs.

use std::path::{Path, PathBuf};
use speccade_spec::Report;

/// Result of a cache lookup.
#[derive(Debug)]
pub struct CachedResult {
    /// The cached report.
    pub report: Report,
    /// Paths to cached artifacts.
    pub artifacts: Vec<PathBuf>,
    /// Cache metadata.
    pub metadata: CacheMetadata,
}

/// Metadata stored alongside cached artifacts.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheMetadata {
    /// When the cache entry was created.
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Original source file path (for debugging).
    pub source_path: Option<String>,
    /// Cache key used.
    pub cache_key: String,
}

/// Cache storage configuration.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Root directory for cache storage.
    pub root: PathBuf,
    /// Whether caching is enabled.
    pub enabled: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from(".speccade/cache"),
            enabled: true,
        }
    }
}

/// File-based cache storage.
pub struct CacheStorage {
    config: CacheConfig,
}

impl CacheStorage {
    /// Creates a new cache storage with the given configuration.
    pub fn new(config: CacheConfig) -> Self {
        Self { config }
    }

    /// Creates a disabled cache storage (for --no-cache).
    pub fn disabled() -> Self {
        Self {
            config: CacheConfig {
                enabled: false,
                ..Default::default()
            },
        }
    }

    /// Looks up a cached result by key.
    ///
    /// Returns `None` if:
    /// - Caching is disabled
    /// - No cache entry exists
    /// - Cache entry is corrupted
    pub fn lookup(&self, key: &super::key::CacheKey) -> Option<CachedResult> {
        if !self.config.enabled {
            return None;
        }
        // Implementation: read from {root}/{prefix}/{full}/
        todo!()
    }

    /// Stores a generation result in the cache.
    ///
    /// This is atomic: writes to a temp directory, then renames.
    pub fn store(
        &self,
        key: &super::key::CacheKey,
        report: &Report,
        artifacts: &[PathBuf],
        source_path: Option<&Path>,
    ) -> std::io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        // Implementation: write to {root}/{prefix}/{full}/
        todo!()
    }

    /// Invalidates a cache entry.
    pub fn invalidate(&self, key: &super::key::CacheKey) -> std::io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        // Implementation: remove {root}/{prefix}/{full}/
        todo!()
    }

    /// Clears all cache entries.
    pub fn clear(&self) -> std::io::Result<()> {
        if !self.config.enabled {
            return Ok(());
        }
        // Implementation: remove {root}/*
        todo!()
    }

    /// Returns the path for a cache entry.
    fn entry_path(&self, key: &super::key::CacheKey) -> PathBuf {
        self.config.root.join(&key.prefix).join(&key.full)
    }
}
```

#### Cache Module (`cache/mod.rs`)

```rust
//! Generation output caching.
//!
//! This module provides file-based caching for generation outputs.
//! Cache keys are computed from the recipe hash, backend version,
//! stdlib version, and budget profile.

mod key;
mod storage;

pub use key::{CacheKey, CacheKeyComponents};
pub use storage::{CacheConfig, CacheMetadata, CacheStorage, CachedResult};

/// Checks if a result should be cached based on the report.
///
/// Returns `false` if:
/// - `git_dirty` is `Some(true)` (uncommitted changes)
/// - The report indicates an error
pub fn should_cache(report: &speccade_spec::Report) -> bool {
    // Don't cache dirty builds
    if report.git_dirty == Some(true) {
        return false;
    }
    // Don't cache failed generations
    if !report.ok {
        return false;
    }
    true
}
```

---

### Extended Validation Functions

#### Updated `validate_for_generate` signature

```rust
// In speccade-spec/src/validation/mod.rs

/// Validates that a spec is suitable for the `generate` command.
///
/// This performs standard validation plus checks that a recipe is present.
///
/// # Arguments
/// * `spec` - The spec to validate
/// * `budget` - Optional budget profile (defaults to `BudgetProfile::default()`)
pub fn validate_for_generate(spec: &Spec) -> ValidationResult {
    validate_for_generate_with_budget(spec, &BudgetProfile::default())
}

/// Validates a spec for generation with a specific budget profile.
pub fn validate_for_generate_with_budget(
    spec: &Spec,
    budget: &BudgetProfile,
) -> ValidationResult {
    let mut result = validate_spec(spec);

    // E010: Recipe required for generate
    if spec.recipe.is_none() {
        result.add_error(ValidationError::with_path(
            ErrorCode::MissingRecipe,
            "recipe is required for generate command",
            "recipe",
        ));
    }

    // Budget enforcement
    if let Some(recipe) = &spec.recipe {
        validate_recipe_budgets(spec, recipe, budget, &mut result);
    }

    // ... rest of validation
    result
}

/// Validates recipe parameters against budget limits.
fn validate_recipe_budgets(
    spec: &Spec,
    recipe: &Recipe,
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    // Dispatch to appropriate budget validator based on recipe kind
    if recipe.kind.starts_with("audio") {
        validate_audio_budgets(recipe, &budget.audio, result);
    } else if recipe.kind.starts_with("music") {
        validate_music_budgets(recipe, &budget.music, result);
    } else if recipe.kind.starts_with("texture") {
        validate_texture_budgets(recipe, &budget.texture, result);
    } else if recipe.kind.contains("mesh") {
        validate_mesh_budgets(recipe, &budget.mesh, result);
    }
}
```

---

### CLI Additions

#### Updated Generate Command

```rust
// In speccade-cli/src/main.rs

#[derive(Subcommand)]
enum Commands {
    /// Generate assets from a spec file
    Generate {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,

        /// Expand `variants[]` into separate generation runs
        #[arg(long)]
        expand_variants: bool,

        /// Disable caching (always regenerate)
        #[arg(long)]
        no_cache: bool,

        /// Custom cache directory (default: .speccade/cache)
        #[arg(long)]
        cache_dir: Option<String>,

        /// Budget profile to use (default, strict, zx-8bit)
        #[arg(long, default_value = "default")]
        budget_profile: String,
    },
    // ...
}
```

---

### Error Codes

```rust
// Add to speccade-spec/src/error.rs

/// Error codes for validation failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorCode {
    // ... existing codes ...

    /// E020: Audio budget exceeded
    AudioBudgetExceeded,
    /// E021: Texture budget exceeded
    TextureBudgetExceeded,
    /// E022: Music budget exceeded
    MusicBudgetExceeded,
    /// E023: Mesh budget exceeded
    MeshBudgetExceeded,
    /// E024: General budget exceeded
    GeneralBudgetExceeded,
}
```

---

## Summary of New Public Types

| Type | Crate | Purpose |
|------|-------|---------|
| `BudgetProfile` | `speccade-spec` | Complete budget configuration |
| `AudioBudget` | `speccade-spec` | Audio generation limits |
| `TextureBudget` | `speccade-spec` | Texture generation limits |
| `MusicBudget` | `speccade-spec` | Music/tracker limits |
| `MeshBudget` | `speccade-spec` | Mesh generation limits |
| `GeneralBudget` | `speccade-spec` | Pipeline-wide limits |
| `BudgetError` | `speccade-spec` | Budget validation error |
| `BudgetCategory` | `speccade-spec` | Error categorization |
| `CacheKey` | `speccade-cli` | Cache lookup key |
| `CacheKeyComponents` | `speccade-cli` | Key components |
| `CacheStorage` | `speccade-cli` | File-based cache |
| `CacheConfig` | `speccade-cli` | Cache configuration |
| `CachedResult` | `speccade-cli` | Cache lookup result |
| `CacheMetadata` | `speccade-cli` | Cache entry metadata |

---

## Existing Types (Verified Complete)

These types already exist and satisfy the provenance requirements:

| Type | Location | Fields |
|------|----------|--------|
| `Report` | `speccade-spec/src/report/mod.rs` | `source_kind`, `source_hash`, `stdlib_version`, `recipe_hash` |
| `ReportBuilder` | `speccade-spec/src/report/builder.rs` | `source_provenance()`, `stdlib_version()`, `recipe_hash()` |
