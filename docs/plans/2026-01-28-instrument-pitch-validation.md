# Instrument Pitch Validation + Auto-Correction Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add pitch deviation measurement (in cents) to XM/IT instrument export, surface it in the loop report, and fix the IT c5_speed truncation bug that causes unnecessary pitch error.

**Architecture:** Two new pure functions (`xm_pitch_deviation_cents`, `it_pitch_deviation_cents`) in the existing `pitch.rs` module simulate what each tracker format will actually play back and compare it to the intended sample rate. A new `pitch_deviation_cents` field on `MusicInstrumentLoopReport` surfaces the result. The IT `calculate_c5_speed_for_base_note` function is fixed to round instead of truncate.

**Tech Stack:** Rust, speccade-backend-music crate, serde for report serialization.

---

### Task 1: Add `xm_pitch_deviation_cents` function (TDD)

**Files:**
- Modify: `crates/speccade-backend-music/src/note/pitch.rs` (add function)
- Modify: `crates/speccade-backend-music/src/note/mod.rs` (re-export)
- Modify: `crates/speccade-backend-music/src/note/tests.rs` (add tests)

**Step 1: Write the failing tests**

Add to the bottom of `crates/speccade-backend-music/src/note/tests.rs`:

```rust
// =========================================================================
// Tests for pitch deviation measurement
// =========================================================================

#[test]
fn test_xm_pitch_deviation_cents_at_reference() {
    // At XM reference rate (8363 Hz) with MIDI 60, correction is (0, 0).
    // Deviation should be exactly 0.
    let cents = xm_pitch_deviation_cents(8363, 60, 0, 0);
    assert!(
        cents.abs() < 0.001,
        "expected ~0 cents at reference, got {:.4}",
        cents
    );
}

#[test]
fn test_xm_pitch_deviation_cents_22050_midi_60() {
    // Standard case: 22050 Hz, MIDI 60
    let (finetune, relative_note) = calculate_xm_pitch_correction(22050, 60);
    let cents = xm_pitch_deviation_cents(22050, 60, finetune, relative_note);
    assert!(
        cents.abs() < 2.0,
        "expected < 2 cents deviation, got {:.4}",
        cents
    );
}

#[test]
fn test_xm_pitch_deviation_cents_sweep() {
    // All standard sample rates and a range of base notes should be < 1 cent.
    let rates = [8363u32, 11025, 16000, 22050, 44100, 48000];
    let notes = [36u8, 48, 60, 72, 84];
    for &sr in &rates {
        for &midi in &notes {
            let (ft, rn) = calculate_xm_pitch_correction(sr, midi);
            let cents = xm_pitch_deviation_cents(sr, midi, ft, rn);
            assert!(
                cents.abs() < 1.0,
                "XM deviation too large: sr={}, midi={}, cents={:.4}",
                sr, midi, cents
            );
        }
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p speccade-backend-music xm_pitch_deviation`
Expected: compilation error — `xm_pitch_deviation_cents` not found.

**Step 3: Implement `xm_pitch_deviation_cents`**

Add to the bottom of `crates/speccade-backend-music/src/note/pitch.rs`:

```rust
/// Compute pitch deviation in cents for XM pitch correction parameters.
///
/// Simulates the XM playback engine's frequency calculation and compares
/// it to the sample's native rate. Returns deviation in cents (positive = sharp).
pub fn xm_pitch_deviation_cents(
    sample_rate: u32,
    base_midi_note: u8,
    finetune: i8,
    relative_note: i8,
) -> f64 {
    let base_xm_note_0indexed = base_midi_note as f64 - 12.0;
    let semitones = (base_xm_note_0indexed + relative_note as f64 - 48.0)
        + (finetune as f64 / 128.0);
    let playback_rate = XM_BASE_FREQ * 2.0_f64.powf(semitones / 12.0);
    1200.0 * (playback_rate / sample_rate as f64).log2()
}
```

Add to the `pub use pitch::` block in `crates/speccade-backend-music/src/note/mod.rs`:

```rust
pub use pitch::{
    calculate_c5_speed, calculate_c5_speed_for_base_note, calculate_pitch_correction,
    calculate_xm_pitch_correction, xm_pitch_deviation_cents,
};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p speccade-backend-music xm_pitch_deviation`
Expected: all 3 new tests PASS.

**Step 5: Commit**

```bash
git add crates/speccade-backend-music/src/note/pitch.rs crates/speccade-backend-music/src/note/mod.rs crates/speccade-backend-music/src/note/tests.rs
git commit -m "feat(music): add xm_pitch_deviation_cents validation function"
```

---

### Task 2: Add `it_pitch_deviation_cents` function (TDD)

**Files:**
- Modify: `crates/speccade-backend-music/src/note/pitch.rs` (add function)
- Modify: `crates/speccade-backend-music/src/note/mod.rs` (re-export)
- Modify: `crates/speccade-backend-music/src/note/tests.rs` (add tests)

**Step 1: Write the failing tests**

Add to `crates/speccade-backend-music/src/note/tests.rs`:

```rust
#[test]
fn test_it_pitch_deviation_cents_at_reference() {
    // MIDI 72 at 22050 Hz → c5_speed = 22050 (no adjustment). Deviation = 0.
    let cents = it_pitch_deviation_cents(22050, 72, 22050);
    assert!(
        cents.abs() < 0.001,
        "expected ~0 cents at reference, got {:.4}",
        cents
    );
}

#[test]
fn test_it_pitch_deviation_cents_22050_midi_60() {
    let c5_speed = calculate_c5_speed_for_base_note(22050, 60);
    let cents = it_pitch_deviation_cents(22050, 60, c5_speed);
    assert!(
        cents.abs() < 2.0,
        "expected < 2 cents deviation, got {:.4}",
        cents
    );
}

#[test]
fn test_it_pitch_deviation_cents_sweep() {
    let rates = [8363u32, 11025, 16000, 22050, 44100, 48000];
    let notes = [36u8, 48, 60, 72, 84];
    for &sr in &rates {
        for &midi in &notes {
            let c5 = calculate_c5_speed_for_base_note(sr, midi);
            let cents = it_pitch_deviation_cents(sr, midi, c5);
            assert!(
                cents.abs() < 1.0,
                "IT deviation too large: sr={}, midi={}, c5_speed={}, cents={:.4}",
                sr, midi, c5, cents
            );
        }
    }
}
```

**Step 2: Run tests to verify they fail**

Run: `cargo test -p speccade-backend-music it_pitch_deviation`
Expected: compilation error — `it_pitch_deviation_cents` not found.

**Step 3: Implement `it_pitch_deviation_cents`**

Add to `crates/speccade-backend-music/src/note/pitch.rs`:

```rust
/// Compute pitch deviation in cents for IT c5_speed parameter.
///
/// Simulates the IT playback engine's frequency calculation and compares
/// it to the sample's native rate. Returns deviation in cents (positive = sharp).
pub fn it_pitch_deviation_cents(sample_rate: u32, base_midi_note: u8, c5_speed: u32) -> f64 {
    let base_it_note = base_midi_note as f64 - 12.0;
    let playback_rate = c5_speed as f64 * 2.0_f64.powf((base_it_note - 60.0) / 12.0);
    1200.0 * (playback_rate / sample_rate as f64).log2()
}
```

Add `it_pitch_deviation_cents` to the re-export in `crates/speccade-backend-music/src/note/mod.rs`:

```rust
pub use pitch::{
    calculate_c5_speed, calculate_c5_speed_for_base_note, calculate_pitch_correction,
    calculate_xm_pitch_correction, it_pitch_deviation_cents, xm_pitch_deviation_cents,
};
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p speccade-backend-music it_pitch_deviation`
Expected: all 3 new tests PASS.

**Step 5: Commit**

```bash
git add crates/speccade-backend-music/src/note/pitch.rs crates/speccade-backend-music/src/note/mod.rs crates/speccade-backend-music/src/note/tests.rs
git commit -m "feat(music): add it_pitch_deviation_cents validation function"
```

---

### Task 3: Fix IT c5_speed truncation (round instead of truncate)

**Files:**
- Modify: `crates/speccade-backend-music/src/note/pitch.rs:104` (change `as u32` to `.round() as u32`)

**Step 1: Write the failing test**

Add to `crates/speccade-backend-music/src/note/tests.rs`:

```rust
#[test]
fn test_c5_speed_rounding_reduces_error() {
    // Non-integer-octave base notes produce fractional c5_speed.
    // For MIDI 65 (F4) at 22050 Hz:
    //   semitone_diff = 60 - (65-12) = 60 - 53 = 7
    //   c5_speed = 22050 * 2^(7/12) = 22050 * 1.4983 = 33038.4...
    //   Truncation → 33038 (error = -0.4), Rounding → 33038 (error = -0.4)
    // For MIDI 63 (Eb4) at 22050 Hz:
    //   semitone_diff = 60 - (63-12) = 60 - 51 = 9
    //   c5_speed = 22050 * 2^(9/12) = 22050 * 1.6818 = 37083.6...
    //   Truncation → 37083 (error = -0.6), Rounding → 37084 (error = +0.4)
    // With rounding, the max error from the cast is 0.5 instead of 1.0.
    let c5_speed = calculate_c5_speed_for_base_note(22050, 63);
    let cents = it_pitch_deviation_cents(22050, 63, c5_speed);
    assert!(
        cents.abs() < 0.05,
        "c5_speed rounding should keep error < 0.05 cents, got {:.4}",
        cents
    );
}
```

**Step 2: Run test (may or may not fail depending on current truncation behavior)**

Run: `cargo test -p speccade-backend-music test_c5_speed_rounding`
Note: This test _might_ already pass since the error is small, but the fix is still correct to apply.

**Step 3: Apply the fix**

In `crates/speccade-backend-music/src/note/pitch.rs`, line 104, change:

```rust
    (sample_rate as f64 * 2.0_f64.powf(semitone_diff as f64 / 12.0)) as u32
```

to:

```rust
    (sample_rate as f64 * 2.0_f64.powf(semitone_diff as f64 / 12.0)).round() as u32
```

**Step 4: Run full test suite to check for regressions**

Run: `cargo test -p speccade-backend-music`
Expected: ALL tests pass. The existing `assert_eq!` tests for c5_speed (MIDI 60 → 44100, MIDI 72 → 22050, MIDI 48 → 88200, MIDI 36 → 384000) all use exact-octave intervals where `2^(n/12)` is an integer, so rounding vs truncating produces the same value.

**Step 5: Commit**

```bash
git add crates/speccade-backend-music/src/note/pitch.rs crates/speccade-backend-music/src/note/tests.rs
git commit -m "fix(music): round IT c5_speed instead of truncating to reduce pitch error"
```

---

### Task 4: Add `pitch_deviation_cents` field to `MusicInstrumentLoopReport`

**Files:**
- Modify: `crates/speccade-backend-music/src/generate/mod.rs:118-159` (add field to struct)

**Step 1: Add the field**

In `crates/speccade-backend-music/src/generate/mod.rs`, add after the `dc_removed_mean` field (line 158):

```rust
    /// Pitch deviation in cents after format-specific correction.
    /// Positive = sharp, negative = flat. Ideally < 1 cent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pitch_deviation_cents: Option<f64>,
```

**Step 2: Fix the compile error in `instrument_baking.rs`**

In `crates/speccade-backend-music/src/generate/instrument_baking.rs`, the `MusicInstrumentLoopReport` construction (around line 451) needs the new field. Add after `dc_removed_mean`:

```rust
        pitch_deviation_cents: None,
```

This is `None` here because baking doesn't know the format-specific correction yet — the generators will fill it in.

**Step 3: Verify compilation**

Run: `cargo build -p speccade-backend-music`
Expected: compiles cleanly.

**Step 4: Commit**

```bash
git add crates/speccade-backend-music/src/generate/mod.rs crates/speccade-backend-music/src/generate/instrument_baking.rs
git commit -m "feat(music): add pitch_deviation_cents field to instrument loop report"
```

---

### Task 5: Populate pitch deviation in XM generator

**Files:**
- Modify: `crates/speccade-backend-music/src/xm_gen/mod.rs:172-209`

**Step 1: Add deviation calculation**

In `generate_xm_instrument` (around line 183, after the `calculate_xm_pitch_correction` call), add an import and calculation. Add to the imports at the top of the file:

```rust
use crate::note::xm_pitch_deviation_cents;
```

Then after `let (finetune, relative_note) = ...;` (line 183), add:

```rust
    let pitch_cents = xm_pitch_deviation_cents(baked.sample_rate, baked.base_midi, finetune, relative_note);
```

Then before `Ok((xm_instr, loop_report))` (line 208), mutate the report:

```rust
    let mut loop_report = loop_report;
    loop_report.pitch_deviation_cents = Some(pitch_cents);
```

**Step 2: Verify compilation and tests**

Run: `cargo test -p speccade-backend-music`
Expected: all pass.

**Step 3: Commit**

```bash
git add crates/speccade-backend-music/src/xm_gen/mod.rs
git commit -m "feat(music): populate pitch_deviation_cents in XM instrument export"
```

---

### Task 6: Populate pitch deviation in IT generator

**Files:**
- Modify: `crates/speccade-backend-music/src/it_gen/instrument.rs`

**Step 1: Add deviation calculation**

Add import:

```rust
use crate::note::it_pitch_deviation_cents;
```

After `let c5_speed = ...;` (line 26), add:

```rust
    let pitch_cents = it_pitch_deviation_cents(baked.sample_rate, baked.base_midi, c5_speed);
```

Before the final `Ok(...)`, mutate the report:

```rust
    let mut loop_report = loop_report;
    loop_report.pitch_deviation_cents = Some(pitch_cents);
```

**Step 2: Verify compilation and tests**

Run: `cargo test -p speccade-backend-music`
Expected: all pass.

**Step 3: Commit**

```bash
git add crates/speccade-backend-music/src/it_gen/instrument.rs
git commit -m "feat(music): populate pitch_deviation_cents in IT instrument export"
```

---

### Task 7: Final verification

**Step 1: Run the full crate test suite**

Run: `cargo test -p speccade-backend-music`
Expected: all tests pass, including the 6 new pitch deviation tests.

**Step 2: Run workspace build**

Run: `cargo build`
Expected: clean build, no warnings related to our changes.

**Step 3: Verify report output (if a test spec is available)**

If there's a way to generate a module from an existing spec (e.g., `cargo run -p speccade-cli -- generate ...`), run it and inspect the JSON loop report for the `pitch_deviation_cents` field. Values should be < 1 cent for standard instrument configurations.

---

## Summary of files changed

| File | Change |
|------|--------|
| `crates/speccade-backend-music/src/note/pitch.rs` | Add `xm_pitch_deviation_cents`, `it_pitch_deviation_cents`; fix `.round()` in c5_speed |
| `crates/speccade-backend-music/src/note/mod.rs` | Re-export new functions |
| `crates/speccade-backend-music/src/note/tests.rs` | 7 new tests |
| `crates/speccade-backend-music/src/generate/mod.rs` | Add `pitch_deviation_cents` field to report |
| `crates/speccade-backend-music/src/generate/instrument_baking.rs` | Initialize new field to `None` |
| `crates/speccade-backend-music/src/xm_gen/mod.rs` | Compute and store XM pitch deviation |
| `crates/speccade-backend-music/src/it_gen/instrument.rs` | Compute and store IT pitch deviation |
