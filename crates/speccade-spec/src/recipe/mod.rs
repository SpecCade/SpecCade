//! Recipe types for different asset backends.
//!
//! Each recipe kind corresponds to a specific backend and asset type.
//! The recipe defines the parameters for how to generate the asset.

pub mod animation;
pub mod audio_instrument;
pub mod audio_sfx;
pub mod character;
pub mod mesh;
pub mod music;
pub mod texture;

pub use animation::*;
pub use audio_instrument::*;
pub use audio_sfx::*;
pub use character::*;
pub use mesh::*;
pub use music::*;
pub use texture::*;

use serde::{Deserialize, Serialize};

/// Recipe kind identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipeKind {
    /// `audio_sfx.layered_synth_v1` - Layered synthesis audio SFX.
    #[serde(rename = "audio_sfx.layered_synth_v1")]
    AudioSfxLayeredSynthV1,
    /// `audio_instrument.synth_patch_v1` - Synthesized instrument patch.
    #[serde(rename = "audio_instrument.synth_patch_v1")]
    AudioInstrumentSynthPatchV1,
    /// `music.tracker_song_v1` - Tracker module song.
    #[serde(rename = "music.tracker_song_v1")]
    MusicTrackerSongV1,
    /// `texture_2d.material_maps_v1` - PBR material maps.
    #[serde(rename = "texture_2d.material_maps_v1")]
    Texture2dMaterialMapsV1,
    /// `texture_2d.normal_map_v1` - Normal map.
    #[serde(rename = "texture_2d.normal_map_v1")]
    Texture2dNormalMapV1,
    /// `static_mesh.blender_primitives_v1` - Static mesh from Blender primitives.
    #[serde(rename = "static_mesh.blender_primitives_v1")]
    StaticMeshBlenderPrimitivesV1,
    /// `skeletal_mesh.blender_rigged_mesh_v1` - Rigged skeletal mesh.
    #[serde(rename = "skeletal_mesh.blender_rigged_mesh_v1")]
    SkeletalMeshBlenderRiggedMeshV1,
    /// `skeletal_animation.blender_clip_v1` - Skeletal animation clip.
    #[serde(rename = "skeletal_animation.blender_clip_v1")]
    SkeletalAnimationBlenderClipV1,
}

impl RecipeKind {
    /// Returns the recipe kind as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeKind::AudioSfxLayeredSynthV1 => "audio_sfx.layered_synth_v1",
            RecipeKind::AudioInstrumentSynthPatchV1 => "audio_instrument.synth_patch_v1",
            RecipeKind::MusicTrackerSongV1 => "music.tracker_song_v1",
            RecipeKind::Texture2dMaterialMapsV1 => "texture_2d.material_maps_v1",
            RecipeKind::Texture2dNormalMapV1 => "texture_2d.normal_map_v1",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh.blender_primitives_v1",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh.blender_rigged_mesh_v1",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation.blender_clip_v1",
        }
    }

    /// Returns the asset type prefix for this recipe kind.
    pub fn asset_type_prefix(&self) -> &'static str {
        match self {
            RecipeKind::AudioSfxLayeredSynthV1 => "audio_sfx",
            RecipeKind::AudioInstrumentSynthPatchV1 => "audio_instrument",
            RecipeKind::MusicTrackerSongV1 => "music",
            RecipeKind::Texture2dMaterialMapsV1 => "texture_2d",
            RecipeKind::Texture2dNormalMapV1 => "texture_2d",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation",
        }
    }

    /// Returns whether this is a Tier 1 (deterministic hash) or Tier 2 (metric validation) backend.
    pub fn is_tier1(&self) -> bool {
        match self {
            RecipeKind::AudioSfxLayeredSynthV1
            | RecipeKind::AudioInstrumentSynthPatchV1
            | RecipeKind::MusicTrackerSongV1
            | RecipeKind::Texture2dMaterialMapsV1
            | RecipeKind::Texture2dNormalMapV1 => true,
            RecipeKind::StaticMeshBlenderPrimitivesV1
            | RecipeKind::SkeletalMeshBlenderRiggedMeshV1
            | RecipeKind::SkeletalAnimationBlenderClipV1 => false,
        }
    }
}

impl std::fmt::Display for RecipeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Recipe parameters enum for all recipe kinds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RecipeParams {
    /// Audio SFX layered synth parameters.
    AudioSfxLayeredSynth(AudioSfxLayeredSynthV1Params),
    /// Audio instrument synth patch parameters.
    AudioInstrumentSynthPatch(AudioInstrumentSynthPatchV1Params),
    /// Music tracker song parameters.
    MusicTrackerSong(MusicTrackerSongV1Params),
    /// Texture 2D material maps parameters.
    Texture2dMaterialMaps(Texture2dMaterialMapsV1Params),
    /// Texture 2D normal map parameters.
    Texture2dNormalMap(Texture2dNormalMapV1Params),
    /// Static mesh Blender primitives parameters.
    StaticMeshBlenderPrimitives(StaticMeshBlenderPrimitivesV1Params),
    /// Skeletal mesh Blender rigged mesh parameters.
    SkeletalMeshBlenderRiggedMesh(SkeletalMeshBlenderRiggedMeshV1Params),
    /// Skeletal animation Blender clip parameters.
    SkeletalAnimationBlenderClip(SkeletalAnimationBlenderClipV1Params),
    /// Unknown/generic parameters (stored as raw JSON).
    Unknown(serde_json::Value),
}

/// Recipe specification containing kind and params.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            "audio_sfx.layered_synth_v1" => Some(RecipeKind::AudioSfxLayeredSynthV1),
            "audio_instrument.synth_patch_v1" => Some(RecipeKind::AudioInstrumentSynthPatchV1),
            "music.tracker_song_v1" => Some(RecipeKind::MusicTrackerSongV1),
            "texture_2d.material_maps_v1" => Some(RecipeKind::Texture2dMaterialMapsV1),
            "texture_2d.normal_map_v1" => Some(RecipeKind::Texture2dNormalMapV1),
            "static_mesh.blender_primitives_v1" => Some(RecipeKind::StaticMeshBlenderPrimitivesV1),
            "skeletal_mesh.blender_rigged_mesh_v1" => {
                Some(RecipeKind::SkeletalMeshBlenderRiggedMeshV1)
            }
            "skeletal_animation.blender_clip_v1" => {
                Some(RecipeKind::SkeletalAnimationBlenderClipV1)
            }
            _ => None,
        }
    }

    /// Returns the asset type prefix from the recipe kind.
    pub fn asset_type_prefix(&self) -> Option<&str> {
        self.kind.split('.').next()
    }

    /// Attempts to parse params as audio SFX layered synth params.
    pub fn as_audio_sfx_layered_synth(
        &self,
    ) -> Result<AudioSfxLayeredSynthV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as audio instrument synth patch params.
    pub fn as_audio_instrument_synth_patch(
        &self,
    ) -> Result<AudioInstrumentSynthPatchV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as music tracker song params.
    pub fn as_music_tracker_song(&self) -> Result<MusicTrackerSongV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as texture 2D material maps params.
    pub fn as_texture_2d_material_maps(
        &self,
    ) -> Result<Texture2dMaterialMapsV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as texture 2D normal map params.
    pub fn as_texture_2d_normal_map(&self) -> Result<Texture2dNormalMapV1Params, serde_json::Error> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recipe_kind_serde() {
        let kind = RecipeKind::AudioSfxLayeredSynthV1;
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "\"audio_sfx.layered_synth_v1\"");

        let parsed: RecipeKind = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, kind);
    }

    #[test]
    fn test_recipe_kind_asset_type_prefix() {
        assert_eq!(
            RecipeKind::AudioSfxLayeredSynthV1.asset_type_prefix(),
            "audio_sfx"
        );
        assert_eq!(
            RecipeKind::Texture2dMaterialMapsV1.asset_type_prefix(),
            "texture_2d"
        );
        assert_eq!(
            RecipeKind::Texture2dNormalMapV1.asset_type_prefix(),
            "texture_2d"
        );
        assert_eq!(
            RecipeKind::StaticMeshBlenderPrimitivesV1.asset_type_prefix(),
            "static_mesh"
        );
    }

    #[test]
    fn test_recipe_kind_tier() {
        assert!(RecipeKind::AudioSfxLayeredSynthV1.is_tier1());
        assert!(RecipeKind::MusicTrackerSongV1.is_tier1());
        assert!(!RecipeKind::StaticMeshBlenderPrimitivesV1.is_tier1());
        assert!(!RecipeKind::SkeletalAnimationBlenderClipV1.is_tier1());
    }

    #[test]
    fn test_recipe_parse_kind() {
        let recipe = Recipe::new(
            "audio_sfx.layered_synth_v1",
            serde_json::json!({"duration_seconds": 0.5}),
        );
        assert_eq!(
            recipe.parse_kind(),
            Some(RecipeKind::AudioSfxLayeredSynthV1)
        );
        assert_eq!(recipe.asset_type_prefix(), Some("audio_sfx"));
    }
}
