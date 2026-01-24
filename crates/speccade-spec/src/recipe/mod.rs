//! Recipe types for different asset backends.
//!
//! Each recipe kind corresponds to a specific backend and asset type.
//! The recipe defines the parameters for how to generate the asset.

pub mod animation;
pub mod audio;
pub mod character;
pub mod font;
pub mod mesh;
pub mod music;
pub mod sprite;
pub mod texture;
pub mod ui;
pub mod vfx;

pub use animation::*;
pub use audio::*;
pub use character::*;
pub use font::*;
pub use mesh::*;
pub use music::*;
pub use sprite::*;
pub use texture::*;
pub use ui::*;
pub use vfx::*;

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
    /// `texture.trimsheet_v1` - Atlas/trimsheet texture packing.
    #[serde(rename = "texture.trimsheet_v1")]
    TextureTrimsheetV1,
    /// `texture.decal_v1` - Decal textures with RGBA, optional normal/roughness, and placement metadata.
    #[serde(rename = "texture.decal_v1")]
    TextureDecalV1,
    /// `texture.splat_set_v1` - Terrain splat set with multiple layers, blend masks, and macro variation.
    #[serde(rename = "texture.splat_set_v1")]
    TextureSplatSetV1,
    /// `texture.matcap_v1` - Matcap texture for stylized NPR shading.
    #[serde(rename = "texture.matcap_v1")]
    TextureMatcapV1,
    /// `texture.material_preset_v1` - Material preset for PBR textures with style presets.
    #[serde(rename = "texture.material_preset_v1")]
    TextureMaterialPresetV1,
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
    /// `sprite.sheet_v1` - Spritesheet/atlas packing with frame metadata.
    #[serde(rename = "sprite.sheet_v1")]
    SpriteSheetV1,
    /// `sprite.animation_v1` - Sprite animation clip definitions.
    #[serde(rename = "sprite.animation_v1")]
    SpriteAnimationV1,
    /// `vfx.flipbook_v1` - VFX flipbook animation with procedural frame generation.
    #[serde(rename = "vfx.flipbook_v1")]
    VfxFlipbookV1,
    /// `vfx.particle_profile_v1` - VFX particle rendering profile preset (metadata-only).
    #[serde(rename = "vfx.particle_profile_v1")]
    VfxParticleProfileV1,
    /// `ui.nine_slice_v1` - Nine-slice panel generation with corner/edge/center regions.
    #[serde(rename = "ui.nine_slice_v1")]
    UiNineSliceV1,
    /// `ui.icon_set_v1` - Icon pack assembly with sprite frames.
    #[serde(rename = "ui.icon_set_v1")]
    UiIconSetV1,
    /// `ui.item_card_v1` - Item card templates with rarity variants and customizable slots.
    #[serde(rename = "ui.item_card_v1")]
    UiItemCardV1,
    /// `ui.damage_number_v1` - Damage number sprites with style variants (normal, critical, healing).
    #[serde(rename = "ui.damage_number_v1")]
    UiDamageNumberV1,
    /// `font.bitmap_v1` - Bitmap pixel font with glyph atlas and metrics.
    #[serde(rename = "font.bitmap_v1")]
    FontBitmapV1,
}

impl RecipeKind {
    /// Returns the recipe kind as a string.
    pub fn as_str(&self) -> &'static str {
        match self {
            RecipeKind::AudioV1 => "audio_v1",
            RecipeKind::MusicTrackerSongV1 => "music.tracker_song_v1",
            RecipeKind::MusicTrackerSongComposeV1 => "music.tracker_song_compose_v1",
            RecipeKind::TextureProceduralV1 => "texture.procedural_v1",
            RecipeKind::TextureTrimsheetV1 => "texture.trimsheet_v1",
            RecipeKind::TextureDecalV1 => "texture.decal_v1",
            RecipeKind::TextureSplatSetV1 => "texture.splat_set_v1",
            RecipeKind::TextureMatcapV1 => "texture.matcap_v1",
            RecipeKind::TextureMaterialPresetV1 => "texture.material_preset_v1",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh.blender_primitives_v1",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh.blender_rigged_mesh_v1",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation.blender_clip_v1",
            RecipeKind::SkeletalAnimationBlenderRiggedV1 => "skeletal_animation.blender_rigged_v1",
            RecipeKind::SpriteSheetV1 => "sprite.sheet_v1",
            RecipeKind::SpriteAnimationV1 => "sprite.animation_v1",
            RecipeKind::VfxFlipbookV1 => "vfx.flipbook_v1",
            RecipeKind::VfxParticleProfileV1 => "vfx.particle_profile_v1",
            RecipeKind::UiNineSliceV1 => "ui.nine_slice_v1",
            RecipeKind::UiIconSetV1 => "ui.icon_set_v1",
            RecipeKind::UiItemCardV1 => "ui.item_card_v1",
            RecipeKind::UiDamageNumberV1 => "ui.damage_number_v1",
            RecipeKind::FontBitmapV1 => "font.bitmap_v1",
        }
    }

    /// Returns the asset type prefix for this recipe kind.
    pub fn asset_type_prefix(&self) -> &'static str {
        match self {
            RecipeKind::AudioV1 => "audio",
            RecipeKind::MusicTrackerSongV1 => "music",
            RecipeKind::MusicTrackerSongComposeV1 => "music",
            RecipeKind::TextureProceduralV1 => "texture",
            RecipeKind::TextureTrimsheetV1 => "texture",
            RecipeKind::TextureDecalV1 => "texture",
            RecipeKind::TextureSplatSetV1 => "texture",
            RecipeKind::TextureMatcapV1 => "texture",
            RecipeKind::TextureMaterialPresetV1 => "texture",
            RecipeKind::StaticMeshBlenderPrimitivesV1 => "static_mesh",
            RecipeKind::SkeletalMeshBlenderRiggedMeshV1 => "skeletal_mesh",
            RecipeKind::SkeletalAnimationBlenderClipV1 => "skeletal_animation",
            RecipeKind::SkeletalAnimationBlenderRiggedV1 => "skeletal_animation",
            RecipeKind::SpriteSheetV1 => "sprite",
            RecipeKind::SpriteAnimationV1 => "sprite",
            RecipeKind::VfxFlipbookV1 => "vfx",
            RecipeKind::VfxParticleProfileV1 => "vfx",
            RecipeKind::UiNineSliceV1 => "ui",
            RecipeKind::UiIconSetV1 => "ui",
            RecipeKind::UiItemCardV1 => "ui",
            RecipeKind::UiDamageNumberV1 => "ui",
            RecipeKind::FontBitmapV1 => "font",
        }
    }

    /// Returns whether this is a Tier 1 (deterministic hash) or Tier 2 (metric validation) backend.
    pub fn is_tier1(&self) -> bool {
        match self {
            RecipeKind::AudioV1
            | RecipeKind::MusicTrackerSongV1
            | RecipeKind::MusicTrackerSongComposeV1
            | RecipeKind::TextureProceduralV1
            | RecipeKind::TextureTrimsheetV1
            | RecipeKind::TextureDecalV1
            | RecipeKind::TextureSplatSetV1
            | RecipeKind::TextureMatcapV1
            | RecipeKind::TextureMaterialPresetV1
            | RecipeKind::SpriteSheetV1
            | RecipeKind::SpriteAnimationV1
            | RecipeKind::VfxFlipbookV1
            | RecipeKind::VfxParticleProfileV1
            | RecipeKind::UiNineSliceV1
            | RecipeKind::UiIconSetV1
            | RecipeKind::UiItemCardV1
            | RecipeKind::UiDamageNumberV1
            | RecipeKind::FontBitmapV1 => true,
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
            "texture.trimsheet_v1" => Some(RecipeKind::TextureTrimsheetV1),
            "texture.decal_v1" => Some(RecipeKind::TextureDecalV1),
            "texture.splat_set_v1" => Some(RecipeKind::TextureSplatSetV1),
            "texture.matcap_v1" => Some(RecipeKind::TextureMatcapV1),
            "texture.material_preset_v1" => Some(RecipeKind::TextureMaterialPresetV1),
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
            "sprite.sheet_v1" => Some(RecipeKind::SpriteSheetV1),
            "sprite.animation_v1" => Some(RecipeKind::SpriteAnimationV1),
            "vfx.flipbook_v1" => Some(RecipeKind::VfxFlipbookV1),
            "vfx.particle_profile_v1" => Some(RecipeKind::VfxParticleProfileV1),
            "ui.nine_slice_v1" => Some(RecipeKind::UiNineSliceV1),
            "ui.icon_set_v1" => Some(RecipeKind::UiIconSetV1),
            "ui.item_card_v1" => Some(RecipeKind::UiItemCardV1),
            "ui.damage_number_v1" => Some(RecipeKind::UiDamageNumberV1),
            "font.bitmap_v1" => Some(RecipeKind::FontBitmapV1),
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

    /// Attempts to parse params as trimsheet texture params.
    pub fn as_texture_trimsheet(&self) -> Result<TextureTrimsheetV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as decal texture params.
    pub fn as_texture_decal(&self) -> Result<TextureDecalV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as splat set texture params.
    pub fn as_texture_splat_set(&self) -> Result<TextureSplatSetV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as matcap texture params.
    pub fn as_texture_matcap(&self) -> Result<TextureMatcapV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as material preset texture params.
    pub fn as_texture_material_preset(
        &self,
    ) -> Result<TextureMaterialPresetV1Params, serde_json::Error> {
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

    /// Attempts to parse params as sprite sheet params.
    pub fn as_sprite_sheet(&self) -> Result<SpriteSheetV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as sprite animation params.
    pub fn as_sprite_animation(&self) -> Result<SpriteAnimationV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as VFX flipbook params.
    pub fn as_vfx_flipbook(&self) -> Result<VfxFlipbookV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as VFX particle profile params.
    pub fn as_vfx_particle_profile(&self) -> Result<VfxParticleProfileV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as UI nine-slice params.
    pub fn as_ui_nine_slice(&self) -> Result<UiNineSliceV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as UI icon set params.
    pub fn as_ui_icon_set(&self) -> Result<UiIconSetV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as UI item card params.
    pub fn as_ui_item_card(&self) -> Result<UiItemCardV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as UI damage number params.
    pub fn as_ui_damage_number(&self) -> Result<UiDamageNumberV1Params, serde_json::Error> {
        serde_json::from_value(self.params.clone())
    }

    /// Attempts to parse params as bitmap font params.
    pub fn as_font_bitmap(&self) -> Result<FontBitmapV1Params, serde_json::Error> {
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
                self.as_music_tracker_song()
                    .map_err(|e| RecipeParamsError {
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
            "texture.trimsheet_v1" => {
                self.as_texture_trimsheet().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "texture.decal_v1" => {
                self.as_texture_decal().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "texture.splat_set_v1" => {
                self.as_texture_splat_set().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "texture.matcap_v1" => {
                self.as_texture_matcap().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "texture.material_preset_v1" => {
                self.as_texture_material_preset()
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
            "sprite.sheet_v1" => {
                self.as_sprite_sheet().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "sprite.animation_v1" => {
                self.as_sprite_animation().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "vfx.flipbook_v1" => {
                self.as_vfx_flipbook().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "vfx.particle_profile_v1" => {
                self.as_vfx_particle_profile()
                    .map_err(|e| RecipeParamsError {
                        recipe_kind: self.kind.clone(),
                        error_message: e.to_string(),
                    })?;
            }
            "ui.nine_slice_v1" => {
                self.as_ui_nine_slice().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "ui.icon_set_v1" => {
                self.as_ui_icon_set().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "ui.item_card_v1" => {
                self.as_ui_item_card().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "ui.damage_number_v1" => {
                self.as_ui_damage_number().map_err(|e| RecipeParamsError {
                    recipe_kind: self.kind.clone(),
                    error_message: e.to_string(),
                })?;
            }
            "font.bitmap_v1" => {
                self.as_font_bitmap().map_err(|e| RecipeParamsError {
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
