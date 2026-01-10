use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub(crate) fn apply_validation_messages(
    mut builder: speccade_spec::ReportBuilder,
    validation: &speccade_spec::ValidationResult,
) -> speccade_spec::ReportBuilder {
    for err in &validation.errors {
        builder = builder.error(speccade_spec::ReportError::from_validation_error(err));
    }
    for warn in &validation.warnings {
        builder = builder.warning(speccade_spec::ReportWarning::from_validation_warning(warn));
    }
    builder
}

pub(crate) fn report_path(spec_path: &str, asset_id: &str) -> String {
    let spec_dir = Path::new(spec_path).parent().unwrap_or(Path::new("."));
    spec_dir
        .join(format!("{}.report.json", asset_id))
        .to_string_lossy()
        .to_string()
}

pub(crate) fn write_report(report: &speccade_spec::Report, path: &str) -> Result<()> {
    let json = serde_json::to_string_pretty(report).context("Failed to serialize report")?;
    fs::write(path, json).with_context(|| format!("Failed to write report to: {}", path))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::{
        ErrorCode, ValidationError, ValidationResult, ValidationWarning, WarningCode,
    };
    use speccade_spec::{OutputFormat, OutputKind, OutputResult, ReportBuilder};

    #[test]
    fn test_report_path_sibling_file() {
        let path = report_path("specs/audio/test.json", "laser-blast-01");
        // Use Path for platform-independent comparison
        let path = Path::new(&path);
        let expected = Path::new("specs")
            .join("audio")
            .join("laser-blast-01.report.json");
        assert_eq!(path, expected);
    }

    #[test]
    fn test_write_report_round_trip() {
        let tmp = tempfile::tempdir().unwrap();
        let report_path = tmp.path().join("report.json");

        let report = ReportBuilder::new("hash".to_string(), "backend v1.0".to_string())
            .ok(true)
            .output(OutputResult::tier1(
                OutputKind::Primary,
                OutputFormat::Wav,
                "sounds/test.wav".into(),
                "outhash".to_string(),
            ))
            .build();

        write_report(&report, report_path.to_str().unwrap()).unwrap();

        let json = fs::read_to_string(&report_path).unwrap();
        let parsed: speccade_spec::Report = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.report_version, report.report_version);
        assert_eq!(parsed.spec_hash, report.spec_hash);
        assert_eq!(parsed.outputs.len(), 1);
    }

    #[test]
    fn test_apply_validation_messages() {
        let validation = ValidationResult::failure_with_warnings(
            vec![ValidationError::new(ErrorCode::NoOutputs, "no outputs")],
            vec![ValidationWarning::new(
                WarningCode::MissingLicense,
                "missing",
            )],
        );

        let report = apply_validation_messages(
            ReportBuilder::new("hash".to_string(), "backend v1.0".to_string()),
            &validation,
        )
        .build();

        assert!(!report.ok);
        assert_eq!(report.errors.len(), 1);
        assert_eq!(report.warnings.len(), 1);
    }
}
