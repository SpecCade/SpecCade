//! Core stdlib functions (spec, output, envelope)

use super::{func, param, FunctionInfo};
use speccade_spec::{AssetType, OutputFormat, OutputKind};

pub(super) fn register_functions() -> Vec<FunctionInfo> {
    let asset_types: Vec<&str> = AssetType::all().iter().map(|t| t.as_str()).collect();
    let output_formats: Vec<&str> = OutputFormat::all().iter().map(|f| f.extension()).collect();
    let output_kinds: Vec<&str> = OutputKind::all().iter().map(|k| k.as_str()).collect();

    vec![
        func!(
            "spec",
            "core",
            "Creates a complete spec dictionary with all required fields.",
            vec![
                param!("asset_id", "string", req),
                param!("asset_type", "string", req, enum: &asset_types),
                param!("seed", "int", req, range: Some(0.0), Some(4294967295.0)),
                param!("outputs", "list", req),
                param!("recipe", "dict", opt_none),
                param!("license", "string", opt, "CC0-1.0"),
                param!("description", "string", opt_none),
                param!("style_tags", "list", opt_none),
            ],
            "A complete spec dict ready for serialization.",
            r#"spec(asset_id="laser-01", asset_type="audio", seed=42, outputs=[...], recipe={...})"#
        ),
        func!(
            "output",
            "core",
            "Creates an output specification for an asset.",
            vec![
                param!("path", "string", req),
                param!("format", "string", req, enum: &output_formats),
                param!("kind", "string", opt, "primary", enum: &output_kinds),
                param!("source", "string", opt_none),
            ],
            "An output dict for the spec outputs list.",
            r#"output("sounds/laser.wav", "wav")"#
        ),
        func!(
            "envelope",
            "core",
            "Creates an ADSR envelope configuration.",
            vec![
                param!("attack", "float", opt, 0.01, range: Some(0.0), None),
                param!("decay", "float", opt, 0.1, range: Some(0.0), None),
                param!("sustain", "float", opt, 0.7, range: Some(0.0), Some(1.0)),
                param!("release", "float", opt, 0.2, range: Some(0.0), None),
            ],
            "An ADSR envelope dict.",
            "envelope(0.01, 0.1, 0.7, 0.2)"
        ),
    ]
}
