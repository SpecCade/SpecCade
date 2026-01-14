//! Builder pattern for creating reports.

use super::{OutputResult, Report, ReportError, ReportWarning, REPORT_VERSION};
use crate::error::{ValidationError, ValidationWarning};
use crate::spec::{AssetType, Spec};

/// Builder for creating reports ergonomically.
pub struct ReportBuilder {
    spec_hash: String,
    base_spec_hash: Option<String>,
    variant_id: Option<String>,
    asset_id: Option<String>,
    asset_type: Option<AssetType>,
    license: Option<String>,
    seed: Option<u32>,
    recipe_kind: Option<String>,
    recipe_hash: Option<String>,
    ok: bool,
    errors: Vec<ReportError>,
    warnings: Vec<ReportWarning>,
    outputs: Vec<OutputResult>,
    duration_ms: u64,
    backend_version: String,
    target_triple: String,
    git_commit: Option<String>,
    git_dirty: Option<bool>,
}

impl ReportBuilder {
    /// Creates a new report builder.
    ///
    /// # Arguments
    ///
    /// * `spec_hash` - Hex-encoded BLAKE3 hash of the spec
    /// * `backend_version` - Backend identifier and version
    ///
    /// # Example
    ///
    /// ```
    /// use speccade_spec::report::ReportBuilder;
    ///
    /// let report = ReportBuilder::new(
    ///     "a1b2c3d4...".to_string(),
    ///     "speccade-backend-audio v0.1.0".to_string()
    /// )
    /// .ok(true)
    /// .duration_ms(1234)
    /// .build();
    /// ```
    pub fn new(spec_hash: String, backend_version: String) -> Self {
        Self {
            spec_hash,
            base_spec_hash: None,
            variant_id: None,
            asset_id: None,
            asset_type: None,
            license: None,
            seed: None,
            recipe_kind: None,
            recipe_hash: None,
            ok: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            outputs: Vec::new(),
            duration_ms: 0,
            backend_version,
            target_triple: Self::detect_target_triple(),
            git_commit: None,
            git_dirty: None,
        }
    }

    /// Sets the ok status.
    pub fn ok(mut self, ok: bool) -> Self {
        self.ok = ok;
        self
    }

    /// Adds provenance metadata from a spec.
    pub fn spec_metadata(mut self, spec: &Spec) -> Self {
        self.asset_id = Some(spec.asset_id.clone());
        self.asset_type = Some(spec.asset_type);
        self.license = Some(spec.license.clone());
        self.seed = Some(spec.seed);
        self.recipe_kind = spec.recipe.as_ref().map(|r| r.kind.clone());
        self
    }

    /// Sets the recipe hash (kind + params).
    pub fn recipe_hash(mut self, hash: impl Into<String>) -> Self {
        self.recipe_hash = Some(hash.into());
        self
    }

    /// Marks this report as being for a derived variant run.
    pub fn variant(
        mut self,
        base_spec_hash: impl Into<String>,
        variant_id: impl Into<String>,
    ) -> Self {
        self.base_spec_hash = Some(base_spec_hash.into());
        self.variant_id = Some(variant_id.into());
        self
    }

    /// Adds an error to the report.
    pub fn error(mut self, error: ReportError) -> Self {
        self.errors.push(error);
        self.ok = false;
        self
    }

    /// Adds multiple errors to the report.
    pub fn errors(mut self, errors: Vec<ReportError>) -> Self {
        if !errors.is_empty() {
            self.ok = false;
        }
        self.errors.extend(errors);
        self
    }

    /// Adds errors from ValidationErrors.
    pub fn validation_errors(mut self, errors: &[ValidationError]) -> Self {
        if !errors.is_empty() {
            self.ok = false;
            self.errors
                .extend(errors.iter().map(ReportError::from_validation_error));
        }
        self
    }

    /// Adds a warning to the report.
    pub fn warning(mut self, warning: ReportWarning) -> Self {
        self.warnings.push(warning);
        self
    }

    /// Adds multiple warnings to the report.
    pub fn warnings(mut self, warnings: Vec<ReportWarning>) -> Self {
        self.warnings.extend(warnings);
        self
    }

    /// Adds warnings from ValidationWarnings.
    pub fn validation_warnings(mut self, warnings: &[ValidationWarning]) -> Self {
        self.warnings
            .extend(warnings.iter().map(ReportWarning::from_validation_warning));
        self
    }

    /// Adds an output to the report.
    pub fn output(mut self, output: OutputResult) -> Self {
        self.outputs.push(output);
        self
    }

    /// Adds multiple outputs to the report.
    pub fn outputs(mut self, outputs: Vec<OutputResult>) -> Self {
        self.outputs.extend(outputs);
        self
    }

    /// Sets the execution duration in milliseconds.
    pub fn duration_ms(mut self, ms: u64) -> Self {
        self.duration_ms = ms;
        self
    }

    /// Sets the target triple.
    pub fn target_triple(mut self, triple: String) -> Self {
        self.target_triple = triple;
        self
    }

    /// Sets git provenance metadata.
    pub fn git_metadata(mut self, commit: impl Into<String>, dirty: bool) -> Self {
        self.git_commit = Some(commit.into());
        self.git_dirty = Some(dirty);
        self
    }

    /// Builds the final report.
    pub fn build(self) -> Report {
        Report {
            report_version: REPORT_VERSION,
            spec_hash: self.spec_hash,
            base_spec_hash: self.base_spec_hash,
            variant_id: self.variant_id,
            asset_id: self.asset_id,
            asset_type: self.asset_type,
            license: self.license,
            seed: self.seed,
            recipe_kind: self.recipe_kind,
            recipe_hash: self.recipe_hash,
            ok: self.ok,
            errors: self.errors,
            warnings: self.warnings,
            outputs: self.outputs,
            duration_ms: self.duration_ms,
            backend_version: self.backend_version,
            target_triple: self.target_triple,
            git_commit: self.git_commit,
            git_dirty: self.git_dirty,
        }
    }

    /// Detects the current target triple.
    fn detect_target_triple() -> String {
        // Use the target triple from the Rust standard library's build configuration
        #[cfg(target_arch = "x86_64")]
        const ARCH: &str = "x86_64";
        #[cfg(target_arch = "x86")]
        const ARCH: &str = "i686";
        #[cfg(target_arch = "aarch64")]
        const ARCH: &str = "aarch64";
        #[cfg(target_arch = "arm")]
        const ARCH: &str = "arm";
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm"
        )))]
        const ARCH: &str = "unknown";

        #[cfg(target_vendor = "pc")]
        const VENDOR: &str = "pc";
        #[cfg(target_vendor = "apple")]
        const VENDOR: &str = "apple";
        #[cfg(target_vendor = "unknown")]
        const VENDOR: &str = "unknown";
        #[cfg(not(any(
            target_vendor = "pc",
            target_vendor = "apple",
            target_vendor = "unknown"
        )))]
        const VENDOR: &str = "unknown";

        #[cfg(target_os = "windows")]
        const OS: &str = "windows";
        #[cfg(target_os = "linux")]
        const OS: &str = "linux";
        #[cfg(target_os = "macos")]
        const OS: &str = "darwin";
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        const OS: &str = "unknown";

        #[cfg(target_env = "msvc")]
        const ENV: &str = "msvc";
        #[cfg(target_env = "gnu")]
        const ENV: &str = "gnu";
        #[cfg(target_env = "")]
        const ENV: &str = "";
        #[cfg(not(any(target_env = "msvc", target_env = "gnu", target_env = "")))]
        const ENV: &str = "";

        // ENV is conditionally compiled, so is_empty() varies by platform
        #[allow(clippy::const_is_empty)]
        if ENV.is_empty() {
            format!("{}-{}-{}", ARCH, VENDOR, OS)
        } else {
            format!("{}-{}-{}-{}", ARCH, VENDOR, OS, ENV)
        }
    }
}
