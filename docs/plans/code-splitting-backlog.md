# Code Splitting Backlog

Files to split in future work, identified by line count.

## Completed

| File | Original Lines | Result | Date |
|------|---------------:|--------|------|
| `blender/entrypoint.py` | 8,708 | Split into `blender/speccade/` package (21 modules) | 2026-01-29 |

## Rust (>1,500 lines)

| File | Lines | Split Strategy |
|------|------:|---------------|
| `crates/speccade-spec/src/validation/recipe_outputs.rs` | 2,029 | Split by asset type |
| `crates/speccade-cli/src/main.rs` | 2,026 | Extract subcommand modules |
| `crates/speccade-lint/src/rules/music.rs` | 1,639 | Split by rule category |
| `crates/speccade-cli/src/dispatch/texture.rs` | 1,599 | Split pipeline stages |
| `crates/speccade-lint/src/rules/mesh.rs` | 1,583 | Split by rule category |

## TypeScript (>1,000 lines)

| File | Lines | Split Strategy |
|------|------:|---------------|
| `editor/src/components/MusicPreview.ts` | 1,369 | Extract renderer/controls |
| `editor/src/components/TexturePreview.ts` | 1,263 | Extract renderer/controls |
| `editor/src/main.ts` | 1,101 | Extract initialization/routing |
