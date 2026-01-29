//! Animation parameter conversion
//!
//! Maps legacy ANIMATION dict keys to canonical `skeletal_animation.blender_clip_v1` params.

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Known legacy animation keys and their status
const KNOWN_KEYS: &[&str] = &[
    "name",
    "character",
    "input_armature",
    "duration_frames",
    "fps",
    "loop",
    "ground_offset",
    "poses",
    "phases",
    "ik_targets",
    "procedural_layers",
    "rig_setup",
    "save_blend",
    // Legacy aliases
    "rig",
    "skeleton",
];

/// Map legacy ANIMATION params to canonical skeletal_animation.blender_clip_v1
pub fn map_animation_params(data: &HashMap<String, Value>) -> Result<(Value, Vec<String>)> {
    let mut warnings = Vec::new();

    // Check for unknown keys
    for key in data.keys() {
        if !KNOWN_KEYS.contains(&key.as_str()) {
            warnings.push(format!(
                "Unknown animation key '{}'. This key will be ignored. \
                Consider updating the legacy spec or filing a feature request.",
                key
            ));
        }
    }

    // Extract skeleton preset from various legacy fields
    let skeleton_preset = extract_skeleton_preset(data, &mut warnings);

    // Extract clip name from legacy 'name' field
    let clip_name = data
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("animation")
        .to_string();

    // Extract duration
    let fps = data.get("fps").and_then(|v| v.as_u64()).unwrap_or(30) as u8;
    let duration_frames = data
        .get("duration_frames")
        .and_then(|v| v.as_u64())
        .unwrap_or(30) as u32;
    let duration_seconds = duration_frames as f64 / fps as f64;

    // Extract loop flag
    let loop_flag = data.get("loop").and_then(|v| v.as_bool()).unwrap_or(false);

    // Convert poses and phases to keyframes
    let keyframes = convert_poses_and_phases(data, fps, &mut warnings)?;

    // Convert IK setup from legacy ik_targets
    let ik_setup = convert_ik_setup(data, &mut warnings);

    // Convert export settings
    let export = convert_export_settings(data);

    // Build the canonical params
    let mut params = serde_json::json!({
        "skeleton_preset": skeleton_preset,
        "clip_name": clip_name,
        "duration_seconds": duration_seconds,
        "fps": fps,
        "loop": loop_flag,
        "keyframes": keyframes,
        "interpolation": "linear"
    });

    // Add IK setup if present
    if let Some(ik) = ik_setup {
        params["ik_keyframes"] = ik;
    }

    // Add export settings if present
    if let Some(exp) = export {
        params["export"] = exp;
    }

    // Handle procedural layers (warn if present, not fully supported)
    if data.contains_key("procedural_layers") {
        warnings.push(
            "Legacy 'procedural_layers' are not directly supported in the canonical format. \
            Consider using keyframe-based animation or IK targets instead."
                .to_string(),
        );
    }

    // Handle ground_offset
    if let Some(offset) = data.get("ground_offset") {
        warnings.push(format!(
            "Legacy 'ground_offset' ({}) not directly mapped. \
            Apply root motion offset in the animation clip or post-process.",
            offset
        ));
    }

    Ok((params, warnings))
}

/// Extract skeleton preset from legacy fields
fn extract_skeleton_preset(data: &HashMap<String, Value>, warnings: &mut Vec<String>) -> String {
    // Try various legacy field names for skeleton reference
    let skeleton_ref = data
        .get("character")
        .or_else(|| data.get("rig"))
        .or_else(|| data.get("skeleton"))
        .or_else(|| data.get("input_armature"))
        .and_then(|v| v.as_str());

    match skeleton_ref {
        Some(rig) => map_rig_to_preset(rig, warnings),
        None => {
            warnings.push(
                "No skeleton reference found (character, rig, skeleton, or input_armature). \
                Defaulting to 'humanoid_basic_v1'."
                    .to_string(),
            );
            "humanoid_basic_v1".to_string()
        }
    }
}

/// Map legacy rig names to canonical skeleton presets
fn map_rig_to_preset(rig: &str, warnings: &mut Vec<String>) -> String {
    let normalized = rig.to_lowercase().replace('-', "_");

    match normalized.as_str() {
        "humanoid" | "humanoid_basic" | "humanoid_basic_v1" | "human" | "biped" => {
            "humanoid_basic_v1".to_string()
        }
        _ => {
            // If it's a direct reference (like an asset_id), pass through
            // but warn that it might need manual review
            if rig.contains('/') || rig.contains(':') {
                // Looks like an asset reference
                warnings.push(format!(
                    "Skeleton reference '{}' looks like an asset reference. \
                    This will be passed through but may need manual verification.",
                    rig
                ));
                rig.to_string()
            } else {
                warnings.push(format!(
                    "Unknown skeleton rig '{}'. Defaulting to 'humanoid_basic_v1'. \
                    Review and update skeleton_preset if needed.",
                    rig
                ));
                "humanoid_basic_v1".to_string()
            }
        }
    }
}

/// Convert legacy poses and phases to canonical keyframes
fn convert_poses_and_phases(
    data: &HashMap<String, Value>,
    fps: u8,
    warnings: &mut Vec<String>,
) -> Result<Vec<Value>> {
    let poses = data.get("poses");
    let phases = data.get("phases");

    // If we have phases, they define the ordering and timing
    // If we only have poses, convert them directly
    match (poses, phases) {
        (Some(poses_val), Some(phases_val)) => {
            convert_phased_poses(poses_val, phases_val, fps, warnings)
        }
        (Some(poses_val), None) => convert_direct_poses(poses_val, fps, warnings),
        (None, Some(_)) => {
            warnings.push(
                "Found 'phases' without 'poses'. Phases reference poses by name, \
                so poses are required. Generating empty keyframes."
                    .to_string(),
            );
            Ok(vec![])
        }
        (None, None) => {
            warnings.push(
                "No poses or phases defined. Animation will have no keyframes. \
                Consider adding pose definitions."
                    .to_string(),
            );
            Ok(vec![])
        }
    }
}

/// Convert poses with phase ordering to keyframes
fn convert_phased_poses(
    poses_val: &Value,
    phases_val: &Value,
    _fps: u8,
    warnings: &mut Vec<String>,
) -> Result<Vec<Value>> {
    let Some(poses_obj) = poses_val.as_object() else {
        warnings.push("'poses' should be an object mapping pose names to definitions.".to_string());
        return Ok(vec![]);
    };

    let phase_names: Vec<String> = match phases_val {
        Value::Array(arr) => arr
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        _ => {
            warnings.push("'phases' should be an array of pose names.".to_string());
            return Ok(vec![]);
        }
    };

    if phase_names.is_empty() {
        warnings.push("'phases' array is empty.".to_string());
        return Ok(vec![]);
    }

    let mut keyframes = Vec::new();
    let frame_per_phase = if phase_names.len() > 1 {
        // Distribute phases across the animation duration
        // Each phase gets equal time
        1.0 / (phase_names.len() - 1) as f64
    } else {
        0.0
    };

    for (i, phase_name) in phase_names.iter().enumerate() {
        let time = if phase_names.len() > 1 {
            i as f64 * frame_per_phase
        } else {
            0.0
        };

        // Look up the pose definition
        if let Some(pose_def) = poses_obj.get(phase_name) {
            let bones = convert_pose_bones(pose_def, warnings);
            keyframes.push(serde_json::json!({
                "time": time,
                "bones": bones
            }));
        } else {
            warnings.push(format!(
                "Phase references unknown pose '{}'. Skipping this keyframe.",
                phase_name
            ));
        }
    }

    Ok(keyframes)
}

/// Convert poses directly (without phase ordering)
fn convert_direct_poses(
    poses_val: &Value,
    fps: u8,
    warnings: &mut Vec<String>,
) -> Result<Vec<Value>> {
    let Some(poses_obj) = poses_val.as_object() else {
        warnings.push("'poses' should be an object mapping pose names to definitions.".to_string());
        return Ok(vec![]);
    };

    let mut keyframes: Vec<(f64, Value)> = Vec::new();

    for (_pose_name, pose_def) in poses_obj {
        // Extract frame number from pose definition
        let frame = pose_def.get("frame").and_then(|v| v.as_i64()).unwrap_or(0);

        let time = frame as f64 / fps as f64;
        let bones = convert_pose_bones(pose_def, warnings);

        keyframes.push((
            time,
            serde_json::json!({
                "time": time,
                "bones": bones
            }),
        ));
    }

    // Sort by time
    keyframes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    Ok(keyframes.into_iter().map(|(_, kf)| kf).collect())
}

/// Convert pose bone definitions to canonical format
fn convert_pose_bones(pose_def: &Value, warnings: &mut Vec<String>) -> HashMap<String, Value> {
    let mut bones = HashMap::new();

    // Legacy format: { "frame": N, "bones": { "bone_name": {...} } }
    // Or: { "bone_name": {...} } directly
    let bones_data = if let Some(bones_obj) = pose_def.get("bones").and_then(|v| v.as_object()) {
        bones_obj.clone()
    } else if let Some(obj) = pose_def.as_object() {
        // Filter out non-bone keys
        obj.iter()
            .filter(|(k, _)| !["frame", "name", "time"].contains(&k.as_str()))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    } else {
        return bones;
    };

    for (bone_name, bone_data) in bones_data {
        let transform = convert_bone_transform(&bone_data, warnings);
        if !transform.is_null() {
            bones.insert(bone_name, transform);
        }
    }

    bones
}

/// Convert a single bone transform
fn convert_bone_transform(bone_data: &Value, _warnings: &mut Vec<String>) -> Value {
    let obj = match bone_data.as_object() {
        Some(o) => o,
        None => return Value::Null,
    };

    let mut transform = serde_json::Map::new();

    // Handle position/location
    if let Some(pos) = obj
        .get("position")
        .or_else(|| obj.get("location"))
        .or_else(|| obj.get("loc"))
    {
        if let Some(arr) = pos.as_array() {
            if arr.len() >= 3 {
                transform.insert(
                    "position".to_string(),
                    serde_json::json!([
                        arr[0].as_f64().unwrap_or(0.0),
                        arr[1].as_f64().unwrap_or(0.0),
                        arr[2].as_f64().unwrap_or(0.0)
                    ]),
                );
            }
        }
    }

    // Handle rotation (various formats)
    if let Some(rot) = obj
        .get("rotation")
        .or_else(|| obj.get("rot"))
        .or_else(|| obj.get("euler"))
    {
        if let Some(arr) = rot.as_array() {
            if arr.len() >= 3 {
                transform.insert(
                    "rotation".to_string(),
                    serde_json::json!([
                        arr[0].as_f64().unwrap_or(0.0),
                        arr[1].as_f64().unwrap_or(0.0),
                        arr[2].as_f64().unwrap_or(0.0)
                    ]),
                );
            }
        }
    } else {
        // Try pitch/yaw/roll format
        let pitch = obj.get("pitch").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let yaw = obj.get("yaw").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let roll = obj.get("roll").and_then(|v| v.as_f64()).unwrap_or(0.0);

        if pitch != 0.0 || yaw != 0.0 || roll != 0.0 {
            transform.insert(
                "rotation".to_string(),
                serde_json::json!([pitch, yaw, roll]),
            );
        }
    }

    // Handle scale
    if let Some(scale) = obj.get("scale").or_else(|| obj.get("scl")) {
        if let Some(arr) = scale.as_array() {
            if arr.len() >= 3 {
                transform.insert(
                    "scale".to_string(),
                    serde_json::json!([
                        arr[0].as_f64().unwrap_or(1.0),
                        arr[1].as_f64().unwrap_or(1.0),
                        arr[2].as_f64().unwrap_or(1.0)
                    ]),
                );
            }
        }
    }

    if transform.is_empty() {
        Value::Null
    } else {
        Value::Object(transform)
    }
}

/// Convert legacy ik_targets to canonical IK keyframes
fn convert_ik_setup(data: &HashMap<String, Value>, warnings: &mut Vec<String>) -> Option<Value> {
    let ik_targets = data.get("ik_targets")?;

    let obj = match ik_targets.as_object() {
        Some(o) => o,
        None => {
            warnings.push(
                "'ik_targets' should be an object mapping IK chain names to target definitions."
                    .to_string(),
            );
            return None;
        }
    };

    if obj.is_empty() {
        return None;
    }

    // Convert legacy IK targets to IK keyframes
    // Legacy format: { "foot_l": { "position": [...], "blend": 1.0 }, ... }
    // Canonical format: [{ "time": 0.0, "targets": { "ik_foot_l": { "position": [...] } } }]

    // For now, create a single keyframe at time 0 with all IK targets
    // More complex animation would need the rig_setup/phases to define timing
    let mut targets = HashMap::new();

    for (target_name, target_data) in obj {
        let canonical_name = normalize_ik_target_name(target_name);
        let target_transform = convert_ik_target_transform(target_data, warnings);
        if !target_transform.is_null() {
            targets.insert(canonical_name, target_transform);
        }
    }

    if targets.is_empty() {
        return None;
    }

    Some(serde_json::json!([{
        "time": 0.0,
        "targets": targets
    }]))
}

/// Normalize IK target names to canonical format
fn normalize_ik_target_name(name: &str) -> String {
    let normalized = name.to_lowercase();

    // Add "ik_" prefix if not present
    if normalized.starts_with("ik_") {
        normalized
    } else {
        format!("ik_{}", normalized)
    }
}

/// Convert a single IK target transform
fn convert_ik_target_transform(target_data: &Value, _warnings: &mut Vec<String>) -> Value {
    let obj = match target_data.as_object() {
        Some(o) => o,
        None => return Value::Null,
    };

    let mut transform = serde_json::Map::new();

    // Handle position
    if let Some(pos) = obj
        .get("position")
        .or_else(|| obj.get("location"))
        .or_else(|| obj.get("loc"))
    {
        if let Some(arr) = pos.as_array() {
            if arr.len() >= 3 {
                transform.insert(
                    "position".to_string(),
                    serde_json::json!([
                        arr[0].as_f64().unwrap_or(0.0),
                        arr[1].as_f64().unwrap_or(0.0),
                        arr[2].as_f64().unwrap_or(0.0)
                    ]),
                );
            }
        }
    }

    // Handle rotation
    if let Some(rot) = obj.get("rotation").or_else(|| obj.get("rot")) {
        if let Some(arr) = rot.as_array() {
            if arr.len() >= 3 {
                transform.insert(
                    "rotation".to_string(),
                    serde_json::json!([
                        arr[0].as_f64().unwrap_or(0.0),
                        arr[1].as_f64().unwrap_or(0.0),
                        arr[2].as_f64().unwrap_or(0.0)
                    ]),
                );
            }
        }
    }

    // Handle IK/FK blend
    if let Some(blend) = obj
        .get("blend")
        .or_else(|| obj.get("ik_fk_blend"))
        .or_else(|| obj.get("ikfk"))
    {
        if let Some(b) = blend.as_f64() {
            transform.insert(
                "ik_fk_blend".to_string(),
                serde_json::json!(b.clamp(0.0, 1.0)),
            );
        }
    }

    if transform.is_empty() {
        Value::Null
    } else {
        Value::Object(transform)
    }
}

/// Convert export settings
fn convert_export_settings(data: &HashMap<String, Value>) -> Option<Value> {
    let save_blend = data.get("save_blend").and_then(|v| v.as_bool());

    // Check for rig_setup export options
    let rig_setup = data.get("rig_setup").and_then(|v| v.as_object());
    let bake_settings = rig_setup
        .and_then(|r| r.get("bake"))
        .and_then(|v| v.as_object());

    let bake_transforms = bake_settings
        .and_then(|b| b.get("bake_transforms"))
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    let optimize_keyframes = bake_settings
        .and_then(|b| b.get("optimize"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if save_blend.is_none() && bake_settings.is_none() {
        return None;
    }

    Some(serde_json::json!({
        "bake_transforms": bake_transforms,
        "optimize_keyframes": optimize_keyframes,
        "separate_file": false,
        "save_blend": save_blend.unwrap_or(false)
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_animation_walk_cycle() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("walk_cycle")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            ("fps".to_string(), serde_json::json!(30)),
            ("duration_frames".to_string(), serde_json::json!(60)),
            ("loop".to_string(), serde_json::json!(true)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "contact_l": {
                        "frame": 0,
                        "bones": {
                            "upper_leg_l": { "pitch": 20.0 },
                            "upper_leg_r": { "pitch": -15.0 }
                        }
                    },
                    "passing_r": {
                        "frame": 15,
                        "bones": {
                            "upper_leg_l": { "pitch": 0.0 },
                            "upper_leg_r": { "pitch": 0.0 }
                        }
                    },
                    "contact_r": {
                        "frame": 30,
                        "bones": {
                            "upper_leg_l": { "pitch": -15.0 },
                            "upper_leg_r": { "pitch": 20.0 }
                        }
                    },
                    "passing_l": {
                        "frame": 45,
                        "bones": {
                            "upper_leg_l": { "pitch": 0.0 },
                            "upper_leg_r": { "pitch": 0.0 }
                        }
                    }
                }),
            ),
            (
                "phases".to_string(),
                serde_json::json!(["contact_l", "passing_r", "contact_r", "passing_l"]),
            ),
        ]);

        let (params, warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "walk_cycle");
        assert_eq!(
            params["skeleton_preset"].as_str().unwrap(),
            "humanoid_basic_v1"
        );
        assert_eq!(params["fps"].as_u64().unwrap(), 30);
        assert!(params["loop"].as_bool().unwrap());
        assert_eq!(params["duration_seconds"].as_f64().unwrap(), 2.0); // 60 frames / 30 fps

        let keyframes = params["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 4);

        // First keyframe at time 0
        assert_eq!(keyframes[0]["time"].as_f64().unwrap(), 0.0);

        // Warnings should be minimal for valid input
        assert!(
            warnings.is_empty() || warnings.iter().all(|w| !w.contains("error")),
            "unexpected error warnings: {:?}",
            warnings
        );
    }

    #[test]
    fn test_map_animation_with_ik_targets() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("idle")),
            (
                "character".to_string(),
                serde_json::json!("humanoid_basic_v1"),
            ),
            ("fps".to_string(), serde_json::json!(24)),
            ("duration_frames".to_string(), serde_json::json!(48)),
            ("loop".to_string(), serde_json::json!(true)),
            (
                "ik_targets".to_string(),
                serde_json::json!({
                    "foot_l": {
                        "position": [0.1, 0.0, 0.0],
                        "blend": 1.0
                    },
                    "foot_r": {
                        "position": [-0.1, 0.0, 0.0],
                        "blend": 1.0
                    }
                }),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "idle");
        assert_eq!(
            params["skeleton_preset"].as_str().unwrap(),
            "humanoid_basic_v1"
        );

        // Check IK keyframes
        let ik_keyframes = params["ik_keyframes"].as_array().unwrap();
        assert_eq!(ik_keyframes.len(), 1);

        let targets = ik_keyframes[0]["targets"].as_object().unwrap();
        assert!(targets.contains_key("ik_foot_l"));
        assert!(targets.contains_key("ik_foot_r"));

        let foot_l = &targets["ik_foot_l"];
        assert_eq!(foot_l["position"][0].as_f64().unwrap(), 0.1);
        assert_eq!(foot_l["ik_fk_blend"].as_f64().unwrap(), 1.0);
    }

    #[test]
    fn test_map_animation_run_cycle() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("run_cycle")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            ("fps".to_string(), serde_json::json!(30)),
            ("duration_frames".to_string(), serde_json::json!(20)),
            ("loop".to_string(), serde_json::json!(true)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "flight_l": {
                        "frame": 0,
                        "bones": {
                            "upper_leg_l": { "pitch": 45.0 },
                            "upper_leg_r": { "pitch": -30.0 },
                            "lower_leg_l": { "pitch": -90.0 },
                            "lower_leg_r": { "pitch": -45.0 }
                        }
                    },
                    "contact_r": {
                        "frame": 5,
                        "bones": {
                            "upper_leg_l": { "pitch": -20.0 },
                            "upper_leg_r": { "pitch": 10.0 }
                        }
                    },
                    "flight_r": {
                        "frame": 10,
                        "bones": {
                            "upper_leg_l": { "pitch": -30.0 },
                            "upper_leg_r": { "pitch": 45.0 }
                        }
                    },
                    "contact_l": {
                        "frame": 15,
                        "bones": {
                            "upper_leg_l": { "pitch": 10.0 },
                            "upper_leg_r": { "pitch": -20.0 }
                        }
                    }
                }),
            ),
            (
                "phases".to_string(),
                serde_json::json!(["flight_l", "contact_r", "flight_r", "contact_l"]),
            ),
        ]);

        let (params, warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "run_cycle");
        assert!(params["loop"].as_bool().unwrap());

        let keyframes = params["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 4);

        assert!(warnings.is_empty() || warnings.iter().all(|w| !w.contains("error")));
    }

    #[test]
    fn test_map_animation_jump() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("jump")),
            ("skeleton".to_string(), serde_json::json!("humanoid_basic")),
            ("fps".to_string(), serde_json::json!(30)),
            ("duration_frames".to_string(), serde_json::json!(45)),
            ("loop".to_string(), serde_json::json!(false)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "crouch": {
                        "frame": 0,
                        "bones": {
                            "hips": { "position": [0.0, 0.0, -0.2] },
                            "upper_leg_l": { "pitch": 60.0 },
                            "upper_leg_r": { "pitch": 60.0 },
                            "lower_leg_l": { "pitch": -120.0 },
                            "lower_leg_r": { "pitch": -120.0 }
                        }
                    },
                    "launch": {
                        "frame": 10,
                        "bones": {
                            "hips": { "position": [0.0, 0.0, 0.0] },
                            "upper_leg_l": { "pitch": -15.0 },
                            "upper_leg_r": { "pitch": -15.0 }
                        }
                    },
                    "apex": {
                        "frame": 25,
                        "bones": {
                            "hips": { "position": [0.0, 0.0, 0.5] },
                            "upper_leg_l": { "pitch": 0.0 },
                            "upper_leg_r": { "pitch": 0.0 }
                        }
                    },
                    "land": {
                        "frame": 45,
                        "bones": {
                            "hips": { "position": [0.0, 0.0, -0.15] },
                            "upper_leg_l": { "pitch": 45.0 },
                            "upper_leg_r": { "pitch": 45.0 }
                        }
                    }
                }),
            ),
            (
                "phases".to_string(),
                serde_json::json!(["crouch", "launch", "apex", "land"]),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "jump");
        assert!(!params["loop"].as_bool().unwrap());
        assert_eq!(
            params["skeleton_preset"].as_str().unwrap(),
            "humanoid_basic_v1"
        );

        let keyframes = params["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 4);

        // Check that position transforms are preserved
        let first_bones = keyframes[0]["bones"].as_object().unwrap();
        let hips = first_bones.get("hips").unwrap();
        assert!(hips.get("position").is_some());
    }

    #[test]
    fn test_map_animation_attack() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("punch")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            ("fps".to_string(), serde_json::json!(60)),
            ("duration_frames".to_string(), serde_json::json!(30)),
            ("loop".to_string(), serde_json::json!(false)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "windup": {
                        "frame": 0,
                        "bones": {
                            "upper_arm_r": { "pitch": -30.0, "yaw": 45.0 },
                            "lower_arm_r": { "pitch": -90.0 },
                            "chest": { "yaw": 20.0 }
                        }
                    },
                    "strike": {
                        "frame": 10,
                        "bones": {
                            "upper_arm_r": { "pitch": 60.0, "yaw": 0.0 },
                            "lower_arm_r": { "pitch": 0.0 },
                            "chest": { "yaw": -10.0 }
                        }
                    },
                    "recover": {
                        "frame": 30,
                        "bones": {
                            "upper_arm_r": { "pitch": 0.0, "yaw": 0.0 },
                            "lower_arm_r": { "pitch": 0.0 },
                            "chest": { "yaw": 0.0 }
                        }
                    }
                }),
            ),
            (
                "phases".to_string(),
                serde_json::json!(["windup", "strike", "recover"]),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "punch");
        assert!(!params["loop"].as_bool().unwrap());
        assert_eq!(params["fps"].as_u64().unwrap(), 60);

        let keyframes = params["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 3);

        // Check that rotation includes yaw
        let windup_bones = keyframes[0]["bones"].as_object().unwrap();
        let upper_arm = windup_bones.get("upper_arm_r").unwrap();
        let rotation = upper_arm["rotation"].as_array().unwrap();
        assert_eq!(rotation[0].as_f64().unwrap(), -30.0); // pitch
        assert_eq!(rotation[1].as_f64().unwrap(), 45.0); // yaw
    }

    #[test]
    fn test_map_animation_idle() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("idle_breathe")),
            (
                "character".to_string(),
                serde_json::json!("humanoid_basic_v1"),
            ),
            ("fps".to_string(), serde_json::json!(24)),
            ("duration_frames".to_string(), serde_json::json!(72)),
            ("loop".to_string(), serde_json::json!(true)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "inhale": {
                        "frame": 0,
                        "bones": {
                            "chest": { "scale": [1.0, 1.02, 1.0] },
                            "spine": { "pitch": -2.0 }
                        }
                    },
                    "exhale": {
                        "frame": 36,
                        "bones": {
                            "chest": { "scale": [1.0, 1.0, 1.0] },
                            "spine": { "pitch": 2.0 }
                        }
                    }
                }),
            ),
            (
                "phases".to_string(),
                serde_json::json!(["inhale", "exhale"]),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        assert_eq!(params["clip_name"].as_str().unwrap(), "idle_breathe");
        assert!(params["loop"].as_bool().unwrap());
        assert_eq!(params["duration_seconds"].as_f64().unwrap(), 3.0); // 72 frames / 24 fps

        let keyframes = params["keyframes"].as_array().unwrap();
        assert_eq!(keyframes.len(), 2);

        // Check that scale transforms are preserved
        let inhale_bones = keyframes[0]["bones"].as_object().unwrap();
        let chest = inhale_bones.get("chest").unwrap();
        assert!(chest.get("scale").is_some());
    }

    #[test]
    fn test_unknown_keys_warning() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("test")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            (
                "unknown_field".to_string(),
                serde_json::json!("should warn"),
            ),
            (
                "another_unknown".to_string(),
                serde_json::json!("also warn"),
            ),
        ]);

        let (_params, warnings) = map_animation_params(&data).unwrap();

        assert!(warnings.iter().any(|w| w.contains("unknown_field")));
        assert!(warnings.iter().any(|w| w.contains("another_unknown")));
    }

    #[test]
    fn test_procedural_layers_warning() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("test")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            (
                "procedural_layers".to_string(),
                serde_json::json!([{ "type": "noise", "bone": "head" }]),
            ),
        ]);

        let (_params, warnings) = map_animation_params(&data).unwrap();

        assert!(warnings.iter().any(|w| w.contains("procedural_layers")));
    }

    #[test]
    fn test_ground_offset_warning() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("test")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            ("ground_offset".to_string(), serde_json::json!(0.05)),
        ]);

        let (_params, warnings) = map_animation_params(&data).unwrap();

        assert!(warnings.iter().any(|w| w.contains("ground_offset")));
    }

    #[test]
    fn test_ik_target_name_normalization() {
        assert_eq!(normalize_ik_target_name("foot_l"), "ik_foot_l");
        assert_eq!(normalize_ik_target_name("ik_foot_l"), "ik_foot_l");
        assert_eq!(normalize_ik_target_name("FOOT_R"), "ik_foot_r");
        assert_eq!(normalize_ik_target_name("IK_HAND_L"), "ik_hand_l");
    }

    #[test]
    fn test_skeleton_preset_mapping() {
        let mut warnings = Vec::new();

        assert_eq!(
            map_rig_to_preset("humanoid", &mut warnings),
            "humanoid_basic_v1"
        );
        assert_eq!(
            map_rig_to_preset("humanoid_basic", &mut warnings),
            "humanoid_basic_v1"
        );
        assert_eq!(
            map_rig_to_preset("human", &mut warnings),
            "humanoid_basic_v1"
        );
        assert_eq!(
            map_rig_to_preset("biped", &mut warnings),
            "humanoid_basic_v1"
        );

        // Unknown rig should default with warning
        warnings.clear();
        assert_eq!(
            map_rig_to_preset("quadruped", &mut warnings),
            "humanoid_basic_v1"
        );
        assert!(!warnings.is_empty());

        // Asset reference should pass through
        warnings.clear();
        assert_eq!(
            map_rig_to_preset("assets/skeletons/custom_rig", &mut warnings),
            "assets/skeletons/custom_rig"
        );
        assert!(warnings.iter().any(|w| w.contains("asset reference")));
    }

    #[test]
    fn test_export_settings() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("test")),
            ("save_blend".to_string(), serde_json::json!(true)),
            (
                "rig_setup".to_string(),
                serde_json::json!({
                    "bake": {
                        "bake_transforms": true,
                        "optimize": true
                    }
                }),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        let export = params["export"].as_object().unwrap();
        assert!(export["save_blend"].as_bool().unwrap());
        assert!(export["bake_transforms"].as_bool().unwrap());
        assert!(export["optimize_keyframes"].as_bool().unwrap());
    }

    #[test]
    fn test_poses_without_phases() {
        let data = HashMap::from([
            ("name".to_string(), serde_json::json!("test")),
            ("rig".to_string(), serde_json::json!("humanoid")),
            ("fps".to_string(), serde_json::json!(30)),
            (
                "poses".to_string(),
                serde_json::json!({
                    "pose_a": { "frame": 0, "bones": { "arm_l": { "pitch": 10.0 } } },
                    "pose_b": { "frame": 15, "bones": { "arm_l": { "pitch": 20.0 } } }
                }),
            ),
        ]);

        let (params, _warnings) = map_animation_params(&data).unwrap();

        let keyframes = params["keyframes"].as_array().unwrap();
        // Should have 2 keyframes, sorted by time
        assert_eq!(keyframes.len(), 2);
        assert!(keyframes[0]["time"].as_f64().unwrap() <= keyframes[1]["time"].as_f64().unwrap());
    }
}
