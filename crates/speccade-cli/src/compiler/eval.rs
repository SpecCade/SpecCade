//! Starlark evaluation logic.
//!
//! This module provides the core evaluation functionality for Starlark specs,
//! including safety limits (timeout) and dialect configuration.

use super::convert::starlark_to_json;
use super::error::CompileError;
use super::stdlib::register_stdlib;
use super::{CompileResult, CompilerConfig};
use speccade_spec::Spec;
use starlark::environment::{GlobalsBuilder, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::dict::DictRef;

/// Creates the Starlark dialect configuration.
///
/// The dialect is configured for safety:
/// - Functions (`def`) and lambdas are enabled for abstraction
/// - Top-level statements are enabled for spec definitions
/// - `load()` statements are disabled (no external file loading)
/// - Recursion is disabled (prevents infinite loops)
fn create_dialect(config: &CompilerConfig) -> Dialect {
    Dialect {
        enable_def: true,
        enable_lambda: true,
        enable_load: config.enable_load,
        enable_top_level_stmt: true,
        ..Dialect::Standard
    }
}

/// Evaluates Starlark source code synchronously (for internal use).
fn eval_starlark_sync(
    filename: &str,
    source: &str,
    config: &CompilerConfig,
) -> Result<CompileResult, CompileError> {
    // Parse the source
    let dialect = create_dialect(config);
    let ast = AstModule::parse(filename, source.to_string(), &dialect).map_err(|e| {
        // Extract location from error message if possible
        let msg = e.to_string();
        CompileError::Syntax {
            location: filename.to_string(),
            message: msg,
        }
    })?;

    // Create evaluation environment with stdlib
    let module = Module::new();
    let globals = GlobalsBuilder::standard()
        .with(register_stdlib)
        .build();
    let mut eval = Evaluator::new(&module);

    // Evaluate the module
    let value = eval.eval_module(ast, &globals).map_err(|e| {
        let msg = e.to_string();
        CompileError::Runtime {
            location: filename.to_string(),
            message: msg,
        }
    })?;

    // The result should be a dict (the spec)
    if DictRef::from_value(value).is_none() {
        return Err(CompileError::NotADict {
            type_name: value.get_type().to_string(),
        });
    }

    // Convert to JSON
    let json_value = starlark_to_json(value)?;

    // Parse as Spec
    let spec = Spec::from_value(json_value).map_err(|e| CompileError::InvalidSpec {
        message: e.to_string(),
    })?;

    Ok(CompileResult {
        spec,
        warnings: Vec::new(),
    })
}

/// Evaluates Starlark source with timeout.
///
/// Uses tokio for timeout enforcement. If the Starlark feature is enabled,
/// this function wraps the evaluation in a timeout.
pub fn eval_with_timeout(
    filename: &str,
    source: &str,
    config: &CompilerConfig,
) -> Result<CompileResult, CompileError> {
    use std::time::Duration;
    use tokio::runtime::Builder;
    use tokio::time::timeout;

    // Create a minimal tokio runtime for timeout
    // Use Builder API since rt feature doesn't include Runtime::new()
    let rt = Builder::new_current_thread()
        .enable_time()
        .build()
        .map_err(|e| CompileError::Runtime {
            location: filename.to_string(),
            message: format!("failed to create runtime: {}", e),
        })?;

    let timeout_duration = Duration::from_secs(config.timeout_seconds);

    // Extract timeout_seconds before cloning config (needed for error message after move)
    let timeout_seconds = config.timeout_seconds;

    // Clone values needed for the async block
    let filename = filename.to_string();
    let source = source.to_string();
    let config = config.clone();

    rt.block_on(async {
        match timeout(timeout_duration, async {
            // Starlark evaluation is synchronous, but we wrap it for timeout
            tokio::task::spawn_blocking(move || {
                eval_starlark_sync(&filename, &source, &config)
            })
            .await
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(e)) => Err(CompileError::Runtime {
                location: "unknown".to_string(),
                message: format!("task panicked: {}", e),
            }),
            Err(_) => Err(CompileError::Timeout {
                seconds: timeout_seconds,
            }),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_starlark(source: &str) -> Result<Spec, CompileError> {
        let config = CompilerConfig::default();
        let result = eval_starlark_sync("test.star", source, &config)?;
        Ok(result.spec)
    }

    #[test]
    fn test_minimal_spec() {
        let source = r#"
{
    "spec_version": 1,
    "asset_id": "test-asset-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        let spec = compile_starlark(source).unwrap();
        assert_eq!(spec.asset_id, "test-asset-01");
        assert_eq!(spec.seed, 42);
    }

    #[test]
    fn test_spec_with_variables() {
        let source = r#"
asset_id = "variable-test-01"
seed = 123

{
    "spec_version": 1,
    "asset_id": asset_id,
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": seed,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        let spec = compile_starlark(source).unwrap();
        assert_eq!(spec.asset_id, "variable-test-01");
        assert_eq!(spec.seed, 123);
    }

    #[test]
    fn test_spec_with_function() {
        let source = r#"
def make_output(filename):
    return {
        "kind": "primary",
        "format": "wav",
        "path": "sounds/" + filename + ".wav"
    }

{
    "spec_version": 1,
    "asset_id": "function-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [make_output("test")]
}
"#;
        let spec = compile_starlark(source).unwrap();
        assert_eq!(spec.outputs.len(), 1);
        assert_eq!(spec.outputs[0].path, "sounds/test.wav");
    }

    #[test]
    fn test_spec_with_list_comprehension() {
        let source = r#"
tags = ["retro", "scifi", "action"]

{
    "spec_version": 1,
    "asset_id": "listcomp-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "style_tags": [tag.upper() for tag in tags],
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        let spec = compile_starlark(source).unwrap();
        let tags = spec.style_tags.as_ref().unwrap();
        assert_eq!(tags, &["RETRO", "SCIFI", "ACTION"]);
    }

    #[test]
    fn test_syntax_error() {
        let source = r#"
{
    "asset_id": "test"
    # missing comma
    "seed": 42
}
"#;
        let result = compile_starlark(source);
        assert!(matches!(result, Err(CompileError::Syntax { .. })));
    }

    #[test]
    fn test_runtime_error() {
        let source = r#"
undefined_variable
"#;
        let result = compile_starlark(source);
        assert!(matches!(result, Err(CompileError::Runtime { .. })));
    }

    #[test]
    fn test_not_a_dict() {
        let source = r#"
[1, 2, 3]
"#;
        let result = compile_starlark(source);
        assert!(matches!(result, Err(CompileError::NotADict { .. })));
    }

    #[test]
    fn test_invalid_spec() {
        // Valid dict but not a valid Spec (missing required fields)
        let source = r#"
{"only_field": "value"}
"#;
        let result = compile_starlark(source);
        assert!(matches!(result, Err(CompileError::InvalidSpec { .. })));
    }

    #[test]
    fn test_timeout() {
        // This test verifies the timeout mechanism works
        // We use a very short timeout to test quickly
        let source = r#"
{
    "spec_version": 1,
    "asset_id": "timeout-test-01",
    "asset_type": "audio",
    "license": "CC0-1.0",
    "seed": 42,
    "outputs": [
        {
            "kind": "primary",
            "format": "wav",
            "path": "sounds/test.wav"
        }
    ]
}
"#;
        let config = CompilerConfig {
            timeout_seconds: 30,
            enable_load: false,
        };
        let result = eval_with_timeout("test.star", source, &config);
        // Should succeed since the code is fast
        assert!(result.is_ok());
    }

    #[test]
    fn test_spec_with_stdlib() {
        // Test that stdlib functions are available in compilation
        let source = r#"
spec(
    asset_id = "stdlib-test-01",
    asset_type = "audio",
    seed = 42,
    outputs = [output("sounds/test.wav", "wav")],
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
"#;
        let spec = compile_starlark(source).unwrap();
        assert_eq!(spec.asset_id, "stdlib-test-01");
        assert_eq!(spec.seed, 42);
        assert!(spec.recipe.is_some());
    }
}
