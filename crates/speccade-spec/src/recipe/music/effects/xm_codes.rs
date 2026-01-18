//! XM effect codes.

/// Arpeggio effect (0xy).
pub const ARPEGGIO: u8 = 0x0;
/// Portamento up (1xx).
pub const PORTA_UP: u8 = 0x1;
/// Portamento down (2xx).
pub const PORTA_DOWN: u8 = 0x2;
/// Tone portamento (3xx).
pub const TONE_PORTA: u8 = 0x3;
/// Vibrato (4xy).
pub const VIBRATO: u8 = 0x4;
/// Tone portamento + volume slide (5xy).
pub const TONE_PORTA_VOL_SLIDE: u8 = 0x5;
/// Vibrato + volume slide (6xy).
pub const VIBRATO_VOL_SLIDE: u8 = 0x6;
/// Tremolo (7xy).
pub const TREMOLO: u8 = 0x7;
/// Set panning (8xx).
pub const SET_PANNING: u8 = 0x8;
/// Sample offset (9xx).
pub const SAMPLE_OFFSET: u8 = 0x9;
/// Volume slide (Axy).
pub const VOL_SLIDE: u8 = 0xA;
/// Position jump (Bxx).
pub const POSITION_JUMP: u8 = 0xB;
/// Set volume (Cxx).
pub const SET_VOLUME: u8 = 0xC;
/// Pattern break (Dxx).
pub const PATTERN_BREAK: u8 = 0xD;
/// Extended effects (Exy).
pub const EXTENDED: u8 = 0xE;
/// Set speed/tempo (Fxx).
pub const SET_SPEED_TEMPO: u8 = 0xF;
/// Set global volume (Gxx).
pub const GLOBAL_VOL: u8 = 0x10;
/// Global volume slide (Hxy).
pub const GLOBAL_VOL_SLIDE: u8 = 0x11;
/// Key off (Kxx).
pub const KEY_OFF: u8 = 0x14;
/// Set envelope position (Lxx).
pub const SET_ENV_POS: u8 = 0x15;
/// Panning slide (Pxy).
pub const PAN_SLIDE: u8 = 0x19;
/// Multi retrigger note (Rxy).
pub const RETRIGGER: u8 = 0x1B;
/// Tremor (Txy).
pub const TREMOR: u8 = 0x1D;
/// Extra fine portamento (Xxy).
pub const EXTRA_FINE_PORTA: u8 = 0x21;
