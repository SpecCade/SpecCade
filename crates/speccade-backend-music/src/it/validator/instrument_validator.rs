//! Instrument validation logic.

use super::error::{ItErrorCategory, ItFormatError};
use super::helpers::extract_string;
use super::info::{ItEnvelopeInfo, ItInstrumentInfo};
use super::report::ItValidationReport;

pub(super) fn validate_instruments(
    data: &[u8],
    instrument_count: u16,
    offset_table_start: usize,
    report: &mut ItValidationReport,
) -> Result<(), ItFormatError> {
    if instrument_count == 0 {
        return Ok(());
    }

    let table_end = offset_table_start + (instrument_count as usize * 4);
    if table_end > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::OffsetTable,
            format!(
                "Instrument offset table extends beyond file: needs {} bytes at offset {}, file has {}",
                instrument_count * 4,
                offset_table_start,
                data.len()
            ),
            offset_table_start,
        ));
    }

    for i in 0..instrument_count as usize {
        let offset_pos = offset_table_start + i * 4;
        let instrument_offset = u32::from_le_bytes([
            data[offset_pos],
            data[offset_pos + 1],
            data[offset_pos + 2],
            data[offset_pos + 3],
        ]);

        if instrument_offset == 0 {
            // Null offset means no instrument data
            continue;
        }

        match validate_single_instrument(data, i + 1, instrument_offset, report) {
            Ok(info) => report.instruments.push(info),
            Err(e) => report.add_error(e),
        }
    }

    Ok(())
}

fn validate_single_instrument(
    data: &[u8],
    index: usize,
    offset: u32,
    report: &mut ItValidationReport,
) -> Result<ItInstrumentInfo, ItFormatError> {
    let offset = offset as usize;

    // Instrument header is 554 bytes (per ITTECH.TXT)
    if offset + 554 > data.len() {
        return Err(ItFormatError::at_offset(
            ItErrorCategory::Instrument,
            format!("Instrument {} header extends beyond file", index),
            offset,
        ));
    }

    let inst = &data[offset..];

    // Check magic "IMPI" at offset 0x00
    if &inst[0..4] != b"IMPI" {
        return Err(ItFormatError::field_at_offset(
            ItErrorCategory::Instrument,
            "magic",
            format!(
                "Instrument {} has invalid magic: expected 'IMPI', got {:02X?}",
                index,
                &inst[0..4]
            ),
            offset,
        ));
    }

    // DOS filename (12 bytes at 0x04)
    let filename = extract_string(&inst[0x04..0x10]);

    // Reserved byte at 0x10 (should be 0)

    // NNA at 0x11
    let nna = inst[0x11];
    if nna > 3 {
        report.add_warning(
            format!(
                "Instrument {} NNA value {} is invalid (should be 0-3)",
                index, nna
            ),
            Some(offset + 0x11),
        );
    }

    // DCT at 0x12
    let dct = inst[0x12];
    if dct > 3 {
        report.add_warning(
            format!(
                "Instrument {} DCT value {} is invalid (should be 0-3)",
                index, dct
            ),
            Some(offset + 0x12),
        );
    }

    // DCA at 0x13
    let dca = inst[0x13];
    if dca > 2 {
        report.add_warning(
            format!(
                "Instrument {} DCA value {} is invalid (should be 0-2)",
                index, dca
            ),
            Some(offset + 0x13),
        );
    }

    // Fadeout at 0x14 (2 bytes)
    let fadeout = u16::from_le_bytes([inst[0x14], inst[0x15]]);
    if fadeout > 1024 {
        report.add_warning(
            format!(
                "Instrument {} fadeout {} exceeds typical maximum of 1024",
                index, fadeout
            ),
            Some(offset + 0x14),
        );
    }

    // PPS (Pitch-Pan Separation) at 0x16
    let pps = inst[0x16] as i8;

    // PPC (Pitch-Pan Center) at 0x17
    let ppc = inst[0x17];
    if ppc > 119 {
        report.add_warning(
            format!(
                "Instrument {} PPC {} exceeds maximum note 119 (B-9)",
                index, ppc
            ),
            Some(offset + 0x17),
        );
    }

    // Global volume at 0x18
    let global_volume = inst[0x18];
    if global_volume > 128 {
        report.add_warning(
            format!(
                "Instrument {} global volume {} exceeds maximum of 128",
                index, global_volume
            ),
            Some(offset + 0x18),
        );
    }

    // Default pan at 0x19 (bit 7 = use pan)
    let dfp = inst[0x19];
    let default_pan = if dfp & 0x80 != 0 {
        Some(dfp & 0x7F)
    } else {
        None
    };

    // Random volume variation at 0x1A
    let random_volume = inst[0x1A];
    if random_volume > 100 {
        report.add_warning(
            format!(
                "Instrument {} random volume {} exceeds 100%",
                index, random_volume
            ),
            Some(offset + 0x1A),
        );
    }

    // Random panning variation at 0x1B
    let random_pan = inst[0x1B];
    if random_pan > 64 {
        report.add_warning(
            format!(
                "Instrument {} random pan {} exceeds maximum of 64",
                index, random_pan
            ),
            Some(offset + 0x1B),
        );
    }

    // Tracker version at 0x1C (2 bytes) - only for instrument files
    // Number of samples at 0x1E - only for instrument files
    // Reserved byte at 0x1F

    // Instrument name at 0x20 (26 bytes)
    let name = extract_string(&inst[0x20..0x3A]);

    // Initial filter cutoff at 0x3A (bit 7 = use filter)
    // Initial filter resonance at 0x3B
    // MIDI channel at 0x3C
    // MIDI program at 0x3D
    // MIDI bank at 0x3E (2 bytes)

    // Note-sample table at 0x40 (240 bytes = 120 pairs)
    // We skip detailed validation here but could check that sample indices are valid

    // Volume envelope starts at 0x130 (offset 304)
    let volume_envelope =
        parse_envelope(&inst[0x130..], "volume", index, offset + 0x130, report);

    // Panning envelope at 0x182 (offset 386)
    let panning_envelope =
        parse_envelope(&inst[0x182..], "panning", index, offset + 0x182, report);

    // Pitch envelope at 0x1D4 (offset 468)
    let pitch_envelope =
        parse_envelope(&inst[0x1D4..], "pitch", index, offset + 0x1D4, report);

    Ok(ItInstrumentInfo {
        index,
        offset: offset as u32,
        name,
        filename,
        nna,
        dct,
        dca,
        fadeout,
        pps,
        ppc,
        global_volume,
        default_pan,
        random_volume,
        random_pan,
        volume_envelope,
        panning_envelope,
        pitch_envelope,
    })
}

fn parse_envelope(
    data: &[u8],
    env_name: &str,
    inst_index: usize,
    base_offset: usize,
    report: &mut ItValidationReport,
) -> ItEnvelopeInfo {
    // Envelope structure: 82 bytes
    // Flags at offset 0
    let flags = data[0];
    let enabled = flags & 0x01 != 0;
    let loop_enabled = flags & 0x02 != 0;
    let sustain_loop_enabled = flags & 0x04 != 0;
    let carry = flags & 0x08 != 0;
    let is_filter = flags & 0x80 != 0; // For pitch envelope

    // Number of nodes at offset 1
    let num_nodes = data[1];
    if num_nodes > 25 {
        report.add_warning(
            format!(
                "Instrument {} {} envelope has {} nodes (maximum is 25)",
                inst_index, env_name, num_nodes
            ),
            Some(base_offset + 1),
        );
    }

    // Loop begin at offset 2
    let loop_begin = data[2];

    // Loop end at offset 3
    let loop_end = data[3];

    // Sustain begin at offset 4
    let sustain_begin = data[4];

    // Sustain end at offset 5
    let sustain_end = data[5];

    // Validate loop points
    if loop_enabled && loop_begin > loop_end {
        report.add_warning(
            format!(
                "Instrument {} {} envelope loop begin ({}) > end ({})",
                inst_index, env_name, loop_begin, loop_end
            ),
            Some(base_offset + 2),
        );
    }

    if sustain_loop_enabled && sustain_begin > sustain_end {
        report.add_warning(
            format!(
                "Instrument {} {} envelope sustain begin ({}) > end ({})",
                inst_index, env_name, sustain_begin, sustain_end
            ),
            Some(base_offset + 4),
        );
    }

    ItEnvelopeInfo {
        enabled,
        loop_enabled,
        sustain_loop_enabled,
        carry,
        is_filter,
        num_nodes,
        loop_begin,
        loop_end,
        sustain_begin,
        sustain_end,
    }
}
