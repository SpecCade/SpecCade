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
}

impl StdlibDump {
    pub fn new() -> Self {
        let mut functions = Vec::new();
        functions.extend(core::register_functions());
        functions.extend(audio::register_functions());
        functions.extend(texture::register_functions());
        functions.extend(mesh::register_functions());
        functions.extend(music::register_functions());

        functions.sort_by(|a, b| {
            a.category
                .cmp(&b.category)
                .then_with(|| a.name.cmp(&b.name))
        });
        Self {
            stdlib_version: STDLIB_VERSION.to_string(),
            coordinate_system: CoordinateSystem::default(),
            functions,
        }
    }
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
#[derive(Debug, Serialize)]
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

        let coord = parsed.get("coordinate_system").expect("coordinate_system field missing");
        assert_eq!(coord.get("handedness").unwrap(), "right");
        assert_eq!(coord.get("up").unwrap(), "+Z");
        assert_eq!(coord.get("forward").unwrap(), "+Y");
        assert_eq!(coord.get("units").unwrap(), "meters");
    }
}
