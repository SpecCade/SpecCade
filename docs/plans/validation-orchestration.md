# Visual Validation Orchestration

This document describes how Claude should validate 3D assets using the preview-grid output.

## For Each Spec to Validate

1. **Read the spec file** (`.star`)
   - Note the `[VALIDATION]` comment block
   - Understand what the asset should look like

2. **Generate the asset**
   ```bash
   cargo run -p speccade-cli -- generate --spec <spec.star> --out-root ./tmp
   ```

3. **Generate the validation grid**
   ```bash
   cargo run -p speccade-cli -- preview-grid --spec <spec.star> --out ./tmp/grid.png
   ```

4. **Read the grid image**
   - Use the Read tool on `./tmp/grid.png`
   - The grid shows 6 views: FRONT, BACK, TOP, LEFT, RIGHT, ISO

5. **Judge correctness**
   - Check: Does the shape match the `SHAPE:` description?
   - Check: Do proportions match `PROPORTIONS:`?
   - Check: Is orientation correct per `ORIENTATION:`?
   - Check: Do individual views match `FRONT VIEW:`, `BACK VIEW:`, etc.?

6. **Report verdict**
   - If PASS: "Validation passed for <asset_id>"
   - If FAIL: Describe what doesn't match

7. **If FAIL and auto-fix requested:**
   a. Identify which spec parameters need adjustment
   b. Edit the spec file
   c. Re-run steps 2-5
   d. Maximum 3 attempts

## Validation Targets (Priority Order)

1. `golden/starlark/mesh_cube.star` - simplest shape
2. `golden/starlark/mesh_organic_sculpt.star` - organic/metaball
3. `golden/starlark/character_humanoid.star` - skeletal mesh
4. `golden/starlark/animation_walk_cycle.star` - animation

## Example Judgment

Given this validation block:
```
# SHAPE: A beveled cube with smooth subdivision
# FRONT VIEW: Square face with beveled edges visible
```

And a grid showing a sphere:
- VERDICT: FAIL
- REASONING: Grid shows a rounded sphere, not a cube. Front view should show a square face with beveled edges, but instead shows a circular silhouette.
- FIXES: Check `base_primitive` parameter - should be "cube" not "sphere"
