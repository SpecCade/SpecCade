//! IT automation and options handling.

use speccade_spec::recipe::music::{AutomationEntry, ItOptions};

use crate::generate::GenerateError;
use crate::it::{ItModule, ItNote, ItPattern};

/// Apply IT-specific options to the module header.
pub fn apply_it_options(module: &mut ItModule, options: &ItOptions) {
    // Set stereo flag
    if options.stereo {
        module.header.flags |= crate::it::flags::STEREO;
    } else {
        module.header.flags &= !crate::it::flags::STEREO;
    }

    // Set global volume
    module.header.global_volume = options.global_volume.min(128);

    // Set mix volume
    module.header.mix_volume = options.mix_volume.min(128);
}

/// Apply automation entries to an IT pattern.
pub fn apply_automation_to_pattern(
    pattern: &mut ItPattern,
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
                    apply_volume_fade_it(
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
                    apply_tempo_change_it(pattern, *row, *bpm)?;
                }
            }
        }
    }
    Ok(())
}

/// Apply volume fade automation to an IT pattern.
pub(super) fn apply_volume_fade_it(
    pattern: &mut ItPattern,
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
    if channel as usize >= pattern.notes.first().map_or(0, Vec::len) {
        return Err(GenerateError::AutomationError(format!(
            "channel {} out of range for pattern with {} channel(s)",
            channel,
            pattern.notes.first().map_or(0, Vec::len)
        )));
    }
    if end_row >= pattern.num_rows {
        return Err(GenerateError::AutomationError(format!(
            "row range {}..={} out of range for pattern with {} row(s)",
            start_row, end_row, pattern.num_rows
        )));
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
            .unwrap_or_else(ItNote::empty);
        note.volume = volume;
        pattern.set_note(row, channel, note);
    }

    Ok(())
}

/// Apply tempo change automation to an IT pattern.
pub(super) fn apply_tempo_change_it(
    pattern: &mut ItPattern,
    row: u16,
    bpm: u8,
) -> Result<(), GenerateError> {
    if bpm < 32 {
        return Err(GenerateError::AutomationError(format!(
            "BPM {} is too low (min 32)",
            bpm
        )));
    }
    if row >= pattern.num_rows {
        return Err(GenerateError::AutomationError(format!(
            "tempo change row {} out of range for pattern with {} row(s)",
            row, pattern.num_rows
        )));
    }

    // IT effect T is tempo set (0x14)
    let effect_code = 0x14; // 'T' command
    let mut note = pattern
        .get_note(row, 0)
        .copied()
        .unwrap_or_else(ItNote::empty);
    note.effect = effect_code;
    note.effect_param = bpm;
    pattern.set_note(row, 0, note);

    Ok(())
}
