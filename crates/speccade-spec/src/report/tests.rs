//! Tests for report types.

use super::*;
use crate::error::{ErrorCode, ValidationError, ValidationWarning, WarningCode};
use crate::output::{OutputFormat, OutputKind};
use std::path::PathBuf;

#[test]
fn test_report_builder() {
    let report = ReportBuilder::new(
        "a1b2c3d4e5f6".to_string(),
        "speccade-backend-audio v0.1.0".to_string(),
    )
    .ok(true)
    .duration_ms(1234)
    .build();

    assert_eq!(report.report_version, 1);
    assert_eq!(report.spec_hash, "a1b2c3d4e5f6");
    assert!(report.ok);
    assert_eq!(report.duration_ms, 1234);
    assert_eq!(report.backend_version, "speccade-backend-audio v0.1.0");
    assert!(!report.target_triple.is_empty());
}

#[test]
fn test_report_builder_with_errors() {
    let error = ReportError::new("E001", "Unsupported spec version");
    let report = ReportBuilder::new("abc123".to_string(), "test-backend v1.0".to_string())
        .error(error)
        .duration_ms(500)
        .build();

    assert!(!report.ok);
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.errors[0].code, "E001");
    assert_eq!(report.errors[0].message, "Unsupported spec version");
}

#[test]
fn test_report_builder_with_warnings() {
    let warning = ReportWarning::new("W001", "Missing license");
    let report = ReportBuilder::new("xyz789".to_string(), "test-backend v1.0".to_string())
        .warning(warning)
        .build();

    assert!(report.ok);
    assert_eq!(report.warnings.len(), 1);
    assert_eq!(report.warnings[0].code, "W001");
}

#[test]
fn test_report_filename() {
    assert_eq!(
        Report::filename("laser-blast-01"),
        "laser-blast-01.report.json"
    );
    assert_eq!(Report::filename("test"), "test.report.json");
}

#[test]
fn test_report_serialization() {
    let report = ReportBuilder::new("testhash".to_string(), "test-backend v1.0".to_string())
        .ok(true)
        .duration_ms(1000)
        .output(OutputResult::tier1(
            OutputKind::Primary,
            OutputFormat::Wav,
            PathBuf::from("sounds/test.wav"),
            "outputhash".to_string(),
        ))
        .build();

    let json = report.to_json().unwrap();
    assert!(json.contains("\"report_version\":1"));
    assert!(json.contains("\"spec_hash\":\"testhash\""));
    assert!(json.contains("\"ok\":true"));

    // Round-trip test
    let parsed = Report::from_json(&json).unwrap();
    assert_eq!(parsed.spec_hash, report.spec_hash);
    assert_eq!(parsed.ok, report.ok);
    assert_eq!(parsed.outputs.len(), 1);
}

#[test]
fn test_report_pretty_json() {
    let report = ReportBuilder::new("prettyhash".to_string(), "test-backend v1.0".to_string())
        .ok(true)
        .build();

    let pretty = report.to_json_pretty().unwrap();
    assert!(pretty.contains('\n'));
    assert!(pretty.contains("  "));
}

#[test]
fn test_validation_error_conversion() {
    let val_err =
        ValidationError::with_path(ErrorCode::InvalidAssetId, "Invalid format", "asset_id");

    let report_err = ReportError::from_validation_error(&val_err);
    assert_eq!(report_err.code, "E002");
    assert_eq!(report_err.message, "Invalid format");
    assert_eq!(report_err.path, Some("asset_id".to_string()));
}

#[test]
fn test_validation_warning_conversion() {
    let val_warn = ValidationWarning::with_path(
        WarningCode::MissingLicense,
        "No license specified",
        "license",
    );

    let report_warn = ReportWarning::from_validation_warning(&val_warn);
    assert_eq!(report_warn.code, "W001");
    assert_eq!(report_warn.message, "No license specified");
    assert_eq!(report_warn.path, Some("license".to_string()));
}

#[test]
fn test_output_result_tier1() {
    let result = OutputResult::tier1(
        OutputKind::Primary,
        OutputFormat::Wav,
        PathBuf::from("sounds/test.wav"),
        "abc123".to_string(),
    );

    assert_eq!(result.kind, OutputKind::Primary);
    assert_eq!(result.format, OutputFormat::Wav);
    assert_eq!(result.hash, Some("abc123".to_string()));
    assert!(result.metrics.is_none());
}

#[test]
fn test_output_result_tier2() {
    let metrics = OutputMetrics::new()
        .with_triangle_count(1234)
        .with_bone_count(22);

    let result = OutputResult::tier2(
        OutputKind::Primary,
        OutputFormat::Glb,
        PathBuf::from("meshes/robot.glb"),
        metrics,
    );

    assert_eq!(result.kind, OutputKind::Primary);
    assert_eq!(result.format, OutputFormat::Glb);
    assert!(result.hash.is_none());
    assert!(result.metrics.is_some());

    let m = result.metrics.unwrap();
    assert_eq!(m.triangle_count, Some(1234));
    assert_eq!(m.bone_count, Some(22));
}

#[test]
fn test_output_metrics_builder() {
    let bbox = BoundingBox::new([-1.0, 0.0, -1.0], [1.0, 2.0, 1.0]);
    let metrics = OutputMetrics::new()
        .with_triangle_count(500)
        .with_bounding_box(bbox)
        .with_uv_island_count(4)
        .with_bone_count(22)
        .with_material_slot_count(2)
        .with_max_bone_influences(4)
        .with_animation_frame_count(30)
        .with_animation_duration_seconds(1.0);

    assert_eq!(metrics.triangle_count, Some(500));
    assert_eq!(metrics.uv_island_count, Some(4));
    assert_eq!(metrics.bone_count, Some(22));
    assert_eq!(metrics.material_slot_count, Some(2));
    assert_eq!(metrics.max_bone_influences, Some(4));
    assert_eq!(metrics.animation_frame_count, Some(30));
    assert_eq!(metrics.animation_duration_seconds, Some(1.0));
    assert!(metrics.bounding_box.is_some());
}

#[test]
fn test_bounding_box() {
    let bbox = BoundingBox::new([-1.0, -2.0, -3.0], [1.0, 2.0, 3.0]);
    assert_eq!(bbox.min, [-1.0, -2.0, -3.0]);
    assert_eq!(bbox.max, [1.0, 2.0, 3.0]);
}

#[test]
fn test_report_error_with_path() {
    let err = ReportError::with_path("E007", "Duplicate path", "outputs[1].path");
    assert_eq!(err.code, "E007");
    assert_eq!(err.message, "Duplicate path");
    assert_eq!(err.path, Some("outputs[1].path".to_string()));
}

#[test]
fn test_report_warning_with_path() {
    let warn = ReportWarning::with_path("W002", "Missing description", "description");
    assert_eq!(warn.code, "W002");
    assert_eq!(warn.message, "Missing description");
    assert_eq!(warn.path, Some("description".to_string()));
}

#[test]
fn test_validation_errors_batch_conversion() {
    let errors = vec![
        ValidationError::new(ErrorCode::InvalidAssetId, "Bad asset ID"),
        ValidationError::new(ErrorCode::NoOutputs, "No outputs declared"),
    ];

    let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
        .validation_errors(&errors)
        .build();

    assert!(!report.ok);
    assert_eq!(report.errors.len(), 2);
    assert_eq!(report.errors[0].code, "E002");
    assert_eq!(report.errors[1].code, "E005");
}

#[test]
fn test_validation_warnings_batch_conversion() {
    let warnings = vec![
        ValidationWarning::new(WarningCode::MissingLicense, "No license"),
        ValidationWarning::new(WarningCode::MissingDescription, "No description"),
    ];

    let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
        .validation_warnings(&warnings)
        .build();

    assert!(report.ok);
    assert_eq!(report.warnings.len(), 2);
    assert_eq!(report.warnings[0].code, "W001");
    assert_eq!(report.warnings[1].code, "W002");
}
