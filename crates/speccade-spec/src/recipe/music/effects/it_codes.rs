//! IT effect codes (letter-based: A=1, B=2, etc.).

/// Set speed (Axx).
pub const SET_SPEED: u8 = 1;
/// Position jump (Bxx).
pub const POSITION_JUMP: u8 = 2;
/// Pattern break (Cxx).
pub const PATTERN_BREAK: u8 = 3;
/// Volume slide (Dxy).
pub const VOLUME_SLIDE: u8 = 4;
/// Portamento down (Exx).
pub const PORTA_DOWN: u8 = 5;
/// Portamento up (Fxx).
pub const PORTA_UP: u8 = 6;
/// Tone portamento (Gxx).
pub const TONE_PORTA: u8 = 7;
/// Vibrato (Hxy).
pub const VIBRATO: u8 = 8;
/// Tremor (Ixy).
pub const TREMOR: u8 = 9;
/// Arpeggio (Jxy).
pub const ARPEGGIO: u8 = 10;
/// Vibrato + volume slide (Kxy).
pub const VIBRATO_VOL_SLIDE: u8 = 11;
/// Tone porta + volume slide (Lxy).
pub const TONE_PORTA_VOL_SLIDE: u8 = 12;
/// Set channel volume (Mxx).
pub const SET_CHANNEL_VOL: u8 = 13;
/// Channel volume slide (Nxy).
pub const CHANNEL_VOL_SLIDE: u8 = 14;
/// Sample offset (Oxx).
pub const SAMPLE_OFFSET: u8 = 15;
/// Panning slide (Pxy).
pub const PANNING_SLIDE: u8 = 16;
/// Retrigger note (Qxy).
pub const RETRIGGER: u8 = 17;
/// Tremolo (Rxy).
pub const TREMOLO: u8 = 18;
/// Extended effects (Sxy).
pub const EXTENDED: u8 = 19;
/// Set tempo (Txx).
pub const TEMPO: u8 = 20;
/// Fine vibrato (Uxy).
pub const FINE_VIBRATO: u8 = 21;
/// Set global volume (Vxx).
pub const SET_GLOBAL_VOL: u8 = 22;
/// Global volume slide (Wxy).
pub const GLOBAL_VOL_SLIDE: u8 = 23;
/// Set panning (Xxx).
pub const SET_PANNING: u8 = 24;
/// Panbrello (Yxy).
pub const PANBRELLO: u8 = 25;
/// MIDI macro (Zxx).
pub const MIDI_MACRO: u8 = 26;
