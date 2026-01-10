//! Determinism testing framework for SpecCade.
//!
//! This module provides utilities for verifying that asset generation produces
//! byte-identical output across multiple runs (Tier 1 requirement).
//!
//! # Overview
//!
//! SpecCade guarantees deterministic output: given the same spec and seed, the
//! generated assets must be identical. This module provides tools to verify
//! this property across:
//!
//! - Multiple runs of the same generation function
//! - Different asset types (audio, texture, music, mesh)
//! - Multiple spec files in batch
//!
//! # Example
//!
//! ```rust,ignore
//! use speccade_tests::determinism::{verify_determinism, DeterminismFixture};
//!
//! // Verify a single generation function
//! let result = verify_determinism(|| generate_audio(&spec), 3);
//! assert!(result.is_deterministic);
//!
//! // Verify multiple specs
//! let fixture = DeterminismFixture::new()
//!     .add_spec("path/to/spec1.json")
//!     .add_spec("path/to/spec2.json")
//!     .runs(5);
//! let report = fixture.run();
//! assert!(report.all_deterministic());
//! ```

use std::fmt;
use std::path::{Path, PathBuf};

/// Result of a determinism verification.
#[derive(Debug, Clone)]
pub struct DeterminismResult {
    /// Whether all runs produced identical output.
    pub is_deterministic: bool,
    /// Number of runs performed.
    pub runs: usize,
    /// Size of the output in bytes.
    pub output_size: usize,
    /// BLAKE3 hash of the output (all runs should match).
    pub hash: String,
    /// If non-deterministic, information about the first difference found.
    pub diff_info: Option<DiffInfo>,
}

/// Information about the first byte difference found between runs.
#[derive(Debug, Clone)]
pub struct DiffInfo {
    /// Byte offset where the difference was found.
    pub offset: usize,
    /// Value from the first run.
    pub expected: u8,
    /// Value from the differing run.
    pub actual: u8,
    /// Which run (0-indexed) produced the differing output.
    pub run_index: usize,
    /// Context bytes before and after the difference (for debugging).
    pub context: DiffContext,
}

/// Context around a byte difference for debugging.
#[derive(Debug, Clone)]
pub struct DiffContext {
    /// Bytes before the difference (up to 8 bytes).
    pub before: Vec<u8>,
    /// Bytes after the difference (up to 8 bytes).
    pub after: Vec<u8>,
}

impl fmt::Display for DiffInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Difference at byte {}: expected 0x{:02X}, got 0x{:02X} (run {})",
            self.offset, self.expected, self.actual, self.run_index
        )?;
        if !self.context.before.is_empty() || !self.context.after.is_empty() {
            write!(f, "\n  Context: ")?;
            for b in &self.context.before {
                write!(f, "{:02X} ", b)?;
            }
            write!(f, "[{:02X}] ", self.expected)?;
            for b in &self.context.after {
                write!(f, "{:02X} ", b)?;
            }
        }
        Ok(())
    }
}

impl DeterminismResult {
    /// Create a successful (deterministic) result.
    pub fn success(runs: usize, output_size: usize, hash: String) -> Self {
        Self {
            is_deterministic: true,
            runs,
            output_size,
            hash,
            diff_info: None,
        }
    }

    /// Create a failure (non-deterministic) result.
    pub fn failure(runs: usize, output_size: usize, hash: String, diff_info: DiffInfo) -> Self {
        Self {
            is_deterministic: false,
            runs,
            output_size,
            hash,
            diff_info: Some(diff_info),
        }
    }

    /// Panic with a detailed message if not deterministic.
    pub fn assert_deterministic(&self) {
        if !self.is_deterministic {
            let diff = self.diff_info.as_ref().unwrap();
            panic!(
                "Non-deterministic output detected!\n\
                 Runs: {}\n\
                 Output size: {} bytes\n\
                 Hash: {}\n\
                 {}",
                self.runs, self.output_size, self.hash, diff
            );
        }
    }
}

/// Run generation N times and verify all outputs are identical.
///
/// This is the core determinism verification function. It runs the provided
/// generation function multiple times and compares all outputs byte-by-byte.
///
/// # Arguments
///
/// * `generate_fn` - A function that generates output data
/// * `runs` - Number of times to run the generation (minimum 2)
///
/// # Returns
///
/// A `DeterminismResult` indicating whether outputs were identical and,
/// if not, details about the first difference found.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::verify_determinism;
///
/// let result = verify_determinism(|| {
///     generate_audio(&spec).wav_data
/// }, 3);
///
/// assert!(result.is_deterministic, "Audio generation must be deterministic");
/// ```
pub fn verify_determinism<F, O>(generate_fn: F, runs: usize) -> DeterminismResult
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    assert!(runs >= 2, "Must run at least 2 times to verify determinism");

    // Generate first output as reference
    let reference = generate_fn();
    let reference_bytes = reference.as_ref();
    let reference_hash = blake3::hash(reference_bytes).to_hex().to_string();

    // Run remaining times and compare
    for run_index in 1..runs {
        let output = generate_fn();
        let output_bytes = output.as_ref();

        // Check length first
        if output_bytes.len() != reference_bytes.len() {
            return DeterminismResult::failure(
                runs,
                reference_bytes.len(),
                reference_hash,
                DiffInfo {
                    offset: reference_bytes.len().min(output_bytes.len()),
                    expected: if reference_bytes.len() > output_bytes.len() {
                        reference_bytes[output_bytes.len()]
                    } else {
                        0
                    },
                    actual: if output_bytes.len() > reference_bytes.len() {
                        output_bytes[reference_bytes.len()]
                    } else {
                        0
                    },
                    run_index,
                    context: DiffContext {
                        before: Vec::new(),
                        after: Vec::new(),
                    },
                },
            );
        }

        // Compare byte-by-byte
        if let Some(diff) = find_first_difference(reference_bytes, output_bytes, run_index) {
            return DeterminismResult::failure(
                runs,
                reference_bytes.len(),
                reference_hash,
                diff,
            );
        }
    }

    DeterminismResult::success(runs, reference_bytes.len(), reference_hash)
}

/// Find the first byte difference between two slices.
fn find_first_difference(expected: &[u8], actual: &[u8], run_index: usize) -> Option<DiffInfo> {
    for (offset, (&e, &a)) in expected.iter().zip(actual.iter()).enumerate() {
        if e != a {
            let context = extract_context(expected, offset);
            return Some(DiffInfo {
                offset,
                expected: e,
                actual: a,
                run_index,
                context,
            });
        }
    }
    None
}

/// Extract context bytes around a given offset.
fn extract_context(data: &[u8], offset: usize) -> DiffContext {
    let before_start = offset.saturating_sub(8);
    let after_end = (offset + 9).min(data.len());

    DiffContext {
        before: data[before_start..offset].to_vec(),
        after: if offset + 1 < data.len() {
            data[(offset + 1)..after_end].to_vec()
        } else {
            Vec::new()
        },
    }
}

/// Compare BLAKE3 hashes of multiple runs.
///
/// A quick check for determinism when you already have computed hashes
/// from multiple runs.
///
/// # Arguments
///
/// * `hashes` - Slice of BLAKE3 hash strings (64-char hex)
///
/// # Returns
///
/// `true` if all hashes are identical, `false` otherwise.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::verify_hash_determinism;
///
/// let hashes = vec![
///     hash_from_run_1,
///     hash_from_run_2,
///     hash_from_run_3,
/// ];
/// assert!(verify_hash_determinism(&hashes));
/// ```
pub fn verify_hash_determinism(hashes: &[String]) -> bool {
    if hashes.is_empty() {
        return true;
    }
    let reference = &hashes[0];
    hashes.iter().all(|h| h == reference)
}

/// Compute BLAKE3 hash of data.
pub fn compute_hash(data: &[u8]) -> String {
    blake3::hash(data).to_hex().to_string()
}

/// Test fixture for determinism across multiple specs.
///
/// This struct provides a convenient way to run determinism tests across
/// multiple spec files with configurable parameters.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::DeterminismFixture;
///
/// let report = DeterminismFixture::new()
///     .add_spec("specs/audio/laser.json")
///     .add_spec("specs/texture/metal.json")
///     .runs(5)
///     .run();
///
/// println!("{}", report);
/// assert!(report.all_deterministic());
/// ```
#[derive(Debug, Clone)]
pub struct DeterminismFixture {
    /// Spec file paths to test.
    pub specs: Vec<PathBuf>,
    /// Number of runs per spec.
    pub runs: usize,
}

impl Default for DeterminismFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl DeterminismFixture {
    /// Create a new empty fixture.
    pub fn new() -> Self {
        Self {
            specs: Vec::new(),
            runs: 3,
        }
    }

    /// Add a spec file path to the fixture.
    pub fn add_spec<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.specs.push(path.as_ref().to_path_buf());
        self
    }

    /// Add multiple spec file paths.
    pub fn add_specs<I, P>(mut self, paths: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            self.specs.push(path.as_ref().to_path_buf());
        }
        self
    }

    /// Set the number of runs per spec.
    pub fn runs(mut self, runs: usize) -> Self {
        assert!(runs >= 2, "Must run at least 2 times");
        self.runs = runs;
        self
    }

    /// Run determinism tests for all specs.
    ///
    /// This method loads each spec, generates output multiple times,
    /// and verifies byte-identical results.
    pub fn run(&self) -> DeterminismReport {
        let mut report = DeterminismReport::new();

        for spec_path in &self.specs {
            let entry = self.test_spec(spec_path);
            report.add_entry(entry);
        }

        report
    }

    /// Test a single spec file.
    fn test_spec(&self, spec_path: &Path) -> DeterminismReportEntry {
        // Check if file exists
        if !spec_path.exists() {
            return DeterminismReportEntry {
                spec_path: spec_path.to_path_buf(),
                result: Err(DeterminismError::SpecNotFound),
            };
        }

        // Read and parse spec
        let spec_content = match std::fs::read_to_string(spec_path) {
            Ok(content) => content,
            Err(e) => {
                return DeterminismReportEntry {
                    spec_path: spec_path.to_path_buf(),
                    result: Err(DeterminismError::IoError(e.to_string())),
                };
            }
        };

        let spec: speccade_spec::Spec = match serde_json::from_str(&spec_content) {
            Ok(s) => s,
            Err(e) => {
                return DeterminismReportEntry {
                    spec_path: spec_path.to_path_buf(),
                    result: Err(DeterminismError::ParseError(e.to_string())),
                };
            }
        };

        // Generate based on asset type
        match self.generate_and_verify(&spec) {
            Ok(result) => DeterminismReportEntry {
                spec_path: spec_path.to_path_buf(),
                result: Ok(result),
            },
            Err(e) => DeterminismReportEntry {
                spec_path: spec_path.to_path_buf(),
                result: Err(e),
            },
        }
    }

    /// Generate output for a spec and verify determinism.
    fn generate_and_verify(
        &self,
        spec: &speccade_spec::Spec,
    ) -> Result<DeterminismResult, DeterminismError> {
        use speccade_spec::AssetType;

        match spec.asset_type {
            AssetType::Audio => {
                // Generic Audio type - same as AudioSfx for now
                let spec_clone = spec.clone();
                let runs = self.runs;
                Ok(verify_determinism(
                    || {
                        speccade_backend_audio::generate(&spec_clone)
                            .map(|r| r.wav.wav_data)
                            .unwrap_or_default()
                    },
                    runs,
                ))
            }
            AssetType::AudioSfx => {
                let spec_clone = spec.clone();
                let runs = self.runs;
                Ok(verify_determinism(
                    || {
                        speccade_backend_audio::generate(&spec_clone)
                            .map(|r| r.wav.wav_data)
                            .unwrap_or_default()
                    },
                    runs,
                ))
            }
            AssetType::AudioInstrument => {
                // AudioInstrument uses generate_instrument with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "AudioInstrument - use verify_determinism with custom generator".to_string()
                ))
            }
            AssetType::Texture => {
                // Texture backend uses generate_material_maps with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Texture - use verify_determinism with custom generator".to_string()
                ))
            }
            AssetType::Music => {
                // Music backend uses generate_music with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Music - use verify_determinism with custom generator".to_string()
                ))
            }
            AssetType::StaticMesh | AssetType::SkeletalMesh | AssetType::SkeletalAnimation => {
                // Blender-based assets require external tooling
                Err(DeterminismError::UnsupportedAssetType(format!(
                    "{:?} - requires Blender",
                    spec.asset_type
                )))
            }
        }
    }
}

/// Builder for custom determinism tests with explicit generation functions.
///
/// Use this when you need to test asset types that don't have a simple
/// `generate(&spec)` function, or when you need custom generation logic.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::DeterminismBuilder;
/// use speccade_backend_texture::generate_material_maps;
///
/// let result = DeterminismBuilder::new()
///     .runs(5)
///     .generate(|| {
///         let params = create_texture_params();
///         generate_material_maps(&params, 42)
///             .maps.get(&TextureMapType::Albedo)
///             .map(|m| m.png_data.clone())
///             .unwrap_or_default()
///     })
///     .verify();
/// ```
pub struct DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    runs: usize,
    generator: Option<F>,
    _phantom: std::marker::PhantomData<O>,
}

impl<F, O> DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    /// Create a new builder with default settings.
    pub fn new() -> Self {
        Self {
            runs: 3,
            generator: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Set the number of runs.
    pub fn runs(mut self, runs: usize) -> Self {
        self.runs = runs;
        self
    }

    /// Set the generation function.
    pub fn generate(mut self, f: F) -> Self {
        self.generator = Some(f);
        self
    }

    /// Verify determinism and return the result.
    ///
    /// # Panics
    /// Panics if no generator was set.
    pub fn verify(self) -> DeterminismResult {
        let generator = self.generator.expect("No generator set - call .generate() first");
        verify_determinism(generator, self.runs)
    }

    /// Verify determinism and panic on failure.
    ///
    /// # Panics
    /// Panics if no generator was set or if output is non-deterministic.
    pub fn assert(self) {
        self.verify().assert_deterministic();
    }
}

impl<F, O> Default for DeterminismBuilder<F, O>
where
    F: Fn() -> O,
    O: AsRef<[u8]>,
{
    fn default() -> Self {
        Self::new()
    }
}

/// Error type for determinism testing.
#[derive(Debug, Clone)]
pub enum DeterminismError {
    /// Spec file was not found.
    SpecNotFound,
    /// IO error reading spec file.
    IoError(String),
    /// Error parsing spec JSON.
    ParseError(String),
    /// Asset type not supported for automated determinism testing.
    UnsupportedAssetType(String),
    /// Generation failed.
    GenerationFailed(String),
}

impl fmt::Display for DeterminismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SpecNotFound => write!(f, "Spec file not found"),
            Self::IoError(e) => write!(f, "IO error: {}", e),
            Self::ParseError(e) => write!(f, "Parse error: {}", e),
            Self::UnsupportedAssetType(t) => write!(f, "Unsupported asset type: {}", t),
            Self::GenerationFailed(e) => write!(f, "Generation failed: {}", e),
        }
    }
}

impl std::error::Error for DeterminismError {}

/// Entry in a determinism report for a single spec.
#[derive(Debug)]
pub struct DeterminismReportEntry {
    /// Path to the spec file.
    pub spec_path: PathBuf,
    /// Result of the determinism test.
    pub result: Result<DeterminismResult, DeterminismError>,
}

impl DeterminismReportEntry {
    /// Check if this entry passed determinism verification.
    pub fn passed(&self) -> bool {
        matches!(&self.result, Ok(r) if r.is_deterministic)
    }
}

/// Report of determinism tests across multiple specs.
#[derive(Debug, Default)]
pub struct DeterminismReport {
    /// Individual spec test entries.
    pub entries: Vec<DeterminismReportEntry>,
}

impl DeterminismReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Add an entry to the report.
    pub fn add_entry(&mut self, entry: DeterminismReportEntry) {
        self.entries.push(entry);
    }

    /// Check if all specs passed determinism verification.
    pub fn all_deterministic(&self) -> bool {
        self.entries.iter().all(|e| e.passed())
    }

    /// Get number of passed tests.
    pub fn passed_count(&self) -> usize {
        self.entries.iter().filter(|e| e.passed()).count()
    }

    /// Get number of failed tests.
    pub fn failed_count(&self) -> usize {
        self.entries.iter().filter(|e| !e.passed()).count()
    }

    /// Get total number of tests.
    pub fn total_count(&self) -> usize {
        self.entries.len()
    }

    /// Get all failed entries.
    pub fn failures(&self) -> Vec<&DeterminismReportEntry> {
        self.entries.iter().filter(|e| !e.passed()).collect()
    }

    /// Panic if any tests failed, with detailed report.
    pub fn assert_all_deterministic(&self) {
        if !self.all_deterministic() {
            let mut msg = format!(
                "Determinism verification failed!\n\
                 Passed: {}/{}\n\n\
                 Failures:\n",
                self.passed_count(),
                self.total_count()
            );

            for entry in self.failures() {
                msg.push_str(&format!("  - {:?}\n", entry.spec_path));
                match &entry.result {
                    Ok(result) => {
                        if let Some(diff) = &result.diff_info {
                            msg.push_str(&format!("    {}\n", diff));
                        }
                    }
                    Err(e) => {
                        msg.push_str(&format!("    Error: {}\n", e));
                    }
                }
            }

            panic!("{}", msg);
        }
    }
}

impl fmt::Display for DeterminismReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "Determinism Report: {}/{} passed",
            self.passed_count(),
            self.total_count()
        )?;

        for entry in &self.entries {
            let status = if entry.passed() { "PASS" } else { "FAIL" };
            writeln!(f, "  [{}] {:?}", status, entry.spec_path)?;

            if !entry.passed() {
                match &entry.result {
                    Ok(result) => {
                        if let Some(diff) = &result.diff_info {
                            writeln!(f, "        {}", diff)?;
                        }
                    }
                    Err(e) => {
                        writeln!(f, "        Error: {}", e)?;
                    }
                }
            }
        }

        Ok(())
    }
}

/// Macro for easy determinism testing.
///
/// This macro generates a test function that runs a generation expression
/// multiple times and verifies byte-identical output. The expression must
/// return a type that implements `AsRef<[u8]>` (e.g., `Vec<u8>`, `&[u8]`).
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::test_determinism;
///
/// test_determinism!(audio_laser_blast, {
///     let spec = create_laser_spec();
///     generate_audio(&spec).wav_data
/// });
///
/// test_determinism!(texture_metal, runs = 5, {
///     let spec = create_metal_texture_spec();
///     generate_texture(&spec).png_data
/// });
/// ```
#[macro_export]
macro_rules! test_determinism {
    ($name:ident, $generate:expr) => {
        #[test]
        fn $name() {
            let mut results: Vec<Vec<u8>> = Vec::with_capacity(3);
            for _ in 0..3 {
                let output = $generate;
                let bytes: &[u8] = output.as_ref();
                results.push(bytes.to_vec());
            }

            let first = &results[0];
            for (i, result) in results.iter().enumerate().skip(1) {
                assert_eq!(
                    first.len(),
                    result.len(),
                    "Output size differs between run 0 and run {}: {} vs {} bytes",
                    i,
                    first.len(),
                    result.len()
                );

                for (offset, (&expected, &actual)) in first.iter().zip(result.iter()).enumerate() {
                    assert_eq!(
                        expected, actual,
                        "Non-deterministic output at byte {}: expected 0x{:02X}, got 0x{:02X} (run {} vs run 0)",
                        offset, expected, actual, i
                    );
                }
            }
        }
    };

    ($name:ident, runs = $runs:expr, $generate:expr) => {
        #[test]
        fn $name() {
            let run_count: usize = $runs;
            assert!(run_count >= 2, "Must run at least 2 times");

            let mut results: Vec<Vec<u8>> = Vec::with_capacity(run_count);
            for _ in 0..run_count {
                let output = $generate;
                let bytes: &[u8] = output.as_ref();
                results.push(bytes.to_vec());
            }

            let first = &results[0];
            for (i, result) in results.iter().enumerate().skip(1) {
                assert_eq!(
                    first.len(),
                    result.len(),
                    "Output size differs between run 0 and run {}: {} vs {} bytes",
                    i,
                    first.len(),
                    result.len()
                );

                for (offset, (&expected, &actual)) in first.iter().zip(result.iter()).enumerate() {
                    assert_eq!(
                        expected, actual,
                        "Non-deterministic output at byte {}: expected 0x{:02X}, got 0x{:02X} (run {} vs run 0)",
                        offset, expected, actual, i
                    );
                }
            }
        }
    };
}

/// Helper to verify determinism of a closure returning Vec<u8>.
///
/// This is useful when you want to verify determinism without the macro,
/// or when you need to capture variables in a closure.
///
/// # Example
///
/// ```rust,ignore
/// use speccade_tests::determinism::assert_deterministic;
///
/// let spec = load_spec("laser.json");
/// assert_deterministic(3, || generate_audio(&spec).wav_data);
/// ```
pub fn assert_deterministic<F>(runs: usize, generate_fn: F)
where
    F: Fn() -> Vec<u8>,
{
    let result = verify_determinism(&generate_fn, runs);
    result.assert_deterministic();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_determinism_identical() {
        let result = verify_determinism(|| vec![1u8, 2, 3, 4, 5], 5);
        assert!(result.is_deterministic);
        assert_eq!(result.runs, 5);
        assert_eq!(result.output_size, 5);
    }

    #[test]
    fn test_verify_determinism_with_counter() {
        use std::sync::atomic::{AtomicU32, Ordering};
        let counter = AtomicU32::new(0);

        // This simulates non-deterministic behavior
        let result = verify_determinism(
            || {
                let n = counter.fetch_add(1, Ordering::SeqCst);
                vec![n as u8, 2, 3]
            },
            3,
        );

        assert!(!result.is_deterministic);
        assert!(result.diff_info.is_some());
        let diff = result.diff_info.unwrap();
        assert_eq!(diff.offset, 0);
        assert_eq!(diff.expected, 0);
        assert_eq!(diff.actual, 1);
    }

    #[test]
    fn test_verify_hash_determinism_same() {
        let hashes = vec![
            "abc123".to_string(),
            "abc123".to_string(),
            "abc123".to_string(),
        ];
        assert!(verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_verify_hash_determinism_different() {
        let hashes = vec![
            "abc123".to_string(),
            "abc123".to_string(),
            "def456".to_string(),
        ];
        assert!(!verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_verify_hash_determinism_empty() {
        let hashes: Vec<String> = vec![];
        assert!(verify_hash_determinism(&hashes));
    }

    #[test]
    fn test_compute_hash() {
        let hash = compute_hash(b"hello world");
        assert_eq!(hash.len(), 64);
        assert_eq!(
            hash,
            "d74981efa70a0c880b8d8c1985d075dbcbf679b99a5f9914e5aaf96b831a9e24"
        );
    }

    #[test]
    fn test_diff_context_extraction() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let context = extract_context(&data, 8);

        assert_eq!(context.before, vec![0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(context.after, vec![9, 10, 11, 12, 13, 14, 15]);
    }

    #[test]
    fn test_diff_context_at_start() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let context = extract_context(&data, 0);

        assert!(context.before.is_empty());
        assert_eq!(context.after, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_diff_context_at_end() {
        let data = vec![0, 1, 2, 3, 4, 5];
        let context = extract_context(&data, 5);

        assert_eq!(context.before, vec![0, 1, 2, 3, 4]);
        assert!(context.after.is_empty());
    }

    #[test]
    fn test_determinism_result_display() {
        let diff = DiffInfo {
            offset: 100,
            expected: 0xAB,
            actual: 0xCD,
            run_index: 2,
            context: DiffContext {
                before: vec![1, 2, 3],
                after: vec![4, 5, 6],
            },
        };

        let display = format!("{}", diff);
        assert!(display.contains("byte 100"));
        assert!(display.contains("0xAB"));
        assert!(display.contains("0xCD"));
        assert!(display.contains("run 2"));
    }

    #[test]
    fn test_fixture_builder() {
        let fixture = DeterminismFixture::new()
            .add_spec("a.json")
            .add_spec("b.json")
            .runs(5);

        assert_eq!(fixture.specs.len(), 2);
        assert_eq!(fixture.runs, 5);
    }

    #[test]
    fn test_report_counting() {
        let mut report = DeterminismReport::new();

        report.add_entry(DeterminismReportEntry {
            spec_path: PathBuf::from("a.json"),
            result: Ok(DeterminismResult::success(3, 100, "hash1".to_string())),
        });

        report.add_entry(DeterminismReportEntry {
            spec_path: PathBuf::from("b.json"),
            result: Err(DeterminismError::SpecNotFound),
        });

        assert_eq!(report.passed_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert_eq!(report.total_count(), 2);
        assert!(!report.all_deterministic());
    }

    // Test macro usage
    test_determinism!(macro_test_constant, { vec![1u8, 2, 3, 4, 5] });

    test_determinism!(macro_test_with_runs, runs = 4, { vec![0u8; 100] });
}
