//! Pattern expression evaluation logic.

use rand::Rng;

use speccade_spec::recipe::music::{ComposePattern, MergePolicy, PatternExpr, PitchSeqKind};

use super::error::ExpandError;
use super::expander_context::{Expander, MAX_CELLS_PER_PATTERN, MAX_RECURSION_DEPTH};
use super::merge::{
    apply_transforms, insert_cell_merge, shift_map, Cell, CellMap, TransformContext,
};
use super::seq::{PitchSeqAccessor, SeqAccessor};
use super::utils::rng_for;

impl<'a> Expander<'a> {
    pub fn expand_pattern(&mut self, pattern: &ComposePattern) -> Result<CellMap, ExpandError> {
        let mut map = self.eval_expr(&pattern.program, 0)?;

        // Merge hand-authored data/notes on top (last_wins).
        let manual = self.manual_cells(pattern)?;
        self.merge_maps(&mut map, manual, MergePolicy::LastWins)?;

        if map.len() > MAX_CELLS_PER_PATTERN {
            return Err(ExpandError::CellLimit {
                pattern: self.pattern_name.to_string(),
                max: MAX_CELLS_PER_PATTERN,
            });
        }

        Ok(map)
    }

    fn manual_cells(&self, pattern: &ComposePattern) -> Result<CellMap, ExpandError> {
        let mut map = CellMap::new();

        if let Some(ref data) = pattern.data {
            for note in data {
                let channel = note.channel.unwrap_or(0);
                let key = (note.row as i32, channel);
                let cell = Cell::from_pattern_note(note);
                insert_cell_merge(
                    &mut map,
                    key,
                    cell,
                    MergePolicy::MergeFields,
                    self.pattern_name,
                )?;
            }
        }

        if let Some(ref notes) = pattern.notes {
            for (channel_str, items) in notes {
                let channel = channel_str
                    .parse::<u8>()
                    .map_err(|_| ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!("invalid channel key '{}'", channel_str),
                    })?;
                for note in items {
                    let key = (note.row as i32, channel);
                    let cell = Cell::from_pattern_note(note);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
            }
        }

        Ok(map)
    }

    pub(super) fn eval_expr(
        &mut self,
        expr: &PatternExpr,
        depth: usize,
    ) -> Result<CellMap, ExpandError> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(ExpandError::RecursionLimit {
                pattern: self.pattern_name.to_string(),
                max: MAX_RECURSION_DEPTH,
            });
        }

        match expr {
            PatternExpr::Stack { merge, parts } => {
                let mut map = CellMap::new();
                for part in parts {
                    let part_map = self.eval_expr(part, depth + 1)?;
                    self.merge_maps(&mut map, part_map, *merge)?;
                    if map.len() > MAX_CELLS_PER_PATTERN {
                        return Err(ExpandError::CellLimit {
                            pattern: self.pattern_name.to_string(),
                            max: MAX_CELLS_PER_PATTERN,
                        });
                    }
                }
                Ok(map)
            }
            PatternExpr::Concat { parts } => {
                let mut map = CellMap::new();
                let mut offset = 0i32;
                let mut total_len = 0i32;
                for part in parts {
                    if part.len_rows == 0 {
                        return Err(ExpandError::InvalidExpr {
                            pattern: self.pattern_name.to_string(),
                            message: "concat parts must have len_rows > 0".to_string(),
                        });
                    }
                    total_len += part.len_rows as i32;
                }
                if total_len > 0 && total_len > self.pattern_rows() as i32 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "concat length {} exceeds pattern rows {}",
                            total_len,
                            self.pattern_rows()
                        ),
                    });
                }
                for part in parts {
                    let part_len = part.len_rows as i32;
                    let mut part_map = self.eval_expr(&part.body, depth + 1)?;
                    // Ensure part stays within its declared window.
                    for (key, _) in part_map.iter() {
                        let row = key.0;
                        if row < 0 || row >= part_len {
                            return Err(ExpandError::CellOutOfBounds {
                                pattern: self.pattern_name.to_string(),
                                row,
                                channel: key.1,
                            });
                        }
                    }
                    shift_map(&mut part_map, offset);
                    self.merge_maps(&mut map, part_map, MergePolicy::Error)?;
                    offset += part_len;
                }
                Ok(map)
            }
            PatternExpr::Repeat {
                times,
                len_rows,
                body,
            } => {
                if *len_rows == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "repeat len_rows must be > 0".to_string(),
                    });
                }
                let total_len = (*times as i32) * (*len_rows as i32);
                if total_len > 0 && total_len > self.pattern_rows() as i32 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: format!(
                            "repeat length {} exceeds pattern rows {}",
                            total_len,
                            self.pattern_rows()
                        ),
                    });
                }
                let base = self.eval_expr(body, depth + 1)?;
                for (key, _) in base.iter() {
                    let row = key.0;
                    if row < 0 || row >= *len_rows as i32 {
                        return Err(ExpandError::CellOutOfBounds {
                            pattern: self.pattern_name.to_string(),
                            row,
                            channel: key.1,
                        });
                    }
                }
                let mut map = CellMap::new();
                for i in 0..*times {
                    let mut part = base.clone();
                    shift_map(&mut part, (i as i32) * (*len_rows as i32));
                    self.merge_maps(&mut map, part, MergePolicy::Error)?;
                }
                Ok(map)
            }
            PatternExpr::Shift { rows, body } => {
                let mut map = self.eval_expr(body, depth + 1)?;
                shift_map(&mut map, *rows);
                Ok(map)
            }
            PatternExpr::Slice { start, len, body } => {
                if *len < 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "slice len must be >= 0".to_string(),
                    });
                }
                let map = self.eval_expr(body, depth + 1)?;
                let end = start.saturating_add(*len);
                let filtered = map
                    .into_iter()
                    .filter(|((row, _), _)| *row >= *start && *row < end)
                    .collect();
                Ok(filtered)
            }
            PatternExpr::Ref { name } => {
                if self.ref_stack.contains(name) {
                    let mut cycle = self.ref_stack.join(" -> ");
                    if !cycle.is_empty() {
                        cycle.push_str(" -> ");
                    }
                    cycle.push_str(name);
                    return Err(ExpandError::RefCycle { cycle });
                }
                let expr = self.defs.get(name).ok_or_else(|| ExpandError::UnknownRef {
                    pattern: self.pattern_name.to_string(),
                    name: name.clone(),
                })?;
                self.ref_stack.push(name.clone());
                let result = self.eval_expr(expr, depth + 1);
                self.ref_stack.pop();
                result
            }
            PatternExpr::Emit { at, cell } => {
                let positions = self.eval_time_expr(at)?;
                let mut map = CellMap::new();
                for row in positions {
                    let channel = self.resolve_channel(&cell.channel, row)?;
                    let inst = cell
                        .inst
                        .as_ref()
                        .map(|inst_ref| self.resolve_instrument(inst_ref))
                        .transpose()?;
                    let key = (row, channel);
                    let cell = Cell::from_template(cell, inst);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
                Ok(map)
            }
            PatternExpr::EmitSeq {
                at,
                cell,
                note_seq,
                pitch_seq,
                inst_seq,
                vol_seq,
                effect_seq,
                param_seq,
                effect_name_seq,
                effect_xy_seq,
            } => {
                let positions = self.eval_time_expr(at)?;
                let mut map = CellMap::new();
                let count = positions.len();

                if note_seq.is_some() && pitch_seq.is_some() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "emit_seq must specify at most one of note_seq or pitch_seq"
                            .to_string(),
                    });
                }

                let note_seq = SeqAccessor::new(note_seq.as_ref(), count, self.pattern_name)?;
                let pitch_seq =
                    PitchSeqAccessor::new(pitch_seq.as_ref(), count, self.pattern_name)?;
                let inst_seq = SeqAccessor::new(inst_seq.as_ref(), count, self.pattern_name)?;
                let vol_seq = SeqAccessor::new(vol_seq.as_ref(), count, self.pattern_name)?;
                let effect_seq = SeqAccessor::new(effect_seq.as_ref(), count, self.pattern_name)?;
                let param_seq = SeqAccessor::new(param_seq.as_ref(), count, self.pattern_name)?;
                let effect_name_seq =
                    SeqAccessor::new(effect_name_seq.as_ref(), count, self.pattern_name)?;
                let effect_xy_seq =
                    SeqAccessor::new(effect_xy_seq.as_ref(), count, self.pattern_name)?;

                let key_ctx = pitch_seq
                    .seq()
                    .filter(|seq| seq.kind == PitchSeqKind::ScaleDegree)
                    .map(|_| self.key_context())
                    .transpose()?;
                let chord_ctx = pitch_seq
                    .seq()
                    .filter(|seq| seq.kind == PitchSeqKind::ChordTone)
                    .map(|_| self.chord_context())
                    .transpose()?;

                let base_inst = cell
                    .inst
                    .as_ref()
                    .map(|inst_ref| self.resolve_instrument(inst_ref))
                    .transpose()?;
                for (idx, row) in positions.into_iter().enumerate() {
                    let channel = self.resolve_channel(&cell.channel, row)?;
                    let mut cell = Cell::from_template(cell, base_inst);
                    if let Some(value) = note_seq.value(idx)? {
                        cell.note = Some(value);
                    }
                    if let Some(seq) = pitch_seq.seq() {
                        if let Some(value) = pitch_seq.value(idx)? {
                            let note = match seq.kind {
                                PitchSeqKind::ScaleDegree => {
                                    let key_ctx = key_ctx.as_ref().ok_or_else(|| {
                                        ExpandError::InvalidExpr {
                                            pattern: self.pattern_name.to_string(),
                                            message: "pitch_seq scale_degree requires harmony.key"
                                                .to_string(),
                                        }
                                    })?;
                                    self.scale_degree_note_name(
                                        key_ctx,
                                        &value,
                                        seq.octave,
                                        seq.allow_accidentals,
                                    )?
                                }
                                PitchSeqKind::ChordTone => {
                                    let chord_ctx = chord_ctx.as_ref().ok_or_else(|| {
                                        ExpandError::InvalidExpr {
                                            pattern: self.pattern_name.to_string(),
                                            message: "pitch_seq chord_tone requires harmony.chords"
                                                .to_string(),
                                        }
                                    })?;
                                    let chord = self.select_chord(chord_ctx, row)?;
                                    self.chord_tone_note_name(chord, &value, seq.octave)?
                                }
                            };
                            cell.note = Some(note);
                        }
                    }
                    if let Some(value) = inst_seq.value(idx)? {
                        cell.inst = Some(self.resolve_instrument(&value)?);
                    }
                    if let Some(value) = vol_seq.value(idx)? {
                        cell.vol = Some(value);
                    }
                    if let Some(value) = effect_seq.value(idx)? {
                        cell.effect = Some(value);
                    }
                    if let Some(value) = param_seq.value(idx)? {
                        cell.param = Some(value);
                    }
                    if let Some(value) = effect_name_seq.value(idx)? {
                        cell.effect_name = Some(value);
                    }
                    if let Some(value) = effect_xy_seq.value(idx)? {
                        cell.effect_xy = Some(value);
                    }

                    let key = (row, channel);
                    insert_cell_merge(
                        &mut map,
                        key,
                        cell,
                        MergePolicy::MergeFields,
                        self.pattern_name,
                    )?;
                }
                Ok(map)
            }
            PatternExpr::Transform { ops, body } => {
                let mut map = self.eval_expr(body, depth + 1)?;
                for (key, cell) in map.iter_mut() {
                    let ctx = TransformContext {
                        seed: self.seed,
                        pattern_name: self.pattern_name,
                        key: *key,
                    };
                    apply_transforms(cell, ops, &ctx)?;
                }
                Ok(map)
            }
            PatternExpr::Prob {
                p_permille,
                seed_salt,
                body,
            } => {
                if *p_permille > 1000 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "prob p_permille must be <= 1000".to_string(),
                    });
                }
                let map = self.eval_expr(body, depth + 1)?;
                if *p_permille == 0 {
                    return Ok(map);
                }
                if *p_permille >= 1000 {
                    return Ok(CellMap::new());
                }
                let mut rng = rng_for(self.seed, self.pattern_name, seed_salt);
                let filtered = map
                    .into_iter()
                    .filter(|_| rng.gen_range(0u16..1000u16) >= *p_permille)
                    .collect();
                Ok(filtered)
            }
            PatternExpr::Choose { seed_salt, choices } => {
                if choices.is_empty() {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "choose requires at least one choice".to_string(),
                    });
                }
                let total_weight: u32 = choices.iter().map(|c| c.weight).sum();
                if total_weight == 0 {
                    return Err(ExpandError::InvalidExpr {
                        pattern: self.pattern_name.to_string(),
                        message: "choose weights must sum to > 0".to_string(),
                    });
                }
                let mut rng = rng_for(self.seed, self.pattern_name, seed_salt);
                let mut roll = rng.gen_range(0u32..total_weight);
                for choice in choices {
                    if roll < choice.weight {
                        return self.eval_expr(&choice.body, depth + 1);
                    }
                    roll = roll.saturating_sub(choice.weight);
                }
                // Fallback (shouldn't happen).
                self.eval_expr(&choices[0].body, depth + 1)
            }
        }
    }

    pub(super) fn merge_maps(
        &self,
        base: &mut CellMap,
        incoming: CellMap,
        policy: MergePolicy,
    ) -> Result<(), ExpandError> {
        for (key, cell) in incoming {
            insert_cell_merge(base, key, cell, policy, self.pattern_name)?;
        }
        Ok(())
    }
}
