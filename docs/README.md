# SpecCade Docs

## Start Here

- `README.md` (repo overview + how to run)
- `SPEC_REFERENCE.md` (index of canonical spec docs under `docs/spec-reference/`)
- `starlark-authoring.md` (authoring `.star` specs)
- `stdlib-reference.md` (Starlark stdlib index)
- `budgets.md` (budget profiles and how to use `--budget`)
- `DETERMINISM.md` (determinism model and expectations)
- `LLM_PROMPT_TO_ASSET_ROADMAP.md` (forward-looking: prompt→spec→asset workflows and tooling gaps)

## LLM/Tool-Friendly Notes

- Prefer `docs/spec-reference/*` and the `speccade-spec` Rust types as the source of truth.
- The stdlib docs are split by domain:
  - `stdlib-core.md`
  - `stdlib-audio.md`
  - `stdlib-music.md`
  - `stdlib-texture.md`
  - `stdlib-mesh.md`
- Large “master list” docs (e.g. preset libraries / genre kits) are reference appendices; they’re not required reading for basic usage.
