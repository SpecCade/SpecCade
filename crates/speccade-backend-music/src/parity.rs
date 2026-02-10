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
use crate::note::it_note_to_xm;
use crate::xm::{XmValidationReport, XmValidator};
use speccade_spec::recipe::music::{decode_it_effect, decode_xm_effect, TrackerEffect};

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

const MAX_CELL_MISMATCHES: usize = 128;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct CanonCell {
    note: u8,
    instrument: u8,
    volume: u8,
    effect: u8,
    effect_param: u8,
}

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

/// Check parity with detailed cell-level comparison.
///
/// This extends the basic structural checks with:
/// - Order-table comparison
/// - Order-expanded pattern cell comparison for note/instrument/volume
///
/// Effect parity is still treated as best-effort due format-specific behavior.
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

    let xm_patterns = decode_xm_patterns(xm_data, &xm_report)?;
    let it_patterns = decode_it_patterns(it_data, &it_report)?;
    let (xm_orders, it_orders) = normalize_orders(&xm_report, &it_report);
    let xm_channels = xm_summary.channel_count as usize;
    let xm_restart = xm_report
        .header
        .as_ref()
        .map(|h| h.restart_position as u8)
        .unwrap_or(0);
    mismatches.extend(compare_cell_level_parity(
        &xm_patterns,
        &it_patterns,
        &xm_orders,
        &it_orders,
        xm_channels,
        xm_restart,
    ));

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
    let (xm_orders, it_orders) = normalize_orders(xm_report, it_report);

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

fn normalize_orders(
    xm_report: &XmValidationReport,
    it_report: &ItValidationReport,
) -> (Vec<u8>, Vec<u8>) {
    let xm_orders = xm_report
        .header
        .as_ref()
        .map(|h| {
            let len = h.song_length as usize;
            h.pattern_order[..len.min(h.pattern_order.len())].to_vec()
        })
        .unwrap_or_default();

    let it_orders = it_report
        .orders
        .iter()
        .copied()
        .filter(|order| *order != 254 && *order != 255)
        .collect();

    (xm_orders, it_orders)
}

fn normalize_xm_volume(vol: u8) -> u8 {
    if (0x10..=0x50).contains(&vol) {
        vol - 0x10
    } else {
        0
    }
}

fn decode_xm_patterns(
    xm_data: &[u8],
    xm_report: &XmValidationReport,
) -> Result<Vec<Vec<Vec<CanonCell>>>, ParityError> {
    let header = xm_report.header.as_ref().ok_or_else(|| {
        ParityError::XmParseError("missing XM header in validation report".to_string())
    })?;
    let num_channels = header.num_channels as usize;
    let mut offset = 60 + header.header_size as usize;
    let mut decoded = Vec::with_capacity(xm_report.patterns.len());

    for pattern in &xm_report.patterns {
        let header_len = pattern.header_length as usize;
        let packed_len = pattern.packed_size as usize;
        let data_start = offset + header_len;
        let data_end = data_start + packed_len;
        if data_end > xm_data.len() {
            return Err(ParityError::XmParseError(format!(
                "pattern {} extends beyond XM data (end={}, len={})",
                pattern.index,
                data_end,
                xm_data.len()
            )));
        }
        let pattern_rows = decode_xm_pattern_data(
            &xm_data[data_start..data_end],
            pattern.index,
            pattern.num_rows as usize,
            num_channels,
        )?;
        decoded.push(pattern_rows);
        offset = data_end;
    }

    Ok(decoded)
}

fn decode_xm_pattern_data(
    data: &[u8],
    pattern_index: usize,
    num_rows: usize,
    num_channels: usize,
) -> Result<Vec<Vec<CanonCell>>, ParityError> {
    let mut rows = vec![vec![CanonCell::default(); num_channels]; num_rows];
    let mut pos = 0usize;

    for row in 0..num_rows {
        for channel in 0..num_channels {
            if pos >= data.len() {
                return Err(ParityError::XmParseError(format!(
                    "pattern {} truncated at row {} channel {}",
                    pattern_index, row, channel
                )));
            }

            let first = data[pos];
            pos += 1;
            let mut cell = CanonCell::default();

            if first & 0x80 != 0 {
                if first & 0x01 != 0 {
                    if pos >= data.len() {
                        return Err(ParityError::XmParseError(format!(
                            "pattern {} note field truncated",
                            pattern_index
                        )));
                    }
                    cell.note = data[pos];
                    pos += 1;
                }
                if first & 0x02 != 0 {
                    if pos >= data.len() {
                        return Err(ParityError::XmParseError(format!(
                            "pattern {} instrument field truncated",
                            pattern_index
                        )));
                    }
                    cell.instrument = data[pos];
                    pos += 1;
                }
                if first & 0x04 != 0 {
                    if pos >= data.len() {
                        return Err(ParityError::XmParseError(format!(
                            "pattern {} volume field truncated",
                            pattern_index
                        )));
                    }
                    cell.volume = normalize_xm_volume(data[pos]);
                    pos += 1;
                }
                if first & 0x08 != 0 {
                    if pos >= data.len() {
                        return Err(ParityError::XmParseError(format!(
                            "pattern {} effect field truncated",
                            pattern_index
                        )));
                    }
                    cell.effect = data[pos];
                    pos += 1;
                }
                if first & 0x10 != 0 {
                    if pos >= data.len() {
                        return Err(ParityError::XmParseError(format!(
                            "pattern {} effect param field truncated",
                            pattern_index
                        )));
                    }
                    cell.effect_param = data[pos];
                    pos += 1;
                }
            } else {
                if pos + 4 > data.len() {
                    return Err(ParityError::XmParseError(format!(
                        "pattern {} uncompressed note truncated",
                        pattern_index
                    )));
                }
                cell.note = first;
                cell.instrument = data[pos];
                cell.volume = normalize_xm_volume(data[pos + 1]);
                cell.effect = data[pos + 2];
                cell.effect_param = data[pos + 3];
                pos += 4;
            }

            rows[row][channel] = cell;
        }
    }

    Ok(rows)
}

fn decode_it_patterns(
    it_data: &[u8],
    it_report: &ItValidationReport,
) -> Result<Vec<Vec<Vec<CanonCell>>>, ParityError> {
    let mut decoded = Vec::with_capacity(it_report.patterns.len());
    for pattern in &it_report.patterns {
        if pattern.offset == 0 {
            decoded.push(vec![
                vec![CanonCell::default(); 64];
                pattern.num_rows as usize
            ]);
            continue;
        }

        let offset = pattern.offset as usize;
        let data_start = offset + 8;
        let data_end = data_start + pattern.packed_length as usize;
        if data_end > it_data.len() {
            return Err(ParityError::ItParseError(format!(
                "pattern {} extends beyond IT data (end={}, len={})",
                pattern.index,
                data_end,
                it_data.len()
            )));
        }

        let rows = decode_it_pattern_data(
            &it_data[data_start..data_end],
            pattern.index,
            pattern.num_rows as usize,
        )?;
        decoded.push(rows);
    }
    Ok(decoded)
}

fn decode_it_pattern_data(
    packed_data: &[u8],
    pattern_index: usize,
    num_rows: usize,
) -> Result<Vec<Vec<CanonCell>>, ParityError> {
    let mut rows = vec![vec![CanonCell::default(); 64]; num_rows];
    let mut pos = 0usize;
    let mut row = 0usize;
    let mut channel_masks = [0u8; 64];
    let mut prev_note = [0u8; 64];
    let mut prev_inst = [0u8; 64];
    let mut prev_vol = [0u8; 64];
    let mut prev_fx = [0u8; 64];
    let mut prev_fx_param = [0u8; 64];

    while pos < packed_data.len() && row < num_rows {
        let channel_var = packed_data[pos];
        pos += 1;

        if channel_var == 0 {
            row += 1;
            continue;
        }

        let channel = ((channel_var - 1) & 63) as usize;
        let mask = if channel_var & 0x80 != 0 {
            if pos >= packed_data.len() {
                return Err(ParityError::ItParseError(format!(
                    "pattern {} truncated before mask byte",
                    pattern_index
                )));
            }
            let m = packed_data[pos];
            pos += 1;
            channel_masks[channel] = m;
            m
        } else {
            channel_masks[channel]
        };

        let mut cell = rows[row][channel];

        if mask & 0x01 != 0 {
            if pos >= packed_data.len() {
                return Err(ParityError::ItParseError(format!(
                    "pattern {} note field truncated",
                    pattern_index
                )));
            }
            let note = packed_data[pos];
            pos += 1;
            prev_note[channel] = note;
            cell.note = it_note_to_xm(note);
        } else if mask & 0x10 != 0 {
            cell.note = it_note_to_xm(prev_note[channel]);
        }

        if mask & 0x02 != 0 {
            if pos >= packed_data.len() {
                return Err(ParityError::ItParseError(format!(
                    "pattern {} instrument field truncated",
                    pattern_index
                )));
            }
            let inst = packed_data[pos];
            pos += 1;
            prev_inst[channel] = inst;
            cell.instrument = inst;
        } else if mask & 0x20 != 0 {
            cell.instrument = prev_inst[channel];
        }

        if mask & 0x04 != 0 {
            if pos >= packed_data.len() {
                return Err(ParityError::ItParseError(format!(
                    "pattern {} volume field truncated",
                    pattern_index
                )));
            }
            let vol = packed_data[pos];
            pos += 1;
            prev_vol[channel] = vol;
            cell.volume = if vol <= 64 { vol } else { 0 };
        } else if mask & 0x40 != 0 {
            let vol = prev_vol[channel];
            cell.volume = if vol <= 64 { vol } else { 0 };
        }

        if mask & 0x08 != 0 {
            if pos + 1 >= packed_data.len() {
                return Err(ParityError::ItParseError(format!(
                    "pattern {} effect field truncated",
                    pattern_index
                )));
            }
            let fx = packed_data[pos];
            let fx_param = packed_data[pos + 1];
            pos += 2;
            prev_fx[channel] = fx;
            prev_fx_param[channel] = fx_param;
            cell.effect = fx;
            cell.effect_param = fx_param;
        } else if mask & 0x80 != 0 {
            cell.effect = prev_fx[channel];
            cell.effect_param = prev_fx_param[channel];
        }

        rows[row][channel] = cell;
    }

    Ok(rows)
}

fn compare_cell_level_parity(
    xm_patterns: &[Vec<Vec<CanonCell>>],
    it_patterns: &[Vec<Vec<CanonCell>>],
    xm_orders: &[u8],
    it_orders: &[u8],
    xm_channels: usize,
    xm_restart: u8,
) -> Vec<ParityMismatch> {
    let mut mismatches = Vec::new();
    let max_orders = xm_orders.len().min(it_orders.len());

    let last_it_pattern_idx = it_orders.last().copied().unwrap_or(0);

    'order_loop: for order_idx in 0..max_orders {
        let xm_pattern_idx = xm_orders[order_idx] as usize;
        let it_pattern_idx = it_orders[order_idx] as usize;

        if xm_pattern_idx >= xm_patterns.len() || it_pattern_idx >= it_patterns.len() {
            continue;
        }

        let xm_rows = &xm_patterns[xm_pattern_idx];
        let it_rows = &it_patterns[it_pattern_idx];
        let max_rows = xm_rows.len().min(it_rows.len());
        for row in 0..max_rows {
            let max_channels = xm_channels.min(xm_rows[row].len()).min(it_rows[row].len());
            let is_last_row_in_pattern = row + 1 == max_rows;
            for channel in 0..max_channels {
                let xm_cell = xm_rows[row][channel];
                let it_cell = it_rows[row][channel];

                if xm_cell.note != it_cell.note {
                    mismatches.push(ParityMismatch {
                        category: "note_cell",
                        message: format!(
                            "order {} row {} ch {} note differs (XM={}, IT={})",
                            order_idx, row, channel, xm_cell.note, it_cell.note
                        ),
                    });
                }
                if xm_cell.instrument != it_cell.instrument {
                    mismatches.push(ParityMismatch {
                        category: "instrument_cell",
                        message: format!(
                            "order {} row {} ch {} instrument differs (XM={}, IT={})",
                            order_idx, row, channel, xm_cell.instrument, it_cell.instrument
                        ),
                    });
                }
                if xm_cell.volume != it_cell.volume {
                    mismatches.push(ParityMismatch {
                        category: "volume_cell",
                        message: format!(
                            "order {} row {} ch {} volume differs (XM={}, IT={})",
                            order_idx, row, channel, xm_cell.volume, it_cell.volume
                        ),
                    });
                }

                if let Some(effect_mismatch) = compare_effect_cell(
                    xm_cell,
                    it_cell,
                    order_idx,
                    row,
                    channel,
                    is_last_row_in_pattern,
                    it_orders[order_idx],
                    last_it_pattern_idx,
                    xm_restart,
                ) {
                    mismatches.push(effect_mismatch);
                }

                if mismatches.len() >= MAX_CELL_MISMATCHES {
                    mismatches.push(ParityMismatch {
                        category: "cell_mismatch_limit",
                        message: format!("stopped after {} cell mismatches", MAX_CELL_MISMATCHES),
                    });
                    break 'order_loop;
                }
            }
        }
    }

    mismatches
}

fn compare_effect_cell(
    xm_cell: CanonCell,
    it_cell: CanonCell,
    order_idx: usize,
    row: usize,
    channel: usize,
    is_last_row_in_pattern: bool,
    current_it_pattern: u8,
    last_it_pattern: u8,
    xm_restart: u8,
) -> Option<ParityMismatch> {
    let xm_none = xm_cell.effect == 0 && xm_cell.effect_param == 0;
    let it_none = it_cell.effect == 0 && it_cell.effect_param == 0;

    if xm_none && it_none {
        return None;
    }

    let xm_effect = decode_xm_effect(xm_cell.effect, xm_cell.effect_param);
    let it_effect = decode_it_effect(it_cell.effect, it_cell.effect_param);

    if xm_effect == it_effect {
        return None;
    }

    // IT loop semantics are emitted as a terminal PositionJump effect inside
    // the final ordered pattern. If that pattern is reused, this jump appears
    // on every occurrence of that pattern in the order list.
    if xm_none && is_last_row_in_pattern && current_it_pattern == last_it_pattern {
        if let Some(TrackerEffect::PositionJump { position }) = it_effect {
            if position == xm_restart {
                return None;
            }
        }
    }

    Some(ParityMismatch {
        category: "effect_cell",
        message: format!(
            "order {} row {} ch {} effect differs (XM={:02X}/{:02X} {:?}, IT={:02X}/{:02X} {:?})",
            order_idx,
            row,
            channel,
            xm_cell.effect,
            xm_cell.effect_param,
            xm_effect,
            it_cell.effect,
            it_cell.effect_param,
            it_effect
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::it::{ItInstrument, ItModule, ItNote, ItPattern, ItSample};
    use crate::xm::{XmInstrument, XmModule, XmNote, XmPattern, XmSample};

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

    fn create_single_note_xm(note: XmNote) -> Vec<u8> {
        let mut xm_module = XmModule::new("CellDetail", 4, 6, 125);
        let mut pattern = XmPattern::empty(64, 4);
        pattern.set_note(0, 0, note);
        xm_module.add_pattern(pattern);
        let sample = XmSample::new("Smp1", vec![0u8; 200], true);
        xm_module.add_instrument(XmInstrument::new("Inst1", sample));
        xm_module.set_order_table(&[0]);
        xm_module.to_bytes().unwrap()
    }

    fn create_single_note_it(note: ItNote) -> Vec<u8> {
        let mut it_module = ItModule::new("CellDetail", 4, 6, 125);
        let mut pattern = ItPattern::empty(64, 4);
        pattern.set_note(0, 0, note);
        it_module.add_pattern(pattern);
        it_module.add_instrument(ItInstrument::new("Inst1"));
        it_module.add_sample(ItSample::new("Smp1", vec![0u8; 100], 22050));
        it_module.set_orders(&[0]);
        it_module.to_bytes().unwrap()
    }

    #[test]
    fn test_parity_detailed_detects_note_cell_mismatch() {
        let xm = create_single_note_xm(XmNote::from_name("C4", 1, Some(64)));
        let it = create_single_note_it(ItNote::from_name("D4", 1, 64));

        let report = check_parity_detailed(&xm, &it).unwrap();
        assert!(!report.is_parity);
        assert!(report.mismatches.iter().any(|m| m.category == "note_cell"));
    }

    #[test]
    fn test_parity_detailed_accepts_matching_note_cells() {
        let xm = create_single_note_xm(XmNote::from_name("C4", 1, Some(64)));
        let it = create_single_note_it(ItNote::from_name("C4", 1, 64));

        let report = check_parity_detailed(&xm, &it).unwrap();
        assert!(report.is_parity, "{:?}", report.mismatches);
    }

    #[test]
    fn test_parity_detailed_accepts_semantically_equivalent_effects() {
        let xm_note = XmNote::from_name("C4", 1, Some(64)).with_effect(0x0, 0x37); // arpeggio
        let it_note = ItNote::from_name("C4", 1, 64).with_effect(10, 0x37); // arpeggio

        let xm = create_single_note_xm(xm_note);
        let it = create_single_note_it(it_note);
        let report = check_parity_detailed(&xm, &it).unwrap();
        assert!(report.is_parity, "{:?}", report.mismatches);
    }

    #[test]
    fn test_parity_detailed_detects_effect_cell_mismatch() {
        let xm_note = XmNote::from_name("C4", 1, Some(64)).with_effect(0x0, 0x37); // arpeggio
        let it_note = ItNote::from_name("C4", 1, 64).with_effect(8, 0x37); // vibrato

        let xm = create_single_note_xm(xm_note);
        let it = create_single_note_it(it_note);
        let report = check_parity_detailed(&xm, &it).unwrap();
        assert!(!report.is_parity);
        assert!(report
            .mismatches
            .iter()
            .any(|m| m.category == "effect_cell"));
    }
}
