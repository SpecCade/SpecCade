# Phase 2 Test Plan: Starlark Stdlib and Examples

## Overview

This test plan covers unit tests for stdlib functions, integration tests for the compiler, golden tests for IR equality, and error message validation. All tests leverage the existing `speccade-tests` infrastructure.

---

## Test Organization

```
crates/speccade-tests/tests/
  starlark_input.rs          # Phase 1 tests (existing)
  starlark_stdlib.rs         # NEW: Stdlib unit tests
  starlark_golden.rs         # NEW: Golden IR equality tests
  starlark_stdlib_errors.rs  # NEW: Error message tests

crates/speccade-cli/src/compiler/
  stdlib/
    mod.rs                   # Contains inline unit tests
    audio.rs                 # Contains inline unit tests
    texture.rs               # Contains inline unit tests
    mesh.rs                  # Contains inline unit tests
    core.rs                  # Contains inline unit tests
```

---

## Unit Tests for Stdlib Functions

### Test File: `crates/speccade-cli/src/compiler/stdlib/audio.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // envelope() tests
    // ========================================================================

    #[test]
    fn test_envelope_defaults() {
        let result = call_envelope(None, None, None, None);
        assert_eq!(result["attack"], 0.01);
        assert_eq!(result["decay"], 0.1);
        assert_eq!(result["sustain"], 0.5);
        assert_eq!(result["release"], 0.2);
    }

    #[test]
    fn test_envelope_custom_values() {
        let result = call_envelope(Some(0.05), Some(0.2), Some(0.7), Some(0.3));
        assert_eq!(result["attack"], 0.05);
        assert_eq!(result["decay"], 0.2);
        assert_eq!(result["sustain"], 0.7);
        assert_eq!(result["release"], 0.3);
    }

    #[test]
    fn test_envelope_partial_overrides() {
        let result = call_envelope(Some(0.1), None, None, Some(0.5));
        assert_eq!(result["attack"], 0.1);
        assert_eq!(result["decay"], 0.1);  // default
        assert_eq!(result["sustain"], 0.5); // default
        assert_eq!(result["release"], 0.5);
    }

    // ========================================================================
    // oscillator() tests
    // ========================================================================

    #[test]
    fn test_oscillator_minimal() {
        let result = call_oscillator(440.0, None, None, None);
        assert_eq!(result["type"], "oscillator");
        assert_eq!(result["frequency"], 440.0);
        assert_eq!(result["waveform"], "sine");
        assert!(result.get("freq_sweep").is_none());
    }

    #[test]
    fn test_oscillator_with_waveform() {
        let result = call_oscillator(880.0, Some("sawtooth"), None, None);
        assert_eq!(result["waveform"], "sawtooth");
    }

    #[test]
    fn test_oscillator_with_sweep() {
        let result = call_oscillator(440.0, None, Some(220.0), Some("exponential"));
        let sweep = &result["freq_sweep"];
        assert_eq!(sweep["end_freq"], 220.0);
        assert_eq!(sweep["curve"], "exponential");
    }

    #[test]
    fn test_oscillator_negative_frequency_error() {
        let result = call_oscillator(-440.0, None, None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("S103"));
        assert!(err.to_string().contains("must be positive"));
    }

    #[test]
    fn test_oscillator_invalid_waveform_error() {
        let result = call_oscillator(440.0, Some("sinwave"), None, None);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("S104"));
        assert!(err.to_string().contains("waveform"));
    }

    // ========================================================================
    // fm_synth() tests
    // ========================================================================

    #[test]
    fn test_fm_synth_basic() {
        let result = call_fm_synth(440.0, 880.0, 5.0, None);
        assert_eq!(result["type"], "fm_synth");
        assert_eq!(result["carrier_freq"], 440.0);
        assert_eq!(result["modulator_freq"], 880.0);
        assert_eq!(result["modulation_index"], 5.0);
    }

    #[test]
    fn test_fm_synth_with_sweep() {
        let result = call_fm_synth(440.0, 880.0, 5.0, Some(220.0));
        assert!(result["freq_sweep"].is_object());
    }

    // ========================================================================
    // lowpass() / highpass() tests
    // ========================================================================

    #[test]
    fn test_lowpass_defaults() {
        let result = call_lowpass(2000.0, None, None);
        assert_eq!(result["type"], "lowpass");
        assert_eq!(result["cutoff"], 2000.0);
        assert_eq!(result["resonance"], 0.707);
        assert!(result.get("cutoff_end").is_none());
    }

    #[test]
    fn test_lowpass_with_sweep() {
        let result = call_lowpass(5000.0, Some(1.5), Some(500.0));
        assert_eq!(result["cutoff"], 5000.0);
        assert_eq!(result["resonance"], 1.5);
        assert_eq!(result["cutoff_end"], 500.0);
    }

    #[test]
    fn test_highpass_basic() {
        let result = call_highpass(100.0, None, None);
        assert_eq!(result["type"], "highpass");
        assert_eq!(result["cutoff"], 100.0);
    }

    // ========================================================================
    // audio_layer() tests
    // ========================================================================

    #[test]
    fn test_audio_layer_minimal() {
        let synth = call_oscillator(440.0, None, None, None);
        let result = call_audio_layer(synth, None, None, None, None, None, None);

        assert_eq!(result["volume"], 0.8);
        assert_eq!(result["pan"], 0.0);
        assert!(result["envelope"].is_object());
        assert!(result["synthesis"].is_object());
    }

    #[test]
    fn test_audio_layer_with_all_options() {
        let synth = call_oscillator(440.0, None, None, None);
        let env = call_envelope(Some(0.1), Some(0.2), Some(0.6), Some(0.3));
        let filt = call_lowpass(2000.0, None, None);

        let result = call_audio_layer(
            synth,
            Some(env),
            Some(0.5),
            Some(-0.5),
            Some(filt),
            None,
            Some(0.1)
        );

        assert_eq!(result["volume"], 0.5);
        assert_eq!(result["pan"], -0.5);
        assert_eq!(result["delay"], 0.1);
        assert!(result["filter"].is_object());
    }

    #[test]
    fn test_audio_layer_volume_out_of_range() {
        let synth = call_oscillator(440.0, None, None, None);
        let result = call_audio_layer(synth, None, Some(1.5), None, None, None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("S103"));
    }

    #[test]
    fn test_audio_layer_pan_out_of_range() {
        let synth = call_oscillator(440.0, None, None, None);
        let result = call_audio_layer(synth, None, None, Some(2.0), None, None, None);
        assert!(result.is_err());
    }
}
```

### Test File: `crates/speccade-cli/src/compiler/stdlib/texture.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // noise_node() tests
    // ========================================================================

    #[test]
    fn test_noise_node_defaults() {
        let result = call_noise_node("n1", None, None, None, None, None);
        assert_eq!(result["id"], "n1");
        assert_eq!(result["type"], "noise");
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "perlin");
        assert_eq!(noise["scale"], 0.1);
        assert_eq!(noise["octaves"], 4);
    }

    #[test]
    fn test_noise_node_custom() {
        let result = call_noise_node("height", Some("simplex"), Some(0.05), Some(6), None, None);
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "simplex");
        assert_eq!(noise["scale"], 0.05);
        assert_eq!(noise["octaves"], 6);
    }

    #[test]
    fn test_noise_node_invalid_algorithm() {
        let result = call_noise_node("n", Some("invalid"), None, None, None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("S104"));
    }

    // ========================================================================
    // gradient_node() tests
    // ========================================================================

    #[test]
    fn test_gradient_node_defaults() {
        let result = call_gradient_node("g1", None, None, None);
        assert_eq!(result["id"], "g1");
        assert_eq!(result["type"], "gradient");
        assert_eq!(result["direction"], "horizontal");
        assert_eq!(result["start"], 0.0);
        assert_eq!(result["end"], 1.0);
    }

    #[test]
    fn test_gradient_node_vertical() {
        let result = call_gradient_node("g2", Some("vertical"), Some(0.2), Some(0.8));
        assert_eq!(result["direction"], "vertical");
        assert_eq!(result["start"], 0.2);
        assert_eq!(result["end"], 0.8);
    }

    // ========================================================================
    // threshold_node() tests
    // ========================================================================

    #[test]
    fn test_threshold_node_default() {
        let result = call_threshold_node("mask", "noise", None);
        assert_eq!(result["id"], "mask");
        assert_eq!(result["type"], "threshold");
        assert_eq!(result["input"], "noise");
        assert_eq!(result["threshold"], 0.5);
    }

    #[test]
    fn test_threshold_node_custom() {
        let result = call_threshold_node("binary", "height", Some(0.7));
        assert_eq!(result["threshold"], 0.7);
    }

    // ========================================================================
    // texture_graph() tests
    // ========================================================================

    #[test]
    fn test_texture_graph_basic() {
        let nodes = vec![call_noise_node("n", None, None, None, None, None)];
        let result = call_texture_graph(vec![64, 64], nodes, None);

        assert_eq!(result["resolution"], vec![64, 64]);
        assert_eq!(result["tileable"], true);
        assert_eq!(result["nodes"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_texture_graph_not_tileable() {
        let nodes = vec![];
        let result = call_texture_graph(vec![128, 128], nodes, Some(false));
        assert_eq!(result["tileable"], false);
    }
}
```

### Test File: `crates/speccade-cli/src/compiler/stdlib/mesh.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_primitive_cube() {
        let result = call_mesh_primitive("cube", vec![1.0, 1.0, 1.0]);
        assert_eq!(result["base_primitive"], "cube");
        assert_eq!(result["dimensions"], vec![1.0, 1.0, 1.0]);
    }

    #[test]
    fn test_mesh_primitive_sphere() {
        let result = call_mesh_primitive("sphere", vec![2.0, 2.0, 2.0]);
        assert_eq!(result["base_primitive"], "sphere");
    }

    #[test]
    fn test_bevel_modifier_defaults() {
        let result = call_bevel_modifier(None, None);
        assert_eq!(result["type"], "bevel");
        assert_eq!(result["width"], 0.02);
        assert_eq!(result["segments"], 2);
    }

    #[test]
    fn test_subdivision_modifier() {
        let result = call_subdivision_modifier(Some(3), None);
        assert_eq!(result["type"], "subdivision");
        assert_eq!(result["levels"], 3);
        assert_eq!(result["render_levels"], 3);
    }

    #[test]
    fn test_mesh_recipe_with_modifiers() {
        let modifiers = vec![
            call_bevel_modifier(Some(0.05), Some(3)),
            call_subdivision_modifier(Some(2), None),
        ];
        let result = call_mesh_recipe("cube", vec![1.0, 1.0, 1.0], Some(modifiers));

        assert_eq!(result["base_primitive"], "cube");
        assert_eq!(result["modifiers"].as_array().unwrap().len(), 2);
    }
}
```

### Test File: `crates/speccade-cli/src/compiler/stdlib/core.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_output_minimal() {
        let result = call_output("sounds/test.wav", "wav", None);
        assert_eq!(result["path"], "sounds/test.wav");
        assert_eq!(result["format"], "wav");
        assert_eq!(result["kind"], "primary");
    }

    #[test]
    fn test_output_secondary() {
        let result = call_output("textures/preview.png", "png", Some("secondary"));
        assert_eq!(result["kind"], "secondary");
    }

    #[test]
    fn test_spec_minimal() {
        let outputs = vec![call_output("test.wav", "wav", None)];
        let result = call_spec(
            "test-asset-01",
            "audio",
            42,
            outputs,
            None, None, None, None
        );

        assert_eq!(result["spec_version"], 1);
        assert_eq!(result["asset_id"], "test-asset-01");
        assert_eq!(result["asset_type"], "audio");
        assert_eq!(result["seed"], 42);
        assert_eq!(result["license"], "CC0-1.0");
    }

    #[test]
    fn test_spec_with_all_options() {
        let outputs = vec![call_output("test.wav", "wav", None)];
        let recipe = serde_json::json!({"kind": "audio_v1", "params": {}});
        let tags = vec!["retro", "sfx"];

        let result = call_spec(
            "full-asset-01",
            "audio",
            12345,
            outputs,
            Some(recipe),
            Some("Test description"),
            Some(tags),
            Some("MIT")
        );

        assert_eq!(result["description"], "Test description");
        assert_eq!(result["license"], "MIT");
        assert!(result["style_tags"].is_array());
        assert!(result["recipe"].is_object());
    }
}
```

---

## Integration Tests

### Test File: `crates/speccade-tests/tests/starlark_stdlib.rs`

```rust
//! Integration tests for Starlark stdlib.

use speccade_cli::compiler::{compile, CompilerConfig};

fn compile_starlark(source: &str) -> serde_json::Value {
    let config = CompilerConfig::default();
    let result = compile("test.star", source, &config).unwrap();
    serde_json::to_value(&result.spec).unwrap()
}

fn compile_starlark_error(source: &str) -> String {
    let config = CompilerConfig::default();
    compile("test.star", source, &config)
        .unwrap_err()
        .to_string()
}

// ========================================================================
// Audio stdlib integration tests
// ========================================================================

#[test]
fn test_audio_layer_integration() {
    let source = r#"
layer = audio_layer(
    synthesis = oscillator(440, "sine"),
    envelope = envelope(0.01, 0.1, 0.5, 0.2),
    volume = 0.8,
    pan = 0.0
)

{
    "spec_version": 1,
    "asset_id": "stdlib-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [{"kind": "primary", "format": "wav", "path": "test.wav"}],
    "recipe": {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "layers": [layer]
        }
    }
}
"#;

    let spec = compile_starlark(source);
    let layers = &spec["recipe"]["params"]["layers"];
    assert_eq!(layers.as_array().unwrap().len(), 1);

    let layer = &layers[0];
    assert_eq!(layer["synthesis"]["type"], "oscillator");
    assert_eq!(layer["synthesis"]["frequency"], 440.0);
    assert_eq!(layer["volume"], 0.8);
}

#[test]
fn test_fm_synth_integration() {
    let source = r#"
layer = audio_layer(
    synthesis = fm_synth(440, 880, 5.0),
    volume = 0.7
)

{
    "spec_version": 1,
    "asset_id": "fm-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [{"kind": "primary", "format": "wav", "path": "test.wav"}],
    "recipe": {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 0.5,
            "layers": [layer]
        }
    }
}
"#;

    let spec = compile_starlark(source);
    let synth = &spec["recipe"]["params"]["layers"][0]["synthesis"];
    assert_eq!(synth["type"], "fm_synth");
    assert_eq!(synth["carrier_freq"], 440.0);
    assert_eq!(synth["modulator_freq"], 880.0);
}

// ========================================================================
// Texture stdlib integration tests
// ========================================================================

#[test]
fn test_texture_graph_integration() {
    let source = r#"
nodes = [
    noise_node("height", "perlin", 0.1, 4),
    threshold_node("mask", "height", 0.5)
]

{
    "spec_version": 1,
    "asset_id": "texture-test-01",
    "asset_type": "texture",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [{"kind": "primary", "format": "png", "path": "test.png"}],
    "recipe": {
        "kind": "texture.procedural_v1",
        "params": texture_graph([64, 64], nodes, True)
    }
}
"#;

    let spec = compile_starlark(source);
    let params = &spec["recipe"]["params"];
    assert_eq!(params["resolution"], vec![64, 64]);
    assert_eq!(params["nodes"].as_array().unwrap().len(), 2);
}

// ========================================================================
// Core stdlib integration tests
// ========================================================================

#[test]
fn test_spec_helper_integration() {
    let source = r#"
spec(
    asset_id = "helper-test-01",
    asset_type = "audio",
    seed = 12345,
    outputs = [output("sounds/test.wav", "wav")],
    description = "Test asset using spec helper"
)
"#;

    let spec = compile_starlark(source);
    assert_eq!(spec["asset_id"], "helper-test-01");
    assert_eq!(spec["seed"], 12345);
    assert_eq!(spec["description"], "Test asset using spec helper");
}

// ========================================================================
// Error handling integration tests
// ========================================================================

#[test]
fn test_oscillator_negative_frequency_error() {
    let source = r#"
oscillator(-440)
"#;
    let error = compile_starlark_error(source);
    assert!(error.contains("S103"));
    assert!(error.contains("frequency"));
    assert!(error.contains("positive"));
}

#[test]
fn test_invalid_waveform_error() {
    let source = r#"
oscillator(440, "sinwave")
"#;
    let error = compile_starlark_error(source);
    assert!(error.contains("S104"));
    assert!(error.contains("waveform"));
}

#[test]
fn test_volume_out_of_range_error() {
    let source = r#"
audio_layer(oscillator(440), volume = 1.5)
"#;
    let error = compile_starlark_error(source);
    assert!(error.contains("S103"));
    assert!(error.contains("volume"));
}
```

---

## Golden Tests

### Test File: `crates/speccade-tests/tests/starlark_golden.rs`

```rust
//! Golden tests for Starlark stdlib examples.
//!
//! Each test compiles a .star file and compares the result to an expected .json file
//! using JCS-canonical JSON comparison.

use speccade_cli::compiler::{compile, CompilerConfig};
use speccade_spec::hash::canonical_json;
use std::path::Path;

fn load_expected(path: &str) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

fn compile_star_file(path: &str) -> serde_json::Value {
    let content = std::fs::read_to_string(path).unwrap();
    let config = CompilerConfig::default();
    let result = compile(path, &content, &config).unwrap();
    serde_json::to_value(&result.spec).unwrap()
}

fn assert_canonical_eq(actual: &serde_json::Value, expected: &serde_json::Value) {
    let actual_canonical = canonical_json(actual).unwrap();
    let expected_canonical = canonical_json(expected).unwrap();
    assert_eq!(
        actual_canonical, expected_canonical,
        "Canonical JSON mismatch.\nActual:\n{}\nExpected:\n{}",
        serde_json::to_string_pretty(actual).unwrap(),
        serde_json::to_string_pretty(expected).unwrap()
    );
}

// ========================================================================
// Audio golden tests
// ========================================================================

#[test]
fn golden_audio_synth_oscillator() {
    let actual = compile_star_file("golden/starlark/audio_synth_oscillator.star");
    let expected = load_expected("golden/starlark/audio_synth_oscillator.expected.json");
    assert_canonical_eq(&actual, &expected);
}

#[test]
fn golden_audio_synth_fm() {
    let actual = compile_star_file("golden/starlark/audio_synth_fm.star");
    let expected = load_expected("golden/starlark/audio_synth_fm.expected.json");
    assert_canonical_eq(&actual, &expected);
}

#[test]
fn golden_audio_synth_layered() {
    let actual = compile_star_file("golden/starlark/audio_synth_layered.star");
    let expected = load_expected("golden/starlark/audio_synth_layered.expected.json");
    assert_canonical_eq(&actual, &expected);
}

// ========================================================================
// Texture golden tests
// ========================================================================

#[test]
fn golden_texture_noise_perlin() {
    let actual = compile_star_file("golden/starlark/texture_noise_perlin.star");
    let expected = load_expected("golden/starlark/texture_noise_perlin.expected.json");
    assert_canonical_eq(&actual, &expected);
}

#[test]
fn golden_texture_graph_threshold() {
    let actual = compile_star_file("golden/starlark/texture_graph_threshold.star");
    let expected = load_expected("golden/starlark/texture_graph_threshold.expected.json");
    assert_canonical_eq(&actual, &expected);
}

#[test]
fn golden_texture_color_ramp() {
    let actual = compile_star_file("golden/starlark/texture_color_ramp.star");
    let expected = load_expected("golden/starlark/texture_color_ramp.expected.json");
    assert_canonical_eq(&actual, &expected);
}

// ========================================================================
// Mesh golden tests
// ========================================================================

#[test]
fn golden_mesh_primitive_cube() {
    let actual = compile_star_file("golden/starlark/mesh_primitive_cube.star");
    let expected = load_expected("golden/starlark/mesh_primitive_cube.expected.json");
    assert_canonical_eq(&actual, &expected);
}

#[test]
fn golden_mesh_primitive_sphere() {
    let actual = compile_star_file("golden/starlark/mesh_primitive_sphere.star");
    let expected = load_expected("golden/starlark/mesh_primitive_sphere.expected.json");
    assert_canonical_eq(&actual, &expected);
}

// ========================================================================
// Existing examples should still work
// ========================================================================

#[test]
fn golden_minimal_unchanged() {
    let actual = compile_star_file("golden/starlark/minimal.star");
    // minimal.star doesn't use stdlib, should compile unchanged
    assert_eq!(actual["asset_id"], "starlark-minimal-01");
}

#[test]
fn golden_with_functions_unchanged() {
    let actual = compile_star_file("golden/starlark/with_functions.star");
    assert_eq!(actual["asset_id"], "starlark-functions-01");
}
```

---

## Golden Test Files

### Audio Examples

**`golden/starlark/audio_synth_oscillator.star`**:
```python
# Simple oscillator with envelope - demonstrates audio stdlib

spec(
    asset_id = "stdlib-audio-osc-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/oscillator.wav", "wav")],
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
```

**`golden/starlark/audio_synth_fm.star`**:
```python
# FM synthesis example

spec(
    asset_id = "stdlib-audio-fm-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/fm.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = fm_synth(440, 880, 5.0),
                    envelope = envelope(0.05, 0.2, 0.6, 0.3),
                    volume = 0.7
                )
            ]
        }
    }
)
```

**`golden/starlark/audio_synth_layered.star`**:
```python
# Multi-layer sound with filter sweep

spec(
    asset_id = "stdlib-audio-layered-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/layered.wav", "wav")],
    recipe = {
        "kind": "audio_v1",
        "params": {
            "duration_seconds": 1.0,
            "sample_rate": 44100,
            "layers": [
                audio_layer(
                    synthesis = oscillator(440, "sawtooth"),
                    envelope = envelope(0.01, 0.2, 0.6, 0.3),
                    volume = 0.6,
                    filter = lowpass(2000, 0.707, 500)
                ),
                audio_layer(
                    synthesis = noise_burst("white", lowpass(5000)),
                    envelope = envelope(0.001, 0.05, 0.0, 0.1),
                    volume = 0.3,
                    delay = 0.0
                )
            ],
            "effects": [reverb(0.4, 0.2)]
        }
    }
)
```

### Texture Examples

**`golden/starlark/texture_noise_perlin.star`**:
```python
# Basic Perlin noise texture

spec(
    asset_id = "stdlib-texture-perlin-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/perlin.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            resolution = [64, 64],
            nodes = [
                noise_node("height", "perlin", 0.1, 4)
            ]
        )
    }
)
```

**`golden/starlark/texture_graph_threshold.star`**:
```python
# Noise with threshold mask

spec(
    asset_id = "stdlib-texture-threshold-01",
    asset_type = "texture",
    seed = 42,
    outputs = [output("textures/threshold.png", "png")],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            resolution = [64, 64],
            nodes = [
                noise_node("noise", "simplex", 0.05, 6),
                threshold_node("mask", "noise", 0.5)
            ]
        )
    }
)
```

### Mesh Examples

**`golden/starlark/mesh_primitive_cube.star`**:
```python
# Simple cube with bevel

spec(
    asset_id = "stdlib-mesh-cube-01",
    asset_type = "static_mesh",
    seed = 42,
    outputs = [output("meshes/cube.glb", "glb")],
    recipe = {
        "kind": "static_mesh.blender_primitives_v1",
        "params": mesh_recipe(
            primitive = "cube",
            dimensions = [1.0, 1.0, 1.0],
            modifiers = [bevel_modifier(0.02, 2)]
        )
    }
)
```

---

## Error Message Tests

### Test File: `crates/speccade-tests/tests/starlark_stdlib_errors.rs`

```rust
//! Tests for stdlib error messages.

use speccade_cli::compiler::{compile, CompilerConfig, CompileError};

fn get_error(source: &str) -> CompileError {
    let config = CompilerConfig::default();
    compile("test.star", source, &config).unwrap_err()
}

// ========================================================================
// S101: Missing argument tests
// ========================================================================

#[test]
fn test_s101_oscillator_missing_frequency() {
    let err = get_error("oscillator()");
    let msg = err.to_string();
    assert!(msg.contains("S101"));
    assert!(msg.contains("oscillator"));
    assert!(msg.contains("frequency"));
    assert!(msg.contains("required"));
}

#[test]
fn test_s101_fm_synth_missing_modulator() {
    let err = get_error("fm_synth(440)");
    let msg = err.to_string();
    assert!(msg.contains("S101"));
    assert!(msg.contains("modulator"));
}

// ========================================================================
// S102: Type mismatch tests
// ========================================================================

#[test]
fn test_s102_oscillator_frequency_string() {
    let err = get_error(r#"oscillator("440")"#);
    let msg = err.to_string();
    assert!(msg.contains("S102"));
    assert!(msg.contains("frequency"));
    assert!(msg.contains("float"));
    assert!(msg.contains("string"));
}

#[test]
fn test_s102_envelope_attack_string() {
    let err = get_error(r#"envelope("fast")"#);
    let msg = err.to_string();
    assert!(msg.contains("S102"));
    assert!(msg.contains("attack"));
}

// ========================================================================
// S103: Range validation tests
// ========================================================================

#[test]
fn test_s103_oscillator_negative_frequency() {
    let err = get_error("oscillator(-100)");
    let msg = err.to_string();
    assert!(msg.contains("S103"));
    assert!(msg.contains("frequency"));
    assert!(msg.contains("positive"));
    assert!(msg.contains("-100"));
}

#[test]
fn test_s103_volume_above_one() {
    let err = get_error("audio_layer(oscillator(440), volume = 1.5)");
    let msg = err.to_string();
    assert!(msg.contains("S103"));
    assert!(msg.contains("volume"));
    assert!(msg.contains("0.0"));
    assert!(msg.contains("1.0"));
}

#[test]
fn test_s103_pan_out_of_range() {
    let err = get_error("audio_layer(oscillator(440), pan = 2.0)");
    let msg = err.to_string();
    assert!(msg.contains("S103"));
    assert!(msg.contains("pan"));
    assert!(msg.contains("-1.0"));
    assert!(msg.contains("1.0"));
}

// ========================================================================
// S104: Invalid enum tests
// ========================================================================

#[test]
fn test_s104_invalid_waveform() {
    let err = get_error(r#"oscillator(440, "sinwave")"#);
    let msg = err.to_string();
    assert!(msg.contains("S104"));
    assert!(msg.contains("waveform"));
    assert!(msg.contains("sine"));
    assert!(msg.contains("sawtooth"));
}

#[test]
fn test_s104_invalid_noise_type() {
    let err = get_error(r#"noise_burst("purple")"#);
    let msg = err.to_string();
    assert!(msg.contains("S104"));
    assert!(msg.contains("noise_type"));
    assert!(msg.contains("white"));
}

#[test]
fn test_s104_invalid_noise_algorithm() {
    let err = get_error(r#"noise_node("n", "fractal")"#);
    let msg = err.to_string();
    assert!(msg.contains("S104"));
    assert!(msg.contains("algorithm"));
    assert!(msg.contains("perlin"));
}

// ========================================================================
// Suggestion tests
// ========================================================================

#[test]
fn test_suggestion_for_typo() {
    let err = get_error(r#"oscillator(440, "sinwave")"#);
    let msg = err.to_string();
    // Should suggest "sine" for "sinwave"
    assert!(msg.contains("Did you mean") || msg.contains("sine"));
}
```

---

## Determinism Tests

### Extend existing `e2e_determinism.rs`

```rust
#[test]
fn starlark_stdlib_specs_are_deterministic() {
    let fixture = DeterminismFixture::new()
        .add_spec("golden/starlark/audio_synth_oscillator.star")
        .add_spec("golden/starlark/audio_synth_fm.star")
        .add_spec("golden/starlark/audio_synth_layered.star")
        .add_spec("golden/starlark/texture_noise_perlin.star")
        .add_spec("golden/starlark/texture_graph_threshold.star")
        .runs(3);

    let report = fixture.run();
    assert!(
        report.all_deterministic(),
        "Determinism failures: {:?}",
        report.failures()
    );
}
```

---

## Test Coverage Summary

| Category | Test Count | Coverage |
|----------|------------|----------|
| Stdlib unit tests | ~50 | All functions, defaults, edge cases |
| Integration tests | ~15 | End-to-end compilation |
| Golden tests | 8+ | IR equality for examples |
| Error tests | ~15 | All S-series error codes |
| Determinism tests | 1 (multi-file) | Hash stability |

---

## Golden File Update Script

Add to `crates/speccade-cli/src/bin/speccade.rs` or as separate script:

```bash
# Update all golden files
for f in golden/starlark/*.star; do
    speccade eval "$f" > "${f%.star}.expected.json"
done
```

Or via Cargo test flag:

```rust
// In test harness
if std::env::var("UPDATE_GOLDEN").is_ok() {
    let content = serde_json::to_string_pretty(&actual).unwrap();
    std::fs::write(expected_path, content).unwrap();
    return;
}
```

---

## Test Execution

```bash
# Run all stdlib tests
cargo test -p speccade-cli stdlib

# Run golden tests
cargo test -p speccade-tests starlark_golden

# Run error tests
cargo test -p speccade-tests starlark_stdlib_errors

# Run with update mode
UPDATE_GOLDEN=1 cargo test -p speccade-tests starlark_golden

# Run determinism tests (slower)
cargo test -p speccade-tests starlark_stdlib_specs_are_deterministic
```
