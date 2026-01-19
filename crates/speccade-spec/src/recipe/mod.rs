//! Recipe types for different asset backends.
//!
//! Each recipe kind corresponds to a specific backend and asset type.
//! The recipe defines the parameters for how to generate the asset.

pub mod animation;
pub mod audio;
pub mod character;
pub mod mesh;
pub mod music;
pub mod texture;

pub use animation::*;
pub use audio::*;
pub use character::*;
pub use mesh::*;
pub use music::*;
pub use texture::*;

use serde::{Deserialize, Serialize};

/// Recipe kind identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeKind {
    /// `audio_v1` - Unified audio synthesis.
    #[serde(rename = "audio_v1")]
    AudioV1,
    /// `music.tracker_song_v1` - Tracker module song.
    #[serde(rename = "music.tracker_song_v1")]
    MusicTrackerSongV1,
    /// `music.tracker_song_compose_v1` - Tracker module song (Pattern IR).
    #[serde(rename = "music.tracker_song_compose_v1")]
    MusicTrackerSongComposeV1,
    /// `texture.procedural_v1` - Unified procedural texture graph.
    #[serde(rename = "texture.procedural_v1")]
    TextureProceduralV1,
    /// `static_mesh.blender_primitives_v1` - Static mesh from Blender primitives.
    #[serde(rename = "static_mesh.blender_primitives_v1")]
    StaticMeshBlenderPrimitivesV1,
    /// `skeletal_mesh.blender_rigged_mesh_v1` - Rigged skeletal mesh.
    #[serde(rename = "skeletal_mesh.blender_rigged_mesh_v1")]
    SkeletalMeshBlenderRiggedMeshV1,
    /// `skeletal_animation.blender_clip_v1` - Skeletal animation clip (simple keyframes).
    #[serde(rename = "skeletal_animation.blender_clip_v1")]
    SkeletalAnimationBlenderClipV1,
    /// `skeletal_animation.blender_rigged_v1` - Skeletal animation with IK rigging support.
    #[serde(rename = "skeletal_animation.blender_rigged_v1")]
    SkeletalAnimationBlenderRiggedV1,
}

impl RecipeKind {
    /// Returns the recipe kind as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeKind::AudioV1 => "audio_v1",
            RecipeKind::MusicTrackerSongV1 => "music.tracker_song_v1",
            RecipeKind::MusicTrackerSongComposeV1 => "music.tracker_song_compose_v1",
            RecipeKind::TextureProceduralV1 => "texture.procedural_v1",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh.blender_primitives_v1",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh.blender_rigged_mesh_v1",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation.blender_clip_v1",
            RecipeKind::SkeletalAnimationBlenderRiggedV1 => "skeletal_animation.blender_rigged_v1",
        }
    }

    /// Returns the asset type prefix for this recipe kind.
    pub fn asset_type_prefix(&self) -> &'static str {
        match self {
            RecipeKind::AudioV1 => "audio",
            RecipeKind::MusicTrackerSongV1 => "music",
            RecipeKind::MusicTrackerSongComposeV1 => "music",
            RecipeKind::TextureProceduralV1 => "texture",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation",
            RecipeKind::SkeletalAnimationBlenderRiggedV1 => "skeletal_animation",
        }
    }

    /// Returns whether this is a Tier 1 (deterministic hash) or Tier 2 (metric validation) backend.
    pub fn is_tier1(&self) -> bool {
        match self {
            RecipeKind::AudioV1
            | RecipeKind::MusicTrackerSongV1
            | RecipeKind::MusicTrackerSongComposeV1
            | RecipeKind::TextureProceduralV1 => true,
            RecipeKind::StaticMeshBlenderPrimitivesV1
            | RecipeKind::SkeletalMeshBlenderRiggedMeshV1
            | RecipeKind::SkeletalAnimationBlenderClipV1
            | RecipeKind::SkeletalAnimationBlenderRiggedV1 => false,
        }
    }
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Recipe specification containing kind and params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recipe {
    /// The recipe kind identifier.
    pub kind: String,
    /// Recipe-specific parameters.
    pub params: serde_json::Value,
}

impl Recipe {
    /// Creates a new recipe with the given kind and parameters.
    pub fn new(kind: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            kind: kind.into(),
            params,
        }
    }

    /// Parses the recipe kind into a typed enum.
    pub fn parse_kind(&self) -> Option<RecipeKind> {
        match self.kind.as_str() {
            "audio_v1" => Some(RecipeKind::AudioV1),
            "music.tracker_song_v1" => Some(RecipeKind::MusicTrackerSongV1),
            "music.tracker_song_compose_v1" => Some(RecipeKind::MusicTrackerSongComposeV1),
            "texture.procedural_v1" => Some(RecipeKind::TextureProceduralV1),
            "static_mesh.blender_primitives_v1" => Some(RecipeKind::StaticMeshBlenderPrimitivesV1),
            "skeletal_mesh.blender_rigged_mesh_v1" => {
                Some(RecipeKind::SkeletalMeshBlenderRiggedMeshV1)
            }
            "skeletal_animation.blender_clip_v1" => {
                Some(RecipeKind::SkeletalAnimationBlenderClipV1)
            }
            "skeletal_animation.blender_rigged_v1" => {
                Some(RecipeKind::SkeletalAnimationBlenderRiggedV1)
            }
            _ => None,
        }
    }

    /// Returns the asset type prefix from the recipe kind.
    /// Handles both dot format (e.g., "texture.procedural_v1") and underscore format (e.g., "audio_v1").
    pub fn asset_type_prefix(&self) -> Option<&str> {
        // First try dot format (e.g., "texture.procedural_v1" -> "texture")
        if self.kind.contains('.') {
            self.kind.split('.').next()
        } else {
            // Fall back to underscore format (e.g., "audio_v1" -> "audio")
            self.kind.split('_').next()
        }
    }

    /// Attempts to parse params as unified audio params.
    pub fn as_audio(&self) -> Result<AudioV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as music tracker song params.
    pub fn as_music_tracker_song(&self) -> Result<MusicTrackerSongV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as music tracker compose params.
    pub fn as_music_tracker_song_compose(
        &self,
    ) -> Result<MusicTrackerSongComposeV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as procedural texture params.
    pub fn as_texture_procedural(&self) -> Result<TextureProceduralV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as static mesh Blender primitives params.
    pub fn as_static_mesh_blender_primitives(
        &self,
    ) -> Result<StaticMeshBlenderPrimitivesV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as skeletal mesh Blender rigged mesh params.
    pub fn as_skeletal_mesh_blender_rigged_mesh(
        &self,
    ) -> Result<SkeletalMeshBlenderRiggedMeshV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as skeletal animation Blender clip params.
    pub fn as_skeletal_animation_blender_clip(
        &self,
    ) -> Result<SkeletalAnimationBlenderClipV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as skeletal animation Blender rigged params (with IK support).
    pub fn as_skeletal_animation_blender_rigged(
        &self,
    ) -> Result<SkeletalAnimationBlenderRiggedV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse recipe params according to the recipe kind.
    ///
    /// This method validates that the params match the expected schema for the
    /// recipe kind. Unknown fields will be rejected due to `deny_unknown_fields`
    /// on the param types.
    ///
    /// # Returns
    ///
    /// - `Ok(())` if params parse successfully
    /// - `Err(RecipeParamsError)` with details about the parsing failure
    pub fn try_parse_params(&self) -> Result<(), RecipeParamsError> {
        match self.kind.as_str() {
            "audio_v1" => {
                self.as_audio().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "music.tracker_song_v1" => {
                self.as_music_tracker_song().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "music.tracker_song_compose_v1" => {
                self.as_music_tracker_song_compose()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "texture.procedural_v1" => {
                self.as_texture_procedural()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "static_mesh.blender_primitives_v1" => {
                self.as_static_mesh_blender_primitives()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "skeletal_mesh.blender_rigged_mesh_v1" => {
                self.as_skeletal_mesh_blender_rigged_mesh()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "skeletal_animation.blender_clip_v1" => {
                self.as_skeletal_animation_blender_clip()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "skeletal_animation.blender_rigged_v1" => {
                self.as_skeletal_animation_blender_rigged()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            _ => {
                // Unknown recipe kind - we don't validate params for unrecognized kinds
                // as they may be handled by external backends
            }
        }
        Ok(())
    }
}

/// Error returned when recipe params fail to parse.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeParamsError {
    /// The recipe kind that failed to parse.
    pub recipe_kind: String,
    /// The error message from serde.
    pub error_message: String,
}

impl std::fmt::Display for RecipeParamsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid params for {}: {}",
            self.recipe_kind, self.error_message
        )
    }
}

impl std::error::Error for RecipeParamsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_kind_serde() {
        let kind = RecipeKind::AudioV1;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"audio_v1\"");

        let parsed: RecipeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }

    #[test]
    fn test_recipe_kind_asset_type_prefix() {
        assert_eq!(RecipeKind::AudioV1.asset_type_prefix(), "audio");
        assert_eq!(
            RecipeKind::MusicTrackerSongComposeV1.asset_type_prefix(),
            "music"
        );
        assert_eq!(
            RecipeKind::TextureProceduralV1.asset_type_prefix(),
            "texture"
        );
        assert_eq!(
            RecipeKind::StaticMeshBlenderPrimitivesV1.asset_type_prefix(),
            "static_mesh"
        );
    }

    #[test]
    fn test_recipe_kind_tier() {
        assert!(RecipeKind::AudioV1.is_tier1());
        assert!(RecipeKind::MusicTrackerSongV1.is_tier1());
        assert!(RecipeKind::MusicTrackerSongComposeV1.is_tier1());
        assert!(!RecipeKind::StaticMeshBlenderPrimitivesV1.is_tier1());
        assert!(!RecipeKind::SkeletalAnimationBlenderClipV1.is_tier1());
    }

    #[test]
    fn test_recipe_parse_kind() {
        let recipe = Recipe::new("audio_v1", serde_json::json!({"duration_seconds": 0.5}));
        assert_eq!(recipe.parse_kind(), Some(RecipeKind::AudioV1));
        assert_eq!(recipe.asset_type_prefix(), Some("audio"));

        let recipe = Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({"resolution": [64, 64], "tileable": true, "nodes": []}),
        );
        assert_eq!(recipe.parse_kind(), Some(RecipeKind::TextureProceduralV1));
        assert_eq!(recipe.asset_type_prefix(), Some("texture"));

        let recipe = Recipe::new(
            "music.tracker_song_compose_v1",
            serde_json::json!({"format": "xm", "bpm": 120, "speed": 6, "channels": 4}),
        );
        assert_eq!(
            recipe.parse_kind(),
            Some(RecipeKind::MusicTrackerSongComposeV1)
        );
        assert_eq!(recipe.asset_type_prefix(), Some("music"));
    }
}
