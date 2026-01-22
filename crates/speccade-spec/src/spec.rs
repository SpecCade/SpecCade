//! Main spec types.

use serde::{Deserialize, Serialize};

use crate::output::{EngineTarget, OutputSpec, VariantSpec};
use crate::recipe::Recipe;

/// Current spec version.
pub const SPEC_VERSION: u32 = 1;

/// Maximum valid seed value (2^32 - 1).
pub const MAX_SEED: u32 = u32::MAX;

/// Asset types supported by SpecCade v1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// Audio assets (SFX, instruments, samples).
    Audio,
    /// Tracker modules (XM, IT).
    Music,
    /// 2D texture maps (PNG).
    Texture,
    /// Sprite sheets and sprite animations.
    Sprite,
    /// VFX flipbook animations.
    Vfx,
    /// UI elements (nine-slice panels, icon sets).
    Ui,
    /// Bitmap and MSDF fonts with glyph metrics.
    Font,
    /// Non-skinned 3D meshes (GLB).
    StaticMesh,
    /// Skinned meshes with skeleton (GLB).
    SkeletalMesh,
    /// Animation clips (GLB).
    SkeletalAnimation,
}

impl AssetType {
    /// Returns the asset type as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::Audio => "audio",
            AssetType::Music => "music",
            AssetType::Texture => "texture",
            AssetType::Sprite => "sprite",
            AssetType::Vfx => "vfx",
            AssetType::Ui => "ui",
            AssetType::Font => "font",
            AssetType::StaticMesh => "static_mesh",
            AssetType::SkeletalMesh => "skeletal_mesh",
            AssetType::SkeletalAnimation => "skeletal_animation",
        }
    }

    /// Checks if a recipe kind is compatible with this asset type.
    pub fn is_compatible_recipe(&self, recipe_kind: &str) -> bool {
        let prefix = if recipe_kind.contains('.') {
            recipe_kind.split('.').next()
        } else {
            // `audio_v1` uses an underscore delimiter; most other recipe kinds are `asset_type.*`.
            recipe_kind.split('_').next()
        };

        prefix == Some(self.as_str())
    }

    /// Returns all asset types.
    pub fn all() -> &'static [AssetType] {
        &[
            AssetType::Audio,
            AssetType::Music,
            AssetType::Texture,
            AssetType::Sprite,
            AssetType::Vfx,
            AssetType::Ui,
            AssetType::Font,
            AssetType::StaticMesh,
            AssetType::SkeletalMesh,
            AssetType::SkeletalAnimation,
        ]
    }
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AssetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "audio" => Ok(AssetType::Audio),
            "music" => Ok(AssetType::Music),
            "texture" => Ok(AssetType::Texture),
            "sprite" => Ok(AssetType::Sprite),
            "vfx" => Ok(AssetType::Vfx),
            "ui" => Ok(AssetType::Ui),
            "font" => Ok(AssetType::Font),
            "static_mesh" => Ok(AssetType::StaticMesh),
            "skeletal_mesh" => Ok(AssetType::SkeletalMesh),
            "skeletal_animation" => Ok(AssetType::SkeletalAnimation),
            _ => Err(format!("unknown asset type: {}", s)),
        }
    }
}

/// A SpecCade canonical spec.
///
/// This is the top-level spec structure that represents a complete asset
/// specification. It contains both contract fields (metadata and outputs)
/// and an optional recipe for generation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// Schema version; must be 1 for v1 specs.
    pub spec_version: u32,

    /// Stable identifier for the asset.
    /// Format: `[a-z][a-z0-9_-]{2,63}`
    pub asset_id: String,

    /// Type of asset to generate.
    pub asset_type: AssetType,

    /// License identifier (SPDX recommended, e.g., "CC0-1.0").
    pub license: String,

    /// RNG seed for deterministic generation.
    /// Range: 0 to 2^32-1 (4294967295).
    pub seed: u32,

    /// Expected output artifacts.
    pub outputs: Vec<OutputSpec>,

    /// Human-readable description of the asset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Semantic tags for filtering/search.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style_tags: Option<Vec<String>>,

    /// Target game engines.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_targets: Option<Vec<EngineTarget>>,

    /// Migration notes (e.g., from legacy `.spec.py` conversion).
    ///
    /// This field is informational and ignored by generators.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration_notes: Option<Vec<String>>,

    /// Variant specifications for procedural variations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<VariantSpec>>,

    /// Recipe for asset generation (required for `generate` command).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<Recipe>,
}

impl Spec {
    /// Creates a new spec builder.
    pub fn builder(asset_id: impl Into<String>, asset_type: AssetType) -> SpecBuilder {
        SpecBuilder::new(asset_id, asset_type)
    }

    /// Parses a spec from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Parses a spec from a JSON value.
    pub fn from_value(value: serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }

    /// Serializes the spec to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Serializes the spec to pretty-printed JSON string.
    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Serializes the spec to a JSON value.
    pub fn to_value(&self) -> Result<serde_json::Value, serde_json::Error> {
        serde_json::to_value(self)
    }

    /// Returns true if the spec has a recipe.
    pub fn has_recipe(&self) -> bool {
        self.recipe.is_some()
    }

    /// Returns true if the spec has at least one primary output.
    pub fn has_primary_output(&self) -> bool {
        self.outputs
            .iter()
            .any(|o| o.kind == crate::output::OutputKind::Primary)
    }

    /// Returns all primary outputs.
    pub fn primary_outputs(&self) -> impl Iterator<Item = &OutputSpec> {
        self.outputs
            .iter()
            .filter(|o| o.kind == crate::output::OutputKind::Primary)
    }

    /// Returns the number of outputs.
    pub fn output_count(&self) -> usize {
        self.outputs.len()
    }

    /// Returns all unique output paths.
    pub fn output_paths(&self) -> impl Iterator<Item = &str> {
        self.outputs.iter().map(|o| o.path.as_str())
    }
}

/// Builder for constructing Spec instances.
#[derive(Debug, Clone)]
pub struct SpecBuilder {
    asset_id: String,
    asset_type: AssetType,
    license: String,
    seed: u32,
    outputs: Vec<OutputSpec>,
    description: Option<String>,
    style_tags: Option<Vec<String>>,
    engine_targets: Option<Vec<EngineTarget>>,
    migration_notes: Option<Vec<String>>,
    variants: Option<Vec<VariantSpec>>,
    recipe: Option<Recipe>,
}

impl SpecBuilder {
    /// Creates a new spec builder.
    pub fn new(asset_id: impl Into<String>, asset_type: AssetType) -> Self {
        Self {
            asset_id: asset_id.into(),
            asset_type,
            license: String::new(),
            seed: 0,
            outputs: Vec::new(),
            description: None,
            style_tags: None,
            engine_targets: None,
            migration_notes: None,
            variants: None,
            recipe: None,
        }
    }

    /// Sets the license.
    pub fn license(mut self, license: impl Into<String>) -> Self {
        self.license = license.into();
        self
    }

    /// Sets the seed.
    pub fn seed(mut self, seed: u32) -> Self {
        self.seed = seed;
        self
    }

    /// Adds an output.
    pub fn output(mut self, output: OutputSpec) -> Self {
        self.outputs.push(output);
        self
    }

    /// Sets all outputs.
    pub fn outputs(mut self, outputs: Vec<OutputSpec>) -> Self {
        self.outputs = outputs;
        self
    }

    /// Sets the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the style tags.
    pub fn style_tags(mut self, tags: Vec<String>) -> Self {
        self.style_tags = Some(tags);
        self
    }

    /// Adds a style tag.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.style_tags
            .get_or_insert_with(Vec::new)
            .push(tag.into());
        self
    }

    /// Sets the engine targets.
    pub fn engine_targets(mut self, targets: Vec<EngineTarget>) -> Self {
        self.engine_targets = Some(targets);
        self
    }

    /// Adds an engine target.
    pub fn target(mut self, target: EngineTarget) -> Self {
        self.engine_targets
            .get_or_insert_with(Vec::new)
            .push(target);
        self
    }

    /// Sets migration notes (informational metadata).
    pub fn migration_notes(mut self, notes: Vec<String>) -> Self {
        self.migration_notes = Some(notes);
        self
    }

    /// Adds a migration note.
    pub fn migration_note(mut self, note: impl Into<String>) -> Self {
        self.migration_notes
            .get_or_insert_with(Vec::new)
            .push(note.into());
        self
    }

    /// Sets the variants.
    pub fn variants(mut self, variants: Vec<VariantSpec>) -> Self {
        self.variants = Some(variants);
        self
    }

    /// Adds a variant.
    pub fn variant(mut self, variant: VariantSpec) -> Self {
        self.variants.get_or_insert_with(Vec::new).push(variant);
        self
    }

    /// Sets the recipe.
    pub fn recipe(mut self, recipe: Recipe) -> Self {
        self.recipe = Some(recipe);
        self
    }

    /// Builds the spec.
    pub fn build(self) -> Spec {
        Spec {
            spec_version: SPEC_VERSION,
            asset_id: self.asset_id,
            asset_type: self.asset_type,
            license: self.license,
            seed: self.seed,
            outputs: self.outputs,
            description: self.description,
            style_tags: self.style_tags,
            engine_targets: self.engine_targets,
            migration_notes: self.migration_notes,
            variants: self.variants,
            recipe: self.recipe,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::output::{OutputFormat, OutputKind};

    #[test]
    fn test_asset_type_serde() {
        let at = AssetType::Audio;
        let json = serde_json::to_string(&at).unwrap();
        assert_eq!(json, "\"audio\"");

        let parsed: AssetType = serde_json::from_str("\"texture\"").unwrap();
        assert_eq!(parsed, AssetType::Texture);
    }

    #[test]
    fn test_asset_type_is_compatible_recipe() {
        assert!(AssetType::Audio.is_compatible_recipe("audio_v1"));
        assert!(!AssetType::Audio.is_compatible_recipe("music.tracker_song_v1"));
        assert!(AssetType::Music.is_compatible_recipe("music.tracker_song_compose_v1"));
        assert!(AssetType::Texture.is_compatible_recipe("texture.procedural_v1"));
    }

    #[test]
    fn test_spec_builder() {
        let spec = Spec::builder("laser-blast-01", AssetType::Audio)
            .license("CC0-1.0")
            .seed(42)
            .description("Sci-fi laser blast sound effect")
            .tag("retro")
            .tag("scifi")
            .output(OutputSpec::primary(
                OutputFormat::Wav,
                "sounds/laser_blast_01.wav",
            ))
            .build();

        assert_eq!(spec.spec_version, 1);
        assert_eq!(spec.asset_id, "laser-blast-01");
        assert_eq!(spec.asset_type, AssetType::Audio);
        assert_eq!(spec.license, "CC0-1.0");
        assert_eq!(spec.seed, 42);
        assert!(spec.description.is_some());
        assert_eq!(spec.style_tags.as_ref().unwrap().len(), 2);
        assert_eq!(spec.outputs.len(), 1);
        assert!(spec.has_primary_output());
    }

    #[test]
    fn test_spec_from_json() {
        let json = r#"{
            "spec_version": 1,
            "asset_id": "test-asset-01",
            "asset_type": "audio",
            "license": "CC0-1.0",
            "seed": 42,
            "outputs": [
                {
                    "kind": "primary",
                    "format": "wav",
                    "path": "sounds/test.wav"
                }
            ]
        }"#;

        let spec = Spec::from_json(json).unwrap();
        assert_eq!(spec.asset_id, "test-asset-01");
        assert_eq!(spec.asset_type, AssetType::Audio);
        assert_eq!(spec.seed, 42);
        assert!(spec.has_primary_output());
    }

    #[test]
    fn test_spec_to_json() {
        let spec = Spec::builder("test-01", AssetType::Music)
            .license("CC-BY-4.0")
            .seed(12345)
            .output(OutputSpec::primary(OutputFormat::Xm, "music/test.xm"))
            .build();

        let json = spec.to_json().unwrap();
        assert!(json.contains("test-01"));
        assert!(json.contains("music"));
        assert!(json.contains("12345"));
    }

    #[test]
    fn test_spec_primary_outputs() {
        let spec = Spec::builder("texture-01", AssetType::Texture)
            .license("CC0-1.0")
            .seed(0)
            .output(OutputSpec::primary(
                OutputFormat::Png,
                "textures/albedo.png",
            ))
            .output(OutputSpec::primary(
                OutputFormat::Png,
                "textures/normal.png",
            ))
            .output(OutputSpec::new(
                OutputKind::Metadata,
                OutputFormat::Json,
                "textures/meta.json",
            ))
            .build();

        let primary_count = spec.primary_outputs().count();
        assert_eq!(primary_count, 2);
        assert_eq!(spec.output_count(), 3);
    }
}
