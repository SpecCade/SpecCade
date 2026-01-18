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
