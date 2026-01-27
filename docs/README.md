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
- `rfcs/` (design proposals and rationale; action items live in `ROADMAP.md`)

## Conventions

- [Coordinate System](conventions/coordinate-system.md) - Axis conventions for meshes and animations

## LLM/Tool-Friendly Notes

- Prefer `docs/spec-reference/*` and the `speccade-spec` Rust types as the source of truth.
- For stdlib accuracy, prefer `speccade stdlib dump --format json` over prose docs.
- The stdlib docs are split by domain:
  - `stdlib-core.md`
  - `stdlib-audio.md`
  - `stdlib-music.md`
  - `stdlib-texture.md`
  - `stdlib-mesh.md`
- Pack inventories live under `packs/`; treat docs that enumerate pack contents as convenience views (not the source of truth).
