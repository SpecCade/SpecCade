//! Tests for compose expansion.

use std::collections::HashMap;

use rand::Rng;
use speccade_spec::recipe::music::{
    BeatDelta, BeatPos, CellTemplate, ChannelRef, ChordSpec, ComposePattern, FilterCriteria,
    Harmony, HarmonyScale, InstrumentRef, InterleavePart, MergePolicy, MirrorAxis,
    MusicTrackerSongComposeV1Params, PatternExpr, PitchSeq, PitchSeqKind, QuantizeScale, Seq,
    SeqMode, TimeBase, TimeExpr, TrackerFormat, TrackerInstrument, TransformOp, WeightedChoice,
};

use super::super::{expand_compose, ExpandError};
use super::expander::Expander;
use super::harmony::parse_chord_symbol;
use super::merge::{apply_transforms, insert_cell_merge, Cell, CellMap, TransformContext};
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

fn test_ctx() -> TransformContext<'static> {
    TransformContext {
        seed: 1,
        pattern_name: "p0",
        key: (0, 0),
    }
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
        &test_ctx(),
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
        &test_ctx(),
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

#[test]
fn humanize_vol_deterministic() {
    // Test that humanize_vol produces consistent results
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell1 = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };
    let mut cell2 = cell1.clone();

    apply_transforms(
        &mut cell1,
        &[TransformOp::HumanizeVol {
            min_vol: 40,
            max_vol: 60,
            seed_salt: "test".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    apply_transforms(
        &mut cell2,
        &[TransformOp::HumanizeVol {
            min_vol: 40,
            max_vol: 60,
            seed_salt: "test".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Same context and salt should produce same result
    assert_eq!(cell1.vol, cell2.vol);
    assert!(cell1.vol.unwrap() >= 40 && cell1.vol.unwrap() <= 60);
}

#[test]
fn humanize_vol_different_salt() {
    // Test that different salts produce different results
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell1 = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };
    let mut cell2 = cell1.clone();

    apply_transforms(
        &mut cell1,
        &[TransformOp::HumanizeVol {
            min_vol: 0,
            max_vol: 64,
            seed_salt: "salt_a".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    apply_transforms(
        &mut cell2,
        &[TransformOp::HumanizeVol {
            min_vol: 0,
            max_vol: 64,
            seed_salt: "salt_b".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Different salts should produce different results (with high probability)
    // Note: there's a tiny chance they could be equal, but range 0-64 makes it unlikely
    assert_ne!(cell1.vol, cell2.vol);
}

#[test]
fn humanize_vol_invalid_range() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::HumanizeVol {
            min_vol: 60,
            max_vol: 40, // Invalid: min > max
            seed_salt: "test".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn swing_offbeat_applies_delay() {
    // Test that swing applies note delay to offbeat positions
    // Row 1 with stride 2 is offbeat (1 % 2 = 1 != 0)
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (1, 0), // offbeat row
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::Swing {
            amount_permille: 500,
            stride: 2,
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Should apply note_delay effect
    assert_eq!(cell.effect_name.as_deref(), Some("note_delay"));
    assert!(cell.effect_xy.is_some());
    let delay = cell.effect_xy.unwrap()[0];
    assert!(delay > 0 && delay <= 15);
}

#[test]
fn swing_onbeat_no_delay() {
    // Test that swing does not apply to onbeat positions
    // Row 0 with stride 2 is onbeat (0 % 2 = 0)
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0), // onbeat row
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::Swing {
            amount_permille: 500,
            stride: 2,
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Should NOT apply note_delay effect
    assert!(cell.effect_name.is_none());
    assert!(cell.effect_xy.is_none());
}

#[test]
fn swing_invalid_amount() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (1, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::Swing {
            amount_permille: 1001, // Invalid: > 1000
            stride: 2,
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn swing_invalid_stride() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (1, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::Swing {
            amount_permille: 500,
            stride: 0, // Invalid: must be > 0
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn swing_deterministic() {
    // Test that swing produces consistent results
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (1, 0),
    };
    let mut cell1 = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };
    let mut cell2 = cell1.clone();

    apply_transforms(
        &mut cell1,
        &[TransformOp::Swing {
            amount_permille: 500,
            stride: 2,
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    apply_transforms(
        &mut cell2,
        &[TransformOp::Swing {
            amount_permille: 500,
            stride: 2,
            seed_salt: "swing".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Same context and salt should produce same result
    assert_eq!(cell1.effect_xy, cell2.effect_xy);
}

#[test]
fn humanize_vol_integration() {
    // Integration test using pattern expansion
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Transform {
                ops: vec![TransformOp::HumanizeVol {
                    min_vol: 40,
                    max_vol: 60,
                    seed_salt: "vel".to_string(),
                }],
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        vol: Some(50), // Will be replaced by humanize
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

    // Should be deterministic
    assert_eq!(first, second);

    // Volumes should vary per row
    let notes = first.patterns.get("p0").unwrap().data.as_ref().unwrap();
    let vols: Vec<u8> = notes.iter().filter_map(|n| n.vol).collect();
    assert_eq!(vols.len(), 4);
    // All volumes should be in range
    assert!(vols.iter().all(|v| *v >= 40 && *v <= 60));
}

#[test]
fn swing_integration() {
    // Integration test using pattern expansion
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Transform {
                ops: vec![TransformOp::Swing {
                    amount_permille: 500,
                    stride: 2,
                    seed_salt: "swing".to_string(),
                }],
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
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

    // Should be deterministic
    assert_eq!(first, second);

    // Check that offbeat rows have note_delay, onbeat rows don't
    let notes = first.patterns.get("p0").unwrap().data.as_ref().unwrap();
    for note in notes {
        if note.row % 2 == 0 {
            // onbeat
            assert!(
                note.effect_name.is_none(),
                "onbeat row {} should not have delay",
                note.row
            );
        } else {
            // offbeat
            assert_eq!(
                note.effect_name.as_deref(),
                Some("note_delay"),
                "offbeat row {} should have delay",
                note.row
            );
        }
    }
}

// ============================================================================
// New Pattern IR Operators (RFC-0011)
// ============================================================================

#[test]
fn reverse_reverses_row_positions() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Reverse {
                len_rows: 8,
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::List {
                        rows: vec![0, 2, 4, 6],
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    },
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();

    // Original rows [0, 2, 4, 6] should become [7, 5, 3, 1]
    // (len_rows - 1) - row = 7 - row
    let rows: Vec<u16> = notes.iter().map(|n| n.row).collect();
    assert_eq!(rows, vec![1, 3, 5, 7]);
}

#[test]
fn reverse_invalid_len_rows() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Reverse {
                len_rows: 0, // Invalid
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::List { rows: vec![0] },
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

    let err = expand_compose(&params, 1).unwrap_err();
    assert!(matches!(err, ExpandError::InvalidExpr { .. }));
}

#[test]
fn mirror_time_equals_reverse() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Mirror {
                len_rows: 8,
                axis: MirrorAxis::Time,
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::List {
                        rows: vec![0, 2, 4, 6],
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    },
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    let rows: Vec<u16> = notes.iter().map(|n| n.row).collect();
    assert_eq!(rows, vec![1, 3, 5, 7]);
}

#[test]
fn interleave_distributes_events() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Interleave {
                stride: 2,
                parts: vec![
                    InterleavePart {
                        body: PatternExpr::Emit {
                            at: TimeExpr::Range {
                                start: 0,
                                step: 2,
                                count: 4,
                            },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("C4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                    },
                    InterleavePart {
                        body: PatternExpr::Emit {
                            at: TimeExpr::Range {
                                start: 0,
                                step: 2,
                                count: 4,
                            },
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

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();

    // Part 0 should handle blocks 0, 2 (rows 0, 4)
    // Part 1 should handle blocks 1, 3 (rows 2, 6)
    let c4_rows: Vec<u16> = notes
        .iter()
        .filter(|n| n.note == "C4")
        .map(|n| n.row)
        .collect();
    let d4_rows: Vec<u16> = notes
        .iter()
        .filter(|n| n.note == "D4")
        .map(|n| n.row)
        .collect();

    assert_eq!(c4_rows, vec![0, 4]);
    assert_eq!(d4_rows, vec![2, 6]);
}

#[test]
fn interleave_invalid_stride() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Interleave {
                stride: 0, // Invalid
                parts: vec![InterleavePart {
                    body: PatternExpr::Emit {
                        at: TimeExpr::List { rows: vec![0] },
                        cell: CellTemplate {
                            channel: ChannelRef::Index(0),
                            inst: Some(InstrumentRef::Index(0)),
                            ..Default::default()
                        },
                    },
                }],
            },
            data: None,
            notes: None,
        },
    );

    let err = expand_compose(&params, 1).unwrap_err();
    assert!(matches!(err, ExpandError::InvalidExpr { .. }));
}

#[test]
fn interleave_too_many_parts() {
    let mut params = base_params();
    let parts: Vec<InterleavePart> = (0..17)
        .map(|_| InterleavePart {
            body: PatternExpr::Emit {
                at: TimeExpr::List { rows: vec![0] },
                cell: CellTemplate {
                    channel: ChannelRef::Index(0),
                    inst: Some(InstrumentRef::Index(0)),
                    ..Default::default()
                },
            },
        })
        .collect();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Interleave { stride: 1, parts },
            data: None,
            notes: None,
        },
    );

    let err = expand_compose(&params, 1).unwrap_err();
    assert!(matches!(err, ExpandError::InvalidExpr { .. }));
}

#[test]
fn remap_channel_changes_channel() {
    let mut params = base_params();
    params.channels = 4;
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::RemapChannel {
                from: 0,
                to: 2,
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 4,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    },
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();

    for note in notes {
        assert_eq!(note.channel, Some(2));
    }
}

#[test]
fn filter_by_row_range() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(8),
            bars: None,
            timebase: None,
            program: PatternExpr::Filter {
                criteria: FilterCriteria {
                    min_row: Some(2),
                    max_row: Some(6),
                    ..Default::default()
                },
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::Range {
                        start: 0,
                        step: 1,
                        count: 8,
                    },
                    cell: CellTemplate {
                        channel: ChannelRef::Index(0),
                        note: Some("C4".to_string()),
                        inst: Some(InstrumentRef::Index(0)),
                        ..Default::default()
                    },
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    let rows: Vec<u16> = notes.iter().map(|n| n.row).collect();

    // Should only include rows 2, 3, 4, 5 (min_row=2, max_row=6 means [2, 6))
    assert_eq!(rows, vec![2, 3, 4, 5]);
}

#[test]
fn filter_by_channel() {
    let mut params = base_params();
    params.channels = 2;
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Filter {
                criteria: FilterCriteria {
                    channel: Some(1),
                    ..Default::default()
                },
                body: Box::new(PatternExpr::Stack {
                    merge: MergePolicy::MergeFields,
                    parts: vec![
                        PatternExpr::Emit {
                            at: TimeExpr::Range {
                                start: 0,
                                step: 1,
                                count: 4,
                            },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("C4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                        PatternExpr::Emit {
                            at: TimeExpr::Range {
                                start: 0,
                                step: 1,
                                count: 4,
                            },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(1),
                                note: Some("D4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                    ],
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();

    // Should only include channel 1 notes
    for note in notes {
        assert_eq!(note.channel, Some(1));
        assert_eq!(note.note, "D4");
    }
}

#[test]
fn filter_by_has_note() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Filter {
                criteria: FilterCriteria {
                    has_note: Some(true),
                    ..Default::default()
                },
                body: Box::new(PatternExpr::Stack {
                    merge: MergePolicy::MergeFields,
                    parts: vec![
                        PatternExpr::Emit {
                            at: TimeExpr::List { rows: vec![0, 2] },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("C4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                        PatternExpr::Emit {
                            at: TimeExpr::List { rows: vec![1, 3] },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("---".to_string()), // No note
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        },
                    ],
                }),
            },
            data: None,
            notes: None,
        },
    );

    let expanded = expand_compose(&params, 1).unwrap();
    let notes = expanded.patterns.get("p0").unwrap().data.as_ref().unwrap();
    let rows: Vec<u16> = notes.iter().map(|n| n.row).collect();

    // Should only include rows with actual notes (0, 2)
    assert_eq!(rows, vec![0, 2]);
}

#[test]
fn filter_requires_criteria() {
    let mut params = base_params();
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(4),
            bars: None,
            timebase: None,
            program: PatternExpr::Filter {
                criteria: FilterCriteria::default(), // No criteria
                body: Box::new(PatternExpr::Emit {
                    at: TimeExpr::List { rows: vec![0] },
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

    let err = expand_compose(&params, 1).unwrap_err();
    assert!(matches!(err, ExpandError::InvalidExpr { .. }));
}

// ============================================================================
// New Transform Operators (RFC-0011)
// ============================================================================

#[test]
fn invert_pitch_reflects_around_pivot() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    // Pivot is C4 (MIDI 60)
    // E4 is MIDI 64, which is +4 from C4
    // Inverted should be G#3 which is MIDI 56 (-4 from C4)
    let mut cell = Cell {
        note: Some("E4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::InvertPitch {
            pivot: "C4".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    assert_eq!(cell.note.as_deref(), Some("G#3"));
}

#[test]
fn invert_pitch_skips_special_notes() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
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
        &[TransformOp::InvertPitch {
            pivot: "C4".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    assert_eq!(cell.note.as_deref(), Some("---"));
}

#[test]
fn invert_pitch_invalid_pivot() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::InvertPitch {
            pivot: "invalid".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn quantize_pitch_snaps_to_scale() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    // C# is not in C major scale, should snap to C or D
    let mut cell = Cell {
        note: Some("C#4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::QuantizePitch {
            scale: QuantizeScale::Major,
            root: "C".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // C# should snap to C (ties snap down)
    assert_eq!(cell.note.as_deref(), Some("C4"));
}

#[test]
fn quantize_pitch_chromatic_is_noop() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C#4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::QuantizePitch {
            scale: QuantizeScale::Chromatic,
            root: "C".to_string(),
        }],
        &ctx,
    )
    .unwrap();

    // Chromatic scale includes all notes, so no change
    assert_eq!(cell.note.as_deref(), Some("C#4"));
}

#[test]
fn quantize_pitch_invalid_root() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: None,
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::QuantizePitch {
            scale: QuantizeScale::Major,
            root: "X".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn ratchet_applies_retrigger_effect() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    // Run multiple times to get one that applies
    let mut applied = false;
    for seed in 0..100 {
        let mut test_cell = cell.clone();
        let test_ctx = TransformContext {
            seed,
            pattern_name: "p0",
            key: (0, 0),
        };
        apply_transforms(
            &mut test_cell,
            &[TransformOp::Ratchet {
                divisions: 4,
                seed_salt: "ratchet".to_string(),
            }],
            &test_ctx,
        )
        .unwrap();
        if test_cell.effect_name.is_some() {
            applied = true;
            assert_eq!(test_cell.effect_name.as_deref(), Some("retrig"));
            assert!(test_cell.effect_xy.is_some());
            break;
        }
    }
    assert!(applied, "ratchet should apply at least once in 100 tries");
}

#[test]
fn ratchet_invalid_divisions() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::Ratchet {
            divisions: 17, // Invalid: > 16
            seed_salt: "ratchet".to_string(),
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn arpeggiate_applies_arpeggio_effect() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    apply_transforms(
        &mut cell,
        &[TransformOp::Arpeggiate {
            semitones_up: 4,
            semitones_down: 7,
        }],
        &ctx,
    )
    .unwrap();

    assert_eq!(cell.effect_name.as_deref(), Some("arpeggio"));
    assert_eq!(cell.effect_xy, Some([4, 7]));
}

#[test]
fn arpeggiate_invalid_semitones() {
    let ctx = TransformContext {
        seed: 42,
        pattern_name: "p0",
        key: (0, 0),
    };
    let mut cell = Cell {
        note: Some("C4".to_string()),
        inst: Some(0),
        vol: Some(64),
        effect: None,
        param: None,
        effect_name: None,
        effect_xy: None,
    };

    let result = apply_transforms(
        &mut cell,
        &[TransformOp::Arpeggiate {
            semitones_up: 16, // Invalid: > 15
            semitones_down: 0,
        }],
        &ctx,
    );

    assert!(result.is_err());
}

#[test]
fn new_operators_are_deterministic() {
    // Test that all new operators produce deterministic output
    let mut params = base_params();
    params.channels = 2;

    // Create a complex pattern using multiple new operators
    params.patterns.insert(
        "p0".to_string(),
        ComposePattern {
            rows: Some(16),
            bars: None,
            timebase: None,
            program: PatternExpr::Transform {
                ops: vec![TransformOp::Arpeggiate {
                    semitones_up: 3,
                    semitones_down: 0,
                }],
                body: Box::new(PatternExpr::Reverse {
                    len_rows: 16,
                    body: Box::new(PatternExpr::Filter {
                        criteria: FilterCriteria {
                            min_row: Some(4),
                            max_row: Some(12),
                            ..Default::default()
                        },
                        body: Box::new(PatternExpr::Emit {
                            at: TimeExpr::Range {
                                start: 0,
                                step: 2,
                                count: 8,
                            },
                            cell: CellTemplate {
                                channel: ChannelRef::Index(0),
                                note: Some("C4".to_string()),
                                inst: Some(InstrumentRef::Index(0)),
                                ..Default::default()
                            },
                        }),
                    }),
                }),
            },
            data: None,
            notes: None,
        },
    );

    let first = expand_compose(&params, 42).unwrap();
    let second = expand_compose(&params, 42).unwrap();
    let third = expand_compose(&params, 42).unwrap();

    assert_eq!(first, second);
    assert_eq!(second, third);
}
