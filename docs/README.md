# SpecCade Docs

## SSOT

See [`AGENTS.md`](../AGENTS.md) for the single source of truth map.

## Start Here

- `README.md` (repo overview + how to run)
- `ROADMAP.md` (single source of truth for planned work + open questions)
- `spec-reference/README.md` (canonical spec contract + per-asset reference)
- `starlark-authoring.md` (authoring `.star` specs)
- `stdlib-reference.md` (Starlark stdlib index)
- `budgets.md` (budget profiles and how to use `--budget`)
- `DETERMINISM.md` (determinism model and expectations)
- `lint-rules.md` (44 semantic quality rules across audio/texture/mesh/music)

## RFCs

Active design proposals (completed RFCs have been removed; their work is in the codebase):

- `rfcs/RFC-0008*` suite — (active)
- `rfcs/RFC-0009*` — (active)
- `rfcs/RFC-0010*` — (active)
- `rfcs/RFC-0012*` — Sprite assets (active)

## Plans

- `plans/code-splitting-backlog.md` — Large file split targets (backlog)
- Other active plan files in `plans/`

## Conventions

- [Coordinate System](conventions/coordinate-system.md) - Axis conventions for meshes and animations

## LLM/Tool-Friendly Notes

- Prefer `docs/spec-reference/*` and the `speccade-spec` Rust types as the source of truth.
- For stdlib accuracy, prefer `speccade stdlib dump --format json` over prose docs.
- The stdlib docs are condensed summary tables pointing to SSOT:
  - `stdlib-core.md`
  - `stdlib-audio.md` — synthesis, filters, effects, modulation
  - `stdlib-music.md` — tracker instruments, patterns, cue templates
  - `stdlib-texture.md` — node graph, trimsheets, decals, splat sets, matcaps
  - `stdlib-mesh.md` — primitives and modifiers
- Pack inventories live under `packs/`; treat docs that enumerate pack contents as convenience views (not the source of truth).
