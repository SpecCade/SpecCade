//! Animation helper functions for building animation keyframes and transforms.

use starlark::environment::GlobalsBuilder;
use starlark::starlark_module;
use starlark::values::{dict::Dict, none::NoneType, Heap, Value, ValueLike};

use super::{hashed_key, new_dict};

/// Registers animation helper functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    register_animation_helper_functions(builder);
}

#[starlark_module]
fn register_animation_helper_functions(builder: &mut GlobalsBuilder) {
    /// Creates a bone transform with position, rotation, and/or scale.
    ///
    /// # Arguments
    /// * `position` - Position offset [X, Y, Z] (optional)
    /// * `rotation` - Rotation in euler angles [X, Y, Z] degrees (optional)
    /// * `scale` - Scale [X, Y, Z] (optional)
    ///
    /// At least one of position, rotation, or scale must be provided.
    ///
    /// # Returns
    /// A dict suitable for use in animation_keyframe's bones map.
    ///
    /// # Example
    /// ```starlark
    /// bone_transform(rotation = [15.0, 0.0, 0.0])
    /// bone_transform(position = [0.1, 0.0, 0.0], rotation = [0.0, 5.0, 0.0])
    /// ```
    fn bone_transform<'v>(
        #[starlark(default = NoneType)] position: Value<'v>,
        #[starlark(default = NoneType)] rotation: Value<'v>,
        #[starlark(default = NoneType)] scale: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Must have at least one transform component
        if position.is_none() && rotation.is_none() && scale.is_none() {
            return Err(anyhow::anyhow!(
                "S101: bone_transform(): at least one of 'position', 'rotation', or 'scale' must be provided"
            ));
        }

        let mut result = new_dict(heap);

        if !position.is_none() {
            result.insert_hashed(hashed_key(heap, "position"), position);
        }

        if !rotation.is_none() {
            result.insert_hashed(hashed_key(heap, "rotation"), rotation);
        }

        if !scale.is_none() {
            result.insert_hashed(hashed_key(heap, "scale"), scale);
        }

        Ok(result)
    }

    /// Creates an animation keyframe at a specific time with bone transforms.
    ///
    /// # Arguments
    /// * `time` - Time in seconds
    /// * `bones` - Dict mapping bone names to bone transforms (from bone_transform())
    ///
    /// # Returns
    /// A dict suitable for use in skeletal_animation_spec's keyframes list.
    ///
    /// # Example
    /// ```starlark
    /// animation_keyframe(
    ///     time = 0.0,
    ///     bones = {
    ///         "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0]),
    ///         "upper_leg_r": bone_transform(rotation = [-15.0, 0.0, 0.0])
    ///     }
    /// )
    /// ```
    fn animation_keyframe<'v>(
        #[starlark(require = named)] time: f64,
        #[starlark(require = named)] bones: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate time
        if time < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: animation_keyframe(): 'time' must be non-negative, got {}",
                time
            ));
        }

        let mut result = new_dict(heap);

        result.insert_hashed(hashed_key(heap, "time"), heap.alloc(time).to_value());
        result.insert_hashed(hashed_key(heap, "bones"), bones);

        Ok(result)
    }

    /// Creates animation export settings.
    ///
    /// # Arguments
    /// * `bake_transforms` - Bake all transforms to keyframes (default: True)
    /// * `optimize_keyframes` - Remove redundant keyframes (default: False)
    /// * `separate_file` - Export as separate file (default: False)
    /// * `save_blend` - Save .blend file alongside output (default: False)
    ///
    /// # Returns
    /// A dict suitable for use in skeletal_animation_spec's export parameter.
    ///
    /// # Example
    /// ```starlark
    /// animation_export_settings(bake_transforms = True, optimize_keyframes = True)
    /// ```
    fn animation_export_settings<'v>(
        #[starlark(default = true)] bake_transforms: bool,
        #[starlark(default = false)] optimize_keyframes: bool,
        #[starlark(default = false)] separate_file: bool,
        #[starlark(default = false)] save_blend: bool,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        let mut result = new_dict(heap);

        result.insert_hashed(
            hashed_key(heap, "bake_transforms"),
            heap.alloc(bake_transforms).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "optimize_keyframes"),
            heap.alloc(optimize_keyframes).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "separate_file"),
            heap.alloc(separate_file).to_value(),
        );
        result.insert_hashed(
            hashed_key(heap, "save_blend"),
            heap.alloc(save_blend).to_value(),
        );

        Ok(result)
    }

    /// Creates an IK target transform for IK-driven animation.
    ///
    /// # Arguments
    /// * `position` - World position [X, Y, Z] (optional)
    /// * `rotation` - World rotation in euler angles [X, Y, Z] degrees (optional)
    /// * `ik_fk_blend` - IK/FK blend value 0.0-1.0 (0=FK, 1=IK) (optional)
    ///
    /// At least position or rotation must be provided.
    ///
    /// # Returns
    /// A dict suitable for use in IK keyframe targets.
    ///
    /// # Example
    /// ```starlark
    /// ik_target_transform(position = [0.1, 0.0, 0.0], ik_fk_blend = 1.0)
    /// ```
    fn ik_target_transform<'v>(
        #[starlark(default = NoneType)] position: Value<'v>,
        #[starlark(default = NoneType)] rotation: Value<'v>,
        #[starlark(default = NoneType)] ik_fk_blend: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Must have at least position or rotation
        if position.is_none() && rotation.is_none() {
            return Err(anyhow::anyhow!(
                "S101: ik_target_transform(): at least 'position' or 'rotation' must be provided"
            ));
        }

        let mut result = new_dict(heap);

        if !position.is_none() {
            result.insert_hashed(hashed_key(heap, "position"), position);
        }

        if !rotation.is_none() {
            result.insert_hashed(hashed_key(heap, "rotation"), rotation);
        }

        if !ik_fk_blend.is_none() {
            result.insert_hashed(hashed_key(heap, "ik_fk_blend"), ik_fk_blend);
        }

        Ok(result)
    }

    /// Creates an IK keyframe at a specific time with IK target transforms.
    ///
    /// # Arguments
    /// * `time` - Time in seconds
    /// * `targets` - Dict mapping IK chain names to IK target transforms
    ///
    /// # Returns
    /// A dict for use in IK-based animation.
    ///
    /// # Example
    /// ```starlark
    /// ik_keyframe(
    ///     time = 0.5,
    ///     targets = {
    ///         "ik_leg_l": ik_target_transform(position = [0.1, 0.0, 0.0]),
    ///         "ik_leg_r": ik_target_transform(position = [-0.1, 0.0, 0.0])
    ///     }
    /// )
    /// ```
    fn ik_keyframe<'v>(
        #[starlark(require = named)] time: f64,
        #[starlark(require = named)] targets: Value<'v>,
        heap: &'v Heap,
    ) -> anyhow::Result<Dict<'v>> {
        // Validate time
        if time < 0.0 {
            return Err(anyhow::anyhow!(
                "S103: ik_keyframe(): 'time' must be non-negative, got {}",
                time
            ));
        }

        let mut result = new_dict(heap);

        result.insert_hashed(hashed_key(heap, "time"), heap.alloc(time).to_value());
        result.insert_hashed(hashed_key(heap, "targets"), targets);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // bone_transform() tests
    // ========================================================================

    #[test]
    fn test_bone_transform_rotation_only() {
        let result = eval_to_json(r#"bone_transform(rotation = [15.0, 0.0, 0.0])"#).unwrap();

        assert!(result["rotation"].is_array());
        assert_eq!(result["rotation"][0], 15.0);
        assert!(result.get("position").is_none());
        assert!(result.get("scale").is_none());
    }

    #[test]
    fn test_bone_transform_position_only() {
        let result = eval_to_json(r#"bone_transform(position = [0.1, 0.2, 0.3])"#).unwrap();

        assert!(result["position"].is_array());
        assert_eq!(result["position"][0], 0.1);
        assert!(result.get("rotation").is_none());
    }

    #[test]
    fn test_bone_transform_all_components() {
        let result = eval_to_json(
            r#"bone_transform(position = [1.0, 0.0, 0.0], rotation = [0.0, 90.0, 0.0], scale = [1.0, 1.0, 1.0])"#,
        )
        .unwrap();

        assert!(result["position"].is_array());
        assert!(result["rotation"].is_array());
        assert!(result["scale"].is_array());
    }

    #[test]
    fn test_bone_transform_requires_at_least_one() {
        let result = eval_to_json(r#"bone_transform()"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
        assert!(err.contains("at least one"));
    }

    // ========================================================================
    // animation_keyframe() tests
    // ========================================================================

    #[test]
    fn test_animation_keyframe_basic() {
        let result = eval_to_json(
            r#"
animation_keyframe(
    time = 0.0,
    bones = {
        "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0])
    }
)
"#,
        )
        .unwrap();

        assert_eq!(result["time"], 0.0);
        assert!(result["bones"].is_object());
        assert!(result["bones"]["upper_leg_l"]["rotation"].is_array());
    }

    #[test]
    fn test_animation_keyframe_multiple_bones() {
        let result = eval_to_json(
            r#"
animation_keyframe(
    time = 0.5,
    bones = {
        "upper_leg_l": bone_transform(rotation = [15.0, 0.0, 0.0]),
        "upper_leg_r": bone_transform(rotation = [-15.0, 0.0, 0.0]),
        "spine": bone_transform(rotation = [0.0, 5.0, 0.0])
    }
)
"#,
        )
        .unwrap();

        assert_eq!(result["time"], 0.5);
        let bones = result["bones"].as_object().unwrap();
        assert_eq!(bones.len(), 3);
    }

    #[test]
    fn test_animation_keyframe_negative_time() {
        let result = eval_to_json(
            r#"animation_keyframe(time = -1.0, bones = {"test": bone_transform(rotation = [0.0, 0.0, 0.0])})"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
        assert!(err.contains("non-negative"));
    }

    // ========================================================================
    // animation_export_settings() tests
    // ========================================================================

    #[test]
    fn test_animation_export_settings_defaults() {
        let result = eval_to_json(r#"animation_export_settings()"#).unwrap();

        assert_eq!(result["bake_transforms"], true);
        assert_eq!(result["optimize_keyframes"], false);
        assert_eq!(result["separate_file"], false);
        assert_eq!(result["save_blend"], false);
    }

    #[test]
    fn test_animation_export_settings_custom() {
        let result = eval_to_json(
            r#"animation_export_settings(bake_transforms = False, optimize_keyframes = True, save_blend = True)"#,
        )
        .unwrap();

        assert_eq!(result["bake_transforms"], false);
        assert_eq!(result["optimize_keyframes"], true);
        assert_eq!(result["separate_file"], false);
        assert_eq!(result["save_blend"], true);
    }

    // ========================================================================
    // ik_target_transform() tests
    // ========================================================================

    #[test]
    fn test_ik_target_transform_position_only() {
        let result = eval_to_json(r#"ik_target_transform(position = [0.1, 0.0, 0.0])"#).unwrap();

        assert!(result["position"].is_array());
        assert_eq!(result["position"][0], 0.1);
        assert!(result.get("rotation").is_none());
    }

    #[test]
    fn test_ik_target_transform_with_blend() {
        let result =
            eval_to_json(r#"ik_target_transform(position = [0.0, 0.1, 0.0], ik_fk_blend = 0.5)"#)
                .unwrap();

        assert!(result["position"].is_array());
        assert_eq!(result["ik_fk_blend"], 0.5);
    }

    #[test]
    fn test_ik_target_transform_requires_position_or_rotation() {
        let result = eval_to_json(r#"ik_target_transform()"#);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    // ========================================================================
    // ik_keyframe() tests
    // ========================================================================

    #[test]
    fn test_ik_keyframe_basic() {
        let result = eval_to_json(
            r#"
ik_keyframe(
    time = 0.5,
    targets = {
        "ik_leg_l": ik_target_transform(position = [0.1, 0.0, 0.0])
    }
)
"#,
        )
        .unwrap();

        assert_eq!(result["time"], 0.5);
        assert!(result["targets"].is_object());
    }

    #[test]
    fn test_ik_keyframe_negative_time() {
        let result = eval_to_json(
            r#"ik_keyframe(time = -1.0, targets = {"ik": ik_target_transform(position = [0.0, 0.0, 0.0])})"#,
        );
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }
}
