//! Effect validation for XM and IT formats.

use super::TrackerEffect;

/// Effect validation error.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectValidationError {
    /// Parameter value out of valid range.
    OutOfRange {
        param: String,
        value: u32,
        min: u32,
        max: u32,
    },
    /// Invalid parameter combination.
    InvalidParameter(String),
    /// Effect not supported in target format.
    UnsupportedFormat { effect: String, format: String },
}

impl std::fmt::Display for EffectValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfRange {
                param,
                value,
                min,
                max,
            } => {
                write!(
                    f,
                    "{} value {} out of range [{}, {}]",
                    param, value, min, max
                )
            }
            Self::InvalidParameter(msg) => write!(f, "invalid parameter: {}", msg),
            Self::UnsupportedFormat { effect, format } => {
                write!(f, "effect {} not supported in {} format", effect, format)
            }
        }
    }
}

impl std::error::Error for EffectValidationError {}

/// Validate that a value fits in a nibble (0-15).
pub fn validate_nibble(value: u8, name: &str) -> Result<(), EffectValidationError> {
    if value > 15 {
        return Err(EffectValidationError::OutOfRange {
            param: name.to_string(),
            value: value as u32,
            min: 0,
            max: 15,
        });
    }
    Ok(())
}

impl TrackerEffect {
    /// Validate effect parameters for XM format.
    pub fn validate_xm(&self) -> Result<(), EffectValidationError> {
        match self {
            Self::Arpeggio { x, y } => {
                validate_nibble(*x, "arpeggio x")?;
                validate_nibble(*y, "arpeggio y")?;
            }
            Self::FinePortamentoUp { speed } | Self::FinePortamentoDown { speed } => {
                validate_nibble(*speed, "fine portamento speed")?;
            }
            Self::ExtraFinePortaUp { speed } | Self::ExtraFinePortaDown { speed } => {
                validate_nibble(*speed, "extra fine portamento speed")?;
            }
            Self::Vibrato { speed, depth } => {
                validate_nibble(*speed, "vibrato speed")?;
                validate_nibble(*depth, "vibrato depth")?;
            }
            Self::Tremolo { speed, depth } => {
                validate_nibble(*speed, "tremolo speed")?;
                validate_nibble(*depth, "tremolo depth")?;
            }
            Self::VolumeSlide { up, down } => {
                validate_nibble(*up, "volume slide up")?;
                validate_nibble(*down, "volume slide down")?;
                if *up > 0 && *down > 0 && *up != 0x0F && *down != 0x0F {
                    return Err(EffectValidationError::InvalidParameter(
                        "volume slide: cannot have both up and down non-zero (except fine slides)"
                            .to_string(),
                    ));
                }
            }
            Self::SetVolume { volume } => {
                if *volume > 64 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "volume".to_string(),
                        value: *volume as u32,
                        min: 0,
                        max: 64,
                    });
                }
            }
            Self::SetSpeed { speed } => {
                if *speed < 1 || *speed > 31 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "speed".to_string(),
                        value: *speed as u32,
                        min: 1,
                        max: 31,
                    });
                }
            }
            Self::SetTempo { bpm } => {
                if *bpm < 32 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "bpm".to_string(),
                        value: *bpm as u32,
                        min: 32,
                        max: 255,
                    });
                }
            }
            Self::Retrigger {
                volume_change,
                interval,
            } => {
                validate_nibble(*volume_change, "retrigger volume_change")?;
                validate_nibble(*interval, "retrigger interval")?;
            }
            Self::NoteDelay { ticks } | Self::NoteCut { ticks } => {
                validate_nibble(*ticks, "ticks")?;
            }
            Self::Tremor { on_time, off_time } => {
                validate_nibble(*on_time, "tremor on_time")?;
                validate_nibble(*off_time, "tremor off_time")?;
            }
            Self::SetGlobalVolume { volume } => {
                if *volume > 64 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "global volume".to_string(),
                        value: *volume as u32,
                        min: 0,
                        max: 64,
                    });
                }
            }
            Self::GlobalVolumeSlide { up, down }
            | Self::PanningSlide {
                left: up,
                right: down,
            } => {
                validate_nibble(*up, "slide up/left")?;
                validate_nibble(*down, "slide down/right")?;
            }
            Self::SetVibratoWaveform { waveform } | Self::SetTremoloWaveform { waveform } => {
                if *waveform > 3 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "waveform".to_string(),
                        value: *waveform as u32,
                        min: 0,
                        max: 3,
                    });
                }
            }
            Self::SetFinetune { value } => {
                validate_nibble(*value, "finetune")?;
            }
            Self::PatternLoop { count } => {
                validate_nibble(*count, "loop count")?;
            }
            Self::TonePortaVolumeSlide { up, down } | Self::VibratoVolumeSlide { up, down } => {
                validate_nibble(*up, "volume slide up")?;
                validate_nibble(*down, "volume slide down")?;
            }
            Self::PatternBreak { row } => {
                if *row > 63 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "pattern break row".to_string(),
                        value: *row as u32,
                        min: 0,
                        max: 63,
                    });
                }
            }
            // IT-only effects
            Self::SetChannelVolume { .. }
            | Self::ChannelVolumeSlide { .. }
            | Self::FineVibrato { .. }
            | Self::Panbrello { .. } => {
                return Err(EffectValidationError::UnsupportedFormat {
                    effect: format!("{:?}", self),
                    format: "XM".to_string(),
                });
            }
            // Effects with no special validation
            Self::PortamentoUp { .. }
            | Self::PortamentoDown { .. }
            | Self::TonePortamento { .. }
            | Self::SetPanning { .. }
            | Self::SampleOffset { .. }
            | Self::PositionJump { .. }
            | Self::KeyOff { .. }
            | Self::SetEnvelopePosition { .. }
            | Self::Raw { .. } => {}
        }
        Ok(())
    }

    /// Validate effect parameters for IT format.
    pub fn validate_it(&self) -> Result<(), EffectValidationError> {
        match self {
            Self::Arpeggio { x, y } => {
                validate_nibble(*x, "arpeggio x")?;
                validate_nibble(*y, "arpeggio y")?;
            }
            Self::FinePortamentoUp { speed } | Self::FinePortamentoDown { speed } => {
                validate_nibble(*speed, "fine portamento speed")?;
            }
            Self::ExtraFinePortaUp { speed } | Self::ExtraFinePortaDown { speed } => {
                validate_nibble(*speed, "extra fine portamento speed")?;
            }
            Self::Vibrato { speed, depth } | Self::FineVibrato { speed, depth } => {
                validate_nibble(*speed, "vibrato speed")?;
                validate_nibble(*depth, "vibrato depth")?;
            }
            Self::Tremolo { speed, depth } => {
                validate_nibble(*speed, "tremolo speed")?;
                validate_nibble(*depth, "tremolo depth")?;
            }
            Self::VolumeSlide { up, down } => {
                validate_nibble(*up, "volume slide up")?;
                validate_nibble(*down, "volume slide down")?;
            }
            Self::SetChannelVolume { volume } => {
                if *volume > 64 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "channel volume".to_string(),
                        value: *volume as u32,
                        min: 0,
                        max: 64,
                    });
                }
            }
            Self::ChannelVolumeSlide { up, down } => {
                validate_nibble(*up, "channel volume slide up")?;
                validate_nibble(*down, "channel volume slide down")?;
            }
            Self::SetTempo { bpm } => {
                if *bpm < 32 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "bpm".to_string(),
                        value: *bpm as u32,
                        min: 32,
                        max: 255,
                    });
                }
            }
            Self::Retrigger {
                volume_change,
                interval,
            } => {
                validate_nibble(*volume_change, "retrigger volume_change")?;
                validate_nibble(*interval, "retrigger interval")?;
            }
            Self::NoteDelay { ticks } | Self::NoteCut { ticks } => {
                validate_nibble(*ticks, "ticks")?;
            }
            Self::Tremor { on_time, off_time } => {
                validate_nibble(*on_time, "tremor on_time")?;
                validate_nibble(*off_time, "tremor off_time")?;
            }
            Self::SetGlobalVolume { volume } => {
                if *volume > 128 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "global volume".to_string(),
                        value: *volume as u32,
                        min: 0,
                        max: 128,
                    });
                }
            }
            Self::GlobalVolumeSlide { up, down }
            | Self::PanningSlide {
                left: up,
                right: down,
            } => {
                validate_nibble(*up, "slide up/left")?;
                validate_nibble(*down, "slide down/right")?;
            }
            Self::Panbrello { speed, depth } => {
                validate_nibble(*speed, "panbrello speed")?;
                validate_nibble(*depth, "panbrello depth")?;
            }
            Self::SetVibratoWaveform { waveform } | Self::SetTremoloWaveform { waveform } => {
                if *waveform > 3 {
                    return Err(EffectValidationError::OutOfRange {
                        param: "waveform".to_string(),
                        value: *waveform as u32,
                        min: 0,
                        max: 3,
                    });
                }
            }
            Self::SetFinetune { value } => {
                validate_nibble(*value, "finetune")?;
            }
            Self::PatternLoop { count } => {
                validate_nibble(*count, "loop count")?;
            }
            Self::TonePortaVolumeSlide { up, down } | Self::VibratoVolumeSlide { up, down } => {
                validate_nibble(*up, "volume slide up")?;
                validate_nibble(*down, "volume slide down")?;
            }
            // XM-only effects
            Self::KeyOff { .. } | Self::SetEnvelopePosition { .. } => {
                return Err(EffectValidationError::UnsupportedFormat {
                    effect: format!("{:?}", self),
                    format: "IT".to_string(),
                });
            }
            // SetVolume uses volume column in IT, not effect column
            Self::SetVolume { .. } => {
                return Err(EffectValidationError::UnsupportedFormat {
                    effect: "SetVolume (use volume column instead)".to_string(),
                    format: "IT".to_string(),
                });
            }
            // Effects with no special validation
            Self::PortamentoUp { .. }
            | Self::PortamentoDown { .. }
            | Self::TonePortamento { .. }
            | Self::SetPanning { .. }
            | Self::SampleOffset { .. }
            | Self::PositionJump { .. }
            | Self::PatternBreak { .. }
            | Self::SetSpeed { .. }
            | Self::Raw { .. } => {}
        }
        Ok(())
    }
}
