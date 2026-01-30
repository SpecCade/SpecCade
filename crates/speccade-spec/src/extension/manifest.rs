//! Extension manifest types for external backend registration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Level of determinism guaranteed by an extension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeterminismLevel {
    /// Output is byte-for-byte identical for the same input.
    /// Required for Tier 1 backends.
    ByteIdentical,
    /// Output is semantically equivalent but may vary in representation.
    /// Used for Tier 2 backends (e.g., Blender-based).
    SemanticEquivalent,
    /// No determinism guarantee.
    /// Outputs are accepted but not cached.
    NonDeterministic,
}

impl DeterminismLevel {
    /// Returns the tier number for this determinism level.
    pub fn tier(&self) -> u8 {
        match self {
            DeterminismLevel::ByteIdentical => 1,
            DeterminismLevel::SemanticEquivalent => 2,
            DeterminismLevel::NonDeterministic => 3,
        }
    }

    /// Returns true if this level requires hash verification.
    pub fn requires_hash(&self) -> bool {
        matches!(self, DeterminismLevel::ByteIdentical)
    }

    /// Returns true if this level requires metric verification.
    pub fn requires_metrics(&self) -> bool {
        matches!(self, DeterminismLevel::SemanticEquivalent)
    }
}

impl std::fmt::Display for DeterminismLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeterminismLevel::ByteIdentical => write!(f, "byte_identical"),
            DeterminismLevel::SemanticEquivalent => write!(f, "semantic_equivalent"),
            DeterminismLevel::NonDeterministic => write!(f, "non_deterministic"),
        }
    }
}

/// Interface type for an extension.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtensionInterface {
    /// Subprocess-based extension.
    /// The extension is invoked as a subprocess with command-line arguments.
    Subprocess {
        /// Executable name or path.
        executable: String,
        /// Additional command-line arguments.
        /// Supports placeholders: {spec_path}, {out_dir}, {seed}
        #[serde(default)]
        args: Vec<String>,
        /// Environment variables to set.
        #[serde(default)]
        env: HashMap<String, String>,
        /// Timeout in seconds (default: 300).
        #[serde(default = "default_timeout")]
        timeout_seconds: u64,
    },
    /// WASM-based extension (future).
    /// The extension is a WASM module loaded and executed in-process.
    #[serde(rename = "wasm")]
    Wasm {
        /// Path to the WASM module.
        module_path: String,
        /// Memory limit in bytes (default: 256MB).
        #[serde(default = "default_wasm_memory")]
        memory_limit: u64,
        /// Execution time limit in seconds (default: 60).
        #[serde(default = "default_wasm_timeout")]
        timeout_seconds: u64,
    },
}

fn default_timeout() -> u64 {
    300 // 5 minutes
}

fn default_wasm_memory() -> u64 {
    256 * 1024 * 1024 // 256 MB
}

fn default_wasm_timeout() -> u64 {
    60 // 1 minute
}

/// Manifest describing an extension's capabilities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionManifest {
    /// Unique name of the extension.
    pub name: String,
    /// Semantic version of the extension.
    pub version: String,
    /// Determinism tier (1, 2, or 3).
    pub tier: u8,
    /// Determinism level.
    pub determinism: DeterminismLevel,
    /// Interface for invoking the extension.
    pub interface: ExtensionInterface,
    /// Recipe kinds this extension handles.
    /// Example: ["texture.custom_v1", "texture.advanced_v1"]
    pub recipe_kinds: Vec<String>,
    /// JSON Schema for the input spec (optional).
    /// If provided, inputs are validated against this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<serde_json::Value>,
    /// JSON Schema for the output manifest (optional).
    /// If provided, outputs are validated against this schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    /// Description of the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Author of the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// License of the extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    /// Homepage URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Repository URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
}

impl ExtensionManifest {
    /// Creates a new subprocess-based extension manifest.
    pub fn subprocess(
        name: impl Into<String>,
        version: impl Into<String>,
        executable: impl Into<String>,
        recipe_kinds: Vec<String>,
        determinism: DeterminismLevel,
    ) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            tier: determinism.tier(),
            determinism,
            interface: ExtensionInterface::Subprocess {
                executable: executable.into(),
                args: vec![],
                env: HashMap::new(),
                timeout_seconds: default_timeout(),
            },
            recipe_kinds,
            input_schema: None,
            output_schema: None,
            description: None,
            author: None,
            license: None,
            homepage: None,
            repository: None,
        }
    }

    /// Returns true if this extension handles the given recipe kind.
    pub fn handles_recipe(&self, recipe_kind: &str) -> bool {
        self.recipe_kinds.iter().any(|k| k == recipe_kind)
    }

    /// Returns the timeout in seconds for this extension.
    pub fn timeout_seconds(&self) -> u64 {
        match &self.interface {
            ExtensionInterface::Subprocess {
                timeout_seconds, ..
            } => *timeout_seconds,
            ExtensionInterface::Wasm {
                timeout_seconds, ..
            } => *timeout_seconds,
        }
    }
}

/// Validation errors for extension manifests.
#[derive(Debug, Clone, PartialEq)]
pub enum ManifestValidationError {
    /// Extension name is invalid.
    InvalidName(String),
    /// Version string is invalid.
    InvalidVersion(String),
    /// Tier doesn't match determinism level.
    TierMismatch { declared: u8, expected: u8 },
    /// No recipe kinds specified.
    NoRecipeKinds,
    /// Recipe kind is invalid.
    InvalidRecipeKind(String),
    /// Executable path is empty.
    EmptyExecutable,
    /// WASM module path is empty.
    EmptyWasmPath,
    /// Timeout is too short.
    TimeoutTooShort(u64),
}

impl std::fmt::Display for ManifestValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "Invalid extension name: {}", name),
            Self::InvalidVersion(version) => write!(f, "Invalid version string: {}", version),
            Self::TierMismatch { declared, expected } => {
                write!(
                    f,
                    "Tier mismatch: declared {}, expected {} for determinism level",
                    declared, expected
                )
            }
            Self::NoRecipeKinds => write!(f, "No recipe kinds specified"),
            Self::InvalidRecipeKind(kind) => write!(f, "Invalid recipe kind: {}", kind),
            Self::EmptyExecutable => write!(f, "Executable path is empty"),
            Self::EmptyWasmPath => write!(f, "WASM module path is empty"),
            Self::TimeoutTooShort(seconds) => {
                write!(f, "Timeout too short: {} seconds (minimum 1)", seconds)
            }
        }
    }
}

impl std::error::Error for ManifestValidationError {}

/// Validates an extension manifest.
pub fn validate_extension_manifest(
    manifest: &ExtensionManifest,
) -> Result<(), Vec<ManifestValidationError>> {
    let mut errors = Vec::new();

    // Validate name (lowercase alphanumeric with hyphens)
    if manifest.name.is_empty() || !is_valid_extension_name(&manifest.name) {
        errors.push(ManifestValidationError::InvalidName(manifest.name.clone()));
    }

    // Validate version (semver-like)
    if manifest.version.is_empty() || !is_valid_version(&manifest.version) {
        errors.push(ManifestValidationError::InvalidVersion(
            manifest.version.clone(),
        ));
    }

    // Validate tier matches determinism level
    let expected_tier = manifest.determinism.tier();
    if manifest.tier != expected_tier {
        errors.push(ManifestValidationError::TierMismatch {
            declared: manifest.tier,
            expected: expected_tier,
        });
    }

    // Validate recipe kinds
    if manifest.recipe_kinds.is_empty() {
        errors.push(ManifestValidationError::NoRecipeKinds);
    }
    for kind in &manifest.recipe_kinds {
        if !is_valid_recipe_kind(kind) {
            errors.push(ManifestValidationError::InvalidRecipeKind(kind.clone()));
        }
    }

    // Validate interface-specific fields
    match &manifest.interface {
        ExtensionInterface::Subprocess {
            executable,
            timeout_seconds,
            ..
        } => {
            if executable.is_empty() {
                errors.push(ManifestValidationError::EmptyExecutable);
            }
            if *timeout_seconds == 0 {
                errors.push(ManifestValidationError::TimeoutTooShort(*timeout_seconds));
            }
        }
        ExtensionInterface::Wasm {
            module_path,
            timeout_seconds,
            ..
        } => {
            if module_path.is_empty() {
                errors.push(ManifestValidationError::EmptyWasmPath);
            }
            if *timeout_seconds == 0 {
                errors.push(ManifestValidationError::TimeoutTooShort(*timeout_seconds));
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Validates an extension name.
/// Names must be lowercase alphanumeric with hyphens, 2-64 characters.
fn is_valid_extension_name(name: &str) -> bool {
    if name.len() < 2 || name.len() > 64 {
        return false;
    }
    let first_char = name.chars().next().unwrap();
    if !first_char.is_ascii_lowercase() {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !name.ends_with('-')
        && !name.contains("--")
}

/// Validates a version string.
/// Must be semver-like: major.minor.patch with optional prerelease.
fn is_valid_version(version: &str) -> bool {
    // Simple validation: at least "X.Y.Z" format
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    // First two parts must be numeric
    for part in parts.iter().take(2) {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }
    true
}

/// Validates a recipe kind.
/// Recipe kinds follow the pattern: category.name_v1 (e.g., texture.custom_v1)
fn is_valid_recipe_kind(kind: &str) -> bool {
    if kind.is_empty() || !kind.contains('.') {
        return false;
    }
    let parts: Vec<&str> = kind.split('.').collect();
    if parts.len() < 2 {
        return false;
    }
    // Each part must be valid identifier
    parts.iter().all(|part| {
        !part.is_empty()
            && part.chars().next().unwrap().is_ascii_lowercase()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    })
}
