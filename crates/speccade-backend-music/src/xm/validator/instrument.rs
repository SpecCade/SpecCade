//! XM instrument and sample validation.

use super::constants::*;
use super::error::{XmFormatError, XmWarning};
use super::types::{XmEnvelopeInfo, XmHeaderInfo, XmInstrumentInfo, XmSampleInfo, XmValidationReport};
use super::header::extract_string;
use crate::xm::instrument::XM_SAMPLE_HEADER_SIZE;

/// Validate all instruments.
pub(super) fn validate_instruments(
    data: &[u8],
    start_offset: usize,
    header: &XmHeaderInfo,
    report: &mut XmValidationReport,
) -> Result<(), XmFormatError> {
    let mut offset = start_offset;

    for inst_idx in 0..header.num_instruments as usize {
        if offset + 4 > data.len() {
            return Err(XmFormatError::FileTruncated {
                expected: offset + 4,
                actual: data.len(),
            });
        }

        // Instrument header size
        let inst_size = u32::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ]);

        if offset + inst_size as usize > data.len() {
            return Err(XmFormatError::FileTruncated {
                expected: offset + inst_size as usize,
                actual: data.len(),
            });
        }

        // Parse instrument header
        let inst_info = parse_instrument_header(data, offset, inst_idx, report)?;

        // Store sample data sizes for offset calculation
        let mut total_sample_data_size: usize = 0;

        // Validate samples
        if inst_info.num_samples > 0 {
            let sample_headers_offset = offset + inst_size as usize;
            let sample_header_size = inst_info.sample_header_size;

            for sample_idx in 0..inst_info.num_samples as usize {
                let sample_offset =
                    sample_headers_offset + sample_idx * sample_header_size as usize;

                if sample_offset + sample_header_size as usize > data.len() {
                    return Err(XmFormatError::FileTruncated {
                        expected: sample_offset + sample_header_size as usize,
                        actual: data.len(),
                    });
                }

                let sample_info = parse_sample_header(
                    data,
                    sample_offset,
                    inst_idx,
                    sample_idx,
                    report,
                )?;
                total_sample_data_size += sample_info.length as usize;

                // Create a mutable copy of inst_info for samples
                // Note: We'll add samples in a collected manner below
            }

            // Skip sample data
            let sample_data_offset = sample_headers_offset
                + inst_info.num_samples as usize * sample_header_size as usize;
            offset = sample_data_offset + total_sample_data_size;
        } else {
            report.add_warning(XmWarning::InstrumentWithoutSamples {
                instrument_index: inst_idx,
            });
            offset += inst_size as usize;
        }

        report.instruments.push(inst_info);
    }

    report.expected_size = offset;
    Ok(())
}

/// Parse instrument header and return instrument info.
fn parse_instrument_header(
    data: &[u8],
    offset: usize,
    inst_idx: usize,
    report: &mut XmValidationReport,
) -> Result<XmInstrumentInfo, XmFormatError> {
    let inst_size = u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);

    // Instrument name (22 bytes at offset +4)
    let name = if offset + 26 <= data.len() {
        extract_string(&data[offset + 4..offset + 26])
    } else {
        String::new()
    };

    // Instrument type at offset +26 (should be 0)
    let instrument_type = if offset + 27 <= data.len() {
        data[offset + 26]
    } else {
        0
    };

    // Number of samples at offset +27
    let num_samples = if offset + 29 <= data.len() {
        u16::from_le_bytes([data[offset + 27], data[offset + 28]])
    } else {
        0
    };

    // Sample header size at offset +29 (only if num_samples > 0)
    let sample_header_size = if num_samples > 0 && offset + 33 <= data.len() {
        u32::from_le_bytes([
            data[offset + 29],
            data[offset + 30],
            data[offset + 31],
            data[offset + 32],
        ])
    } else {
        XM_SAMPLE_HEADER_SIZE
    };

    // Parse envelopes and other data only if we have samples
    let (volume_envelope, panning_envelope) = if num_samples > 0 && inst_size >= 243 {
        parse_envelopes(data, offset, inst_idx, report)?
    } else {
        (
            XmEnvelopeInfo {
                envelope_type: "volume",
                num_points: 0,
                sustain_point: 0,
                loop_start: 0,
                loop_end: 0,
                enabled: false,
                sustain_enabled: false,
                loop_enabled: false,
                points: Vec::new(),
            },
            XmEnvelopeInfo {
                envelope_type: "panning",
                num_points: 0,
                sustain_point: 0,
                loop_start: 0,
                loop_end: 0,
                enabled: false,
                sustain_enabled: false,
                loop_enabled: false,
                points: Vec::new(),
            },
        )
    };

    // Vibrato parameters
    let vibrato_type = if num_samples > 0 && offset + 236 <= data.len() {
        data[offset + 235]
    } else {
        0
    };
    let vibrato_sweep = if num_samples > 0 && offset + 237 <= data.len() {
        data[offset + 236]
    } else {
        0
    };
    let vibrato_depth = if num_samples > 0 && offset + 238 <= data.len() {
        data[offset + 237]
    } else {
        0
    };
    let vibrato_rate = if num_samples > 0 && offset + 239 <= data.len() {
        data[offset + 238]
    } else {
        0
    };

    // Volume fadeout
    let volume_fadeout = if num_samples > 0 && offset + 241 <= data.len() {
        u16::from_le_bytes([data[offset + 239], data[offset + 240]])
    } else {
        0
    };

    Ok(XmInstrumentInfo {
        index: inst_idx,
        name,
        instrument_type,
        num_samples,
        sample_header_size,
        volume_envelope,
        panning_envelope,
        vibrato_type,
        vibrato_sweep,
        vibrato_depth,
        vibrato_rate,
        volume_fadeout,
        samples: Vec::new(), // Filled later
    })
}

/// Parse volume and panning envelopes.
fn parse_envelopes(
    data: &[u8],
    offset: usize,
    inst_idx: usize,
    report: &mut XmValidationReport,
) -> Result<(XmEnvelopeInfo, XmEnvelopeInfo), XmFormatError> {
    // Volume envelope points: offset + 33, 48 bytes (12 points * 4 bytes)
    let vol_points_offset = offset + 33;
    // Panning envelope points: offset + 81 (0x51), 48 bytes
    let pan_points_offset = offset + 129;

    // Number of envelope points
    let num_vol_points = data[offset + 225];
    let num_pan_points = data[offset + 226];

    // Validate point counts
    if num_vol_points > XM_MAX_ENVELOPE_POINTS {
        return Err(XmFormatError::EnvelopeError {
            instrument: inst_idx,
            envelope_type: "volume",
            message: format!(
                "Too many points: {} (max {})",
                num_vol_points, XM_MAX_ENVELOPE_POINTS
            ),
        });
    }
    if num_pan_points > XM_MAX_ENVELOPE_POINTS {
        return Err(XmFormatError::EnvelopeError {
            instrument: inst_idx,
            envelope_type: "panning",
            message: format!(
                "Too many points: {} (max {})",
                num_pan_points, XM_MAX_ENVELOPE_POINTS
            ),
        });
    }

    // Sustain/loop points
    let vol_sustain = data[offset + 227];
    let vol_loop_start = data[offset + 228];
    let vol_loop_end = data[offset + 229];
    let pan_sustain = data[offset + 230];
    let pan_loop_start = data[offset + 231];
    let pan_loop_end = data[offset + 232];

    // Envelope flags
    let vol_flags = data[offset + 233];
    let pan_flags = data[offset + 234];

    let vol_enabled = (vol_flags & 1) != 0;
    let vol_sustain_enabled = (vol_flags & 2) != 0;
    let vol_loop_enabled = (vol_flags & 4) != 0;

    let pan_enabled = (pan_flags & 1) != 0;
    let pan_sustain_enabled = (pan_flags & 2) != 0;
    let pan_loop_enabled = (pan_flags & 4) != 0;

    // Warn if envelope is enabled but has no points
    if vol_enabled && num_vol_points == 0 {
        report.add_warning(XmWarning::EmptyEnabledEnvelope {
            instrument: inst_idx,
            envelope_type: "volume",
        });
    }
    if pan_enabled && num_pan_points == 0 {
        report.add_warning(XmWarning::EmptyEnabledEnvelope {
            instrument: inst_idx,
            envelope_type: "panning",
        });
    }

    // Parse envelope points
    let mut vol_points = Vec::new();
    for i in 0..num_vol_points as usize {
        let pt_offset = vol_points_offset + i * 4;
        if pt_offset + 4 <= data.len() {
            let frame = u16::from_le_bytes([data[pt_offset], data[pt_offset + 1]]);
            let value = u16::from_le_bytes([data[pt_offset + 2], data[pt_offset + 3]]);

            // Validate value range
            if value > 64 {
                report.add_error(XmFormatError::EnvelopeError {
                    instrument: inst_idx,
                    envelope_type: "volume",
                    message: format!("Point {} value {} exceeds maximum 64", i, value),
                });
            }

            // Warn about high frame values
            if frame > 512 {
                report.add_warning(XmWarning::HighEnvelopeFrameValue {
                    instrument: inst_idx,
                    envelope_type: "volume",
                    frame,
                });
            }

            vol_points.push((frame, value));
        }
    }

    let mut pan_points = Vec::new();
    for i in 0..num_pan_points as usize {
        let pt_offset = pan_points_offset + i * 4;
        if pt_offset + 4 <= data.len() {
            let frame = u16::from_le_bytes([data[pt_offset], data[pt_offset + 1]]);
            let value = u16::from_le_bytes([data[pt_offset + 2], data[pt_offset + 3]]);

            // Panning envelope values should be 0-64 centered at 32
            if value > 64 {
                report.add_error(XmFormatError::EnvelopeError {
                    instrument: inst_idx,
                    envelope_type: "panning",
                    message: format!("Point {} value {} exceeds maximum 64", i, value),
                });
            }

            pan_points.push((frame, value));
        }
    }

    // Validate sustain and loop point indices
    if vol_sustain_enabled && num_vol_points > 0 && vol_sustain >= num_vol_points {
        report.add_error(XmFormatError::EnvelopeError {
            instrument: inst_idx,
            envelope_type: "volume",
            message: format!(
                "Sustain point {} exceeds point count {}",
                vol_sustain, num_vol_points
            ),
        });
    }
    if vol_loop_enabled
        && num_vol_points > 0
        && (vol_loop_start >= num_vol_points
            || vol_loop_end >= num_vol_points
            || vol_loop_start > vol_loop_end)
    {
        report.add_error(XmFormatError::EnvelopeError {
            instrument: inst_idx,
            envelope_type: "volume",
            message: format!(
                "Invalid loop points: start={}, end={}, points={}",
                vol_loop_start, vol_loop_end, num_vol_points
            ),
        });
    }

    Ok((
        XmEnvelopeInfo {
            envelope_type: "volume",
            num_points: num_vol_points,
            sustain_point: vol_sustain,
            loop_start: vol_loop_start,
            loop_end: vol_loop_end,
            enabled: vol_enabled,
            sustain_enabled: vol_sustain_enabled,
            loop_enabled: vol_loop_enabled,
            points: vol_points,
        },
        XmEnvelopeInfo {
            envelope_type: "panning",
            num_points: num_pan_points,
            sustain_point: pan_sustain,
            loop_start: pan_loop_start,
            loop_end: pan_loop_end,
            enabled: pan_enabled,
            sustain_enabled: pan_sustain_enabled,
            loop_enabled: pan_loop_enabled,
            points: pan_points,
        },
    ))
}

/// Parse sample header.
fn parse_sample_header(
    data: &[u8],
    offset: usize,
    inst_idx: usize,
    sample_idx: usize,
    report: &mut XmValidationReport,
) -> Result<XmSampleInfo, XmFormatError> {
    // Sample length (4 bytes)
    let length = u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]);

    // Loop start (4 bytes)
    let loop_start = u32::from_le_bytes([
        data[offset + 4],
        data[offset + 5],
        data[offset + 6],
        data[offset + 7],
    ]);

    // Loop length (4 bytes)
    let loop_length = u32::from_le_bytes([
        data[offset + 8],
        data[offset + 9],
        data[offset + 10],
        data[offset + 11],
    ]);

    // Volume (1 byte, 0-64)
    let volume = data[offset + 12];
    if volume > XM_MAX_VOLUME {
        report.add_error(XmFormatError::InvalidVolume {
            value: volume,
            context: format!("instrument {} sample {}", inst_idx, sample_idx),
        });
    }

    // Finetune (1 signed byte)
    let finetune = data[offset + 13] as i8;

    // Type/flags (1 byte)
    let flags = data[offset + 14];
    let loop_type = flags & 0x03;
    let is_16bit = (flags & 0x10) != 0;

    // Validate loop type
    if loop_type > 2 {
        return Err(XmFormatError::InvalidLoopType {
            loop_type,
            instrument: inst_idx,
            sample: sample_idx,
        });
    }

    // Validate loop bounds
    if loop_type != 0 && length > 0 && loop_start + loop_length > length {
        return Err(XmFormatError::InvalidLoopBounds {
            instrument: inst_idx,
            sample: sample_idx,
            loop_start,
            loop_length,
            sample_length: length,
        });
    }

    // Panning (1 byte, 0-255)
    let panning = data[offset + 15];

    // Relative note (1 signed byte)
    let relative_note = data[offset + 16] as i8;

    // Reserved byte at offset + 17

    // Sample name (22 bytes at offset + 18)
    let name = if offset + 40 <= data.len() {
        extract_string(&data[offset + 18..offset + 40])
    } else {
        String::new()
    };

    Ok(XmSampleInfo {
        index: sample_idx,
        name,
        length,
        loop_start,
        loop_length,
        volume,
        finetune,
        flags,
        is_16bit,
        loop_type,
        panning,
        relative_note,
    })
}
