//! Music parameter conversion

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use super::audio::convert_synthesis_type;
use super::helpers::default_envelope_json;

/// Map legacy SONG params to canonical music.tracker_song_v1
pub fn map_music_params(data: &HashMap<String, Value>) -> Result<(Value, Vec<String>)> {
    let mut warnings = Vec::new();

    // Required fields
    let format = data
        .get("format")
        .and_then(|v| v.as_str())
        .unwrap_or("xm")
        .to_lowercase();
    let bpm = data.get("bpm").and_then(|v| v.as_u64()).unwrap_or(120) as u16;
    let speed = data.get("speed").and_then(|v| v.as_u64()).unwrap_or(6) as u8;
    let channels = data.get("channels").and_then(|v| v.as_u64()).unwrap_or(4) as u8;

    // Optional fields
    let name = data.get("name").and_then(|v| v.as_str()).map(String::from);
    let title = data.get("title").and_then(|v| v.as_str()).map(String::from);
    let restart_position = data
        .get("restart_position")
        .and_then(|v| v.as_u64())
        .map(|v| v as u16);
    let looping = data.get("loop").and_then(|v| v.as_bool()).unwrap_or(false);

    // Convert instruments
    let instruments = convert_song_instruments(data.get("instruments"), &mut warnings);

    // Convert patterns
    let patterns = convert_song_patterns(data.get("patterns"), &mut warnings);

    // Convert arrangement
    let arrangement = convert_arrangement(data.get("arrangement"), &mut warnings);

    // Convert automation
    let automation = convert_automation(data.get("automation"), &mut warnings);

    // Convert IT options
    let it_options = convert_it_options(data.get("it_options"));

    let mut params = serde_json::json!({
        "format": format,
        "bpm": bpm,
        "speed": speed,
        "channels": channels,
        "loop": looping,
        "instruments": instruments,
        "patterns": patterns,
        "arrangement": arrangement
    });

    if let Some(n) = name {
        params["name"] = serde_json::json!(n);
    }
    if let Some(t) = title {
        params["title"] = serde_json::json!(t);
    }
    if let Some(rp) = restart_position {
        params["restart_position"] = serde_json::json!(rp);
    }
    if !automation.is_empty() {
        params["automation"] = serde_json::json!(automation);
    }
    if let Some(it_opts) = it_options {
        params["it_options"] = it_opts;
    }

    Ok((params, warnings))
}

/// Convert song instruments
fn convert_song_instruments(instruments: Option<&Value>, warnings: &mut Vec<String>) -> Vec<Value> {
    let Some(Value::Array(insts)) = instruments else {
        return vec![];
    };

    insts
        .iter()
        .map(|inst| {
            let obj = inst.as_object();
            let name = obj
                .and_then(|o| o.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or("instrument")
                .to_string();

            let envelope = obj
                .and_then(|o| o.get("envelope"))
                .cloned()
                .unwrap_or_else(default_envelope_json);

            let base_note = obj
                .and_then(|o| o.get("base_note"))
                .and_then(|v| v.as_str())
                .map(String::from);

            let sample_rate = obj
                .and_then(|o| o.get("sample_rate"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            // Convert synthesis if present
            let synthesis = obj.and_then(|o| o.get("synthesis")).and_then(|s| {
                let synth_obj = s.as_object();
                convert_synthesis_type(synth_obj, warnings).ok()
            });

            let mut result = serde_json::json!({
                "name": name,
                "envelope": envelope
            });

            if let Some(note) = base_note {
                result["base_note"] = serde_json::json!(note);
            }
            if let Some(sr) = sample_rate {
                result["sample_rate"] = serde_json::json!(sr);
            }
            if let Some(synth) = synthesis {
                result["synthesis"] = synth;
            }

            result
        })
        .collect()
}

/// Convert song patterns
fn convert_song_patterns(
    patterns: Option<&Value>,
    _warnings: &mut Vec<String>,
) -> HashMap<String, Value> {
    let Some(Value::Object(pats)) = patterns else {
        return HashMap::new();
    };

    pats.iter()
        .map(|(name, pattern)| {
            let obj = pattern.as_object();
            let rows = obj
                .and_then(|o| o.get("rows"))
                .and_then(|v| v.as_u64())
                .unwrap_or(64) as u16;

            let notes = obj.and_then(|o| o.get("notes")).cloned();
            let data = obj.and_then(|o| o.get("data")).cloned();

            let mut result = serde_json::json!({ "rows": rows });
            if let Some(n) = notes {
                result["notes"] = n;
            }
            if let Some(d) = data {
                result["data"] = d;
            }

            (name.clone(), result)
        })
        .collect()
}

/// Convert arrangement entries
fn convert_arrangement(arrangement: Option<&Value>, _warnings: &mut Vec<String>) -> Vec<Value> {
    let Some(Value::Array(arr)) = arrangement else {
        return vec![];
    };

    arr.iter()
        .filter_map(|entry| {
            let obj = entry.as_object()?;
            let pattern = obj.get("pattern").and_then(|v| v.as_str())?.to_string();
            let repeat = obj.get("repeat").and_then(|v| v.as_u64()).unwrap_or(1) as u16;
            Some(serde_json::json!({
                "pattern": pattern,
                "repeat": repeat
            }))
        })
        .collect()
}

/// Convert automation entries
fn convert_automation(automation: Option<&Value>, _warnings: &mut Vec<String>) -> Vec<Value> {
    let Some(Value::Array(arr)) = automation else {
        return vec![];
    };

    // Pass through automation as-is for now
    arr.clone()
}

/// Convert IT-specific options
fn convert_it_options(it_options: Option<&Value>) -> Option<Value> {
    let obj = it_options?.as_object()?;

    let stereo = obj.get("stereo").and_then(|v| v.as_bool()).unwrap_or(true);
    let global_volume = obj
        .get("global_volume")
        .and_then(|v| v.as_u64())
        .unwrap_or(128) as u8;
    let mix_volume = obj
        .get("mix_volume")
        .and_then(|v| v.as_u64())
        .unwrap_or(128) as u8;

    Some(serde_json::json!({
        "stereo": stereo,
        "global_volume": global_volume,
        "mix_volume": mix_volume
    }))
}
