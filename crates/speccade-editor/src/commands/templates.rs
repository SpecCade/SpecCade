//! Template loading commands for the editor.
//!
//! Provides pre-defined Starlark spec templates for common asset types.

use serde::{Deserialize, Serialize};

/// Information about a template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Unique template identifier
    pub id: String,
    /// Human-readable template name
    pub name: String,
    /// Template description
    pub description: String,
    /// Asset type this template creates (audio, texture, mesh, etc.)
    pub asset_type: String,
}

/// Basic audio oscillator template - simple oscillator with ADSR envelope.
const AUDIO_BASIC: &str = r#"# Basic Audio Template
# A simple oscillator with ADSR envelope
#
# Modify the frequency, waveform, and envelope to shape your sound.

spec(
    asset_id = "my-sound-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/my_sound.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sine"),
                    envelope = envelope(0.01, 0.1, 0.5, 0.2),
                    volume = 0.8
                )
            ]
        }
    }
)
"#;

/// Laser sound effect template - pitch sweep with filter.
const AUDIO_LASER: &str = r#"# Laser Sound Effect Template
# A sawtooth oscillator with pitch sweep and lowpass filter
#
# Adjust pitch_end for sweep range, filter cutoff for brightness.

spec(
    asset_id = "laser-blast-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/laser.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.3,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(880, "sawtooth", 220, "exponential"),
                    envelope = envelope(0.001, 0.05, 0.0, 0.2),
                    filter = lowpass(4000, 0.707, 500),
                    volume = 0.9
                )
            ],
            "effects": [
                reverb(0.1, 0.3)
            ]
        }
    }
)
"#;

/// Basic music tracker template - 4-channel tracker song.
const MUSIC_BASIC: &str = r#"# Basic Music Tracker Template
# A simple 4-channel tracker song with bass and lead instruments
#
# Add more patterns and extend the arrangement to build your song.

# Define instruments
bass_inst = tracker_instrument(
    name = "bass",
    synthesis = instrument_synthesis("sawtooth"),
    envelope = envelope(0.01, 0.1, 0.8, 0.1),
    default_volume = 48
)

lead_inst = tracker_instrument(
    name = "lead",
    synthesis = instrument_synthesis("pulse", 0.25),
    envelope = envelope(0.01, 0.05, 0.6, 0.3)
)

# Define a simple pattern
main_pattern = tracker_pattern(64, notes = {
    "0": [
        pattern_note(0, "C3", 0),
        pattern_note(16, "E3", 0),
        pattern_note(32, "G3", 0),
        pattern_note(48, "C4", 0)
    ],
    "1": [
        pattern_note(0, "E4", 1, vol = 40),
        pattern_note(16, "G4", 1, vol = 40),
        pattern_note(32, "C5", 1, vol = 40),
        pattern_note(48, "E5", 1, vol = 40)
    ]
})

# Create the music spec
music_spec(
    asset_id = "my-song-01",
    seed = 42,
    output_path = "music/my_song.xm",
    format = "xm",
    bpm = 120,
    speed = 6,
    channels = 4,
    instruments = [bass_inst, lead_inst],
    patterns = {
        "main": main_pattern
    },
    arrangement = [
        arrangement_entry("main", 4)
    ],
    name = "My Song",
    title = "My First Tracker Song"
)
"#;

/// Basic procedural texture template - Perlin noise with color ramp.
const TEXTURE_BASIC: &str = r##"# Basic Texture Template
# Procedural noise texture with color ramp
#
# Modify noise parameters and color stops to customize appearance.

spec(
    asset_id = "noise-texture-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/noise.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [256, 256],
            [
                noise_node("base", "perlin", 0.05, 4, 0.5, 2.0),
                color_ramp_node("colored", "base", ["#1a1a2e", "#16213e", "#0f3460", "#e94560"])
            ],
            True  # tileable
        )
    }
)
"##;

/// PBR material preset texture template - BrushedMetal preset.
const TEXTURE_PBR: &str = r#"# PBR Material Preset Template
# Material preset with albedo, normal, roughness, and metallic maps
#
# Available presets: brushed_metal, wood_planks, concrete, marble,
# toon_metal, fabric, leather, plastic, stone, rust

material_preset_v1(
    asset_id = "material-brushed-metal-01",
    seed = 42,
    output_prefix = "materials/brushed_metal",
    resolution = [512, 512],
    preset = "brushed_metal",
    base_color = [0.8, 0.75, 0.7],
    description = "Brushed metal PBR material",
    style_tags = ["metal", "pbr", "brushed"],
    license = "CC0-1.0"
)
"#;

/// Basic mesh template - cube with bevel and subdivision.
const MESH_BASIC: &str = r#"# Basic Mesh Template
# A cube with bevel and subdivision modifiers
#
# Change primitive type: cube, sphere, cylinder, cone, torus, plane
# Add more modifiers: mirror, array, decimate, solidify

spec(
    asset_id = "rounded-cube-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            "cube",
            [1.0, 1.0, 1.0],
            [
                bevel_modifier(0.05, 3),
                subdivision_modifier(2)
            ]
        )
    }
)
"#;

/// Character skeletal mesh template - basic humanoid.
const CHARACTER_BASIC: &str = r#"# Basic Character Template
# A skeletal mesh with humanoid skeleton and body parts
#
# Add more body_part() calls to build out the character.
# Skeleton presets: humanoid_basic_v1, quadruped_v1, biped_simple_v1

skeletal_mesh_spec(
    asset_id = "humanoid-character-01",
    seed = 42,
    output_path = "characters/humanoid.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    description = "Basic humanoid character",
    body_parts = [
        # Torso
        body_part(
            bone = "spine",
            primitive = "cylinder",
            dimensions = [0.25, 0.4, 0.25],
            segments = 8,
            offset = [0, 0, 0.3],
            material_index = 0
        ),
        body_part(
            bone = "chest",
            primitive = "cylinder",
            dimensions = [0.3, 0.3, 0.28],
            segments = 8,
            offset = [0, 0, 0.6],
            material_index = 0
        ),
        # Head
        body_part(
            bone = "head",
            primitive = "sphere",
            dimensions = [0.15, 0.18, 0.15],
            segments = 12,
            offset = [0, 0, 0.95],
            material_index = 1
        )
    ],
    material_slots = [
        material_slot(name = "body", base_color = [0.8, 0.6, 0.5, 1.0]),
        material_slot(name = "head", base_color = [0.9, 0.7, 0.6, 1.0])
    ],
    skinning = skinning_config(max_bone_influences = 4, auto_weights = True),
    constraints = skeletal_constraints(max_triangles = 5000, max_bones = 64)
)
"#;

/// Animation template - basic walk cycle.
const ANIMATION_BASIC: &str = r#"# Basic Animation Template
# A simple walk cycle with leg and arm movement
#
# Add more keyframes and bones for more complex animations.
# Use IK keyframes for foot/hand placement.

skeletal_animation_spec(
    asset_id = "walk-cycle-01",
    seed = 42,
    output_path = "animations/walk.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    description = "Basic walk cycle animation",
    keyframes = [
        # Contact pose - left foot forward
        animation_keyframe(
            time = 0.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [-15.0, 0.0, -5.0]),
                "upper_arm_r": bone_transform(rotation = [15.0, 0.0, 5.0])
            }
        ),
        # Passing pose - legs swap
        animation_keyframe(
            time = 0.5,
            bones = {
                "upper_leg_l": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [15.0, 0.0, 5.0]),
                "upper_arm_r": bone_transform(rotation = [-15.0, 0.0, -5.0])
            }
        ),
        # Return to start (looping)
        animation_keyframe(
            time = 1.0,
            bones = {
                "upper_leg_l": bone_transform(rotation = [25.0, 0.0, 0.0]),
                "upper_leg_r": bone_transform(rotation = [-25.0, 0.0, 0.0]),
                "upper_arm_l": bone_transform(rotation = [-15.0, 0.0, -5.0]),
                "upper_arm_r": bone_transform(rotation = [15.0, 0.0, 5.0])
            }
        )
    ],
    interpolation = "linear",
    export = animation_export_settings(bake_transforms = True)
)
"#;

/// Sprite sheet template - render mesh to sprite atlas.
const SPRITE_BASIC: &str = r#"# Sprite Sheet Template
# Renders a 3D mesh from multiple angles into a sprite atlas
#
# Adjust rotation_angles for different directional sprites.
# Change mesh_recipe to render different objects.

spec(
    asset_id = "sprite-sheet-01",
    asset_type = "sprite",
    seed = 42,
    outputs = [
        output("sprites/sheet.png", "png"),
        output("sprites/sheet.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "sprite.render_from_mesh_v1",
        "params": {
            "mesh": mesh_recipe(
                "cube",
                [1.0, 1.0, 1.0],
                [bevel_modifier(0.05, 2)]
            ),
            "camera": "orthographic",
            "lighting": "three_point",
            "frame_resolution": [64, 64],
            "rotation_angles": [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0],
            "atlas_padding": 2,
            "camera_distance": 2.5,
            "camera_elevation": 30.0
        }
    }
)
"#;

/// VFX flipbook template - particle effect flipbook animation.
const VFX_BASIC: &str = r#"# VFX Flipbook Template
# A particle effect rendered as a flipbook texture atlas
#
# Adjust profile and color_tint for different effect styles.
# Profiles: additive, alpha_blend, soft_additive, premultiplied

spec(
    asset_id = "vfx-explosion-01",
    asset_type = "vfx",
    seed = 42,
    outputs = [
        output("vfx/explosion.png", "png"),
        output("vfx/explosion.json", "json", kind = "metadata")
    ],
    recipe = {
        "kind": "vfx.flipbook_v1",
        "params": {
            "effect_type": "explosion",
            "frame_count": 16,
            "frame_resolution": [128, 128],
            "duration_seconds": 0.5,
            "profile": "additive",
            "color_tint": [1.0, 0.8, 0.3],
            "intensity": 1.2,
            "grid_layout": [4, 4]
        }
    }
)
"#;

/// List of all available templates.
const TEMPLATES: &[(&str, &str, &str, &str, &str)] = &[
    (
        "audio_basic",
        "Basic Audio",
        "Simple oscillator with ADSR envelope - a starting point for sound effects",
        "audio",
        AUDIO_BASIC,
    ),
    (
        "audio_laser",
        "Laser Sound",
        "Pitch sweep sawtooth with lowpass filter - classic sci-fi laser effect",
        "audio",
        AUDIO_LASER,
    ),
    (
        "music_basic",
        "Basic Tracker Song",
        "4-channel tracker song with bass and lead instruments",
        "music",
        MUSIC_BASIC,
    ),
    (
        "texture_basic",
        "Procedural Noise",
        "Perlin noise texture with color ramp - tileable procedural pattern",
        "texture",
        TEXTURE_BASIC,
    ),
    (
        "texture_pbr",
        "PBR Material",
        "Brushed metal PBR material preset with albedo, normal, roughness maps",
        "texture",
        TEXTURE_PBR,
    ),
    (
        "mesh_basic",
        "Basic Mesh",
        "Cube with bevel and subdivision modifiers - starting point for props",
        "static_mesh",
        MESH_BASIC,
    ),
    (
        "character_basic",
        "Basic Character",
        "Humanoid skeletal mesh with body parts and skinning",
        "character",
        CHARACTER_BASIC,
    ),
    (
        "animation_basic",
        "Walk Cycle",
        "Basic walk cycle animation with leg and arm keyframes",
        "animation",
        ANIMATION_BASIC,
    ),
    (
        "sprite_basic",
        "Sprite Sheet",
        "Render 3D mesh to 8-directional sprite atlas",
        "sprite",
        SPRITE_BASIC,
    ),
    (
        "vfx_basic",
        "VFX Flipbook",
        "Particle effect rendered as flipbook texture atlas",
        "vfx",
        VFX_BASIC,
    ),
];

/// List all available templates.
///
/// Returns metadata about each template without the full content.
#[tauri::command]
pub fn list_templates() -> Vec<TemplateInfo> {
    TEMPLATES
        .iter()
        .map(|(id, name, description, asset_type, _)| TemplateInfo {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            asset_type: asset_type.to_string(),
        })
        .collect()
}

/// Get the content of a specific template by ID.
///
/// Returns `None` if the template ID is not found.
#[tauri::command]
pub fn get_template(id: String) -> Option<String> {
    TEMPLATES
        .iter()
        .find(|(template_id, _, _, _, _)| *template_id == id)
        .map(|(_, _, _, _, content)| content.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_templates_returns_all() {
        let templates = list_templates();
        assert_eq!(templates.len(), 10);
    }

    #[test]
    fn test_list_templates_has_required_fields() {
        let templates = list_templates();
        for template in templates {
            assert!(!template.id.is_empty());
            assert!(!template.name.is_empty());
            assert!(!template.description.is_empty());
            assert!(!template.asset_type.is_empty());
        }
    }

    #[test]
    fn test_get_template_audio_basic() {
        let content = get_template("audio_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("oscillator"));
        assert!(content.contains("envelope"));
    }

    #[test]
    fn test_get_template_audio_laser() {
        let content = get_template("audio_laser".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("sawtooth"));
        assert!(content.contains("lowpass"));
    }

    #[test]
    fn test_get_template_music_basic() {
        let content = get_template("music_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("tracker_instrument"));
        assert!(content.contains("tracker_pattern"));
    }

    #[test]
    fn test_get_template_texture_basic() {
        let content = get_template("texture_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("noise_node"));
        assert!(content.contains("color_ramp_node"));
    }

    #[test]
    fn test_get_template_texture_pbr() {
        let content = get_template("texture_pbr".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("material_preset_v1"));
        assert!(content.contains("brushed_metal"));
    }

    #[test]
    fn test_get_template_mesh_basic() {
        let content = get_template("mesh_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("bevel_modifier"));
        assert!(content.contains("subdivision_modifier"));
    }

    #[test]
    fn test_get_template_character_basic() {
        let content = get_template("character_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("skeletal_mesh_spec"));
        assert!(content.contains("body_part"));
    }

    #[test]
    fn test_get_template_animation_basic() {
        let content = get_template("animation_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("skeletal_animation_spec"));
        assert!(content.contains("animation_keyframe"));
    }

    #[test]
    fn test_get_template_sprite_basic() {
        let content = get_template("sprite_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("sprite.render_from_mesh_v1"));
        assert!(content.contains("rotation_angles"));
    }

    #[test]
    fn test_get_template_vfx_basic() {
        let content = get_template("vfx_basic".to_string());
        assert!(content.is_some());
        let content = content.unwrap();
        assert!(content.contains("vfx.flipbook_v1"));
        assert!(content.contains("frame_count"));
    }

    #[test]
    fn test_get_template_not_found() {
        let content = get_template("nonexistent".to_string());
        assert!(content.is_none());
    }

    #[test]
    fn test_all_template_ids_unique() {
        let templates = list_templates();
        let mut ids: Vec<_> = templates.iter().map(|t| &t.id).collect();
        let original_len = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), original_len, "Template IDs must be unique");
    }
}
