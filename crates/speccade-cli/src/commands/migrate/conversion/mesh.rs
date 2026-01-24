//! Mesh parameter conversion

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Map legacy MESH params to canonical static_mesh.blender_primitives_v1
pub fn map_mesh_params(data: &HashMap<String, Value>) -> Result<(Value, Vec<String>)> {
    let mut warnings = Vec::new();

    // Extract primitive type
    let primitive = data
        .get("primitive")
        .and_then(|v| v.as_str())
        .unwrap_or("cube");

    let base_primitive = match primitive {
        "cube" | "box" => "cube",
        "sphere" | "uv_sphere" => "sphere",
        "ico_sphere" | "icosphere" => "ico_sphere",
        "cylinder" => "cylinder",
        "cone" => "cone",
        "torus" => "torus",
        "plane" => "plane",
        _ => {
            warnings.push(format!(
                "Unknown primitive '{}'. Defaulting to cube.",
                primitive
            ));
            "cube"
        }
    };

    // Extract dimensions from params, scale, and primitive-specific fields
    let dimensions = extract_dimensions(data, primitive);

    // Extract modifiers
    let modifiers = convert_modifiers(data.get("modifiers"), &mut warnings);

    // Extract UV projection
    let uv_projection = convert_uv_settings(data.get("uv"), &mut warnings);

    // Extract export settings
    let export_settings = convert_export_settings(data.get("export"));

    // Build params
    let mut params = serde_json::json!({
        "base_primitive": base_primitive,
        "dimensions": dimensions
    });

    if !modifiers.is_empty() {
        params["modifiers"] = serde_json::json!(modifiers);
    }
    if let Some(uv) = uv_projection {
        params["uv_projection"] = uv;
    }
    if let Some(exp) = export_settings {
        params["export"] = exp;
    }

    // Handle shade mode
    if let Some(shade) = data.get("shade").and_then(|v| v.as_str()) {
        if shade == "smooth" {
            params["normals"] = serde_json::json!({
                "preset": "auto_smooth",
                "angle": 30.0
            });
        }
    }

    // Warn about unsupported fields
    if data.contains_key("location") || data.contains_key("rotation") {
        warnings.push(
            "Legacy 'location'/'rotation' transforms not migrated. Apply transforms in recipe or post-process.".to_string()
        );
    }

    Ok((params, warnings))
}

/// Extract dimensions from various legacy fields
fn extract_dimensions(data: &HashMap<String, Value>, primitive: &str) -> [f64; 3] {
    // Try scale first
    if let Some(scale) = data.get("scale") {
        if let Some(arr) = scale.as_array() {
            if arr.len() >= 3 {
                let x = arr[0].as_f64().unwrap_or(1.0);
                let y = arr[1].as_f64().unwrap_or(1.0);
                let z = arr[2].as_f64().unwrap_or(1.0);
                return [x, y, z];
            }
        }
    }

    // Try params
    if let Some(params) = data.get("params").and_then(|p| p.as_object()) {
        match primitive {
            "cube" | "box" => {
                let size = params.get("size").and_then(|v| v.as_f64()).unwrap_or(1.0);
                return [size, size, size];
            }
            "sphere" | "uv_sphere" | "ico_sphere" | "icosphere" => {
                let radius = params.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let diameter = radius * 2.0;
                return [diameter, diameter, diameter];
            }
            "cylinder" => {
                let radius = params.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.5);
                let depth = params.get("depth").and_then(|v| v.as_f64()).unwrap_or(2.0);
                let diameter = radius * 2.0;
                return [diameter, diameter, depth];
            }
            "cone" => {
                let radius1 = params
                    .get("radius1")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.5);
                let depth = params.get("depth").and_then(|v| v.as_f64()).unwrap_or(2.0);
                let diameter = radius1 * 2.0;
                return [diameter, diameter, depth];
            }
            "torus" => {
                let major_radius = params
                    .get("major_radius")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(1.0);
                let minor_radius = params
                    .get("minor_radius")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.25);
                let outer = (major_radius + minor_radius) * 2.0;
                let thickness = minor_radius * 2.0;
                return [outer, outer, thickness];
            }
            "plane" => {
                let size = params.get("size").and_then(|v| v.as_f64()).unwrap_or(2.0);
                return [size, size, 0.0];
            }
            _ => {}
        }
    }

    // Default dimensions
    [1.0, 1.0, 1.0]
}

/// Convert legacy modifiers to canonical format
fn convert_modifiers(modifiers: Option<&Value>, warnings: &mut Vec<String>) -> Vec<Value> {
    let Some(Value::Array(mods)) = modifiers else {
        return vec![];
    };

    mods.iter()
        .filter_map(|m| {
            let obj = m.as_object()?;
            let mod_type = obj.get("type").and_then(|v| v.as_str())?;

            match mod_type {
                "bevel" => {
                    let width = obj.get("width").and_then(|v| v.as_f64()).unwrap_or(0.02);
                    let segments = obj.get("segments").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
                    let angle_limit = obj.get("angle_limit").and_then(|v| v.as_f64());

                    let mut result = serde_json::json!({
                        "type": "bevel",
                        "width": width,
                        "segments": segments
                    });
                    if let Some(al) = angle_limit {
                        result["angle_limit"] = serde_json::json!(al);
                    }
                    Some(result)
                }
                "subdivision" | "subsurf" => {
                    let levels = obj.get("levels").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
                    let render_levels = obj
                        .get("render_levels")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(levels as u64) as u8;
                    Some(serde_json::json!({
                        "type": "subdivision",
                        "levels": levels,
                        "render_levels": render_levels
                    }))
                }
                "decimate" => {
                    let ratio = obj.get("ratio").and_then(|v| v.as_f64()).unwrap_or(0.5);
                    Some(serde_json::json!({
                        "type": "decimate",
                        "ratio": ratio
                    }))
                }
                "mirror" => {
                    let axis_x = obj.get("axis_x").and_then(|v| v.as_bool()).unwrap_or(true);
                    let axis_y = obj.get("axis_y").and_then(|v| v.as_bool()).unwrap_or(false);
                    let axis_z = obj.get("axis_z").and_then(|v| v.as_bool()).unwrap_or(false);
                    Some(serde_json::json!({
                        "type": "mirror",
                        "axis_x": axis_x,
                        "axis_y": axis_y,
                        "axis_z": axis_z
                    }))
                }
                "solidify" => {
                    let thickness = obj.get("thickness").and_then(|v| v.as_f64()).unwrap_or(0.1);
                    let offset = obj.get("offset").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    Some(serde_json::json!({
                        "type": "solidify",
                        "thickness": thickness,
                        "offset": offset
                    }))
                }
                "array" => {
                    let count = obj.get("count").and_then(|v| v.as_u64()).unwrap_or(2) as u32;
                    let offset = obj
                        .get("offset")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            [
                                arr.first().and_then(|v| v.as_f64()).unwrap_or(1.0),
                                arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
                                arr.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0),
                            ]
                        })
                        .unwrap_or([1.0, 0.0, 0.0]);
                    Some(serde_json::json!({
                        "type": "array",
                        "count": count,
                        "offset": offset
                    }))
                }
                "edge_split" => {
                    let angle = obj.get("angle").and_then(|v| v.as_f64()).unwrap_or(30.0);
                    Some(serde_json::json!({
                        "type": "edge_split",
                        "angle": angle
                    }))
                }
                _ => {
                    warnings.push(format!("Unknown modifier type '{}'. Skipping.", mod_type));
                    None
                }
            }
        })
        .collect()
}

/// Convert legacy UV settings
fn convert_uv_settings(uv: Option<&Value>, _warnings: &mut Vec<String>) -> Option<Value> {
    let obj = uv?.as_object()?;
    let mode = obj.get("mode").and_then(|v| v.as_str())?;

    let method = match mode {
        "smart_project" | "smart" => "smart",
        "cube_project" | "box" => "box",
        "cylinder_project" | "cylinder" => "cylinder",
        "sphere_project" | "sphere" => "sphere",
        "lightmap" | "lightmap_pack" => "lightmap",
        _ => return None,
    };

    let angle_limit = obj.get("angle_limit").and_then(|v| v.as_f64());
    let cube_size = obj.get("cube_size").and_then(|v| v.as_f64());

    let mut result = serde_json::json!({ "method": method });
    if let Some(al) = angle_limit {
        result["angle_limit"] = serde_json::json!(al);
    }
    if let Some(cs) = cube_size {
        result["cube_size"] = serde_json::json!(cs);
    }

    Some(result)
}

/// Convert export settings
fn convert_export_settings(export: Option<&Value>) -> Option<Value> {
    let obj = export?.as_object()?;

    let tangents = obj
        .get("tangents")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let apply_modifiers = obj
        .get("apply_modifiers")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
    let triangulate = obj
        .get("triangulate")
        .and_then(|v| v.as_bool())
        .unwrap_or(true);

    Some(serde_json::json!({
        "tangents": tangents,
        "apply_modifiers": apply_modifiers,
        "triangulate": triangulate,
        "include_normals": true,
        "include_uvs": true,
        "include_vertex_colors": false
    }))
}
