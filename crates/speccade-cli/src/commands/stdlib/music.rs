//! Music tracker functions (instruments, patterns, songs)

use super::{func, param, FunctionInfo};

pub(super) fn register_functions() -> Vec<FunctionInfo> {
    vec![
        func!(
            "instrument_synthesis",
            "music.instruments",
            "Creates a tracker instrument synthesis configuration.",
            vec![
                param!("synth_type", "string", req, enum: &["pulse", "square", "triangle", "sawtooth", "sine", "noise"]),
                param!("duty_cycle", "float", opt, 0.5, range: Some(0.0), Some(1.0)),
                param!("periodic", "bool", opt, false),
            ],
            "An instrument synthesis dict.",
            r#"instrument_synthesis("pulse", 0.25)"#
        ),
        func!(
            "tracker_instrument",
            "music.instruments",
            "Creates a tracker instrument definition.",
            vec![
                param!("name", "string", req),
                param!("synthesis", "dict", opt_none),
                param!("wav", "string", opt_none),
                param!("envelope", "dict", opt_none),
                param!("loop_mode", "string", opt_none, enum: &["auto", "none", "forward", "pingpong"]),
            ],
            "An instrument dict.",
            r#"tracker_instrument(name="bass", synthesis=instrument_synthesis("sawtooth"))"#
        ),
        func!(
            "pattern_note",
            "music.patterns",
            "Creates a pattern note event.",
            vec![
                param!("row", "int", req, range: Some(0.0), None),
                param!("note", "string", req),
                param!("inst", "int", req, range: Some(0.0), None),
                param!("vol", "int", opt_none, range: Some(0.0), Some(64.0)),
            ],
            "A pattern note dict.",
            r#"pattern_note(0, "C4", 0)"#
        ),
        func!(
            "tracker_pattern",
            "music.patterns",
            "Creates a tracker pattern definition.",
            vec![
                param!("rows", "int", req, range: Some(1.0), None),
                param!("notes", "dict", opt_none),
                param!("data", "list", opt_none),
            ],
            "A pattern dict.",
            r#"tracker_pattern(64, notes={"0": [pattern_note(0, "C4", 0)]})"#
        ),
        func!(
            "arrangement_entry",
            "music.patterns",
            "Creates an arrangement entry.",
            vec![
                param!("pattern", "string", req),
                param!("repeat", "int", opt, 1, range: Some(1.0), None),
            ],
            "An arrangement entry dict.",
            r#"arrangement_entry("intro", 4)"#
        ),
        func!(
            "tracker_song",
            "music.song",
            "Creates a complete tracker song recipe.",
            vec![
                param!("format", "string", req, enum: &["xm", "it"]),
                param!("bpm", "int", req, range: Some(30.0), Some(300.0)),
                param!("speed", "int", req, range: Some(1.0), Some(31.0)),
                param!("channels", "int", req, range: Some(1.0), Some(64.0)),
                param!("instruments", "list", req),
                param!("patterns", "dict", req),
                param!("arrangement", "list", req),
                param!("loop", "bool", opt, false),
            ],
            "A tracker song recipe dict."
        ),
        func!(
            "music_spec",
            "music.song",
            "Creates a music spec with a tracker song recipe.",
            vec![
                param!("asset_id", "string", req),
                param!("seed", "int", req, range: Some(0.0), Some(4294967295.0)),
                param!("output_path", "string", req),
                param!("format", "string", req, enum: &["xm", "it"]),
                param!("bpm", "int", req, range: Some(30.0), Some(300.0)),
                param!("speed", "int", req, range: Some(1.0), Some(31.0)),
                param!("channels", "int", req, range: Some(1.0), Some(64.0)),
                param!("instruments", "list", req),
                param!("patterns", "dict", req),
                param!("arrangement", "list", req),
            ],
            "A complete music spec dict."
        ),
    ]
}
