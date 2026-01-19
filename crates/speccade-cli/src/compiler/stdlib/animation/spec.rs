//! Skeletal animation spec creation function.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::list::AllocList;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use super::super::validation::{validate_enum, validate_non_empty};
use super::{hashed_key, new_dict, ANIMATION_FORMATS, INTERPOLATION_MODES, SKELETON_PRESETS};

/// Registers skeletal animation spec functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_skeletal_animation_spec_functions(builder);
}

#[starlark_module]
fn register_skeletal_animation_spec_functions(builder: &mut GlobalsBuilder) {
    /// Creates a complete skeletal animation spec with blender_clip_v1 recipe.
    ///
    /// # Arguments
    /// * `asset_id` - Kebab-case identifier for the asset
    /// * `seed` - Deterministic seed (0 to 2^32-1)
    /// * `output_path` - Output file path
    /// * `format` - Animation format: "glb" or "gltf"
    /// * `skeleton_preset` - Skeleton preset name (e.g., "humanoid_basic_v1")
    /// * `clip_name` - Name of the animation clip
    /// * `duration_seconds` - Duration of the animation in seconds
    /// * `fps` - Frames per second (default: 30)
    /// * `loop` - Whether animation should loop (default: False)
    /// * `keyframes` - List of animation keyframes (from animation_keyframe())
    /// * `interpolation` - Interpolation mode: "linear", "bezier", "constant" (default: "linear")
    /// * `export` - Optional export settings (from animation_export_settings())
    /// * `description` - Asset description (optional)
    /// * `tags` - Style tags (optional)
    /// * `license` - SPDX license identifier (default: "CC0-1.0")
    ///
    /// # Returns
    /// A complete spec dict ready for serialization.
    ///
    /// # Example
    /// ```starlark
    /// skeletal_animation_spec(
    ///     asset_id = "walk-cycle",
    ///     seed = 42,
    ///     output_path = "animations/walk.glb",
    ///     format = "glb",
    ///     skeleton_preset = "humanoid_basic_v1",
    ///     clip_name = "walk",
    ///     duration_seconds = 1.0,
    ///     keyframes = [
    ///         animation_keyframe(time = 0.0, bones = {
    ///             "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0])
    ///         }),
    ///         animation_keyframe(time = 0.5, bones = {
    ///             "upper_leg_l": bone_transform(rotation = [-15.0, 0.0, 0.0])
    ///         })
    ///     ],
    ///     loop = True
    /// )
    /// ```
    fn skeletal_animation_spec<'v>(
        #[starlark(require = named)] asset_id: &str,
        #[starlark(require = named)] seed: i64,
        #[starlark(require = named)] output_path: &str,
        #[starlark(require = named)] format: &str,
        #[starlark(require = named)] skeleton_preset: &str,
        #[starlark(require = named)] clip_name: &str,
        #[starlark(require = named)] duration_seconds: f64,
        #[starlark(default = 30)] fps: i32,
        #[starlark(default = false)] r#loop: bool,
        #[starlark(default = NoneType)] keyframes: Value<'v>,
        #[starlark(default = "linear")] interpolation: &str,
        #[starlark(default = NoneType)] export: Value<'v>,
        #[starlark(default = NoneType)] description: Value<'v>,
        #[starlark(default = NoneType)] tags: Value<'v>,
        #[starlark(default = "CC0-1.0")] license: &str,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate asset_id
        validate_non_empty(asset_id, "skeletal_animation_spec", "asset_id")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate clip_name
        validate_non_empty(clip_name, "skeletal_animation_spec", "clip_name")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate format
        validate_enum(format, ANIMATION_FORMATS, "skeletal_animation_spec", "format")
            .map_err(|e| anyhow::anyhow!(e))?;

        // Validate skeleton_preset
        validate_enum(
            skeleton_preset,
            SKELETON_PRESETS,
            "skeletal_animation_spec",
            "skeleton_preset",
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        // Validate interpolation
        validate_enum(
            interpolation,
            INTERPOLATION_MODES,
            "skeletal_animation_spec",
            "interpolation",
        )
        .map_err(|e| anyhow::anyhow!(e))?;

        // Validate seed
        if seed < 0 || seed > u32::MAX as i64 {
            return Err(anyhow::anyhow!(
                "S103: skeletal_animation_spec(): 'seed' must be in range 0 to {}, got {}",
                u32::MAX,
                seed
            ));
        }

        // Validate duration_seconds
        if duration_seconds <= 0.0 {
            return Err(anyhow::anyhow!(
                "S103: skeletal_animation_spec(): 'duration_seconds' must be positive, got {}",
                duration_seconds
            ));
        }

        // Validate fps
        if fps < 1 || fps > 120 {
            return Err(anyhow::anyhow!(
                "S103: skeletal_animation_spec(): 'fps' must be in range 1-120, got {}",
                fps
            ));
        }

        let mut spec = new_dict(heap);

        // spec_version
        spec.insert_hashed(hashed_key(heap, "spec_version"), heap.alloc(1).to_value());

        // asset_id
        spec.insert_hashed(
            hashed_key(heap, "asset_id"),
            heap.alloc_str(asset_id).to_value(),
        );

        // asset_type
        spec.insert_hashed(
            hashed_key(heap, "asset_type"),
            heap.alloc_str("skeletal_animation").to_value(),
        );

        // license
        spec.insert_hashed(
            hashed_key(heap, "license"),
            heap.alloc_str(license).to_value(),
        );

        // seed
        spec.insert_hashed(hashed_key(heap, "seed"), heap.alloc(seed).to_value());

        // outputs
        let mut output = new_dict(heap);
        output.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("primary").to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "format"),
            heap.alloc_str(format).to_value(),
        );
        output.insert_hashed(
            hashed_key(heap, "path"),
            heap.alloc_str(output_path).to_value(),
        );
        let outputs_list = heap.alloc(AllocList(vec![heap.alloc(output).to_value()]));
        spec.insert_hashed(hashed_key(heap, "outputs"), outputs_list);

        // Optional: description
        if !description.is_none() {
            if let Some(desc) = description.unpack_str() {
                spec.insert_hashed(
                    hashed_key(heap, "description"),
                    heap.alloc_str(desc).to_value(),
                );
            }
        }

        // Optional: style_tags
        if !tags.is_none() {
            spec.insert_hashed(hashed_key(heap, "style_tags"), tags);
        }

        // Build recipe
        let mut recipe = new_dict(heap);
        recipe.insert_hashed(
            hashed_key(heap, "kind"),
            heap.alloc_str("skeletal_animation.blender_clip_v1").to_value(),
        );

        // Create params
        let mut params = new_dict(heap);

        // skeleton_preset
        params.insert_hashed(
            hashed_key(heap, "skeleton_preset"),
            heap.alloc_str(skeleton_preset).to_value(),
        );

        // clip_name
        params.insert_hashed(
            hashed_key(heap, "clip_name"),
            heap.alloc_str(clip_name).to_value(),
        );

        // duration_seconds
        params.insert_hashed(
            hashed_key(heap, "duration_seconds"),
            heap.alloc(duration_seconds).to_value(),
        );

        // fps
        params.insert_hashed(hashed_key(heap, "fps"), heap.alloc(fps).to_value());

        // loop
        params.insert_hashed(hashed_key(heap, "loop"), heap.alloc(r#loop).to_value());

        // keyframes
        if !keyframes.is_none() {
            params.insert_hashed(hashed_key(heap, "keyframes"), keyframes);
        } else {
            // Default to empty list
            let empty_list: Vec<Value> = vec![];
            params.insert_hashed(
                hashed_key(heap, "keyframes"),
                heap.alloc(AllocList(empty_list)),
            );
        }

        // interpolation
        params.insert_hashed(
            hashed_key(heap, "interpolation"),
            heap.alloc_str(interpolation).to_value(),
        );

        // export
        if !export.is_none() {
            params.insert_hashed(hashed_key(heap, "export"), export);
        }

        recipe.insert_hashed(hashed_key(heap, "params"), heap.alloc(params).to_value());

        spec.insert_hashed(hashed_key(heap, "recipe"), heap.alloc(recipe).to_value());

        Ok(spec)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // skeletal_animation_spec() tests
    // ========================================================================

    #[test]
    fn test_skeletal_animation_spec_basic() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test-animation",
    seed = 42,
    output_path = "animations/test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "idle",
    duration_seconds = 2.0
)
"#,
        )
        .unwrap();

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-animation");
        assert_eq!(result["asset_type"], "skeletal_animation");
        assert_eq!(result["seed"], 42);
        assert_eq!(
            result["recipe"]["kind"],
            "skeletal_animation.blender_clip_v1"
        );
        assert_eq!(
            result["recipe"]["params"]["skeleton_preset"],
            "humanoid_basic_v1"
        );
        assert_eq!(result["recipe"]["params"]["clip_name"], "idle");
        assert_eq!(result["recipe"]["params"]["duration_seconds"], 2.0);
        assert_eq!(result["recipe"]["params"]["fps"], 30);
        assert_eq!(result["recipe"]["params"]["loop"], false);
        assert_eq!(result["recipe"]["params"]["interpolation"], "linear");
    }

    #[test]
    fn test_skeletal_animation_spec_with_keyframes() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "walk-cycle",
    seed = 100,
    output_path = "animations/walk.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "walk",
    duration_seconds = 1.0,
    fps = 24,
    loop = True,
    keyframes = [
        animation_keyframe(time = 0.0, bones = {
            "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0]),
            "upper_leg_r": bone_transform(rotation = [-15.0, 0.0, 0.0])
        }),
        animation_keyframe(time = 0.5, bones = {
            "upper_leg_l": bone_transform(rotation = [-15.0, 0.0, 0.0]),
            "upper_leg_r": bone_transform(rotation = [15.0, 0.0, 0.0])
        }),
        animation_keyframe(time = 1.0, bones = {
            "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0]),
            "upper_leg_r": bone_transform(rotation = [-15.0, 0.0, 0.0])
        })
    ]
)
"#,
        )
        .unwrap();

        assert_eq!(result["recipe"]["params"]["clip_name"], "walk");
        assert_eq!(result["recipe"]["params"]["fps"], 24);
        assert_eq!(result["recipe"]["params"]["loop"], true);

        let keyframes = result["recipe"]["params"]["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 3);
        assert_eq!(keyframes[0]["time"], 0.0);
        assert_eq!(keyframes[1]["time"], 0.5);
        assert_eq!(keyframes[2]["time"], 1.0);
    }

    #[test]
    fn test_skeletal_animation_spec_with_all_options() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "full-animation",
    seed = 42,
    output_path = "anims/full.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "attack",
    duration_seconds = 0.5,
    fps = 60,
    loop = False,
    keyframes = [
        animation_keyframe(time = 0.0, bones = {
            "spine": bone_transform(rotation = [0.0, 0.0, 0.0])
        })
    ],
    interpolation = "bezier",
    export = animation_export_settings(bake_transforms = True, optimize_keyframes = True),
    description = "Full animation test",
    tags = ["combat", "attack"]
)
"#,
        )
        .unwrap();

        assert_eq!(result["description"], "Full animation test");
        assert_eq!(result["style_tags"][0], "combat");
        assert_eq!(result["recipe"]["params"]["interpolation"], "bezier");
        assert!(result["recipe"]["params"]["export"].is_object());
        assert_eq!(result["recipe"]["params"]["export"]["optimize_keyframes"], true);
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_format() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.fbx",
    format = "fbx",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 1.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("glb"));
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_preset() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "invalid_skeleton",
    clip_name = "test",
    duration_seconds = 1.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("humanoid_basic_v1"));
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_interpolation() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 1.0,
    interpolation = "invalid"
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("linear"));
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_seed() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = -1,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 1.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("seed"));
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_duration() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 0.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("duration_seconds"));
        assert!(err.contains("positive"));
    }

    #[test]
    fn test_skeletal_animation_spec_invalid_fps() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 1.0,
    fps = 0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("fps"));
        assert!(err.contains("1-120"));
    }

    #[test]
    fn test_skeletal_animation_spec_empty_asset_id() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "test",
    duration_seconds = 1.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("asset_id"));
    }

    #[test]
    fn test_skeletal_animation_spec_empty_clip_name() {
        let result = eval_to_json(
            r#"
skeletal_animation_spec(
    asset_id = "test",
    seed = 42,
    output_path = "test.glb",
    format = "glb",
    skeleton_preset = "humanoid_basic_v1",
    clip_name = "",
    duration_seconds = 1.0
)
"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("clip_name"));
    }
}
