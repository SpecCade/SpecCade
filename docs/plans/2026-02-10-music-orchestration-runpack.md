# Music Orchestration Runpack

This runpack is for long-running XM/IT confidence work with repeatable test gates and optional staged checkpoint commits.

## Goal

- Keep music work moving in cycles with deterministic gates.
- Catch regressions in spec semantics, backend generation, CLI dispatch, and integration parity.
- Support unattended runs that produce logs per cycle.

## Script

- Path: `scripts/music-runpack.ps1`

## What Each Cycle Runs

1. `cargo test -p speccade-spec validation::tests::music_tests`
2. `cargo test -p speccade-backend-music`
3. `cargo test -p speccade-cli`
4. `cargo test -p speccade-tests --test music_parity`
5. `cargo test -p speccade-tests --test compose`
6. Optional deep gate: `cargo test -p speccade-tests --test golden_hash_verification -- --nocapture`

## Usage

```powershell
# One cycle, fast gate
./scripts/music-runpack.ps1

# Multi-hour run: 12 cycles, 5 minute pause between cycles
./scripts/music-runpack.ps1 -Cycles 12 -SleepSeconds 300

# Include deep gate in every cycle
./scripts/music-runpack.ps1 -Cycles 4 -DeepGate

# Commit staged changes after each successful cycle
./scripts/music-runpack.ps1 -Cycles 6 -CheckpointStaged -CheckpointPrefix "music hardening"
```

## Checkpoint Model

- `-CheckpointStaged` commits only what is already staged.
- The script never stages files automatically.
- This avoids accidentally committing unrelated local edits.

## Logs

- Logs are written under `target/runpack/music/<timestamp>/`.
- One log file per cycle: `cycle-01.log`, `cycle-02.log`, etc.

## Suggested Operator Loop

1. Stage the intended batch of changes for the current workstream.
2. Run 1+ cycles of the runpack.
3. If all gates are green, let `-CheckpointStaged` create the commit.
4. Repeat with the next staged batch.
