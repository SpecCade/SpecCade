//! Test fixture for running determinism tests across multiple specs.

use std::path::{Path, PathBuf};

use crate::determinism::core::{verify_determinism, DeterminismResult};
use crate::determinism::report::{DeterminismError, DeterminismReportEntry};

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
    pub fn run(&self) -> crate::determinism::report::DeterminismReport {
        let mut report = crate::determinism::report::DeterminismReport::new();

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
            AssetType::Texture => {
                // Texture backend uses generate_material_maps with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Texture - use verify_determinism with custom generator".to_string(),
                ))
            }
            AssetType::Music => {
                // Music backend uses generate_music with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Music - use verify_determinism with custom generator".to_string(),
                ))
            }
            AssetType::Sprite => {
                // Sprite backend uses texture generation with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Sprite - use verify_determinism with custom generator".to_string(),
                ))
            }
            AssetType::Vfx => {
                // VFX backend uses texture generation with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Vfx - use verify_determinism with custom generator".to_string(),
                ))
            }
            AssetType::Ui => {
                // UI backend uses texture generation with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Ui - use verify_determinism with custom generator".to_string(),
                ))
            }
            AssetType::Font => {
                // Font backend uses texture generation with params directly
                // For full spec-based generation, use verify_determinism with custom generator
                Err(DeterminismError::UnsupportedAssetType(
                    "Font - use verify_determinism with custom generator".to_string(),
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
