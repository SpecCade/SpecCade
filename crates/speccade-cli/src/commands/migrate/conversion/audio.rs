//! Audio parameter conversion

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

use super::helpers::{create_default_layer, default_envelope_json};

/// Map legacy SOUND/INSTRUMENT params to canonical audio_v1
pub fn map_audio_params(
    data: &HashMap<String, Value>,
    dict_name: &str,
) -> Result<(Value, Vec<String>)> {
    let mut warnings = Vec::new();

    // Extract duration - required for SOUND, derived from output.duration for INSTRUMENT
    let duration = if dict_name == "SOUND" {
        data.get("duration").and_then(|v| v.as_f64()).unwrap_or(1.0)
    } else {
        // INSTRUMENT: check output.duration
        data.get("output")
            .and_then(|o| o.get("duration"))
            .and_then(|d| d.as_f64())
            .unwrap_or(1.0)
    };

    // Extract sample_rate
    let sample_rate = data
        .get("sample_rate")
        .and_then(|v| v.as_u64())
        .unwrap_or(44100) as u32;

    // Extract base_note (for INSTRUMENT)
    let base_note = data
        .get("base_note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Convert layers
    let layers = if dict_name == "SOUND" {
        convert_sound_layers(data.get("layers"), &mut warnings)?
    } else {
        // INSTRUMENT has a single synthesis definition, not layers
        convert_instrument_synthesis(data, &mut warnings)?
    };

    // Build params
    let mut params = serde_json::json!({
        "duration_seconds": duration,
        "sample_rate": sample_rate,
        "layers": layers
    });

    // Add base_note if present
    if let Some(note) = base_note {
        params["base_note"] = serde_json::json!(note);
    }

    // Check for normalize/peak_db - these are post-processing hints we can't fully support
    if data.contains_key("normalize") || data.contains_key("peak_db") {
        warnings.push(
            "Legacy 'normalize'/'peak_db' fields not directly supported. Consider adding audio effects for normalization.".to_string()
        );
    }

    Ok((params, warnings))
}

/// Convert legacy SOUND layers to canonical AudioLayer array
fn convert_sound_layers(
    layers_value: Option<&Value>,
    warnings: &mut Vec<String>,
) -> Result<Vec<Value>> {
    let Some(Value::Array(layers)) = layers_value else {
        return Ok(vec![create_default_layer()]);
    };

    let mut result = Vec::new();
    for layer in layers {
        let Some(obj) = layer.as_object() else {
            warnings.push("Skipping non-object layer".to_string());
            continue;
        };
        result.push(convert_single_layer(obj, warnings)?);
    }

    if result.is_empty() {
        result.push(create_default_layer());
    }

    Ok(result)
}

/// Convert INSTRUMENT synthesis to a single AudioLayer
fn convert_instrument_synthesis(
    data: &HashMap<String, Value>,
    warnings: &mut Vec<String>,
) -> Result<Vec<Value>> {
    // Extract envelope (instruments have a top-level envelope)
    let envelope = data
        .get("envelope")
        .cloned()
        .unwrap_or_else(default_envelope_json);

    // Extract synthesis config
    let synthesis = if let Some(synth) = data.get("synthesis") {
        convert_synthesis_type(synth.as_object(), warnings)?
    } else {
        // Default to simple oscillator
        serde_json::json!({
            "type": "oscillator",
            "waveform": "sine",
            "frequency": 440.0
        })
    };

    Ok(vec![serde_json::json!({
        "synthesis": synthesis,
        "envelope": envelope,
        "volume": 1.0,
        "pan": 0.0
    })])
}

/// Convert a single legacy layer to canonical AudioLayer
fn convert_single_layer(
    layer: &serde_json::Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Result<Value> {
    let layer_type = layer.get("type").and_then(|v| v.as_str()).unwrap_or("sine");

    // Extract envelope
    let envelope = layer
        .get("envelope")
        .cloned()
        .unwrap_or_else(default_envelope_json);

    // Extract volume/amplitude
    let volume = layer
        .get("amplitude")
        .or_else(|| layer.get("volume"))
        .and_then(|v| v.as_f64())
        .unwrap_or(1.0);

    // Extract pan
    let pan = layer.get("pan").and_then(|v| v.as_f64()).unwrap_or(0.0);

    // Extract delay
    let delay = layer.get("delay").and_then(|v| v.as_f64());

    // Convert synthesis type
    let synthesis = convert_layer_synthesis(layer_type, layer, warnings)?;

    // Convert filter if present
    let filter = layer
        .get("filter")
        .and_then(|f| convert_filter(f.as_object(), warnings));

    let mut result = serde_json::json!({
        "synthesis": synthesis,
        "envelope": envelope,
        "volume": volume,
        "pan": pan
    });

    if let Some(d) = delay {
        result["delay"] = serde_json::json!(d);
    }
    if let Some(f) = filter {
        result["filter"] = f;
    }

    Ok(result)
}

/// Convert legacy layer type to canonical Synthesis
fn convert_layer_synthesis(
    layer_type: &str,
    layer: &serde_json::Map<String, Value>,
    warnings: &mut Vec<String>,
) -> Result<Value> {
    match layer_type {
        "sine" | "square" | "triangle" | "sawtooth" | "saw" | "pulse" => {
            let waveform = match layer_type {
                "saw" => "sawtooth",
                other => other,
            };
            let frequency = layer
                .get("freq")
                .or_else(|| layer.get("frequency"))
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);

            let duty = layer.get("duty").and_then(|v| v.as_f64());

            let mut synth = serde_json::json!({
                "type": "oscillator",
                "waveform": waveform,
                "frequency": frequency
            });
            if let Some(d) = duty {
                synth["duty"] = serde_json::json!(d);
            }
            Ok(synth)
        }

        "fm_synth" | "fm" => {
            let carrier_freq = layer
                .get("carrier_freq")
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);
            let mod_ratio = layer.get("mod_ratio").and_then(|v| v.as_f64());
            let mod_index = layer
                .get("mod_index")
                .or_else(|| layer.get("index"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            // Calculate modulator frequency from ratio or use direct value
            let modulator_freq = if let Some(ratio) = mod_ratio {
                carrier_freq * ratio
            } else {
                layer
                    .get("modulator_freq")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(carrier_freq * 2.0)
            };

            // Legacy index_decay is not directly supported
            if layer.contains_key("index_decay") {
                warnings.push(
                    "Legacy 'index_decay' for FM synthesis not directly supported. Consider using pitch envelope or effects.".to_string()
                );
            }

            Ok(serde_json::json!({
                "type": "fm_synth",
                "carrier_freq": carrier_freq,
                "modulator_freq": modulator_freq,
                "modulation_index": mod_index
            }))
        }

        "noise_burst" | "noise" => {
            let noise_color = layer
                .get("color")
                .or_else(|| layer.get("noise_type"))
                .and_then(|v| v.as_str())
                .unwrap_or("white");

            let noise_type = match noise_color {
                "brown" | "red" => "brown",
                "pink" => "pink",
                _ => "white",
            };

            // Filter is handled separately in convert_single_layer
            Ok(serde_json::json!({
                "type": "noise_burst",
                "noise_type": noise_type
            }))
        }

        "pitched_body" => {
            let start_freq = layer
                .get("start_freq")
                .and_then(|v| v.as_f64())
                .unwrap_or(200.0);
            let end_freq = layer
                .get("end_freq")
                .and_then(|v| v.as_f64())
                .unwrap_or(50.0);

            Ok(serde_json::json!({
                "type": "pitched_body",
                "start_freq": start_freq,
                "end_freq": end_freq
            }))
        }

        "karplus_strong" => {
            let frequency = layer
                .get("freq")
                .or_else(|| layer.get("frequency"))
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);
            let decay = layer
                .get("damping")
                .or_else(|| layer.get("decay"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.996);
            let blend = layer
                .get("brightness")
                .or_else(|| layer.get("blend"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5);

            Ok(serde_json::json!({
                "type": "karplus_strong",
                "frequency": frequency,
                "decay": decay,
                "blend": blend
            }))
        }

        "additive" => {
            let base_freq = layer
                .get("base_freq")
                .or_else(|| layer.get("frequency"))
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);
            let harmonics = layer
                .get("harmonics")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_f64()).collect::<Vec<_>>())
                .unwrap_or_else(|| vec![1.0, 0.5, 0.25]);

            Ok(serde_json::json!({
                "type": "additive",
                "base_freq": base_freq,
                "harmonics": harmonics
            }))
        }

        "subtractive" => {
            // Subtractive synthesis maps to multi_oscillator with filter
            let frequency = layer
                .get("frequency")
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);

            let oscillators = if let Some(Value::Array(oscs)) = layer.get("oscillators") {
                oscs.iter()
                    .map(|osc| {
                        let waveform = osc
                            .get("waveform")
                            .and_then(|v| v.as_str())
                            .unwrap_or("sawtooth");
                        let volume = osc.get("volume").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        let detune = osc.get("detune").and_then(|v| v.as_f64());
                        let mut o = serde_json::json!({
                            "waveform": waveform,
                            "volume": volume
                        });
                        if let Some(d) = detune {
                            o["detune"] = serde_json::json!(d);
                        }
                        o
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![serde_json::json!({
                    "waveform": "sawtooth",
                    "volume": 1.0
                })]
            };

            warnings.push(
                "Legacy 'subtractive' synthesis mapped to 'multi_oscillator'. Filter moved to layer level.".to_string()
            );

            Ok(serde_json::json!({
                "type": "multi_oscillator",
                "frequency": frequency,
                "oscillators": oscillators
            }))
        }

        _ => {
            warnings.push(format!(
                "Unknown layer type '{}'. Defaulting to sine oscillator.",
                layer_type
            ));
            let frequency = layer
                .get("freq")
                .or_else(|| layer.get("frequency"))
                .and_then(|v| v.as_f64())
                .unwrap_or(440.0);
            Ok(serde_json::json!({
                "type": "oscillator",
                "waveform": "sine",
                "frequency": frequency
            }))
        }
    }
}

/// Convert instrument-level synthesis definition
pub fn convert_synthesis_type(
    synth: Option<&serde_json::Map<String, Value>>,
    warnings: &mut Vec<String>,
) -> Result<Value> {
    let Some(synth) = synth else {
        return Ok(serde_json::json!({
            "type": "oscillator",
            "waveform": "sine",
            "frequency": 440.0
        }));
    };

    let synth_type = synth.get("type").and_then(|v| v.as_str()).unwrap_or("sine");
    convert_layer_synthesis(synth_type, synth, warnings)
}

/// Convert legacy filter to canonical Filter
fn convert_filter(
    filter: Option<&serde_json::Map<String, Value>>,
    warnings: &mut Vec<String>,
) -> Option<Value> {
    let filter = filter?;
    let filter_type = filter.get("type").and_then(|v| v.as_str())?;

    match filter_type {
        "lowpass" => {
            let cutoff = filter
                .get("cutoff")
                .and_then(|v| v.as_f64())
                .unwrap_or(1000.0);
            let resonance = filter
                .get("q")
                .or_else(|| filter.get("resonance"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.707);
            let cutoff_end = filter.get("cutoff_end").and_then(|v| v.as_f64());

            let mut f = serde_json::json!({
                "type": "lowpass",
                "cutoff": cutoff,
                "resonance": resonance
            });
            if let Some(end) = cutoff_end {
                f["cutoff_end"] = serde_json::json!(end);
            }
            Some(f)
        }
        "highpass" => {
            let cutoff = filter
                .get("cutoff")
                .and_then(|v| v.as_f64())
                .unwrap_or(1000.0);
            let resonance = filter
                .get("q")
                .or_else(|| filter.get("resonance"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.707);
            let cutoff_end = filter.get("cutoff_end").and_then(|v| v.as_f64());

            let mut f = serde_json::json!({
                "type": "highpass",
                "cutoff": cutoff,
                "resonance": resonance
            });
            if let Some(end) = cutoff_end {
                f["cutoff_end"] = serde_json::json!(end);
            }
            Some(f)
        }
        "bandpass" => {
            let center = filter
                .get("center")
                .or_else(|| filter.get("cutoff"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1000.0);
            let resonance = filter
                .get("q")
                .or_else(|| filter.get("resonance"))
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0);

            Some(serde_json::json!({
                "type": "bandpass",
                "center": center,
                "resonance": resonance
            }))
        }
        _ => {
            warnings.push(format!(
                "Unknown filter type '{}'. Filter not migrated.",
                filter_type
            ));
            None
        }
    }
}
