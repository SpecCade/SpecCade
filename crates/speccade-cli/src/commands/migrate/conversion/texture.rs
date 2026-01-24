//! Texture parameter conversion

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Map legacy texture/normal params to canonical texture.procedural_v1
pub fn map_texture_params(
    data: &HashMap<String, Value>,
    category: &str,
) -> Result<(Value, Vec<String>)> {
    let mut warnings = Vec::new();

    let resolution = match data.get("size").or_else(|| data.get("resolution")) {
        Some(Value::Number(n)) => {
            let size = n.as_u64().unwrap_or(256) as u32;
            [size, size]
        }
        Some(Value::Array(values)) if values.len() == 2 => {
            let w = values[0].as_u64().unwrap_or(256) as u32;
            let h = values[1].as_u64().unwrap_or(256) as u32;
            [w, h]
        }
        _ => [256, 256],
    };

    let tileable = data
        .get("tileable")
        .or_else(|| data.get("tile"))
        .or_else(|| data.get("seamless"))
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let nodes = if category == "normals" {
        serde_json::json!([
            { "id": "height", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
            { "id": "normal", "type": "normal_from_height", "input": "height", "strength": 1.0 }
        ])
    } else {
        serde_json::json!([
            { "id": "height", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.08 } },
            { "id": "albedo", "type": "color_ramp", "input": "height", "ramp": ["#2b2b2b", "#cfcfcf"] }
        ])
    };

    warnings.push(
        "Legacy texture specs migrated to placeholder procedural graphs. Manual review recommended."
            .to_string(),
    );

    Ok((
        serde_json::json!({
            "resolution": resolution,
            "tileable": tileable,
            "nodes": nodes
        }),
        warnings,
    ))
}
