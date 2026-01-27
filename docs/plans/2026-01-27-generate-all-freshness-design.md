# generate-all freshness check

Skip unchanged specs in `generate-all` to avoid redundant regeneration.

## Problem

`generate-all` regenerates every spec unconditionally. On a growing spec library this wastes time when most specs haven't changed.

## Design

### Freshness check (per spec, inside `process_spec`)

Before dispatching to a backend:

1. Compute `canonical_spec_hash` for the current spec.
2. Compute `backend_version` (`speccade-cli vX.Y.Z`).
3. Look for `{spec_dir}/{asset_id}.report.json`.
4. If the report exists, deserialize it and verify **all** of:
   - `report.ok == true`
   - `report.spec_hash == current_spec_hash`
   - `report.backend_version == current_backend_version`
5. Resolve each `report.outputs[].path` against the output directory (same layout `generate-all` uses) and check the file exists.
6. If every check passes, mark the spec as **fresh** and skip generation.
7. If any check fails, regenerate normally.

### CLI changes

Add `--force` / `-f` flag to `generate-all`. When set, skip the freshness check entirely.

### Output changes

| Mode | Fresh spec | Generated | Failed |
|------|-----------|-----------|--------|
| non-verbose | `s` (yellow) | `.` (green) | `x` (red) |
| verbose | `SKIPPED (fresh) {asset_id}` | `SUCCESS {asset_id} ({ms}ms)` | `FAILED {asset_id} - {err}` |

### Summary changes

- Print "Fresh (skipped): N" in the summary block.
- `GenerationSummary` gains `fresh_skipped: usize`.
- `SpecResult` gains `skipped_fresh: bool`.

### Files to change

1. `crates/speccade-cli/src/main.rs` — add `--force` arg to `GenerateAll`, pass to `run()`.
2. `crates/speccade-cli/src/commands/generate_all.rs` — add `force` param, implement freshness check in `process_spec`, update summary output and structs.
3. Tests in both files for the new flag and skip logic.
