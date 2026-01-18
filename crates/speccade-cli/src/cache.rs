//! Content-addressed caching for generated assets.
//!
//! This module implements a local cache keyed by:
//! - Canonical recipe hash
//! - Spec seed
//! - Backend version string
//! - Preview mode flag
//!
//! Cache entries are stored in an XDG-compatible directory structure.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use speccade_spec::{canonical_recipe_hash, OutputResult, Recipe};
use std::fs;
use std::path::{Path, PathBuf};

/// Cache key components for deterministic cache lookups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    /// BLAKE3 hash of the canonical recipe
    pub recipe_hash: String,
    /// Spec seed
    pub seed: u32,
    /// Backend version string
    pub backend_version: String,
    /// Preview mode flag
    pub preview: bool,
}

impl CacheKey {
    /// Create a new cache key
    pub fn new(recipe: &Recipe, seed: u32, backend_version: String, preview: bool) -> Result<Self> {
        let recipe_hash =
            canonical_recipe_hash(recipe).context("Failed to compute canonical recipe hash")?;
        Ok(Self {
            recipe_hash,
            seed,
            backend_version,
            preview,
        })
    }

    /// Compute the cache entry hash (deterministic cache directory name)
    pub fn compute_hash(&self) -> String {
        let canonical = format!(
            "recipe:{},seed:{},backend:{},preview:{}",
            self.recipe_hash, self.seed, self.backend_version, self.preview
        );
        blake3::hash(canonical.as_bytes()).to_hex().to_string()
    }
}

/// Cache manifest stored alongside cached outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheManifest {
    /// Cache key components
    pub key: CacheKey,
    /// Timestamp when this entry was created
    pub created_at: String,
    /// Cached output results
    pub outputs: Vec<CachedOutput>,
}

/// A cached output file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedOutput {
    /// Output kind (primary, etc.)
    pub kind: speccade_spec::OutputKind,
    /// Output format (wav, png, etc.)
    pub format: speccade_spec::OutputFormat,
    /// Relative path within the cache entry
    pub cache_path: String,
    /// BLAKE3 hash of the output file
    pub hash: Option<String>,
    /// Validation metrics (for Tier 2 outputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<speccade_spec::OutputMetrics>,
    /// Whether this was generated in preview mode
    pub preview: Option<bool>,
}

/// Cache manager for reading/writing cached generation results
pub struct CacheManager {
    cache_dir: PathBuf,
}

impl CacheManager {
    /// Create a new cache manager using the default XDG cache directory
    pub fn new() -> Result<Self> {
        let cache_dir = Self::default_cache_dir().context("Failed to determine cache directory")?;
        Ok(Self { cache_dir })
    }

    /// Get the default cache directory (XDG-compatible)
    pub fn default_cache_dir() -> Option<PathBuf> {
        dirs::cache_dir().map(|d| d.join("speccade").join("generate"))
    }

    /// Get the path to a cache entry directory
    pub fn entry_path(&self, key: &CacheKey) -> PathBuf {
        let hash = key.compute_hash();
        self.cache_dir.join(format!("{}.cache", hash))
    }

    /// Check if a cache entry exists
    pub fn has_entry(&self, key: &CacheKey) -> bool {
        let entry_path = self.entry_path(key);
        entry_path.join("manifest.json").exists()
    }

    /// Retrieve cached outputs (returns None if cache miss)
    pub fn get(&self, key: &CacheKey, out_root: &Path) -> Result<Option<Vec<OutputResult>>> {
        let entry_path = self.entry_path(key);
        let manifest_path = entry_path.join("manifest.json");

        if !manifest_path.exists() {
            return Ok(None);
        }

        // Read manifest
        let manifest_json =
            fs::read_to_string(&manifest_path).context("Failed to read cache manifest")?;
        let manifest: CacheManifest =
            serde_json::from_str(&manifest_json).context("Failed to parse cache manifest")?;

        // Copy cached files to output directory
        let mut outputs = Vec::new();
        for cached in &manifest.outputs {
            let cache_file = entry_path.join(&cached.cache_path);
            let output_path = out_root.join(&cached.cache_path);

            // Create parent directory if needed
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create output directory: {}", parent.display())
                })?;
            }

            // Copy file from cache to output
            fs::copy(&cache_file, &output_path)
                .with_context(|| format!("Failed to copy cached file: {}", cache_file.display()))?;

            // Reconstruct OutputResult with relative path (same as what dispatch produces)
            outputs.push(OutputResult {
                kind: cached.kind,
                format: cached.format,
                path: PathBuf::from(&cached.cache_path),
                hash: cached.hash.clone(),
                metrics: cached.metrics.clone(),
                preview: cached.preview,
            });
        }

        Ok(Some(outputs))
    }

    /// Store outputs in the cache
    pub fn put(&self, key: &CacheKey, outputs: &[OutputResult], out_root: &Path) -> Result<()> {
        let entry_path = self.entry_path(key);

        // Create cache entry directory
        fs::create_dir_all(&entry_path).with_context(|| {
            format!(
                "Failed to create cache entry directory: {}",
                entry_path.display()
            )
        })?;

        // Copy output files to cache and build manifest
        let mut cached_outputs = Vec::new();
        for output in outputs {
            // OutputResult paths are relative paths from dispatch
            // Use them directly as cache paths
            let cache_path_str = output.path.to_string_lossy().to_string();

            // Actual file location is out_root/path
            let actual_file = out_root.join(&output.path);

            // Copy file to cache
            let cache_file = entry_path.join(&cache_path_str);
            if let Some(parent) = cache_file.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create cache subdirectory: {}", parent.display())
                })?;
            }

            fs::copy(&actual_file, &cache_file).with_context(|| {
                format!("Failed to copy file to cache: {}", actual_file.display())
            })?;

            cached_outputs.push(CachedOutput {
                kind: output.kind,
                format: output.format,
                cache_path: cache_path_str,
                hash: output.hash.clone(),
                metrics: output.metrics.clone(),
                preview: output.preview,
            });
        }

        // Write manifest
        let manifest = CacheManifest {
            key: key.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            outputs: cached_outputs,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)
            .context("Failed to serialize cache manifest")?;
        fs::write(entry_path.join("manifest.json"), manifest_json)
            .context("Failed to write cache manifest")?;

        Ok(())
    }

    /// Clear all cache entries
    pub fn clear(&self) -> Result<u64> {
        if !self.cache_dir.exists() {
            return Ok(0);
        }

        let mut count = 0u64;
        for entry in fs::read_dir(&self.cache_dir).context("Failed to read cache directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("cache") {
                fs::remove_dir_all(&path)
                    .with_context(|| format!("Failed to remove cache entry: {}", path.display()))?;
                count += 1;
            }
        }

        Ok(count)
    }

    /// Get cache info (total entries, total size)
    pub fn info(&self) -> Result<CacheInfo> {
        if !self.cache_dir.exists() {
            return Ok(CacheInfo {
                cache_dir: self.cache_dir.clone(),
                entry_count: 0,
                total_size_bytes: 0,
            });
        }

        let mut entry_count = 0u64;
        let mut total_size_bytes = 0u64;

        for entry in fs::read_dir(&self.cache_dir).context("Failed to read cache directory")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();

            if path.is_dir() && path.extension().and_then(|s| s.to_str()) == Some("cache") {
                entry_count += 1;
                total_size_bytes += Self::dir_size(&path)?;
            }
        }

        Ok(CacheInfo {
            cache_dir: self.cache_dir.clone(),
            entry_count,
            total_size_bytes,
        })
    }

    /// Compute total size of a directory (recursive)
    fn dir_size(path: &Path) -> Result<u64> {
        let mut total = 0u64;

        for entry in walkdir::WalkDir::new(path) {
            let entry = entry.context("Failed to walk directory")?;
            if entry.file_type().is_file() {
                total += entry.metadata()?.len();
            }
        }

        Ok(total)
    }
}

/// Cache information
#[derive(Debug, Clone)]
pub struct CacheInfo {
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Number of cache entries
    pub entry_count: u64,
    /// Total size in bytes
    pub total_size_bytes: u64,
}

/// Add chrono for timestamps
use chrono;

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{OutputFormat, OutputKind};
    use tempfile::TempDir;

    fn create_test_recipe() -> Recipe {
        speccade_spec::Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 0.5,
                "sample_rate": 22050,
                "layers": []
            }),
        )
    }

    #[test]
    fn test_cache_key_compute_hash() {
        let recipe = create_test_recipe();
        let key1 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let key2 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();

        // Same key components should produce same hash
        assert_eq!(key1.compute_hash(), key2.compute_hash());
    }

    #[test]
    fn test_cache_key_different_seeds() {
        let recipe = create_test_recipe();
        let key1 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let key2 = CacheKey::new(&recipe, 43, "v1.0.0".to_string(), false).unwrap();

        // Different seeds should produce different hashes
        assert_ne!(key1.compute_hash(), key2.compute_hash());
    }

    #[test]
    fn test_cache_key_different_versions() {
        let recipe = create_test_recipe();
        let key1 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let key2 = CacheKey::new(&recipe, 42, "v1.0.1".to_string(), false).unwrap();

        // Different versions should produce different hashes
        assert_ne!(key1.compute_hash(), key2.compute_hash());
    }

    #[test]
    fn test_cache_key_different_preview() {
        let recipe = create_test_recipe();
        let key1 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let key2 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), true).unwrap();

        // Different preview flags should produce different hashes
        assert_ne!(key1.compute_hash(), key2.compute_hash());
    }

    #[test]
    fn test_cache_roundtrip() {
        let tmp_cache = TempDir::new().unwrap();
        let tmp_out = TempDir::new().unwrap();

        let mut cache_mgr = CacheManager::new().unwrap();
        cache_mgr.cache_dir = tmp_cache.path().to_path_buf();

        let recipe = create_test_recipe();
        let key = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();

        // Initially no cache entry
        assert!(!cache_mgr.has_entry(&key));

        // Create a test output file
        let output_path = tmp_out.path().join("test.wav");
        fs::write(&output_path, b"test data").unwrap();

        // Use relative path (as dispatch does)
        let outputs = vec![OutputResult {
            kind: OutputKind::Primary,
            format: OutputFormat::Wav,
            path: PathBuf::from("test.wav"),
            hash: Some("testhash".to_string()),
            metrics: None,
            preview: None,
        }];

        // Store in cache
        cache_mgr.put(&key, &outputs, tmp_out.path()).unwrap();

        // Now entry should exist
        assert!(cache_mgr.has_entry(&key));

        // Retrieve from cache to a different output dir
        let tmp_out2 = TempDir::new().unwrap();
        let cached_outputs = cache_mgr.get(&key, tmp_out2.path()).unwrap().unwrap();

        assert_eq!(cached_outputs.len(), 1);
        assert_eq!(cached_outputs[0].kind, OutputKind::Primary);
        assert_eq!(cached_outputs[0].format, OutputFormat::Wav);
        assert_eq!(cached_outputs[0].hash, Some("testhash".to_string()));

        // Verify file was copied
        let output_path2 = tmp_out2.path().join("test.wav");
        assert!(output_path2.exists());
        let data = fs::read(&output_path2).unwrap();
        assert_eq!(data, b"test data");
    }

    #[test]
    fn test_cache_clear() {
        let tmp_cache = TempDir::new().unwrap();
        let tmp_out = TempDir::new().unwrap();

        let mut cache_mgr = CacheManager::new().unwrap();
        cache_mgr.cache_dir = tmp_cache.path().to_path_buf();

        let recipe = create_test_recipe();
        let key1 = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let key2 = CacheKey::new(&recipe, 43, "v1.0.0".to_string(), false).unwrap();

        // Create test outputs
        let output_path = tmp_out.path().join("test.wav");
        fs::write(&output_path, b"test data").unwrap();
        let outputs = vec![OutputResult {
            kind: OutputKind::Primary,
            format: OutputFormat::Wav,
            path: PathBuf::from("test.wav"),
            hash: None,
            metrics: None,
            preview: None,
        }];

        // Store two entries
        cache_mgr.put(&key1, &outputs, tmp_out.path()).unwrap();
        cache_mgr.put(&key2, &outputs, tmp_out.path()).unwrap();

        assert!(cache_mgr.has_entry(&key1));
        assert!(cache_mgr.has_entry(&key2));

        // Clear cache
        let count = cache_mgr.clear().unwrap();
        assert_eq!(count, 2);

        assert!(!cache_mgr.has_entry(&key1));
        assert!(!cache_mgr.has_entry(&key2));
    }

    #[test]
    fn test_cache_info() {
        let tmp_cache = TempDir::new().unwrap();
        let tmp_out = TempDir::new().unwrap();

        let mut cache_mgr = CacheManager::new().unwrap();
        cache_mgr.cache_dir = tmp_cache.path().to_path_buf();

        // Initially empty
        let info = cache_mgr.info().unwrap();
        assert_eq!(info.entry_count, 0);
        assert_eq!(info.total_size_bytes, 0);

        // Add an entry
        let recipe = create_test_recipe();
        let key = CacheKey::new(&recipe, 42, "v1.0.0".to_string(), false).unwrap();
        let output_path = tmp_out.path().join("test.wav");
        fs::write(&output_path, b"test data").unwrap();
        let outputs = vec![OutputResult {
            kind: OutputKind::Primary,
            format: OutputFormat::Wav,
            path: PathBuf::from("test.wav"),
            hash: None,
            metrics: None,
            preview: None,
        }];
        cache_mgr.put(&key, &outputs, tmp_out.path()).unwrap();

        // Now should have 1 entry
        let info = cache_mgr.info().unwrap();
        assert_eq!(info.entry_count, 1);
        assert!(info.total_size_bytes > 0);
    }
}
