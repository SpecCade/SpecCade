# SpecCade TODOs (Curated Backlog)

This is a curated backlog of *future work*. For what’s implemented today, see `PARITY_MATRIX.md` and `docs/spec-reference/`.

## Music (Compose / Pattern IR)

- Add/expand snapshot tests for `docs/examples/music/*.expanded.params.json` via `speccade expand`.
- Extend Pattern IR operators and hard limits where needed (RFC-0003 / RFC-0004), keeping determinism and reviewability.
- Improve `speccade expand` UX for review workflows (stable formatting, optional file output, diff-friendly mode).
- Tighten XM/IT parity checks and document known differences (looping, envelopes, effect coverage).

## Music (Content / Workflow)

- Prefer `TrackerInstrument.ref` (external `audio_v1` specs) in music examples/goldens so instruments are reusable and specs stay small; keep inline synthesis only when demonstrating a feature.
- Add a higher-quality drum example (filtered noise / metallic synthesis and/or WAV drum samples) and tune it by ear so kick/snare/hat sound correct in common XM/IT players.
- Add a small “XM vs IT parity” listening checklist and automate basic structural checks (loop flags/envelope flags) in tests.
- Grow “genre kits” as data packages: curated compose `defs` + instrument refs + timebase/harmony defaults (with workflows that focus edits on patterns, not instruments).

## Audio

- Keep expanding and tuning the `audio_v1` preset library (kick/snare/hat/clap/bass/lead/pad/etc.) and gate changes with QA metrics (no clipping, sane RMS, low DC offset).
- Add an “audio audit” report (peak/RMS/DC offset) to catch broken presets and regressions.
- Improve loop-point generation + click-free envelope defaults for tracker instrument baking; document best practices.

## Tooling / QA

- Grow the Tier-1 golden corpus (`golden/`) for audio/music/textures and run it in CI.
- Add “expand/inspect” style commands/flags where helpful for review (compose → expanded JSON, packed-texture intermediate maps).
