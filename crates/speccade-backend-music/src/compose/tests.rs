//! Tests for compose expansion.

use std::collections::HashMap;

use rand::Rng;
use speccade_spec::recipe::music::{
    BeatDelta, BeatPos, CellTemplate, ChannelRef, ChordSpec, ComposePattern, Harmony,
    HarmonyScale, InstrumentRef, MergePolicy, MusicTrackerSongComposeV1Params, PatternExpr,
    PitchSeq, PitchSeqKind, Seq, SeqMode, TimeBase, TimeExpr, TrackerFormat, TrackerInstrument,
    TransformOp, WeightedChoice,
};

use super::super::{expand_compose, ExpandError};
use super::expander::Expander;
use super::harmony::parse_chord_symbol;
use super::merge::{apply_transforms, insert_cell_merge, Cell, CellMap};
use super::utils::rng_for;

fn base_params() -> MusicTrackerSongComposeV1Params {
    MusicTrackerSongComposeV1Params {
        format: TrackerFormat::Xm,
        bpm: 120,
        speed: 6,
        channels: 4,
        instruments: vec![TrackerInstrument::default()],
        ..Default::default()
    }
}

#[test]
fn time_expr_range_list_pattern() {
    let params = base_params();
    let defs = HashMap::new();
    let expander = Expander::new(&params, "p0", &defs, 1, 64, None);
    let rows = expander
        .eval_time_expr(&TimeExpr::Range {
            start: 0,
            step: 2,
            count: 4,
        })
        .unwrap();
    assert_eq!(rows, vec![0, 2, 4, 6]);

    let rows = expander
        .eval_time_expr(&TimeExpr::List {
            rows: vec![1, 3, 5],
        })
        .unwrap();
    assert_eq!(rows, vec![1, 3, 5]);

    let rows = expander
        .eval_time_expr(&TimeExpr::Pattern {
            pattern: "x.x.".to_string(),
            stride: 1,
            offset: 0,
        })
        .unwrap();
    assert_eq!(rows, vec![0, 2]);

    let rows = expander
        .eval_time_expr(&TimeExpr::Euclid {
            pulses: 3,
            steps: 8,
            rotate: 0,
            stride: 1,
            offset: 0,
        })
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|r| *r >= 0 && *r < 8));
}

#[test]
fn emit_seq_cycle_vs_once() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::EmitSeq {
                at: TimeExpr::Range {
                    start: 0,
                    step: 1,
                    count: 4,
                },
                cell: Box::new(CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                }),
                note_seq: Some(Seq {
                    mode: SeqMode::Cycle,
                    values: vec!["C4".to_string(), "D4".to_string()],
                }),
                pitch_seq: None,
                inst_seq: None,
                vol_seq: None,
                effect_seq: None,
                param_seq: None,
                effect_name_seq: None,
                effect_xy_seq: None,
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let pattern = expanded.patterns.get("p0").unwrap();
    let notes = pattern.data.as_ref().unwrap();
    assert_eq!(notes[0].note, "C4");
    assert_eq!(notes[1].note, "D4");
    assert_eq!(notes[2].note, "C4");
    assert_eq!(notes[3].note, "D4");

    params.patterns.insert(
        "p1".to_string(),
        ComposePattern {
            rows: Some(2),
            bars: None,
            timebase: None,
            program: PatternExpr::EmitSeq {
                at: TimeExpr::Range {
                    start: 0,
                    step: 1,
                    count: 2,
                },
                cell: Box::new(CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                }),
                note_seq: Some(Seq {
                    mode: SeqMode::Once,
                    values: vec!["E4".to_string(), "F4".to_string()],
                }),
                pitch_seq: None,
                inst_seq: None,
                vol_seq: None,
                effect_seq: None,
                param_seq: None,
                effect_name_seq: None,
                effect_xy_seq: None,
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p1").unwrap().data.as_ref().unwrap();
    assert_eq!(notes[0].note, "E4");
    assert_eq!(notes[1].note, "F4");
}

#[test]
fn emit_expands_cells() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Emit {
                at: TimeExpr::Range {
                    start: 0,
                    step: 2,
                    count: 3,
                },
                cell: CellTemplate {
                    channel: ChannelRef::Index(1),
                    note: Some("C4".to_string()),
                    inst: Some(InstrumentRef::Index(0)),
                    vol: Some(48),
                    ..Default::default()
                },
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[0].row, 0);
    assert_eq!(notes[1].row, 2);
    assert_eq!(notes[2].row, 4);
    assert_eq!(notes[0].channel, Some(1));
    assert_eq!(notes[0].note, "C4");
    assert_eq!(notes[0].inst, 0);
    assert_eq!(notes[0].vol, Some(48));
}

#[test]
fn ref_resolution_and_cycles() {
    let mut params = base_params();
    params.defs.insert(
        "beat".to_string(),
        PatternExpr::Emit {
            at: TimeExpr::List { rows: vec![0, 2] },
            cell: CellTemplate {
                channel: ChannelRef::Index(0),
                note: Some("C4".to_string()),
                inst: Some(InstrumentRef::Index(0)),
                ..Default::default()
            },
        },
    );
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Ref {
                name: "beat".to_string(),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    assert_eq!(notes.len(), 2);
    assert_eq!(notes[0].row, 0);
    assert_eq!(notes[1].row, 2);

    let mut missing = base_params();
    missing.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Ref {
                name: "nope".to_string(),
            },
            data: None,
            notes: None,
        },
    );
    let err = expand_compose(&missing, 1).unwrap_err();
    assert!(matches!(err, ExpandError::UnknownRef { .. }));

    let mut cycle = base_params();
    cycle.defs.insert(
        "a".to_string(),
        PatternExpr::Ref {
            name: "b".to_string(),
        },
    );
    cycle.defs.insert(
        "b".to_string(),
        PatternExpr::Ref {
            name: "a".to_string(),
        },
    );
    cycle.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Ref {
                name: "a".to_string(),
            },
            data: None,
            notes: None,
        },
    );
    let err = expand_compose(&cycle, 1).unwrap_err();
    assert!(matches!(err, ExpandError::RefCycle { .. }));
}

#[test]
fn choose_is_deterministic() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(1),
            bars: None,
            timebase: None,
            program: PatternExpr::Choose {
                seed_salt: "fill".to_string(),
                choices: vec![
                    WeightedChoice {
                        weight: 1,
                        body: PatternExpr::Emit {
                            at: TimeExpr::List { rows: vec![0] },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("C4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                    },
                    WeightedChoice {
                        weight: 1,
                        body: PatternExpr::Emit {
                            at: TimeExpr::List { rows: vec![0] },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("D4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                    },
                ],
            },
            data: None,
            notes: None,
        },
    );

    let first = expand_compose(&params, 42).unwrap();
    let second = expand_compose(&params, 42).unwrap();
    assert_eq!(first, second);
}

#[test]
fn seed_salt_changes_rng_stream() {
    let mut rng_a = rng_for(1, "p0", "salt_a");
    let mut rng_b = rng_for(1, "p0", "salt_b");
    let a = rng_a.gen::<u32>();
    let b = rng_b.gen::<u32>();
    assert_ne!(a, b);
}

#[test]
fn merge_policies() {
    let params = base_params();
    let defs = HashMap::new();
    let expander = Expander::new(&params, "p0", &defs, 1, 64, None);
    let mut base = CellMap::new();
    let key = (0, 0);
    insert_cell_merge(
        &mut base,
        key,
        Cell {
            note: Some("C4".to_string()),
            inst: Some(0),
            vol: Some(32),
            effect: None,
            param: None,
            effect_name: None,
            effect_xy: None,
        },
        MergePolicy::MergeFields,
        "p0",
    )
    .unwrap();

    let mut incoming = CellMap::new();
    insert_cell_merge(
        &mut incoming,
        key,
        Cell {
            note: Some("C4".to_string()),
            inst: Some(0),
            vol: Some(48),
            effect: None,
            param: None,
            effect_name: None,
            effect_xy: None,
        },
        MergePolicy::MergeFields,
        "p0",
    )
    .unwrap();

    let err = expander.merge_maps(&mut base, incoming, MergePolicy::MergeFields);
    assert!(err.is_err());

    let mut incoming = CellMap::new();
    insert_cell_merge(
        &mut incoming,
        key,
        Cell {
            note: Some("C4".to_string()),
            inst: Some(0),
            vol: Some(48),
            effect: None,
            param: None,
            effect_name: None,
            effect_xy: None,
        },
        MergePolicy::MergeFields,
        "p0",
    )
    .unwrap();

    expander
        .merge_maps(&mut base, incoming, MergePolicy::LastWins)
        .unwrap();
    let cell = base.get(&key).unwrap();
    assert_eq!(cell.vol, Some(48));
}

#[test]
fn transpose_skips_special_notes() {
    let mut cell = Cell {
        note: Some("---".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };
    apply_transforms(
        &mut cell,
        &[TransformOp::TransposeSemitones { semitones: 12 }],
        "p0",
    )
    .unwrap();
    assert_eq!(cell.note.as_deref(), Some("---"));
}

#[test]
fn transform_transpose_vol_mul_set() {
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: None,
        vol: Some(32),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };
    apply_transforms(
        &mut cell,
        &[
            TransformOp::TransposeSemitones { semitones: 12 },
            TransformOp::VolMul { mul: 3, div: 2 },
            TransformOp::Set {
                inst: Some(1),
                vol: Some(10),
                effect: Some(2),
                param: Some(3),
                effect_name: Some("arpeggio".to_string()),
                effect_xy: Some([1, 2]),
            },
        ],
        "p0",
    )
    .unwrap();

    assert_eq!(cell.note.as_deref(), Some("C5"));
    assert_eq!(cell.vol, Some(48));
    assert_eq!(cell.inst, Some(1));
    assert_eq!(cell.effect, Some(2));
    assert_eq!(cell.param, Some(3));
    assert_eq!(cell.effect_name.as_deref(), Some("arpeggio"));
    assert_eq!(cell.effect_xy, Some([1, 2]));
}

#[test]
fn rng_determinism_prob() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Prob {
                p_permille: 500,
                seed_salt: "hats".to_string(),
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 8,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    },
                }),
            },
            data: None,
            notes: None,
        },
    );

    let first = expand_compose(&params, 123).unwrap();
    let second = expand_compose(&params, 123).unwrap();
    assert_eq!(first, second);
}

#[test]
fn aliases_resolve_channel_and_instrument() {
    let mut params = base_params();
    params.channels = 2;
    params.instruments = vec![TrackerInstrument::default(), TrackerInstrument::default()];
    params.channel_ids.insert("kick".to_string(), 1);
    params.instrument_ids.insert("kick".to_string(), 1);

    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Emit {
                at: TimeExpr::List { rows: vec![0] },
                cell: CellTemplate {
                    channel: ChannelRef::Name("kick".to_string()),
                    inst: Some(InstrumentRef::Name("kick".to_string())),
                    note: Some("C4".to_string()),
                    ..Default::default()
                },
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].channel, Some(1));
    assert_eq!(notes[0].inst, 1);
}

#[test]
fn time_expr_beat_range_and_list() {
    let params = base_params();
    let defs = HashMap::new();
    let expander = Expander::new(
        &params,
        "p0",
        &defs,
        1,
        64,
        Some(TimeBase {
            beats_per_bar: 4,
            rows_per_beat: 4,
        }),
    );

    let rows = expander
        .eval_time_expr(&TimeExpr::BeatRange {
            start: BeatPos {
                bar: 0,
                beat: 0,
                sub: 0,
            },
            step: BeatDelta { beats: 1, sub: 0 },
            count: 4,
        })
        .unwrap();
    assert_eq!(rows, vec![0, 4, 8, 12]);

    let rows = expander
        .eval_time_expr(&TimeExpr::BeatList {
            beats: vec![
                BeatPos {
                    bar: 0,
                    beat: 0,
                    sub: 0,
                },
                BeatPos {
                    bar: 0,
                    beat: 2,
                    sub: 2,
                },
            ],
        })
        .unwrap();
    assert_eq!(rows, vec![0, 10]);
}

#[test]
fn pattern_bars_expands_to_rows() {
    let mut params = base_params();
    params.timebase = Some(TimeBase {
        beats_per_bar: 4,
        rows_per_beat: 4,
    });
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: None,
            bars: Some(2),
            timebase: None,
            program: PatternExpr::Emit {
                at: TimeExpr::List { rows: vec![0] },
                cell: CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    note: Some("C4".to_string()),
                    ..Default::default()
                },
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    assert_eq!(expanded.patterns.get("p0").unwrap().rows, 32);
}

#[test]
fn chord_symbol_parses_intervals_and_bass() {
    let chord = parse_chord_symbol("Cmaj7#11/G", "p0").unwrap();
    assert_eq!(chord.root_pc, 0);
    assert_eq!(chord._bass_pc, Some(7));
    assert_eq!(chord.intervals, vec![0, 4, 7, 11, 18]);
}

#[test]
fn pitch_seq_scale_degree_maps_notes() {
    let mut params = base_params();
    params.harmony = Some(Harmony {
        key: speccade_spec::recipe::music::HarmonyKey {
            root: "A".to_string(),
            scale: HarmonyScale::Minor,
        },
        chords: vec![],
    });
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::EmitSeq {
                at: TimeExpr::Range {
                    start: 0,
                    step: 1,
                    count: 4,
                },
                cell: Box::new(CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                }),
                note_seq: None,
                pitch_seq: Some(PitchSeq {
                    kind: PitchSeqKind::ScaleDegree,
                    mode: SeqMode::Once,
                    values: vec![
                        "1".to_string(),
                        "2".to_string(),
                        "3".to_string(),
                        "5".to_string(),
                    ],
                    octave: 5,
                    allow_accidentals: false,
                }),
                inst_seq: None,
                vol_seq: None,
                effect_seq: None,
                param_seq: None,
                effect_name_seq: None,
                effect_xy_seq: None,
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    assert_eq!(notes[0].note, "A5");
    assert_eq!(notes[1].note, "B5");
    assert_eq!(notes[2].note, "C6");
    assert_eq!(notes[3].note, "E6");
}

#[test]
fn pitch_seq_chord_tone_maps_notes() {
    let mut params = base_params();
    params.timebase = Some(TimeBase {
        beats_per_bar: 4,
        rows_per_beat: 4,
    });
    params.harmony = Some(Harmony {
        key: speccade_spec::recipe::music::HarmonyKey {
            root: "A".to_string(),
            scale: HarmonyScale::Minor,
        },
        chords: vec![speccade_spec::recipe::music::HarmonyChordEntry {
            at: BeatPos {
                bar: 0,
                beat: 0,
                sub: 0,
            },
            chord: ChordSpec::Symbol(speccade_spec::recipe::music::ChordSpecSymbol {
                symbol: "Am7".to_string(),
            }),
        }],
    });

    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: None,
            bars: Some(1),
            timebase: None,
            program: PatternExpr::EmitSeq {
                at: TimeExpr::Range {
                    start: 0,
                    step: 1,
                    count: 4,
                },
                cell: Box::new(CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                }),
                note_seq: None,
                pitch_seq: Some(PitchSeq {
                    kind: PitchSeqKind::ChordTone,
                    mode: SeqMode::Once,
                    values: vec![
                        "1".to_string(),
                        "3".to_string(),
                        "5".to_string(),
                        "7".to_string(),
                    ],
                    octave: 2,
                    allow_accidentals: false,
                }),
                inst_seq: None,
                vol_seq: None,
                effect_seq: None,
                param_seq: None,
                effect_name_seq: None,
                effect_xy_seq: None,
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    assert_eq!(notes[0].note, "A2");
    assert_eq!(notes[1].note, "C3");
    assert_eq!(notes[2].note, "E3");
    assert_eq!(notes[3].note, "G3");
}
