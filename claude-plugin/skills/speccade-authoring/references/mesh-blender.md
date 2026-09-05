# Blender mesh and animation authoring

Blender backends are Tier 2: pin the Blender version, compare metrics and inspect output rather than expecting identical GLB bytes. Run `speccade doctor` before a Blender task; a working Rust audio backend does not verify Blender.

## Source map (repository-relative)

- `crates/speccade-spec/src/recipe/mesh/`: primitive, modifier, static/organic/modular mesh contracts.
- `crates/speccade-spec/src/recipe/character/`: armature-driven and skinned-mesh contracts.
- `crates/speccade-spec/src/recipe/animation/`: clips, rigs, poses and constraints.
- `docs/spec-reference/mesh.md`, `character.md`, `animation.md`: corresponding guides.
- `speccade stdlib dump --format json`: current authoring helper signatures.

Reuse a checked-in spec for the selected recipe kind, validate, and generate one representative asset. Do not assume a Blender primitive or modifier is exposed by SpecCade merely because Blender supports it. Keep naming, orientation, scale, materials, skin and animation selection explicit.

Check the generated GLB's meshes/primitives, bounds, triangles, skin/bone order, clip names and frames. View the imported result and animate it in the consumer before a batch.

## Nethercore boundary

Current Nethercore mesh import selects the first mesh's first primitive, not a whole scene. Prepare one intended runtime primitive or separate exports. Skeleton export accepts at most256 bones; packed animation accepts at most255 bones and65535 sampled frames. Animated assets must fit the lower bone ceiling. Use explicit `skin_name`/`animation_name` in `nether.toml` where necessary. See [ZX integration](nethercore-zx-integration.md).
