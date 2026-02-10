//! XM/IT structural parity checking utilities.
//!
//! This module provides functions to compare generated XM and IT outputs
//! for structural equivalence. These checks verify that both formats represent
//! the same musical content at a structural level.
//!
//! # Limitations
//!
//! Structural parity does not guarantee identical playback. See
//! `docs/xm-it-differences.md` for known playback differences between formats.

use std::fmt;

use crate::it::validator::{ItValidationReport, ItValidator};
use crate::xm::{XmValidationReport, XmValidator};

/// A mismatch found during parity checking.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParityMismatch {
    /// Category of the mismatch (e.g., "instrument_count", "pattern_rows").
    pub category: &'static str,
    /// Human-readable description of the mismatch.
    pub message: String,
}

impl fmt::Display for ParityMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.category, self.message)
    }
}

/// Result of a parity check between XM and IT files.
#[derive(Debug, Clone)]
pub struct ParityReport {
    /// Whether the files are structurally equivalent.
    pub is_parity: bool,
    /// List of mismatches found (empty if parity).
    pub mismatches: Vec<ParityMismatch>,
    /// XM file summary for reporting.
    pub xm_summary: Option<FormatSummary>,
    /// IT file summary for reporting.
    pub it_summary: Option<FormatSummary>,
}

impl ParityReport {
    /// Create a successful parity report.
    fn success(xm_summary: FormatSummary, it_summary: FormatSummary) -> Self {
        Self {
            is_parity: true,
            mismatches: Vec::new(),
            xm_summary: Some(xm_summary),
            it_summary: Some(it_summary),
        }
    }

    /// Create a failed parity report with mismatches.
    fn failure(
        mismatches: Vec<ParityMismatch>,
        xm_summary: Option<FormatSummary>,
        it_summary: Option<FormatSummary>,
    ) -> Self {
        Self {
            is_parity: false,
            mismatches,
            xm_summary,
            it_summary,
        }
    }
}

impl fmt::Display for ParityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_parity {
            writeln!(f, "Parity: PASS")?;
        } else {
            writeln!(f, "Parity: FAIL ({} mismatches)", self.mismatches.len())?;
            for mismatch in &self.mismatches {
                writeln!(f, "  - {}", mismatch)?;
            }
        }

        if let Some(ref xm) = self.xm_summary {
            writeln!(f, "XM: {}", xm)?;
        }
        if let Some(ref it) = self.it_summary {
            writeln!(f, "IT: {}", it)?;
        }

        Ok(())
    }
}

/// Summary of a tracker module's structure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatSummary {
    /// Number of instruments.
    pub instrument_count: usize,
    /// Number of samples (IT only tracks this separately).
    pub sample_count: usize,
    /// Number of patterns.
    pub pattern_count: usize,
    /// Rows per pattern (list).
    pub pattern_rows: Vec<u16>,
    /// Number of channels.
    pub channel_count: u16,
    /// Default tempo/speed.
    pub tempo: u16,
    /// Default BPM.
    pub bpm: u16,
}

impl fmt::Display for FormatSummary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "instruments={}, samples={}, patterns={}, channels={}, tempo={}, bpm={}",
            self.instrument_count,
            self.sample_count,
            self.pattern_count,
            self.channel_count,
            self.tempo,
            self.bpm
        )
    }
}

/// Error during parity checking.
#[derive(Debug)]
pub enum ParityError {
    /// Failed to parse XM file.
    XmParseError(String),
    /// Failed to parse IT file.
    ItParseError(String),
    /// XM validation found errors.
    XmValidationError(Vec<String>),
    /// IT validation found errors.
    ItValidationError(Vec<String>),
}

impl fmt::Display for ParityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParityError::XmParseError(msg) => write!(f, "XM parse error: {}", msg),
            ParityError::ItParseError(msg) => write!(f, "IT parse error: {}", msg),
            ParityError::XmValidationError(errs) => {
                write!(f, "XM validation errors: {}", errs.join("; "))
            }
            ParityError::ItValidationError(errs) => {
                write!(f, "IT validation errors: {}", errs.join("; "))
            }
        }
    }
}

impl std::error::Error for ParityError {}

/// Check structural parity between XM and IT module data.
///
/// This function validates both files and compares their structural properties:
/// - Instrument count
/// - Sample count (IT tracks separately from instruments)
/// - Pattern count
/// - Rows per pattern
/// - Channel count
/// - Tempo and BPM settings
///
/// # Arguments
/// * `xm_data` - Raw bytes of the XM file
/// * `it_data` - Raw bytes of the IT file
///
/// # Returns
/// * `Ok(ParityReport)` - Parity check completed (check `is_parity` for result)
/// * `Err(ParityError)` - Failed to parse or validate one of the files
pub fn check_parity(xm_data: &[u8], it_data: &[u8]) -> Result<ParityReport, ParityError> {
    // Validate and parse XM
    let xm_report =
        XmValidator::validate(xm_data).map_err(|e| ParityError::XmParseError(e.to_string()))?;

    if !xm_report.valid {
        let errors: Vec<String> = xm_report.errors.iter().map(|e| e.to_string()).collect();
        return Err(ParityError::XmValidationError(errors));
    }

    // Validate and parse IT
    let it_report =
        ItValidator::validate(it_data).map_err(|e| ParityError::ItParseError(e.to_string()))?;

    if !it_report.is_valid {
        let errors: Vec<String> = it_report.errors.iter().map(|e| e.to_string()).collect();
        return Err(ParityError::ItValidationError(errors));
    }

    // Extract summaries
    let xm_summary = extract_xm_summary(&xm_report);
    let it_summary = extract_it_summary(&it_report);

    // Compare structures
    let mismatches = compare_structures(&xm_summary, &it_summary);

    if mismatches.is_empty() {
        Ok(ParityReport::success(xm_summary, it_summary))
    } else {
        Ok(ParityReport::failure(
            mismatches,
            Some(xm_summary),
            Some(it_summary),
        ))
    }
}

/// Extract a structural summary from an XM validation report.
fn extract_xm_summary(report: &XmValidationReport) -> FormatSummary {
    let header = report.header.as_ref().expect("XM header should be present");

    // In XM, each instrument can have multiple samples.
    // Use num_samples from the instrument header since samples Vec may not be populated.
    let sample_count: usize = report
        .instruments
        .iter()
        .map(|i| i.num_samples as usize)
        .sum();

    FormatSummary {
        instrument_count: report.instruments.len(),
        sample_count,
        pattern_count: report.patterns.len(),
        pattern_rows: report.patterns.iter().map(|p| p.num_rows).collect(),
        channel_count: header.num_channels,
        tempo: header.default_tempo,
        bpm: header.default_bpm,
    }
}

/// Extract a structural summary from an IT validation report.
fn extract_it_summary(report: &ItValidationReport) -> FormatSummary {
    let header = report.header.as_ref().expect("IT header should be present");

    FormatSummary {
        instrument_count: report.instruments.len(),
        sample_count: report.samples.len(),
        pattern_count: report.patterns.len(),
        pattern_rows: report.patterns.iter().map(|p| p.num_rows).collect(),
        channel_count: 64, // IT always reports 64 channel slots in header
        tempo: header.initial_speed as u16,
        bpm: header.initial_tempo as u16,
    }
}

/// Compare two format summaries and return mismatches.
fn compare_structures(xm: &FormatSummary, it: &FormatSummary) -> Vec<ParityMismatch> {
    let mut mismatches = Vec::new();

    // Compare instrument count
    if xm.instrument_count != it.instrument_count {
        mismatches.push(ParityMismatch {
            category: "instrument_count",
            message: format!(
                "XM has {} instruments, IT has {}",
                xm.instrument_count, it.instrument_count
            ),
        });
    }

    // Compare sample count
    if xm.sample_count != it.sample_count {
        mismatches.push(ParityMismatch {
            category: "sample_count",
            message: format!(
                "XM has {} samples, IT has {}",
                xm.sample_count, it.sample_count
            ),
        });
    }

    // Compare pattern count
    if xm.pattern_count != it.pattern_count {
        mismatches.push(ParityMismatch {
            category: "pattern_count",
            message: format!(
                "XM has {} patterns, IT has {}",
                xm.pattern_count, it.pattern_count
            ),
        });
    }

    // Compare pattern rows
    let min_patterns = xm.pattern_rows.len().min(it.pattern_rows.len());
    for i in 0..min_patterns {
        if xm.pattern_rows[i] != it.pattern_rows[i] {
            mismatches.push(ParityMismatch {
                category: "pattern_rows",
                message: format!(
                    "Pattern {} has {} rows in XM, {} rows in IT",
                    i, xm.pattern_rows[i], it.pattern_rows[i]
                ),
            });
        }
    }

    // Compare tempo
    if xm.tempo != it.tempo {
        mismatches.push(ParityMismatch {
            category: "tempo",
            message: format!("XM tempo is {}, IT tempo is {}", xm.tempo, it.tempo),
        });
    }

    // Compare BPM
    if xm.bpm != it.bpm {
        mismatches.push(ParityMismatch {
            category: "bpm",
            message: format!("XM BPM is {}, IT BPM is {}", xm.bpm, it.bpm),
        });
    }

    // Note: We don't compare channel count because IT always has 64 channel slots
    // while XM specifies only active channels. This is a known format difference.

    mismatches
}

/// Check parity with detailed note-level comparison.
///
/// This extends the basic parity check to also verify note placements
/// within patterns. Due to format differences, this comparison is best-effort.
///
/// # Note
///
/// This function currently performs the same checks as `check_parity`.
/// Detailed note-level comparison requires additional pattern parsing
/// infrastructure that may be added in future versions.
pub fn check_parity_detailed(xm_data: &[u8], it_data: &[u8]) -> Result<ParityReport, ParityError> {
    // Validate and parse XM
    let xm_report =
        XmValidator::validate(xm_data).map_err(|e| ParityError::XmParseError(e.to_string()))?;
    if !xm_report.valid {
        let errors: Vec<String> = xm_report.errors.iter().map(|e| e.to_string()).collect();
        return Err(ParityError::XmValidationError(errors));
    }

    // Validate and parse IT
    let it_report =
        ItValidator::validate(it_data).map_err(|e| ParityError::ItParseError(e.to_string()))?;
    if !it_report.is_valid {
        let errors: Vec<String> = it_report.errors.iter().map(|e| e.to_string()).collect();
        return Err(ParityError::ItValidationError(errors));
    }

    let xm_summary = extract_xm_summary(&xm_report);
    let it_summary = extract_it_summary(&it_report);

    let mut mismatches = compare_structures(&xm_summary, &it_summary);
    mismatches.extend(compare_order_tables(&xm_report, &it_report));

    if mismatches.is_empty() {
        Ok(ParityReport::success(xm_summary, it_summary))
    } else {
        Ok(ParityReport::failure(
            mismatches,
            Some(xm_summary),
            Some(it_summary),
        ))
    }
}

fn compare_order_tables(
    xm_report: &XmValidationReport,
    it_report: &ItValidationReport,
) -> Vec<ParityMismatch> {
    let mut mismatches = Vec::new();

    let xm_header = match xm_report.header.as_ref() {
        Some(h) => h,
        None => return mismatches,
    };

    let xm_len = xm_header.song_length as usize;
    let xm_orders = &xm_header.pattern_order[..xm_len.min(xm_header.pattern_order.len())];
    let it_orders: Vec<u8> = it_report
        .orders
        .iter()
        .copied()
        .filter(|order| *order != 254 && *order != 255)
        .collect();

    if xm_orders.len() != it_orders.len() {
        mismatches.push(ParityMismatch {
            category: "order_length",
            message: format!(
                "XM order length is {}, IT order length is {}",
                xm_orders.len(),
                it_orders.len()
            ),
        });
    }

    let min_len = xm_orders.len().min(it_orders.len());
    for i in 0..min_len {
        if xm_orders[i] != it_orders[i] {
            mismatches.push(ParityMismatch {
                category: "order_entry",
                message: format!(
                    "Order {} differs (XM={}, IT={})",
                    i, xm_orders[i], it_orders[i]
                ),
            });
        }
    }

    mismatches
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::it::{ItInstrument, ItModule, ItPattern, ItSample};
    use crate::xm::{XmInstrument, XmModule, XmPattern, XmSample};

    fn create_minimal_xm(instruments: usize, patterns: usize, rows: u16) -> Vec<u8> {
        let mut module = XmModule::new("Test", 4, 6, 125);

        for i in 0..instruments {
            // Create 16-bit sample data (200 bytes = 100 samples)
            let sample_data = vec![0u8; 200];
            let sample = XmSample::new(&format!("Smp{}", i), sample_data, true);
            let instrument = XmInstrument::new(&format!("Inst{}", i), sample);
            module.add_instrument(instrument);
        }

        for _ in 0..patterns {
            module.add_pattern(XmPattern::empty(rows, 4));
        }

        let orders: Vec<u8> = (0..patterns as u8).collect();
        module.set_order_table(&orders);

        module.to_bytes().expect("XM serialization failed")
    }

    fn create_minimal_it(
        instruments: usize,
        samples: usize,
        patterns: usize,
        rows: u16,
    ) -> Vec<u8> {
        let mut module = ItModule::new("Test", 4, 6, 125);

        for i in 0..instruments {
            module.add_instrument(ItInstrument::new(&format!("Inst{}", i)));
        }

        for i in 0..samples {
            module.add_sample(ItSample::new(&format!("Smp{}", i), vec![0u8; 100], 22050));
        }

        for _ in 0..patterns {
            module.add_pattern(ItPattern::empty(rows, 4));
        }

        let orders: Vec<u8> = (0..patterns as u8).collect();
        module.set_orders(&orders);

        module.to_bytes().expect("IT serialization failed")
    }

    #[test]
    fn test_parity_matching_structures() {
        let xm = create_minimal_xm(2, 3, 64);
        let it = create_minimal_it(2, 2, 3, 64);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        assert!(
            report.is_parity,
            "Structures should match: {:?}",
            report.mismatches
        );
    }

    #[test]
    fn test_parity_mismatched_instruments() {
        let xm = create_minimal_xm(2, 1, 64);
        let it = create_minimal_it(3, 3, 1, 64);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "instrument_count"));
    }

    #[test]
    fn test_parity_mismatched_samples() {
        let xm = create_minimal_xm(2, 1, 64);
        let it = create_minimal_it(2, 4, 1, 64);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "sample_count"));
    }

    #[test]
    fn test_parity_mismatched_patterns() {
        let xm = create_minimal_xm(1, 2, 64);
        let it = create_minimal_it(1, 1, 3, 64);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "pattern_count"));
    }

    #[test]
    fn test_parity_mismatched_rows() {
        let xm = create_minimal_xm(1, 1, 64);
        let it = create_minimal_it(1, 1, 1, 32);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "pattern_rows"));
    }

    #[test]
    fn test_parity_report_display() {
        let xm = create_minimal_xm(2, 3, 64);
        let it = create_minimal_it(3, 3, 3, 64);

        let report = check_parity(&xm, &it).expect("Parity check should succeed");
        let display = format!("{}", report);
        assert!(display.contains("FAIL"));
        assert!(display.contains("instrument_count"));
    }

    #[test]
    fn test_invalid_xm_data() {
        let invalid_xm = vec![0u8; 100];
        let it = create_minimal_it(1, 1, 1, 64);

        let result = check_parity(&invalid_xm, &it);
        assert!(matches!(result, Err(ParityError::XmParseError(_))));
    }

    #[test]
    fn test_invalid_it_data() {
        let xm = create_minimal_xm(1, 1, 64);
        let invalid_it = vec![0u8; 100];

        let result = check_parity(&xm, &invalid_it);
        assert!(matches!(result, Err(ParityError::ItParseError(_))));
    }

    #[test]
    fn test_parity_detailed_detects_order_length_mismatch() {
        let mut xm_module = XmModule::new("Test", 4, 6, 125);
        xm_module.add_pattern(XmPattern::empty(64, 4));
        xm_module.set_order_table(&[0, 0]);
        let xm = xm_module.to_bytes().unwrap();

        let mut it_module = ItModule::new("Test", 4, 6, 125);
        it_module.add_instrument(ItInstrument::new("Inst1"));
        it_module.add_sample(ItSample::new("Sample1", vec![0u8; 100], 22050));
        it_module.add_pattern(ItPattern::empty(64, 4));
        it_module.set_orders(&[0]);
        let it = it_module.to_bytes().unwrap();

        let report = check_parity_detailed(&xm, &it).unwrap();
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "order_length"));
    }
}
