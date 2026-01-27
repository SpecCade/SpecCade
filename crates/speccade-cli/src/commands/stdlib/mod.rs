//! Stdlib command implementation
//!
//! Provides commands for inspecting the Starlark stdlib, including
//! dumping function metadata in machine-readable JSON format.

mod audio;
mod core;
mod mesh;
mod music;
mod texture;

use anyhow::Result;
use serde::Serialize;
use speccade_lint::RuleRegistry;
use std::collections::HashMap;
use std::process::ExitCode;

#[cfg(feature = "starlark")]
use crate::compiler::STDLIB_VERSION;

#[cfg(not(feature = "starlark"))]
const STDLIB_VERSION: &str = "0.1.0";

/// Output format for the stdlib dump command.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DumpFormat {
    Json,
}

impl std::str::FromStr for DumpFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(DumpFormat::Json),
            _ => Err(format!("Unknown format: {}. Supported: json", s)),
        }
    }
}

/// Run the stdlib dump command.
pub fn run_dump(format: DumpFormat) -> Result<ExitCode> {
    match format {
        DumpFormat::Json => {
            let dump = StdlibDump::new();
            let json = serde_json::to_string_pretty(&dump)?;
            println!("{}", json);
            Ok(ExitCode::SUCCESS)
        }
    }
}

/// Complete stdlib dump structure.
#[derive(Debug, Serialize)]
pub struct StdlibDump {
    pub stdlib_version: String,
    pub coordinate_system: CoordinateSystem,
    pub functions: Vec<FunctionInfo>,
    pub lint_rules: Vec<speccade_lint::RuleMetadata>,
}

impl StdlibDump {
    pub fn new() -> Self {
        #[cfg(feature = "starlark")]
        let mut functions = collect_functions_from_starlark_docs();

        #[cfg(not(feature = "starlark"))]
        let mut functions = {
            let mut functions = Vec::new();
            functions.extend(core::register_functions());
            functions.extend(audio::register_functions());
            functions.extend(texture::register_functions());
            functions.extend(mesh::register_functions());
            functions.extend(music::register_functions());
            functions
        };

        functions.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name))
        });

        // Get lint rule metadata from the registry
        let registry = RuleRegistry::default_rules();
        let lint_rules = registry.rule_metadata();

        Self {
            stdlib_version: STDLIB_VERSION.to_string(),
            coordinate_system: CoordinateSystem::default(),
            functions,
            lint_rules,
        }
    }
}

#[cfg(feature = "starlark")]
fn collect_functions_from_starlark_docs() -> Vec<FunctionInfo> {
    use starlark::docs::{DocMember, DocParam};
    use starlark::environment::GlobalsBuilder;

    // Manual metadata (ranges/enums/examples) for functions we already describe.
    // We only use this for enrichment; the function list + parameter defaults come
    // from the real stdlib registration.
    let manual: HashMap<String, FunctionInfo> = {
        let mut map = HashMap::new();
        for f in core::register_functions()
            .into_iter()
            .chain(audio::register_functions())
            .chain(texture::register_functions())
            .chain(mesh::register_functions())
            .chain(music::register_functions())
        {
            map.insert(f.name.clone(), f);
        }
        map
    };

    let globals = GlobalsBuilder::new()
        .with(crate::compiler::stdlib::register_stdlib)
        .build();
    let module_docs = globals.documentation();

    let mut functions = Vec::new();
    for (name, member) in module_docs.members.iter() {
        let DocMember::Function(f) = member else {
            continue;
        };

        let description = f
            .docs
            .as_ref()
            .map(|d| d.summary.clone())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| manual.get(name).map(|m| m.description.clone()))
            .unwrap_or_else(|| format!("Starlark function `{}`.", name));

        let returns = f
            .ret
            .docs
            .as_ref()
            .map(|d| d.summary.clone())
            .filter(|s| !s.trim().is_empty())
            .or_else(|| manual.get(name).map(|m| m.returns.clone()))
            .unwrap_or_else(|| format!("Returns {}.", f.ret.typ));

        let mut params: Vec<ParamInfo> = Vec::new();
        for p in &f.params {
            let DocParam::Arg {
                name: param_name,
                typ,
                default_value,
                ..
            } = p
            else {
                // Skip separators (`*`/`/`) and varargs/kwargs for now.
                continue;
            };

            let mut info = ParamInfo {
                name: param_name.clone(),
                param_type: typ.to_string(),
                required: default_value.is_none(),
                default: default_value.as_deref().and_then(|d| {
                    if d.trim() == "_" {
                        None
                    } else {
                        Some(parse_starlark_default_value(d))
                    }
                }),
                enum_values: None,
                range: None,
            };

            if let Some(m) = manual.get(name) {
                if let Some(mp) = m.params.iter().find(|mp| mp.name == info.name) {
                    info.enum_values = mp.enum_values.clone();
                    info.range = mp.range.clone();
                }
            }

            params.push(info);
        }

        let mut out = FunctionInfo {
            name: name.clone(),
            category: manual
                .get(name)
                .map(|m| m.category.clone())
                .unwrap_or_else(|| guess_category(name).to_owned()),
            description,
            params,
            returns,
            example: manual.get(name).and_then(|m| m.example.clone()),
        };

        // Ensure required fields are never empty (tests rely on this).
        if out.category.trim().is_empty() {
            out.category = "misc".to_owned();
        }
        if out.description.trim().is_empty() {
            out.description = format!("Starlark function `{}`.", out.name);
        }
        if out.returns.trim().is_empty() {
            out.returns = "Return value.".to_owned();
        }

        functions.push(out);
    }

    // Deterministic ordering.
    functions.sort_by(|a, b| {
        a.category
            .cmp(&b.category)
            .then_with(|| a.name.cmp(&b.name))
    });
    functions
}

#[cfg(feature = "starlark")]
fn parse_starlark_default_value(raw: &str) -> serde_json::Value {
    let s = raw.trim();

    match s {
        "None" => return serde_json::Value::Null,
        "True" => return serde_json::Value::Bool(true),
        "False" => return serde_json::Value::Bool(false),
        _ => {}
    }

    if let Ok(i) = s.parse::<i64>() {
        return serde_json::Value::Number(i.into());
    }
    if let Ok(f) = s.parse::<f64>() {
        if let Some(n) = serde_json::Number::from_f64(f) {
            return serde_json::Value::Number(n);
        }
    }

    // Try to parse JSON-like values (lists, dicts, quoted strings).
    let normalized = s
        .replace("True", "true")
        .replace("False", "false")
        .replace("None", "null");
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&normalized) {
        return v;
    }

    serde_json::Value::String(s.to_owned())
}

#[cfg(feature = "starlark")]
fn guess_category(name: &str) -> &'static str {
    if matches!(name, "spec" | "output" | "envelope") {
        return "core";
    }

    if name.ends_with("_node") {
        return "texture.nodes";
    }
    if name == "texture_graph" {
        return "texture.graph";
    }
    if matches!(
        name,
        "matcap_v1"
            | "material_preset_v1"
            | "decal_metadata"
            | "decal_spec"
            | "trimsheet_tile"
            | "trimsheet_spec"
            | "splat_layer"
            | "splat_set_spec"
    ) {
        return "texture";
    }
    if name.starts_with("texture_") {
        return "texture";
    }

    if name.ends_with("_modifier") {
        return "mesh.modifiers";
    }
    if name == "baking_settings" {
        return "mesh.baking";
    }
    if name.starts_with("mesh_") {
        return "mesh";
    }

    if matches!(
        name,
        "body_part"
            | "material_slot"
            | "skinning_config"
            | "custom_bone"
            | "skeletal_export_settings"
            | "skeletal_constraints"
            | "skeletal_texturing"
            | "skeletal_mesh_spec"
    ) {
        return "character";
    }

    if matches!(
        name,
        "bone_transform"
            | "animation_keyframe"
            | "animation_export_settings"
            | "ik_target_transform"
            | "ik_keyframe"
            | "skeletal_animation_spec"
    ) {
        return "animation";
    }

    if name.contains("instrument")
        || name.contains("pattern")
        || name.contains("song")
        || name.contains("tempo")
        || name == "volume_fade"
        || name == "it_options"
        || name.contains("swing")
        || name.contains("humanize")
        || name.contains("stinger")
        || name.contains("transition")
        || name.contains("cue")
        || name.starts_with("loop_")
    {
        return "music";
    }

    // Default to audio since most remaining helpers are audio-related.
    "audio"
}

impl Default for StdlibDump {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a single stdlib function.
#[derive(Debug, Serialize)]
pub struct FunctionInfo {
    pub name: String,
    pub category: String,
    pub description: String,
    pub params: Vec<ParamInfo>,
    pub returns: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}

/// Information about a function parameter.
#[derive(Debug, Serialize)]
pub struct ParamInfo {
    pub name: String,
    #[serde(rename = "type")]
    pub param_type: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<RangeInfo>,
}

/// Range constraint for a parameter.
#[derive(Debug, Clone, Serialize)]
pub struct RangeInfo {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,
}

/// Coordinate system metadata for the stdlib.
#[derive(Debug, Serialize)]
pub struct CoordinateSystem {
    pub handedness: String,
    pub up: String,
    pub forward: String,
    pub right: String,
    pub units: String,
    pub rotation_order: String,
    pub rotation_units: String,
}

impl Default for CoordinateSystem {
    fn default() -> Self {
        Self {
            handedness: "right".into(),
            up: "+Z".into(),
            forward: "+Y".into(),
            right: "+X".into(),
            units: "meters".into(),
            rotation_order: "XYZ".into(),
            rotation_units: "degrees".into(),
        }
    }
}

// Helper macros for compact function definitions
macro_rules! func {
    ($name:expr, $cat:expr, $desc:expr, $params:expr, $ret:expr) => {
        $crate::commands::stdlib::FunctionInfo {
            name: $name.into(),
            category: $cat.into(),
            description: $desc.into(),
            params: $params,
            returns: $ret.into(),
            example: None,
        }
    };
    ($name:expr, $cat:expr, $desc:expr, $params:expr, $ret:expr, $ex:expr) => {
        $crate::commands::stdlib::FunctionInfo {
            name: $name.into(),
            category: $cat.into(),
            description: $desc.into(),
            params: $params,
            returns: $ret.into(),
            example: Some($ex.into()),
        }
    };
}

macro_rules! param {
    ($name:expr, $ty:expr, req) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: true,
            default: None,
            enum_values: None,
            range: None,
        }
    };
    ($name:expr, $ty:expr, req, range: $min:expr, $max:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: true,
            default: None,
            enum_values: None,
            range: Some($crate::commands::stdlib::RangeInfo {
                min: $min,
                max: $max,
            }),
        }
    };
    ($name:expr, $ty:expr, req, enum: $vals:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: true,
            default: None,
            enum_values: Some($vals.iter().map(|s| s.to_string()).collect()),
            range: None,
        }
    };
    ($name:expr, $ty:expr, opt, $def:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: Some(serde_json::json!($def)),
            enum_values: None,
            range: None,
        }
    };
    ($name:expr, $ty:expr, opt, $def:expr, range: $min:expr, $max:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: Some(serde_json::json!($def)),
            enum_values: None,
            range: Some($crate::commands::stdlib::RangeInfo {
                min: $min,
                max: $max,
            }),
        }
    };
    ($name:expr, $ty:expr, opt, $def:expr, enum: $vals:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: Some(serde_json::json!($def)),
            enum_values: Some($vals.iter().map(|s| s.to_string()).collect()),
            range: None,
        }
    };
    ($name:expr, $ty:expr, opt_none) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: None,
            enum_values: None,
            range: None,
        }
    };
    ($name:expr, $ty:expr, opt_none, range: $min:expr, $max:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: None,
            enum_values: None,
            range: Some($crate::commands::stdlib::RangeInfo {
                min: $min,
                max: $max,
            }),
        }
    };
    ($name:expr, $ty:expr, opt_none, enum: $vals:expr) => {
        $crate::commands::stdlib::ParamInfo {
            name: $name.into(),
            param_type: $ty.into(),
            required: false,
            default: None,
            enum_values: Some($vals.iter().map(|s| s.to_string()).collect()),
            range: None,
        }
    };
}

pub(crate) use func;
pub(crate) use param;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dump_produces_valid_json() {
        let dump = StdlibDump::new();
        let json = serde_json::to_string(&dump).unwrap();
        assert!(!json.is_empty());
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.is_object());
    }

    #[test]
    fn test_dump_has_expected_top_level_keys() {
        let dump = StdlibDump::new();
        let json = serde_json::to_string(&dump).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert!(parsed.get("stdlib_version").is_some());
        assert!(parsed.get("functions").is_some());
        assert!(parsed["functions"].is_array());
    }

    #[test]
    fn test_dump_has_at_least_one_function() {
        let dump = StdlibDump::new();
        assert!(!dump.functions.is_empty());
    }

    #[test]
    fn test_dump_is_deterministic() {
        let json1 = serde_json::to_string(&StdlibDump::new()).unwrap();
        let json2 = serde_json::to_string(&StdlibDump::new()).unwrap();
        assert_eq!(json1, json2, "Dump output should be deterministic");
    }

    #[test]
    fn test_dump_functions_have_required_fields() {
        for func in &StdlibDump::new().functions {
            assert!(!func.name.is_empty());
            assert!(!func.category.is_empty());
            assert!(!func.description.is_empty());
            assert!(!func.returns.is_empty());
        }
    }

    #[test]
    fn test_dump_includes_oscillator_function() {
        let dump = StdlibDump::new();
        let osc = dump
            .functions
            .iter()
            .find(|f| f.name == "oscillator")
            .unwrap();
        assert_eq!(osc.category, "audio.synthesis");
        assert!(!osc.params.is_empty());
        let freq = osc.params.iter().find(|p| p.name == "frequency").unwrap();
        assert!(freq.required);
        assert_eq!(freq.param_type, "float");
    }

    #[test]
    fn test_dump_includes_waveform_enum() {
        let dump = StdlibDump::new();
        let osc = dump
            .functions
            .iter()
            .find(|f| f.name == "oscillator")
            .unwrap();
        let wf = osc.params.iter().find(|p| p.name == "waveform").unwrap();
        let enums = wf.enum_values.as_ref().unwrap();
        assert!(enums.contains(&"sine".to_string()));
        assert!(enums.contains(&"square".to_string()));
    }

    #[test]
    fn test_dump_format_parsing() {
        assert_eq!("json".parse::<DumpFormat>().unwrap(), DumpFormat::Json);
        assert_eq!("JSON".parse::<DumpFormat>().unwrap(), DumpFormat::Json);
        assert!("yaml".parse::<DumpFormat>().is_err());
    }

    #[test]
    fn test_dump_includes_coordinate_system() {
        let dump = StdlibDump::new();
        let json = serde_json::to_string(&dump).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let coord = parsed
            .get("coordinate_system")
            .expect("coordinate_system field missing");
        assert_eq!(coord.get("handedness").unwrap(), "right");
        assert_eq!(coord.get("up").unwrap(), "+Z");
        assert_eq!(coord.get("forward").unwrap(), "+Y");
        assert_eq!(coord.get("units").unwrap(), "meters");
    }

    #[test]
    fn test_dump_includes_lint_rules() {
        let dump = StdlibDump::new();
        let json = serde_json::to_string(&dump).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        let lint_rules = parsed.get("lint_rules").expect("lint_rules field missing");
        assert!(lint_rules.is_array());
        // Should have 44 lint rules total (audio: 10, texture: 10, mesh: 12, music: 12)
        assert_eq!(lint_rules.as_array().unwrap().len(), 44);

        // Verify structure of first rule
        let first_rule = &lint_rules[0];
        assert!(first_rule.get("id").is_some());
        assert!(first_rule.get("severity").is_some());
        assert!(first_rule.get("description").is_some());
        assert!(first_rule.get("applies_to").is_some());
    }

    #[test]
    fn test_dump_lint_rules_include_audio_clipping() {
        let dump = StdlibDump::new();
        let clipping_rule = dump
            .lint_rules
            .iter()
            .find(|r| r.id == "audio/clipping")
            .expect("audio/clipping rule should exist");

        assert!(!clipping_rule.description.is_empty());
        assert!(clipping_rule
            .applies_to
            .contains(&speccade_lint::AssetType::Audio));
    }
}
