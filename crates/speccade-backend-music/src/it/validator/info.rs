//! Information structures extracted during IT validation.

/// Information extracted from IT header.
#[derive(Debug, Clone)]
pub struct ItHeaderInfo {
    /// Song name (up to 26 characters).
    pub name: String,
    /// Pattern highlight (minor, major).
    pub pattern_highlight: (u8, u8),
    /// Number of orders.
    pub order_count: u16,
    /// Number of instruments.
    pub instrument_count: u16,
    /// Number of samples.
    pub sample_count: u16,
    /// Number of patterns.
    pub pattern_count: u16,
    /// Created with tracker version.
    pub created_with: u16,
    /// Compatible with tracker version.
    pub compatible_with: u16,
    /// Flags.
    pub flags: ItFlags,
    /// Special flags.
    pub special: ItSpecialFlags,
    /// Global volume (0-128).
    pub global_volume: u8,
    /// Mix volume (0-128).
    pub mix_volume: u8,
    /// Initial speed (ticks per row).
    pub initial_speed: u8,
    /// Initial tempo (BPM).
    pub initial_tempo: u8,
    /// Panning separation (0-128).
    pub panning_separation: u8,
    /// Pitch wheel depth.
    pub pitch_wheel_depth: u8,
    /// Message length.
    pub message_length: u16,
    /// Message offset.
    pub message_offset: u32,
    /// Channel panning values.
    pub channel_pan: [u8; 64],
    /// Channel volume values.
    pub channel_vol: [u8; 64],
}

/// IT header flags parsed from the flags field.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItFlags {
    /// Stereo output.
    pub stereo: bool,
    /// Vol0MixOptimizations.
    pub vol0_mix_optimizations: bool,
    /// Use instruments (vs samples only).
    pub use_instruments: bool,
    /// Linear slides.
    pub linear_slides: bool,
    /// Old effects.
    pub old_effects: bool,
    /// Link effect G memory with E/F.
    pub link_g_memory: bool,
    /// Use MIDI pitch controller.
    pub midi_pitch_controller: bool,
    /// Embedded MIDI configuration.
    pub embedded_midi_config: bool,
}

impl ItFlags {
    pub(super) fn from_u16(value: u16) -> Self {
        Self {
            stereo: value & 0x01 != 0,
            vol0_mix_optimizations: value & 0x02 != 0,
            use_instruments: value & 0x04 != 0,
            linear_slides: value & 0x08 != 0,
            old_effects: value & 0x10 != 0,
            link_g_memory: value & 0x20 != 0,
            midi_pitch_controller: value & 0x40 != 0,
            embedded_midi_config: value & 0x80 != 0,
        }
    }
}

/// IT special flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItSpecialFlags {
    /// Song message attached.
    pub message_attached: bool,
    /// Embedded MIDI configuration.
    pub midi_configuration: bool,
}

impl ItSpecialFlags {
    pub(super) fn from_u16(value: u16) -> Self {
        Self {
            message_attached: value & 0x01 != 0,
            midi_configuration: value & 0x08 != 0,
        }
    }
}

/// Information extracted from IT instrument.
#[derive(Debug, Clone)]
pub struct ItInstrumentInfo {
    /// Instrument index (1-based).
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Instrument name.
    pub name: String,
    /// DOS filename.
    pub filename: String,
    /// New Note Action.
    pub nna: u8,
    /// Duplicate Check Type.
    pub dct: u8,
    /// Duplicate Check Action.
    pub dca: u8,
    /// Fadeout value.
    pub fadeout: u16,
    /// Pitch-pan separation.
    pub pps: i8,
    /// Pitch-pan center note.
    pub ppc: u8,
    /// Global volume.
    pub global_volume: u8,
    /// Default pan (None if not set).
    pub default_pan: Option<u8>,
    /// Random volume variation.
    pub random_volume: u8,
    /// Random panning variation.
    pub random_pan: u8,
    /// Volume envelope info.
    pub volume_envelope: ItEnvelopeInfo,
    /// Panning envelope info.
    pub panning_envelope: ItEnvelopeInfo,
    /// Pitch envelope info.
    pub pitch_envelope: ItEnvelopeInfo,
}

/// Information about an IT envelope.
#[derive(Debug, Clone, Default)]
pub struct ItEnvelopeInfo {
    /// Envelope enabled.
    pub enabled: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Sustain loop enabled.
    pub sustain_loop_enabled: bool,
    /// Carry envelope.
    pub carry: bool,
    /// Filter envelope (for pitch envelope).
    pub is_filter: bool,
    /// Number of nodes.
    pub num_nodes: u8,
    /// Loop begin node.
    pub loop_begin: u8,
    /// Loop end node.
    pub loop_end: u8,
    /// Sustain loop begin node.
    pub sustain_begin: u8,
    /// Sustain loop end node.
    pub sustain_end: u8,
}

/// Information extracted from IT sample.
#[derive(Debug, Clone)]
pub struct ItSampleInfo {
    /// Sample index (1-based).
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Sample name.
    pub name: String,
    /// DOS filename.
    pub filename: String,
    /// Global volume.
    pub global_volume: u8,
    /// Sample flags.
    pub flags: ItSampleFlags,
    /// Default volume.
    pub default_volume: u8,
    /// Default pan (None if not set).
    pub default_pan: Option<u8>,
    /// Sample length in samples.
    pub length: u32,
    /// Loop begin.
    pub loop_begin: u32,
    /// Loop end.
    pub loop_end: u32,
    /// C5 speed.
    pub c5_speed: u32,
    /// Sustain loop begin.
    pub sustain_loop_begin: u32,
    /// Sustain loop end.
    pub sustain_loop_end: u32,
    /// Sample data offset.
    pub data_offset: u32,
    /// Vibrato speed.
    pub vibrato_speed: u8,
    /// Vibrato depth.
    pub vibrato_depth: u8,
    /// Vibrato rate.
    pub vibrato_rate: u8,
    /// Vibrato type.
    pub vibrato_type: u8,
    /// Conversion flags.
    pub convert_flags: ItConvertFlags,
}

/// IT sample flags parsed.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItSampleFlags {
    /// Sample has data.
    pub has_data: bool,
    /// 16-bit sample.
    pub is_16bit: bool,
    /// Stereo sample.
    pub is_stereo: bool,
    /// Compressed sample.
    pub is_compressed: bool,
    /// Loop enabled.
    pub loop_enabled: bool,
    /// Sustain loop enabled.
    pub sustain_loop_enabled: bool,
    /// Ping-pong loop.
    pub ping_pong_loop: bool,
    /// Ping-pong sustain loop.
    pub ping_pong_sustain: bool,
}

impl ItSampleFlags {
    pub(super) fn from_u8(value: u8) -> Self {
        Self {
            has_data: value & 0x01 != 0,
            is_16bit: value & 0x02 != 0,
            is_stereo: value & 0x04 != 0,
            is_compressed: value & 0x08 != 0,
            loop_enabled: value & 0x10 != 0,
            sustain_loop_enabled: value & 0x20 != 0,
            ping_pong_loop: value & 0x40 != 0,
            ping_pong_sustain: value & 0x80 != 0,
        }
    }
}

/// IT sample conversion flags.
#[derive(Debug, Clone, Copy, Default)]
pub struct ItConvertFlags {
    /// Signed samples.
    pub signed: bool,
    /// Big endian (Motorola byte order).
    pub big_endian: bool,
    /// Delta encoded.
    pub delta: bool,
    /// Byte delta (8-bit only).
    pub byte_delta: bool,
    /// TX-Wave 12-bit.
    pub tx_wave: bool,
    /// Stereo prompt.
    pub stereo_prompt: bool,
}

impl ItConvertFlags {
    pub(super) fn from_u8(value: u8) -> Self {
        Self {
            signed: value & 0x01 != 0,
            big_endian: value & 0x02 != 0,
            delta: value & 0x04 != 0,
            byte_delta: value & 0x08 != 0,
            tx_wave: value & 0x10 != 0,
            stereo_prompt: value & 0x20 != 0,
        }
    }
}

/// Information extracted from IT pattern.
#[derive(Debug, Clone)]
pub struct ItPatternInfo {
    /// Pattern index.
    pub index: usize,
    /// File offset.
    pub offset: u32,
    /// Packed data length.
    pub packed_length: u16,
    /// Number of rows.
    pub num_rows: u16,
}
