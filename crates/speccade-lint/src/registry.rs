//! Rule registry for managing lint rules.

use crate::report::{AssetType, LintReport};
use crate::rules::{audio, mesh, music, texture, AssetData, LintRule};
use speccade_spec::Spec;
use std::collections::HashSet;
use std::path::Path;

/// Registry of all available lint rules.
pub struct RuleRegistry {
    rules: Vec<Box<dyn LintRule>>,
    disabled_rules: HashSet<String>,
    enabled_only: Option<HashSet<String>>,
}

impl RuleRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            disabled_rules: HashSet::new(),
            enabled_only: None,
        }
    }

    /// Creates a registry with all default rules registered.
    pub fn default_rules() -> Self {
        let mut registry = Self::new();

        // Register all domain-specific rules
        for rule in audio::all_rules() {
            registry.register(rule);
        }
        for rule in texture::all_rules() {
            registry.register(rule);
        }
        for rule in mesh::all_rules() {
            registry.register(rule);
        }
        for rule in music::all_rules() {
            registry.register(rule);
        }

        registry
    }

    /// Registers a new lint rule.
    pub fn register(&mut self, rule: Box<dyn LintRule>) {
        self.rules.push(rule);
    }

    /// Disables a rule by ID.
    pub fn disable_rule(&mut self, rule_id: &str) {
        self.disabled_rules.insert(rule_id.to_string());
    }

    /// Enables only the specified rules (disables all others).
    pub fn enable_only(&mut self, rule_ids: &[&str]) {
        self.enabled_only = Some(rule_ids.iter().map(|s| s.to_string()).collect());
    }

    /// Returns all registered rules.
    pub fn rules(&self) -> &[Box<dyn LintRule>] {
        &self.rules
    }

    /// Returns rule metadata for documentation/introspection.
    pub fn rule_metadata(&self) -> Vec<RuleMetadata> {
        self.rules
            .iter()
            .map(|r| RuleMetadata {
                id: r.id().to_string(),
                description: r.description().to_string(),
                severity: r.default_severity(),
                applies_to: r.applies_to().to_vec(),
            })
            .collect()
    }

    /// Returns the number of registered rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Returns true if no rules are registered.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Checks if a rule is enabled.
    fn is_rule_enabled(&self, rule_id: &str) -> bool {
        if self.disabled_rules.contains(rule_id) {
            return false;
        }
        if let Some(ref enabled) = self.enabled_only {
            return enabled.contains(rule_id);
        }
        true
    }

    /// Determines asset type from file extension.
    fn asset_type_from_path(path: &Path) -> Option<AssetType> {
        let ext = path.extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "wav" | "ogg" | "mp3" | "flac" => Some(AssetType::Audio),
            "png" | "jpg" | "jpeg" | "tga" | "bmp" => Some(AssetType::Texture),
            "glb" | "gltf" | "obj" | "fbx" => Some(AssetType::Mesh),
            "xm" | "mod" | "it" | "s3m" => Some(AssetType::Music),
            _ => None,
        }
    }

    /// Runs all applicable rules on the given asset.
    ///
    /// Returns a `LintReport` containing all detected issues.
    pub fn lint(&self, asset_path: &Path, spec: Option<&Spec>) -> Result<LintReport, std::io::Error> {
        let bytes = std::fs::read(asset_path)?;
        self.lint_bytes(asset_path, &bytes, spec)
    }

    /// Runs all applicable rules on asset data already in memory.
    pub fn lint_bytes(
        &self,
        asset_path: &Path,
        bytes: &[u8],
        spec: Option<&Spec>,
    ) -> Result<LintReport, std::io::Error> {
        let mut report = LintReport::new();

        let asset_type = match Self::asset_type_from_path(asset_path) {
            Some(t) => t,
            None => return Ok(report), // Unknown asset type, no rules apply
        };

        let asset_data = AssetData {
            path: asset_path,
            bytes,
        };

        for rule in &self.rules {
            // Skip disabled rules
            if !self.is_rule_enabled(rule.id()) {
                continue;
            }

            // Skip rules that don't apply to this asset type
            if !rule.applies_to().contains(&asset_type) {
                continue;
            }

            // Run the rule and collect issues
            let issues = rule.check(&asset_data, spec);
            for issue in issues {
                report.add_issue(issue);
            }
        }

        Ok(report)
    }
}

impl Default for RuleRegistry {
    fn default() -> Self {
        Self::default_rules()
    }
}

/// Metadata about a lint rule for documentation/introspection.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RuleMetadata {
    /// Rule identifier.
    pub id: String,
    /// Human-readable description.
    pub description: String,
    /// Default severity level.
    pub severity: crate::report::Severity,
    /// Asset types this rule applies to.
    pub applies_to: Vec<AssetType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_registry() {
        let registry = RuleRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_default_registry() {
        let registry = RuleRegistry::default_rules();
        // Should have all rules from all domains registered
        // Audio: 10, Texture: 10, Mesh: 12, Music: 12 = 44 total
        assert_eq!(registry.len(), 44);
    }

    #[test]
    fn test_disable_rule() {
        let mut registry = RuleRegistry::new();
        registry.disable_rule("audio/clipping");
        assert!(!registry.is_rule_enabled("audio/clipping"));
        assert!(registry.is_rule_enabled("audio/silence"));
    }

    #[test]
    fn test_enable_only() {
        let mut registry = RuleRegistry::new();
        registry.enable_only(&["audio/clipping", "audio/silence"]);
        assert!(registry.is_rule_enabled("audio/clipping"));
        assert!(registry.is_rule_enabled("audio/silence"));
        assert!(!registry.is_rule_enabled("audio/too-quiet"));
    }

    #[test]
    fn test_asset_type_detection() {
        assert_eq!(
            RuleRegistry::asset_type_from_path(Path::new("sound.wav")),
            Some(AssetType::Audio)
        );
        assert_eq!(
            RuleRegistry::asset_type_from_path(Path::new("texture.png")),
            Some(AssetType::Texture)
        );
        assert_eq!(
            RuleRegistry::asset_type_from_path(Path::new("model.glb")),
            Some(AssetType::Mesh)
        );
        assert_eq!(
            RuleRegistry::asset_type_from_path(Path::new("track.xm")),
            Some(AssetType::Music)
        );
        assert_eq!(
            RuleRegistry::asset_type_from_path(Path::new("file.unknown")),
            None
        );
    }
}
