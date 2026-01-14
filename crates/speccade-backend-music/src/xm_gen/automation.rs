//! XM automation application functions.
//!
//! Handles applying automation entries like volume fades and tempo changes
//! to XM patterns.

use speccade_spec::recipe::music::AutomationEntry;

use crate::generate::GenerateError;
use crate::xm::{XmNote, XmPattern};

/// Apply automation entries to an XM pattern.
pub fn apply_automation_to_xm_pattern(
    pattern: &mut XmPattern,
    pattern_name: &str,
    automation: &[AutomationEntry],
    _num_channels: u8,
) -> Result<(), GenerateError> {
    for auto in automation {
        match auto {
            AutomationEntry::VolumeFade {
                pattern: target,
                channel,
                start_row,
                end_row,
                start_vol,
                end_vol,
            } => {
                if target == pattern_name {
                    apply_volume_fade_xm(
                        pattern, *channel, *start_row, *end_row, *start_vol, *end_vol,
                    )?;
                }
            }
            AutomationEntry::TempoChange {
                pattern: target,
                row,
                bpm,
            } => {
                if target == pattern_name {
                    apply_tempo_change_xm(pattern, *row, *bpm)?;
                }
            }
        }
    }
    Ok(())
}

/// Apply volume fade automation to an XM pattern.
pub fn apply_volume_fade_xm(
    pattern: &mut XmPattern,
    channel: u8,
    start_row: u16,
    end_row: u16,
    start_vol: u8,
    end_vol: u8,
) -> Result<(), GenerateError> {
    if start_row >= end_row {
        return Err(GenerateError::AutomationError(
            "start_row must be less than end_row".to_string(),
        ));
    }

    let num_steps = (end_row - start_row) as f64;
    let vol_diff = end_vol as f64 - start_vol as f64;

    for row in start_row..=end_row {
        let progress = (row - start_row) as f64 / num_steps;
        let volume = (start_vol as f64 + vol_diff * progress).round() as u8;
        let volume = volume.min(64);

        // Get existing note or create volume command
        let mut note = pattern
            .get_note(row, channel)
            .copied()
            .unwrap_or_else(XmNote::empty);
        // XM volume column: 0x10-0x50 for volume 0-64
        note.volume = 0x10 + volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an XM pattern.
pub fn apply_tempo_change_xm(pattern: &mut XmPattern, row: u16, bpm: u8) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }

    // XM effect F is tempo/BPM set
    let effect_code = 0x0F;
    let mut note = pattern
        .get_note(row, 0)
        .copied()
        .unwrap_or_else(XmNote::empty);
    note = note.with_effect(effect_code, bpm);
    pattern.set_note(row, 0, note);

    Ok(())
}
