//! Unit tests for tracker effects.

use super::*;

#[test]
fn test_arpeggio_to_xm() {
    let effect = TrackerEffect::Arpeggio { x: 3, y: 7 };
    assert_eq!(effect.to_xm(), Some((0x0, 0x37)));
}

#[test]
fn test_arpeggio_to_it() {
    let effect = TrackerEffect::Arpeggio { x: 3, y: 7 };
    assert_eq!(effect.to_it(), Some((10, 0x37)));
}

#[test]
fn test_vibrato_to_xm() {
    let effect = TrackerEffect::Vibrato { speed: 4, depth: 8 };
    assert_eq!(effect.to_xm(), Some((0x4, 0x48)));
}

#[test]
fn test_set_tempo_to_xm() {
    let effect = TrackerEffect::SetTempo { bpm: 140 };
    assert_eq!(effect.to_xm(), Some((0xF, 140)));
}

#[test]
fn test_set_speed_to_it() {
    let effect = TrackerEffect::SetSpeed { speed: 6 };
    assert_eq!(effect.to_it(), Some((1, 6)));
}

#[test]
fn test_retrigger_to_xm() {
    let effect = TrackerEffect::Retrigger {
        volume_change: 0,
        interval: 3,
    };
    assert_eq!(effect.to_xm(), Some((0x1B, 0x03)));
}

#[test]
fn test_note_delay_to_xm() {
    let effect = TrackerEffect::NoteDelay { ticks: 4 };
    assert_eq!(effect.to_xm(), Some((0xE, 0xD4)));
}

#[test]
fn test_pattern_break_to_it() {
    let effect = TrackerEffect::PatternBreak { row: 32 };
    assert_eq!(effect.to_it(), Some((3, 32)));
}

#[test]
fn test_fine_porta_to_it() {
    let effect = TrackerEffect::FinePortamentoUp { speed: 4 };
    assert_eq!(effect.to_it(), Some((6, 0xF4)));
}

#[test]
fn test_validate_xm_arpeggio_valid() {
    let effect = TrackerEffect::Arpeggio { x: 15, y: 15 };
    assert!(effect.validate_xm().is_ok());
}

#[test]
fn test_validate_xm_arpeggio_invalid() {
    let effect = TrackerEffect::Arpeggio { x: 16, y: 0 };
    assert!(matches!(
        effect.validate_xm(),
        Err(EffectValidationError::OutOfRange { .. })
    ));
}

#[test]
fn test_validate_xm_speed_valid() {
    let effect = TrackerEffect::SetSpeed { speed: 6 };
    assert!(effect.validate_xm().is_ok());
}

#[test]
fn test_validate_xm_speed_invalid_zero() {
    let effect = TrackerEffect::SetSpeed { speed: 0 };
    assert!(matches!(
        effect.validate_xm(),
        Err(EffectValidationError::OutOfRange { .. })
    ));
}

#[test]
fn test_validate_xm_speed_invalid_high() {
    let effect = TrackerEffect::SetSpeed { speed: 32 };
    assert!(matches!(
        effect.validate_xm(),
        Err(EffectValidationError::OutOfRange { .. })
    ));
}

#[test]
fn test_validate_xm_tempo_valid() {
    let effect = TrackerEffect::SetTempo { bpm: 140 };
    assert!(effect.validate_xm().is_ok());
}

#[test]
fn test_validate_xm_tempo_invalid() {
    let effect = TrackerEffect::SetTempo { bpm: 31 };
    assert!(matches!(
        effect.validate_xm(),
        Err(EffectValidationError::OutOfRange { .. })
    ));
}

#[test]
fn test_validate_xm_it_only_effect() {
    let effect = TrackerEffect::SetChannelVolume { volume: 64 };
    assert!(matches!(
        effect.validate_xm(),
        Err(EffectValidationError::UnsupportedFormat { .. })
    ));
}

#[test]
fn test_validate_it_channel_volume() {
    let effect = TrackerEffect::SetChannelVolume { volume: 64 };
    assert!(effect.validate_it().is_ok());
}

#[test]
fn test_validate_it_channel_volume_invalid() {
    let effect = TrackerEffect::SetChannelVolume { volume: 65 };
    assert!(matches!(
        effect.validate_it(),
        Err(EffectValidationError::OutOfRange { .. })
    ));
}

#[test]
fn test_validate_it_xm_only_effect() {
    let effect = TrackerEffect::KeyOff { tick: 0 };
    assert!(matches!(
        effect.validate_it(),
        Err(EffectValidationError::UnsupportedFormat { .. })
    ));
}

#[test]
fn test_parse_effect_name_vibrato() {
    let effect = parse_effect_name("vibrato", Some(0x48), None);
    assert_eq!(effect, Some(TrackerEffect::Vibrato { speed: 4, depth: 8 }));
}

#[test]
fn test_parse_effect_name_arpeggio_xy() {
    let effect = parse_effect_name("arpeggio", None, Some([3, 7]));
    assert_eq!(effect, Some(TrackerEffect::Arpeggio { x: 3, y: 7 }));
}

#[test]
fn test_parse_effect_name_retrigger() {
    let effect = parse_effect_name("retrigger", Some(0x03), None);
    assert_eq!(
        effect,
        Some(TrackerEffect::Retrigger {
            volume_change: 0,
            interval: 3
        })
    );
}

#[test]
fn test_parse_effect_name_unknown() {
    let effect = parse_effect_name("unknown_effect", None, None);
    assert_eq!(effect, None);
}

#[test]
fn test_serialization_roundtrip() {
    let effect = TrackerEffect::Vibrato { speed: 4, depth: 8 };
    let json = serde_json::to_string(&effect).unwrap();
    let parsed: TrackerEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, parsed);
}

#[test]
fn test_serialization_retrigger() {
    let effect = TrackerEffect::Retrigger {
        volume_change: 5,
        interval: 3,
    };
    let json = serde_json::to_string(&effect).unwrap();
    assert!(json.contains("retrigger"));
    let parsed: TrackerEffect = serde_json::from_str(&json).unwrap();
    assert_eq!(effect, parsed);
}
