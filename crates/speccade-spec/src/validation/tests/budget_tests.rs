//! Budget enforcement tests.

use crate::output::{OutputFormat, OutputSpec};
use crate::recipe::Recipe;
use crate::spec::AssetType;
use crate::validation::*;

#[test]
fn test_budget_audio_duration_default_profile_passes() {
    // 25 seconds is within default budget of 30 seconds
    let spec = crate::spec::Spec::builder("test-budget-01", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 25.0,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate_with_budget(&spec, &BudgetProfile::default());
    assert!(result.is_ok(), "{:?}", result.errors);
}

#[test]
fn test_budget_audio_duration_strict_profile_fails() {
    // 15 seconds exceeds strict budget of 10 seconds
    let spec = crate::spec::Spec::builder("test-budget-02", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 15.0,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate_with_budget(&spec, &BudgetProfile::strict());
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::BudgetExceeded));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("duration_seconds")));
}

#[test]
fn test_budget_audio_duration_zx_8bit_profile_fails() {
    // 10 seconds exceeds zx-8bit budget of 5 seconds
    let spec = crate::spec::Spec::builder("test-budget-03", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 10.0,
                "sample_rate": 22050,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate_with_budget(&spec, &BudgetProfile::zx_8bit());
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::BudgetExceeded));
}

#[test]
fn test_budget_audio_sample_rate_zx_8bit_profile_fails() {
    // 44100 is not in zx-8bit allowed rates [22050]
    let spec = crate::spec::Spec::builder("test-budget-04", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 1.0,
                "sample_rate": 44100,
                "layers": []
            }),
        ))
        .build();

    let result = validate_for_generate_with_budget(&spec, &BudgetProfile::zx_8bit());
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::BudgetExceeded));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("sample_rate")));
}

#[test]
fn test_budget_audio_layers_strict_profile_fails() {
    // 20 layers exceeds strict budget of 16 layers
    let layers: Vec<serde_json::Value> = (0..20)
        .map(|_| {
            serde_json::json!({
                "synthesis": { "type": "oscillator", "waveform": "sine", "frequency": 440.0 },
                "envelope": { "attack": 0.0, "decay": 0.0, "sustain": 1.0, "release": 0.0 },
                "volume": 0.1,
                "pan": 0.0
            })
        })
        .collect();

    let spec = crate::spec::Spec::builder("test-budget-05", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 1.0,
                "layers": layers
            }),
        ))
        .build();

    let result = validate_for_generate_with_budget(&spec, &BudgetProfile::strict());
    assert!(!result.is_ok());
    assert!(result
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::BudgetExceeded));
    assert!(result
        .errors
        .iter()
        .any(|e| e.message.contains("layers count")));
}

#[test]
fn test_budget_spec_passes_with_default_then_fails_with_strict() {
    // This spec should pass with default budget but fail with strict budget
    let spec = crate::spec::Spec::builder("test-budget-06", AssetType::Audio)
        .license("CC0-1.0")
        .seed(42)
        .output(OutputSpec::primary(OutputFormat::Wav, "sounds/test.wav"))
        .recipe(Recipe::new(
            "audio_v1",
            serde_json::json!({
                "duration_seconds": 20.0,  // Passes default (30s), fails strict (10s)
                "sample_rate": 48000,      // Passes default, fails strict (only 22050, 44100)
                "layers": []
            }),
        ))
        .build();

    // Should pass with default budget
    let result_default = validate_for_generate_with_budget(&spec, &BudgetProfile::default());
    assert!(result_default.is_ok(), "{:?}", result_default.errors);

    // Should fail with strict budget
    let result_strict = validate_for_generate_with_budget(&spec, &BudgetProfile::strict());
    assert!(!result_strict.is_ok());
    assert!(result_strict
        .errors
        .iter()
        .any(|e| e.code == crate::error::ErrorCode::BudgetExceeded));
}
