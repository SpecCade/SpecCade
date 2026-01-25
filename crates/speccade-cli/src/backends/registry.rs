//! Extension registry for managing external backends.

use speccade_spec::extension::{ExtensionManifest, validate_extension_manifest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Registry for managing external backend extensions.
#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    /// Extensions indexed by name.
    extensions: HashMap<String, ExtensionManifest>,
    /// Recipe kind to extension name mapping.
    recipe_map: HashMap<String, String>,
    /// Search paths for extension manifests.
    search_paths: Vec<PathBuf>,
}

/// Errors that can occur during registry operations.
#[derive(Debug, Clone, PartialEq)]
pub enum RegistryError {
    /// Extension manifest is invalid.
    InvalidManifest(String),
    /// Extension with this name already registered.
    AlreadyRegistered(String),
    /// Recipe kind already claimed by another extension.
    RecipeKindConflict { kind: String, existing: String },
    /// Extension not found.
    NotFound(String),
    /// Failed to read manifest file.
    ReadError(String),
    /// Failed to parse manifest JSON.
    ParseError(String),
}

impl std::fmt::Display for RegistryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest(msg) => write!(f, "Invalid extension manifest: {}", msg),
            Self::AlreadyRegistered(name) => write!(f, "Extension already registered: {}", name),
            Self::RecipeKindConflict { kind, existing } => {
                write!(f, "Recipe kind '{}' already claimed by extension '{}'", kind, existing)
            }
            Self::NotFound(name) => write!(f, "Extension not found: {}", name),
            Self::ReadError(msg) => write!(f, "Failed to read manifest: {}", msg),
            Self::ParseError(msg) => write!(f, "Failed to parse manifest: {}", msg),
        }
    }
}

impl std::error::Error for RegistryError {}

impl ExtensionRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a registry with default search paths.
    pub fn with_default_paths() -> Self {
        let mut registry = Self::new();

        // Add default search paths
        // 1. Current directory extensions/
        if let Ok(cwd) = std::env::current_dir() {
            registry.search_paths.push(cwd.join("extensions"));
        }

        // 2. User config directory
        if let Some(config_dir) = dirs::config_dir() {
            registry.search_paths.push(config_dir.join("speccade").join("extensions"));
        }

        // 3. System-wide (Unix-like)
        #[cfg(unix)]
        {
            registry.search_paths.push(PathBuf::from("/usr/share/speccade/extensions"));
            registry.search_paths.push(PathBuf::from("/usr/local/share/speccade/extensions"));
        }

        registry
    }

    /// Adds a search path for extension manifests.
    pub fn add_search_path(&mut self, path: impl Into<PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Registers an extension manifest.
    pub fn register(&mut self, manifest: ExtensionManifest) -> Result<(), RegistryError> {
        // Validate the manifest
        if let Err(errors) = validate_extension_manifest(&manifest) {
            let msg = errors.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; ");
            return Err(RegistryError::InvalidManifest(msg));
        }

        // Check for name conflicts
        if self.extensions.contains_key(&manifest.name) {
            return Err(RegistryError::AlreadyRegistered(manifest.name.clone()));
        }

        // Check for recipe kind conflicts
        for kind in &manifest.recipe_kinds {
            if let Some(existing) = self.recipe_map.get(kind) {
                return Err(RegistryError::RecipeKindConflict {
                    kind: kind.clone(),
                    existing: existing.clone(),
                });
            }
        }

        // Register recipe kinds
        for kind in &manifest.recipe_kinds {
            self.recipe_map.insert(kind.clone(), manifest.name.clone());
        }

        // Register extension
        self.extensions.insert(manifest.name.clone(), manifest);

        Ok(())
    }

    /// Loads and registers an extension from a manifest file.
    pub fn load_manifest(&mut self, path: &Path) -> Result<(), RegistryError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| RegistryError::ReadError(format!("{}: {}", path.display(), e)))?;

        let manifest: ExtensionManifest = serde_json::from_str(&content)
            .map_err(|e| RegistryError::ParseError(format!("{}: {}", path.display(), e)))?;

        self.register(manifest)
    }

    /// Discovers and loads all extensions from search paths.
    pub fn discover(&mut self) -> Vec<RegistryError> {
        let mut errors = Vec::new();

        for search_path in self.search_paths.clone() {
            if !search_path.exists() {
                continue;
            }

            // Look for manifest.json files in subdirectories
            if let Ok(entries) = std::fs::read_dir(&search_path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let manifest_path = entry.path().join("manifest.json");
                    if manifest_path.exists() {
                        if let Err(e) = self.load_manifest(&manifest_path) {
                            errors.push(e);
                        }
                    }
                }
            }
        }

        errors
    }

    /// Gets an extension by name.
    pub fn get(&self, name: &str) -> Option<&ExtensionManifest> {
        self.extensions.get(name)
    }

    /// Gets the extension that handles a recipe kind.
    pub fn get_for_recipe(&self, recipe_kind: &str) -> Option<&ExtensionManifest> {
        self.recipe_map.get(recipe_kind)
            .and_then(|name| self.extensions.get(name))
    }

    /// Returns true if an extension is registered for the recipe kind.
    pub fn has_extension_for(&self, recipe_kind: &str) -> bool {
        self.recipe_map.contains_key(recipe_kind)
    }

    /// Lists all registered extensions.
    pub fn list(&self) -> impl Iterator<Item = &ExtensionManifest> {
        self.extensions.values()
    }

    /// Lists all registered recipe kinds.
    pub fn recipe_kinds(&self) -> impl Iterator<Item = &str> {
        self.recipe_map.keys().map(|s| s.as_str())
    }

    /// Unregisters an extension by name.
    pub fn unregister(&mut self, name: &str) -> Result<ExtensionManifest, RegistryError> {
        let manifest = self.extensions.remove(name)
            .ok_or_else(|| RegistryError::NotFound(name.to_string()))?;

        // Remove recipe kind mappings
        for kind in &manifest.recipe_kinds {
            self.recipe_map.remove(kind);
        }

        Ok(manifest)
    }

    /// Returns the number of registered extensions.
    pub fn len(&self) -> usize {
        self.extensions.len()
    }

    /// Returns true if no extensions are registered.
    pub fn is_empty(&self) -> bool {
        self.extensions.is_empty()
    }
}
