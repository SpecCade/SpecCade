//! Mesh and animation Tier-2 recipe validation tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::recipe::Recipe;
use crate::spec::AssetType;
use crate::validation::*;

// =============================================================================
// Static Mesh Tests
// =============================================================================

fn make_valid_static_mesh_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("static-mesh-01", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "mesh.glb"))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0]
            }),
        ))
        .build()
}

#[test]
fn test_static_mesh_valid_spec() {
    let spec = make_valid_static_mesh_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_static_mesh_with_all_fields() {
    let spec = crate::spec::Spec::builder("static-mesh-full", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "mesh.glb"))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "sphere",
                "dimensions": [2.0, 2.0, 2.0],
                "modifiers": [
                    {"type": "bevel", "width": 0.02, "segments": 2}
                ],
                "uv_projection": "box",
                "material_slots": [],
                "export": {
                    "apply_modifiers": true,
                    "triangulate": true,
                    "include_normals": true,
                    "include_uvs": true,
                    "include_vertex_colors": false
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_static_mesh_rejects_unknown_fields() {
    let spec = crate::spec::Spec::builder("static-mesh-bad", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "mesh.glb"))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [1.0, 1.0, 1.0],
                "unknown_field": "should_fail"
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams),
        "expected InvalidRecipeParams error, got: {:?}",
        result.errors
    );
    assert!(
        result.errors.iter().any(|e| e.message.contains("unknown")),
        "expected error message to mention unknown field, got: {:?}",
        result.errors
    );
}

#[test]
fn test_static_mesh_rejects_missing_required_field() {
    let spec = crate::spec::Spec::builder("static-mesh-missing", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "mesh.glb"))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube"
                // missing "dimensions"
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_static_mesh_rejects_negative_dimensions() {
    let spec = crate::spec::Spec::builder("static-mesh-neg", AssetType::StaticMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "mesh.glb"))
        .recipe(Recipe::new(
            "static_mesh.blender_primitives_v1",
            serde_json::json!({
                "base_primitive": "cube",
                "dimensions": [-1.0, 1.0, 1.0]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result.errors.iter().any(|e| e.message.contains("positive")),
        "expected error about positive dimensions, got: {:?}",
        result.errors
    );
}

// =============================================================================
// Skeletal Mesh Tests
// =============================================================================

fn make_valid_skeletal_mesh_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("skeletal-mesh-01", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "body_parts": []
            }),
        ))
        .build()
}

#[test]
fn test_skeletal_mesh_valid_spec() {
    let spec = make_valid_skeletal_mesh_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_skeletal_mesh_with_custom_skeleton() {
    let spec = crate::spec::Spec::builder("skeletal-mesh-custom", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::json!({
                "skeleton": [
                    {"bone": "root", "head": [0, 0, 0], "tail": [0, 0, 0.1]},
                    {"bone": "spine", "head": [0, 0, 0.1], "tail": [0, 0, 0.5], "parent": "root"}
                ],
                "body_parts": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_skeletal_mesh_with_parts() {
    let spec = crate::spec::Spec::builder("skeletal-mesh-parts", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::json!({
                "parts": {
                    "torso": {
                        "bone": "spine",
                        "base": "hexagon(6)",
                        "base_radius": 0.15,
                        "steps": [{"extrude": 0.4, "scale": 1.2}]
                    }
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_skeletal_mesh_rejects_unknown_fields() {
    let spec = crate::spec::Spec::builder("skeletal-mesh-bad", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "unknown_field": "should_fail"
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams),
        "expected InvalidRecipeParams error, got: {:?}",
        result.errors
    );
}

#[test]
fn test_skeletal_mesh_rejects_empty_params() {
    let spec = crate::spec::Spec::builder("skeletal-mesh-empty", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.blender_rigged_mesh_v1",
            serde_json::json!({}),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams),
        "expected InvalidRecipeParams error for empty params, got: {:?}",
        result.errors
    );
}

// =============================================================================
// Skeletal Animation (clip_v1) Tests
// =============================================================================

fn make_valid_animation_clip_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("animation-clip-01", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "idle",
                "duration_seconds": 1.0,
                "fps": 30,
                "keyframes": []
            }),
        ))
        .build()
}

#[test]
fn test_animation_clip_valid_spec() {
    let spec = make_valid_animation_clip_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_animation_clip_with_keyframes() {
    let spec = crate::spec::Spec::builder("animation-clip-kf", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "walk",
                "duration_seconds": 2.0,
                "fps": 24,
                "loop": true,
                "keyframes": [
                    {
                        "time": 0.0,
                        "bones": {
                            "upper_leg_l": {"rotation": [15.0, 0.0, 0.0]}
                        }
                    }
                ],
                "interpolation": "linear"
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_animation_clip_rejects_unknown_fields() {
    let spec = crate::spec::Spec::builder("animation-clip-bad", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "idle",
                "duration_seconds": 1.0,
                "fps": 30,
                "keyframes": [],
                "invalid_field": "should_fail"
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams),
        "expected InvalidRecipeParams error, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_missing_required() {
    // Missing clip_name
    let spec = crate::spec::Spec::builder("animation-clip-missing", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "duration_seconds": 1.0,
                "fps": 30,
                "keyframes": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams));
}

#[test]
fn test_animation_clip_rejects_zero_duration() {
    let spec = crate::spec::Spec::builder("animation-clip-zero", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "idle",
                "duration_seconds": 0.0,
                "fps": 30,
                "keyframes": []
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("duration_seconds")),
        "expected error about duration, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_empty_clip_name() {
    let spec =
        crate::spec::Spec::builder("animation-clip-empty-name", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
            .recipe(Recipe::new(
                "skeletal_animation.blender_clip_v1",
                serde_json::json!({
                    "skeleton_preset": "humanoid_basic_v1",
                    "clip_name": "",
                    "duration_seconds": 1.0,
                    "fps": 30,
                    "keyframes": []
                }),
            ))
            .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("clip_name")),
        "expected error about clip_name, got: {:?}",
        result.errors
    );
}

// =============================================================================
// Skeletal Animation (clip_v1) - IK Field Rejection Tests
// =============================================================================

#[test]
fn test_animation_clip_rejects_rig_setup_with_helpful_message() {
    let spec = crate::spec::Spec::builder("animation-clip-ik-bad", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "walk",
                "duration_seconds": 1.0,
                "fps": 30,
                "keyframes": [],
                "rig_setup": {
                    "presets": ["humanoid_legs"]
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.code == crate::error::ErrorCode::InvalidRecipeParams),
        "expected InvalidRecipeParams error, got: {:?}",
        result.errors
    );
    // Check for helpful error message directing to blender_rigged_v1
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("blender_rigged_v1")),
        "expected error to suggest blender_rigged_v1, got: {:?}",
        result.errors
    );
    // Check for specific field mention
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("rig_setup")),
        "expected error to mention rig_setup field, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_poses_field() {
    let spec = crate::spec::Spec::builder("animation-clip-poses-bad", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_clip_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "walk",
                "duration_seconds": 1.0,
                "fps": 30,
                "keyframes": [],
                "poses": {
                    "standing": { "bones": {} }
                }
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("poses") && e.message.contains("blender_rigged_v1")),
        "expected error to mention poses and blender_rigged_v1, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_phases_field() {
    let spec =
        crate::spec::Spec::builder("animation-clip-phases-bad", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
            .recipe(Recipe::new(
                "skeletal_animation.blender_clip_v1",
                serde_json::json!({
                    "skeleton_preset": "humanoid_basic_v1",
                    "clip_name": "walk",
                    "duration_seconds": 1.0,
                    "fps": 30,
                    "keyframes": [],
                    "phases": [
                        { "name": "contact", "start_frame": 0, "end_frame": 15 }
                    ]
                }),
            ))
            .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("phases") && e.message.contains("blender_rigged_v1")),
        "expected error to mention phases and blender_rigged_v1, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_ik_keyframes_field() {
    let spec = crate::spec::Spec::builder(
        "animation-clip-ik-keyframes-bad",
        AssetType::SkeletalAnimation,
    )
    .license("CC0-1.0")
    .seed(123)
    .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
    .recipe(Recipe::new(
        "skeletal_animation.blender_clip_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "clip_name": "walk",
            "duration_seconds": 1.0,
            "fps": 30,
            "keyframes": [],
            "ik_keyframes": [
                { "time": 0.5, "targets": {} }
            ]
        }),
    ))
    .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("ik_keyframes") && e.message.contains("blender_rigged_v1")),
        "expected error to mention ik_keyframes and blender_rigged_v1, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_procedural_layers_field() {
    let spec = crate::spec::Spec::builder(
        "animation-clip-procedural-bad",
        AssetType::SkeletalAnimation,
    )
    .license("CC0-1.0")
    .seed(123)
    .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
    .recipe(Recipe::new(
        "skeletal_animation.blender_clip_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "clip_name": "walk",
            "duration_seconds": 1.0,
            "fps": 30,
            "keyframes": [],
            "procedural_layers": [
                { "type": "breathing", "intensity": 0.5 }
            ]
        }),
    ))
    .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("procedural_layers")
                && e.message.contains("blender_rigged_v1")),
        "expected error to mention procedural_layers and blender_rigged_v1, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_clip_rejects_duration_frames_field() {
    let spec = crate::spec::Spec::builder(
        "animation-clip-duration-frames-bad",
        AssetType::SkeletalAnimation,
    )
    .license("CC0-1.0")
    .seed(123)
    .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
    .recipe(Recipe::new(
        "skeletal_animation.blender_clip_v1",
        serde_json::json!({
            "skeleton_preset": "humanoid_basic_v1",
            "clip_name": "walk",
            "duration_seconds": 1.0,
            "fps": 30,
            "keyframes": [],
            "duration_frames": 60
        }),
    ))
    .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("duration_frames")
                && e.message.contains("blender_rigged_v1")),
        "expected error to mention duration_frames and blender_rigged_v1, got: {:?}",
        result.errors
    );
}

// =============================================================================
// Skeletal Animation (rigged_v1) Tests
// =============================================================================

fn make_valid_animation_rigged_spec() -> crate::spec::Spec {
    crate::spec::Spec::builder("animation-rigged-01", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_rigged_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "walk_ik",
                "duration_frames": 60,
                "fps": 30
            }),
        ))
        .build()
}

#[test]
fn test_animation_rigged_valid_spec() {
    let spec = make_valid_animation_rigged_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_animation_rigged_with_ik_features() {
    let spec = crate::spec::Spec::builder("animation-rigged-ik-full", AssetType::SkeletalAnimation)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
        .recipe(Recipe::new(
            "skeletal_animation.blender_rigged_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "clip_name": "walk_ik",
                "duration_frames": 60,
                "fps": 30,
                "loop": true,
                "rig_setup": {
                    "presets": ["humanoid_legs"]
                },
                "poses": {
                    "contact_left": {
                        "bones": {
                            "spine": { "pitch": 2.0, "yaw": 0.0, "roll": 0.0 }
                        }
                    }
                },
                "phases": [
                    {
                        "name": "contact_left",
                        "start_frame": 0,
                        "end_frame": 30,
                        "curve": "ease_in_out",
                        "pose": "contact_left",
                        "ik_targets": {
                            "ik_leg_l": [
                                { "frame": 0, "location": [0.1, 0.3, 0.0] }
                            ]
                        }
                    }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_animation_rigged_rejects_zero_duration_frames() {
    let spec =
        crate::spec::Spec::builder("animation-rigged-zero-frames", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
            .recipe(Recipe::new(
                "skeletal_animation.blender_rigged_v1",
                serde_json::json!({
                    "skeleton_preset": "humanoid_basic_v1",
                    "clip_name": "walk",
                    "duration_frames": 0,
                    "fps": 30
                }),
            ))
            .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("duration_frames")),
        "expected error about duration_frames, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_rigged_rejects_zero_fps() {
    let spec =
        crate::spec::Spec::builder("animation-rigged-zero-fps", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
            .recipe(Recipe::new(
                "skeletal_animation.blender_rigged_v1",
                serde_json::json!({
                    "skeleton_preset": "humanoid_basic_v1",
                    "clip_name": "walk",
                    "duration_frames": 60,
                    "fps": 0
                }),
            ))
            .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result.errors.iter().any(|e| e.message.contains("fps")),
        "expected error about fps, got: {:?}",
        result.errors
    );
}

#[test]
fn test_animation_rigged_rejects_empty_clip_name() {
    let spec =
        crate::spec::Spec::builder("animation-rigged-empty-name", AssetType::SkeletalAnimation)
            .license("CC0-1.0")
            .seed(123)
            .output(OutputSpec::primary(OutputFormat::Glb, "animation.glb"))
            .recipe(Recipe::new(
                "skeletal_animation.blender_rigged_v1",
                serde_json::json!({
                    "skeleton_preset": "humanoid_basic_v1",
                    "clip_name": "",
                    "duration_frames": 60,
                    "fps": 30
                }),
            ))
            .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(
        result
            .errors
            .iter()
            .any(|e| e.message.contains("clip_name")),
        "expected error about clip_name, got: {:?}",
        result.errors
    );
}

// =============================================================================
// Recipe::try_parse_params() Tests
// =============================================================================

#[test]
fn test_try_parse_params_valid_static_mesh() {
    let recipe = Recipe::new(
        "static_mesh.blender_primitives_v1",
        serde_json::json!({
            "base_primitive": "cube",
            "dimensions": [1.0, 1.0, 1.0]
        }),
    );
    assert!(recipe.try_parse_params().is_ok());
}

#[test]
fn test_try_parse_params_unknown_field() {
    let recipe = Recipe::new(
        "static_mesh.blender_primitives_v1",
        serde_json::json!({
            "base_primitive": "cube",
            "dimensions": [1.0, 1.0, 1.0],
            "unknown_field": true
        }),
    );
    let result = recipe.try_parse_params();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_message.contains("unknown"));
}

#[test]
fn test_try_parse_params_missing_field() {
    let recipe = Recipe::new(
        "static_mesh.blender_primitives_v1",
        serde_json::json!({
            "base_primitive": "cube"
            // missing dimensions
        }),
    );
    let result = recipe.try_parse_params();
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.error_message.contains("dimensions"));
}

#[test]
fn test_try_parse_params_unknown_recipe_kind() {
    // Unknown recipe kinds should pass (no validation performed)
    let recipe = Recipe::new(
        "custom.my_backend_v1",
        serde_json::json!({
            "any": "params",
            "are": "allowed"
        }),
    );
    assert!(recipe.try_parse_params().is_ok());
}
