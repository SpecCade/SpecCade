//! Effect conversion to XM and IT format codes.

use super::{it_codes, xm_codes, TrackerEffect};

impl TrackerEffect {
    /// Convert to XM effect code and parameter.
    pub fn to_xm(&self) -> Option<(u8, u8)> {
        use xm_codes::*;
        match self {
            Self::Arpeggio { x, y } => Some((ARPEGGIO, (x << 4) | (y & 0x0F))),
            Self::PortamentoUp { speed } => Some((PORTA_UP, *speed)),
            Self::PortamentoDown { speed } => Some((PORTA_DOWN, *speed)),
            Self::FinePortamentoUp { speed } => Some((EXTENDED, 0x10 | (speed & 0x0F))),
            Self::FinePortamentoDown { speed } => Some((EXTENDED, 0x20 | (speed & 0x0F))),
            Self::ExtraFinePortaUp { speed } => Some((EXTRA_FINE_PORTA, 0x10 | (speed & 0x0F))),
            Self::ExtraFinePortaDown { speed } => Some((EXTRA_FINE_PORTA, 0x20 | (speed & 0x0F))),
            Self::TonePortamento { speed } => Some((TONE_PORTA, *speed)),
            Self::Vibrato { speed, depth } => Some((VIBRATO, (speed << 4) | (depth & 0x0F))),
            Self::Tremolo { speed, depth } => Some((TREMOLO, (speed << 4) | (depth & 0x0F))),
            Self::VolumeSlide { up, down } => Some((VOL_SLIDE, (up << 4) | (down & 0x0F))),
            Self::SetVolume { volume } => Some((SET_VOLUME, *volume)),
            Self::SetPanning { pan } => Some((SET_PANNING, *pan)),
            Self::SampleOffset { offset } => Some((SAMPLE_OFFSET, *offset)),
            Self::PositionJump { position } => Some((POSITION_JUMP, *position)),
            Self::PatternBreak { row } => Some((PATTERN_BREAK, *row)),
            Self::SetSpeed { speed } => Some((SET_SPEED_TEMPO, *speed)),
            Self::SetTempo { bpm } => Some((SET_SPEED_TEMPO, *bpm)),
            Self::Retrigger {
                volume_change,
                interval,
            } => Some((RETRIGGER, (volume_change << 4) | (interval & 0x0F))),
            Self::NoteDelay { ticks } => Some((EXTENDED, 0xD0 | (ticks & 0x0F))),
            Self::NoteCut { ticks } => Some((EXTENDED, 0xC0 | (ticks & 0x0F))),
            Self::Tremor { on_time, off_time } => {
                Some((TREMOR, (on_time << 4) | (off_time & 0x0F)))
            }
            Self::SetGlobalVolume { volume } => Some((GLOBAL_VOL, *volume)),
            Self::GlobalVolumeSlide { up, down } => {
                Some((GLOBAL_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::PanningSlide { left, right } => Some((PAN_SLIDE, (right << 4) | (left & 0x0F))),
            Self::SetVibratoWaveform { waveform } => Some((EXTENDED, 0x40 | (waveform & 0x0F))),
            Self::SetTremoloWaveform { waveform } => Some((EXTENDED, 0x70 | (waveform & 0x0F))),
            Self::SetFinetune { value } => Some((EXTENDED, 0x50 | (value & 0x0F))),
            Self::PatternLoop { count } => Some((EXTENDED, 0x60 | (count & 0x0F))),
            Self::TonePortaVolumeSlide { up, down } => {
                Some((TONE_PORTA_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::VibratoVolumeSlide { up, down } => {
                Some((VIBRATO_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::KeyOff { tick } => Some((KEY_OFF, *tick)),
            Self::SetEnvelopePosition { position } => Some((SET_ENV_POS, *position)),
            Self::Raw { code, param } => Some((*code, *param)),
            // IT-only effects
            Self::SetChannelVolume { .. }
            | Self::ChannelVolumeSlide { .. }
            | Self::FineVibrato { .. }
            | Self::Panbrello { .. } => None,
        }
    }

    /// Convert to IT effect code and parameter.
    pub fn to_it(&self) -> Option<(u8, u8)> {
        use it_codes::*;
        match self {
            Self::Arpeggio { x, y } => Some((ARPEGGIO, (x << 4) | (y & 0x0F))),
            Self::PortamentoUp { speed } => Some((PORTA_UP, *speed)),
            Self::PortamentoDown { speed } => Some((PORTA_DOWN, *speed)),
            Self::FinePortamentoUp { speed } => Some((PORTA_UP, 0xF0 | (speed & 0x0F))),
            Self::FinePortamentoDown { speed } => Some((PORTA_DOWN, 0xF0 | (speed & 0x0F))),
            Self::ExtraFinePortaUp { speed } => Some((PORTA_UP, 0xE0 | (speed & 0x0F))),
            Self::ExtraFinePortaDown { speed } => Some((PORTA_DOWN, 0xE0 | (speed & 0x0F))),
            Self::TonePortamento { speed } => Some((TONE_PORTA, *speed)),
            Self::Vibrato { speed, depth } => Some((VIBRATO, (speed << 4) | (depth & 0x0F))),
            Self::Tremolo { speed, depth } => Some((TREMOLO, (speed << 4) | (depth & 0x0F))),
            Self::VolumeSlide { up, down } => Some((VOLUME_SLIDE, (up << 4) | (down & 0x0F))),
            Self::SetVolume { .. } => None, // IT uses volume column
            Self::SetPanning { pan } => Some((SET_PANNING, *pan)),
            Self::SampleOffset { offset } => Some((SAMPLE_OFFSET, *offset)),
            Self::PositionJump { position } => Some((POSITION_JUMP, *position)),
            Self::PatternBreak { row } => Some((PATTERN_BREAK, *row)),
            Self::SetSpeed { speed } => Some((SET_SPEED, *speed)),
            Self::SetTempo { bpm } => Some((TEMPO, *bpm)),
            Self::Retrigger {
                volume_change,
                interval,
            } => Some((RETRIGGER, (volume_change << 4) | (interval & 0x0F))),
            Self::NoteDelay { ticks } => Some((EXTENDED, 0xD0 | (ticks & 0x0F))),
            Self::NoteCut { ticks } => Some((EXTENDED, 0xC0 | (ticks & 0x0F))),
            Self::Tremor { on_time, off_time } => {
                Some((TREMOR, (on_time << 4) | (off_time & 0x0F)))
            }
            Self::SetGlobalVolume { volume } => Some((SET_GLOBAL_VOL, *volume)),
            Self::GlobalVolumeSlide { up, down } => {
                Some((GLOBAL_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::PanningSlide { left, right } => {
                Some((PANNING_SLIDE, (left << 4) | (right & 0x0F)))
            }
            Self::SetChannelVolume { volume } => Some((SET_CHANNEL_VOL, *volume)),
            Self::ChannelVolumeSlide { up, down } => {
                Some((CHANNEL_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::FineVibrato { speed, depth } => {
                Some((FINE_VIBRATO, (speed << 4) | (depth & 0x0F)))
            }
            Self::SetVibratoWaveform { waveform } => Some((EXTENDED, 0x30 | (waveform & 0x0F))),
            Self::SetTremoloWaveform { waveform } => Some((EXTENDED, 0x40 | (waveform & 0x0F))),
            Self::SetFinetune { value } => Some((EXTENDED, 0x20 | (value & 0x0F))),
            Self::PatternLoop { count } => Some((EXTENDED, 0xB0 | (count & 0x0F))),
            Self::TonePortaVolumeSlide { up, down } => {
                Some((TONE_PORTA_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::VibratoVolumeSlide { up, down } => {
                Some((VIBRATO_VOL_SLIDE, (up << 4) | (down & 0x0F)))
            }
            Self::Panbrello { speed, depth } => Some((PANBRELLO, (speed << 4) | (depth & 0x0F))),
            Self::Raw { code, param } => Some((*code, *param)),
            // XM-only effects that don't map to IT
            Self::KeyOff { .. } | Self::SetEnvelopePosition { .. } => None,
        }
    }
}

/// Parse an effect name string to a TrackerEffect.
///
/// Supports both simple names (e.g., "vibrato") and parameterized forms.
/// Returns None if the effect name is not recognized.
pub fn parse_effect_name(
    name: &str,
    param: Option<u8>,
    xy: Option<[u8; 2]>,
) -> Option<TrackerEffect> {
    let name_lower = name.to_lowercase();
    let (x, y) = xy.map(|[a, b]| (a, b)).unwrap_or((0, 0));
    let p = param.unwrap_or(0);

    match name_lower.as_str() {
        "arpeggio" => Some(TrackerEffect::Arpeggio {
            x: xy.map(|[a, _]| a).unwrap_or((p >> 4) & 0x0F),
            y: xy.map(|[_, b]| b).unwrap_or(p & 0x0F),
        }),
        "porta_up" | "portamento_up" => Some(TrackerEffect::PortamentoUp { speed: p }),
        "porta_down" | "portamento_down" => Some(TrackerEffect::PortamentoDown { speed: p }),
        "fine_porta_up" | "fine_portamento_up" => Some(TrackerEffect::FinePortamentoUp {
            speed: xy.map(|[_, b]| b).unwrap_or(p & 0x0F),
        }),
        "fine_porta_down" | "fine_portamento_down" => Some(TrackerEffect::FinePortamentoDown {
            speed: xy.map(|[_, b]| b).unwrap_or(p & 0x0F),
        }),
        "tone_porta" | "tone_portamento" => Some(TrackerEffect::TonePortamento { speed: p }),
        "vibrato" => Some(TrackerEffect::Vibrato {
            speed: x.max((p >> 4) & 0x0F),
            depth: y.max(p & 0x0F),
        }),
        "tremolo" => Some(TrackerEffect::Tremolo {
            speed: x.max((p >> 4) & 0x0F),
            depth: y.max(p & 0x0F),
        }),
        "vol_slide" | "volume_slide" => Some(TrackerEffect::VolumeSlide {
            up: x.max((p >> 4) & 0x0F),
            down: y.max(p & 0x0F),
        }),
        "set_volume" => Some(TrackerEffect::SetVolume { volume: p }),
        "set_panning" | "panning" => Some(TrackerEffect::SetPanning { pan: p }),
        "sample_offset" => Some(TrackerEffect::SampleOffset { offset: p }),
        "position_jump" | "jump" => Some(TrackerEffect::PositionJump { position: p }),
        "pattern_break" | "break" => Some(TrackerEffect::PatternBreak { row: p }),
        "set_speed" | "speed" => Some(TrackerEffect::SetSpeed { speed: p }),
        "set_tempo" | "tempo" => Some(TrackerEffect::SetTempo { bpm: p }),
        "retrigger" => Some(TrackerEffect::Retrigger {
            volume_change: x.max((p >> 4) & 0x0F),
            interval: y.max(p & 0x0F),
        }),
        "note_delay" => Some(TrackerEffect::NoteDelay {
            ticks: xy.map(|[_, b]| b).unwrap_or(p & 0x0F),
        }),
        "note_cut" => Some(TrackerEffect::NoteCut {
            ticks: xy.map(|[_, b]| b).unwrap_or(p & 0x0F),
        }),
        "tremor" => Some(TrackerEffect::Tremor {
            on_time: x.max((p >> 4) & 0x0F),
            off_time: y.max(p & 0x0F),
        }),
        "global_volume" | "set_global_volume" => Some(TrackerEffect::SetGlobalVolume { volume: p }),
        "global_vol_slide" | "global_volume_slide" => Some(TrackerEffect::GlobalVolumeSlide {
            up: x.max((p >> 4) & 0x0F),
            down: y.max(p & 0x0F),
        }),
        "pan_slide" | "panning_slide" => Some(TrackerEffect::PanningSlide {
            left: x.max((p >> 4) & 0x0F),
            right: y.max(p & 0x0F),
        }),
        "channel_volume" | "set_channel_volume" => {
            Some(TrackerEffect::SetChannelVolume { volume: p })
        }
        "channel_vol_slide" | "channel_volume_slide" => Some(TrackerEffect::ChannelVolumeSlide {
            up: x.max((p >> 4) & 0x0F),
            down: y.max(p & 0x0F),
        }),
        "fine_vibrato" => Some(TrackerEffect::FineVibrato {
            speed: x.max((p >> 4) & 0x0F),
            depth: y.max(p & 0x0F),
        }),
        "vibrato_waveform" | "set_vibrato_waveform" => {
            Some(TrackerEffect::SetVibratoWaveform { waveform: p & 0x0F })
        }
        "tremolo_waveform" | "set_tremolo_waveform" => {
            Some(TrackerEffect::SetTremoloWaveform { waveform: p & 0x0F })
        }
        "finetune" | "set_finetune" => Some(TrackerEffect::SetFinetune { value: p & 0x0F }),
        "pattern_loop" | "loop" => Some(TrackerEffect::PatternLoop { count: p & 0x0F }),
        "tone_porta_vol_slide" => Some(TrackerEffect::TonePortaVolumeSlide {
            up: x.max((p >> 4) & 0x0F),
            down: y.max(p & 0x0F),
        }),
        "vibrato_vol_slide" => Some(TrackerEffect::VibratoVolumeSlide {
            up: x.max((p >> 4) & 0x0F),
            down: y.max(p & 0x0F),
        }),
        "key_off" => Some(TrackerEffect::KeyOff { tick: p }),
        "envelope_position" | "set_envelope_position" => {
            Some(TrackerEffect::SetEnvelopePosition { position: p })
        }
        "panbrello" => Some(TrackerEffect::Panbrello {
            speed: x.max((p >> 4) & 0x0F),
            depth: y.max(p & 0x0F),
        }),
        _ => None,
    }
}

/// Decode XM effect code+param into a typed tracker effect.
pub fn decode_xm_effect(code: u8, param: u8) -> Option<TrackerEffect> {
    use xm_codes::*;

    if code == 0 && param == 0 {
        return None;
    }

    let hi = (param >> 4) & 0x0F;
    let lo = param & 0x0F;

    match code {
        ARPEGGIO => Some(TrackerEffect::Arpeggio { x: hi, y: lo }),
        PORTA_UP => Some(TrackerEffect::PortamentoUp { speed: param }),
        PORTA_DOWN => Some(TrackerEffect::PortamentoDown { speed: param }),
        TONE_PORTA => Some(TrackerEffect::TonePortamento { speed: param }),
        VIBRATO => Some(TrackerEffect::Vibrato {
            speed: hi,
            depth: lo,
        }),
        TONE_PORTA_VOL_SLIDE => Some(TrackerEffect::TonePortaVolumeSlide { up: hi, down: lo }),
        VIBRATO_VOL_SLIDE => Some(TrackerEffect::VibratoVolumeSlide { up: hi, down: lo }),
        TREMOLO => Some(TrackerEffect::Tremolo {
            speed: hi,
            depth: lo,
        }),
        SET_PANNING => Some(TrackerEffect::SetPanning { pan: param }),
        SAMPLE_OFFSET => Some(TrackerEffect::SampleOffset { offset: param }),
        VOL_SLIDE => Some(TrackerEffect::VolumeSlide { up: hi, down: lo }),
        POSITION_JUMP => Some(TrackerEffect::PositionJump { position: param }),
        SET_VOLUME => Some(TrackerEffect::SetVolume { volume: param }),
        PATTERN_BREAK => Some(TrackerEffect::PatternBreak { row: param }),
        SET_SPEED_TEMPO => {
            if param <= 31 {
                Some(TrackerEffect::SetSpeed { speed: param })
            } else {
                Some(TrackerEffect::SetTempo { bpm: param })
            }
        }
        GLOBAL_VOL => Some(TrackerEffect::SetGlobalVolume { volume: param }),
        GLOBAL_VOL_SLIDE => Some(TrackerEffect::GlobalVolumeSlide { up: hi, down: lo }),
        KEY_OFF => Some(TrackerEffect::KeyOff { tick: param }),
        SET_ENV_POS => Some(TrackerEffect::SetEnvelopePosition { position: param }),
        PAN_SLIDE => Some(TrackerEffect::PanningSlide {
            left: lo,
            right: hi,
        }),
        RETRIGGER => Some(TrackerEffect::Retrigger {
            volume_change: hi,
            interval: lo,
        }),
        TREMOR => Some(TrackerEffect::Tremor {
            on_time: hi,
            off_time: lo,
        }),
        EXTRA_FINE_PORTA => match hi {
            0x1 => Some(TrackerEffect::ExtraFinePortaUp { speed: lo }),
            0x2 => Some(TrackerEffect::ExtraFinePortaDown { speed: lo }),
            _ => Some(TrackerEffect::Raw { code, param }),
        },
        EXTENDED => match hi {
            0x1 => Some(TrackerEffect::FinePortamentoUp { speed: lo }),
            0x2 => Some(TrackerEffect::FinePortamentoDown { speed: lo }),
            0x4 => Some(TrackerEffect::SetVibratoWaveform { waveform: lo }),
            0x5 => Some(TrackerEffect::SetFinetune { value: lo }),
            0x6 => Some(TrackerEffect::PatternLoop { count: lo }),
            0x7 => Some(TrackerEffect::SetTremoloWaveform { waveform: lo }),
            0xC => Some(TrackerEffect::NoteCut { ticks: lo }),
            0xD => Some(TrackerEffect::NoteDelay { ticks: lo }),
            _ => Some(TrackerEffect::Raw { code, param }),
        },
        _ => Some(TrackerEffect::Raw { code, param }),
    }
}

/// Decode IT effect code+param into a typed tracker effect.
pub fn decode_it_effect(code: u8, param: u8) -> Option<TrackerEffect> {
    use it_codes::*;

    if code == 0 && param == 0 {
        return None;
    }

    let hi = (param >> 4) & 0x0F;
    let lo = param & 0x0F;

    match code {
        SET_SPEED => Some(TrackerEffect::SetSpeed { speed: param }),
        POSITION_JUMP => Some(TrackerEffect::PositionJump { position: param }),
        PATTERN_BREAK => Some(TrackerEffect::PatternBreak { row: param }),
        VOLUME_SLIDE => Some(TrackerEffect::VolumeSlide { up: hi, down: lo }),
        PORTA_DOWN => {
            if hi == 0xF {
                Some(TrackerEffect::FinePortamentoDown { speed: lo })
            } else if hi == 0xE {
                Some(TrackerEffect::ExtraFinePortaDown { speed: lo })
            } else {
                Some(TrackerEffect::PortamentoDown { speed: param })
            }
        }
        PORTA_UP => {
            if hi == 0xF {
                Some(TrackerEffect::FinePortamentoUp { speed: lo })
            } else if hi == 0xE {
                Some(TrackerEffect::ExtraFinePortaUp { speed: lo })
            } else {
                Some(TrackerEffect::PortamentoUp { speed: param })
            }
        }
        TONE_PORTA => Some(TrackerEffect::TonePortamento { speed: param }),
        VIBRATO => Some(TrackerEffect::Vibrato {
            speed: hi,
            depth: lo,
        }),
        TREMOR => Some(TrackerEffect::Tremor {
            on_time: hi,
            off_time: lo,
        }),
        ARPEGGIO => Some(TrackerEffect::Arpeggio { x: hi, y: lo }),
        VIBRATO_VOL_SLIDE => Some(TrackerEffect::VibratoVolumeSlide { up: hi, down: lo }),
        TONE_PORTA_VOL_SLIDE => Some(TrackerEffect::TonePortaVolumeSlide { up: hi, down: lo }),
        SET_CHANNEL_VOL => Some(TrackerEffect::SetChannelVolume { volume: param }),
        CHANNEL_VOL_SLIDE => Some(TrackerEffect::ChannelVolumeSlide { up: hi, down: lo }),
        SAMPLE_OFFSET => Some(TrackerEffect::SampleOffset { offset: param }),
        PANNING_SLIDE => Some(TrackerEffect::PanningSlide {
            left: hi,
            right: lo,
        }),
        RETRIGGER => Some(TrackerEffect::Retrigger {
            volume_change: hi,
            interval: lo,
        }),
        TREMOLO => Some(TrackerEffect::Tremolo {
            speed: hi,
            depth: lo,
        }),
        EXTENDED => match hi {
            0x2 => Some(TrackerEffect::SetFinetune { value: lo }),
            0x3 => Some(TrackerEffect::SetVibratoWaveform { waveform: lo }),
            0x4 => Some(TrackerEffect::SetTremoloWaveform { waveform: lo }),
            0xB => Some(TrackerEffect::PatternLoop { count: lo }),
            0xC => Some(TrackerEffect::NoteCut { ticks: lo }),
            0xD => Some(TrackerEffect::NoteDelay { ticks: lo }),
            _ => Some(TrackerEffect::Raw { code, param }),
        },
        TEMPO => Some(TrackerEffect::SetTempo { bpm: param }),
        FINE_VIBRATO => Some(TrackerEffect::FineVibrato {
            speed: hi,
            depth: lo,
        }),
        SET_GLOBAL_VOL => Some(TrackerEffect::SetGlobalVolume { volume: param }),
        GLOBAL_VOL_SLIDE => Some(TrackerEffect::GlobalVolumeSlide { up: hi, down: lo }),
        SET_PANNING => Some(TrackerEffect::SetPanning { pan: param }),
        PANBRELLO => Some(TrackerEffect::Panbrello {
            speed: hi,
            depth: lo,
        }),
        _ => Some(TrackerEffect::Raw { code, param }),
    }
}
