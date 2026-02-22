//! Texture validation tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::recipe::Recipe;
use crate::spec::AssetType;
use crate::validation::*;

fn make_valid_texture_procedural_spec() -> crate::spec::Spec {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/mask.png");
    output.source = Some("mask".to_string());

    crate::spec::Spec::builder("procedural-test-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(123)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [16, 16],
                "tileable": true,
                "nodes": [
                    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                    { "id": "mask", "type": "threshold", "input": "n", "threshold": 0.5 }
                ]
            }),
        ))
        .build()
}

#[test]
fn test_texture_procedural_valid_spec() {
    let spec = make_valid_texture_procedural_spec();
    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}

#[test]
fn test_texture_procedural_rejects_missing_output_source() {
    let mut spec = make_valid_texture_procedural_spec();
    spec.outputs[0].source = None;

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("must set 'source'")));
}

#[test]
fn test_texture_procedural_rejects_unknown_output_source() {
    let mut spec = make_valid_texture_procedural_spec();
    spec.outputs[0].source = Some("missing".to_string());

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("does not match any recipe.params.nodes")));
}

#[test]
fn test_texture_procedural_rejects_cycles() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/a.png");
    output.source = Some("a".to_string());

    let spec = crate::spec::Spec::builder("procedural-cycle-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(1)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [8, 8],
                "tileable": true,
                "nodes": [
                    { "id": "a", "type": "invert", "input": "b" },
                    { "id": "b", "type": "invert", "input": "a" }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("cycle detected")));
}

#[test]
fn test_texture_procedural_rejects_obvious_type_mismatch() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/bad.png");
    output.source = Some("bad".to_string());

    let spec = crate::spec::Spec::builder("procedural-types-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(1)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [8, 8],
                "tileable": true,
                "nodes": [
                    { "id": "n", "type": "noise", "noise": { "algorithm": "perlin", "scale": 0.1 } },
                    { "id": "bad", "type": "palette", "input": "n", "palette": ["#000000", "#ffffff"] }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("type mismatch")));
}

#[test]
fn test_texture_procedural_accepts_reaction_diffusion() {
    let mut output = OutputSpec::primary(OutputFormat::Png, "textures/rd.png");
    output.source = Some("rd".to_string());

    let spec = crate::spec::Spec::builder("procedural-rd-01", AssetType::Texture)
        .license("CC0-1.0")
        .seed(9)
        .output(output)
        .recipe(Recipe::new(
            "texture.procedural_v1",
            serde_json::json!({
                "resolution": [32, 32],
                "tileable": true,
                "nodes": [
                    {
                        "id": "rd",
                        "type": "reaction_diffusion",
                        "steps": 64,
                        "feed": 0.055,
                        "kill": 0.062,
                        "diffuse_a": 1.0,
                        "diffuse_b": 0.5,
                        "dt": 1.0,
                        "seed_density": 0.03
                    }
                ]
            }),
        ))
        .build();

    let result = validate_for_generate(&spec);
    assert!(result.is_ok(), "errors: {:?}", result.errors);
}
