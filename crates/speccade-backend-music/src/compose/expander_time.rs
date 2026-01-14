//! Time expression evaluation logic.

use speccade_spec::recipe::music::TimeExpr;

use super::error::ExpandError;
use super::expander_context::{Expander, MAX_PATTERN_STRING_LEN, MAX_TIME_LIST_LEN};
use super::utils::{bjorklund, modulo};

impl<'a> Expander<'a> {
    pub(super) fn eval_time_expr(&self, expr: &TimeExpr) -> Result<Vec<i32>, ExpandError> {
        match expr {
            TimeExpr::Range { start, step, count } => {
                if *step == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "range step must be non-zero".to_string(),
                    });
                }
                let count = *count as usize;
                if count > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let mut rows = Vec::with_capacity(count);
                for i in 0..count {
                    let row = *start as i64 + (*step as i64) * (i as i64);
                    rows.push(row as i32);
                }
                Ok(rows)
            }
            TimeExpr::List { rows } => {
                if rows.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                Ok(rows.clone())
            }
            TimeExpr::BeatList { beats } => {
                if beats.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let mut rows = Vec::with_capacity(beats.len());
                for pos in beats {
                    rows.push(self.beat_pos_to_row(pos)? as i32);
                }
                Ok(rows)
            }
            TimeExpr::BeatRange { start, step, count } => {
                let count = *count as usize;
                if count > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }

                let start_row = self.beat_pos_to_row(start)?;
                let delta_rows = self.beat_delta_to_rows(step)?;
                if delta_rows == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "beat_range step must be non-zero".to_string(),
                    });
                }

                let mut rows = Vec::with_capacity(count);
                for i in 0..count {
                    let offset = delta_rows.checked_mul(i as i64).ok_or_else(|| {
                        ExpandError::InvalidTime {
                            pattern: self.pattern_name.to_string(),
                            message: "beat_range overflow".to_string(),
                        }
                    })?;
                    let row =
                        start_row
                            .checked_add(offset)
                            .ok_or_else(|| ExpandError::InvalidTime {
                                pattern: self.pattern_name.to_string(),
                                message: "beat_range overflow".to_string(),
                            })?;
                    if row < 0 || row >= self.pattern_rows as i64 {
                        return Err(ExpandError::InvalidTime {
                            pattern: self.pattern_name.to_string(),
                            message: format!(
                                "beat_range produced row {} outside pattern rows {}",
                                row, self.pattern_rows
                            ),
                        });
                    }
                    rows.push(row as i32);
                }
                Ok(rows)
            }
            TimeExpr::Euclid {
                pulses,
                steps,
                rotate,
                stride,
                offset,
            } => {
                let steps_usize = *steps as usize;
                let pulses_usize = *pulses as usize;
                if *steps == 0 {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "euclid steps must be > 0".to_string(),
                    });
                }
                if pulses_usize > steps_usize {
                    return Err(ExpandError::InvalidTime {
                        pattern: self.pattern_name.to_string(),
                        message: "euclid pulses must be <= steps".to_string(),
                    });
                }
                if steps_usize > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                let pattern = bjorklund(steps_usize, pulses_usize);
                let rot = modulo(*rotate, *steps as i32);
                let mut rows = Vec::with_capacity(pulses_usize);
                for (idx, hit) in pattern.iter().enumerate() {
                    if *hit {
                        let rotated = (idx as i32 + rot) % (*steps as i32);
                        let row = rotated * *stride + *offset;
                        rows.push(row);
                    }
                }
                Ok(rows)
            }
            TimeExpr::Pattern {
                pattern,
                stride,
                offset,
            } => {
                if pattern.len() > MAX_PATTERN_STRING_LEN {
                    return Err(ExpandError::PatternStringLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_PATTERN_STRING_LEN,
                    });
                }
                let mut rows = Vec::new();
                let mut idx = 0i32;
                for ch in pattern.chars() {
                    match ch {
                        'x' | 'X' => {
                            let row = idx * *stride + *offset;
                            rows.push(row);
                            idx += 1;
                        }
                        '.' => {
                            idx += 1;
                        }
                        c if c.is_whitespace() => {}
                        _ => {
                            return Err(ExpandError::InvalidTime {
                                pattern: self.pattern_name.to_string(),
                                message: format!("invalid pattern character '{}'", ch),
                            })
                        }
                    }
                }
                if rows.len() > MAX_TIME_LIST_LEN {
                    return Err(ExpandError::TimeListLimit {
                        pattern: self.pattern_name.to_string(),
                        max: MAX_TIME_LIST_LEN,
                    });
                }
                Ok(rows)
            }
        }
    }
}
