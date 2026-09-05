# Spec format and reports

Author Starlark (`.star`) with current helpers, or JSON when explicitly required. Evaluation produces the canonical spec; helper syntax is not the underlying recipe schema.

## Authority

- `crates/speccade-spec/src/recipe/`: recipe variants and parameter contracts.
- `crates/speccade-spec/src/`: top-level spec, output, validation and budget types.
- `speccade stdlib dump --format json`: helper signatures/defaults.
- `schemas/speccade-spec-v1.schema.json`: editor assistance, not backend acceptance.

A complete spec identifies version, asset ID/type, seed, outputs and recipe. Use the `spec`/`output` helpers to populate defaults rather than maintaining a parallel JSON template. Declare rights/license accurately; do not invent rights for imported material.

Outputs need safe relative paths, matching formats and a primary output. For procedural textures, `outputs[].source` names the intended terminal node ID. Keep generated files inside the explicit output root; validate before generation.

```bash
speccade eval --spec asset.star --pretty
speccade validate --spec asset.star --budget nethercore --json
speccade generate --spec asset.star --out-root ./output --budget nethercore --json
```

These use the Nethercore ZX budget. Other consumers need their appropriate preset. Read actual errors, warnings and output records from the installed CLI; do not use a stale copied error-code table. On nonzero exit, retain the concrete failure and do not claim generated success.

Verify reported paths exist and decode in their intended format. Tier1 determinism requires the same validated source/spec/seed and pinned implementation/dependencies; Tier2 Blender uses metrics and pinned versions. Consumer packing, initialization, runtime use and human quality are separate acceptance gates.
