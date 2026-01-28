# Visual LLM Validation for 3D Assets

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Validate that golden static meshes, characters, and animations are "100% accurate" by generating multi-angle preview grids and using Claude's vision to verify correctness against embedded spec comments.

**Architecture:** A CLI command generates a 6-view PNG grid from any 3D spec. Claude reads the grid + validation comments from the spec, judges correctness (shape, proportions, orientation), and if incorrect, edits the spec and re-validates in a loop (max 3 attempts).

---

## Workflow

```
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ .star spec  │───▶│ speccade     │───▶│ .glb mesh   │
│ + comments  │    │ generate     │    │ (Blender)   │
└─────────────┘    └──────────────┘    └─────────────┘
                                              │
                                              ▼
┌─────────────┐    ┌──────────────┐    ┌─────────────┐
│ Claude      │◀───│ multi-angle  │◀───│ preview-grid│
│ vision      │    │ PNG grid     │    │ (new cmd)   │
└─────────────┘    └──────────────┘    └─────────────┘
      │
      ├─── PASS ──▶ done
      │
      └─── FAIL ──▶ edit spec ──▶ loop (max 3)
```

---

## Multi-Angle Preview Grid

**6-view layout:**

```
┌────────────────────────────────────┐
│  FRONT     │  BACK      │  TOP     │
│  (0°)      │  (180°)    │  (90° ↑) │
├────────────┼────────────┼──────────┤
│  LEFT      │  RIGHT     │  ISO     │
│  (90°)     │  (270°)    │ (45°,30°)│
└────────────────────────────────────┘
```

- Each panel: 256x256 pixels (configurable)
- Labels overlaid on each panel
- Total grid: 768x512 pixels

**For animations:** Render 3 keyframes (0, N/2, N-1), each as a 6-view row = 18 panels.

---

## Spec Comment Format

Specs include a `[VALIDATION]` comment block:

```python
# [VALIDATION]
# SHAPE: A humanoid figure with cylindrical limbs
# PROPORTIONS: Head is ~0.15 units, torso is ~0.4 units tall
# ORIENTATION: Head at top (+Z), facing forward (+Y)
# FRONT VIEW: Should show chest, face looking at camera
# BACK VIEW: Should show spine, back of head
# LEFT VIEW: Should show left arm extended
# NOTES: Arms should be attached at chest level, not floating

skeletal_mesh_spec(
    asset_id = "stdlib-character-humanoid-01",
    ...
)
```

**Sections:**
- `SHAPE:` - What the object fundamentally is
- `PROPORTIONS:` - Relative sizes of parts
- `ORIENTATION:` - Which way is up/front/back
- `FRONT VIEW:` / `BACK VIEW:` / `LEFT VIEW:` / `RIGHT VIEW:` / `TOP VIEW:` / `ISO VIEW:` - What should be visible from each angle
- `NOTES:` - Gotchas or edge cases

**For animations:**
```python
# FRAME 0: Contact pose - left foot forward
# FRAME 12: Passing pose - legs crossed
# MOTION: Smooth sinusoidal leg swing
```

---

## Validation Judgment

Claude's judgment prompt:

```
You are validating a 3D asset render against expectations.

SPEC FILE: {spec_content}
VALIDATION COMMENTS:
{extracted_comments}

IMAGE: [attached grid.png]

Check:
1. Does the shape match the SHAPE description?
2. Do proportions match PROPORTIONS?
3. Is orientation correct per ORIENTATION?
4. Do individual views match FRONT VIEW, BACK VIEW, etc.?

Respond with:
- VERDICT: PASS or FAIL
- REASONING: What specifically matches or doesn't match
- FIXES: If FAIL, what spec parameter changes would fix it
```

---

## Implementation Tasks

### Task 1: Add `preview-grid` CLI command

**Files:**
- Modify: `crates/speccade-cli/src/main.rs` (add PreviewGrid command variant)
- Create: `crates/speccade-cli/src/commands/preview_grid.rs`

**Step 1:** Add the PreviewGrid command to the Commands enum:

```rust
/// Generate a multi-angle validation grid PNG
PreviewGrid {
    /// Path to the spec file
    #[arg(short, long)]
    spec: String,

    /// Output PNG path (default: <spec_stem>.grid.png)
    #[arg(short, long)]
    out: Option<String>,

    /// Panel size in pixels (default: 256)
    #[arg(long, default_value = "256")]
    panel_size: u32,
},
```

**Step 2:** Add dispatch in main.rs:

```rust
Commands::PreviewGrid { spec, out, panel_size } => {
    commands::preview_grid::run(&spec, out.as_deref(), panel_size)
}
```

**Step 3:** Create `preview_grid.rs` with stub implementation:

```rust
pub fn run(spec_path: &str, out: Option<&str>, panel_size: u32) -> Result<ExitCode> {
    println!("preview-grid not yet implemented");
    Ok(ExitCode::SUCCESS)
}
```

**Step 4:** Verify compilation: `cargo check -p speccade-cli`

**Step 5:** Commit:
```bash
git add crates/speccade-cli/src/main.rs crates/speccade-cli/src/commands/preview_grid.rs crates/speccade-cli/src/commands/mod.rs
git commit -m "feat(cli): add preview-grid command stub"
```

---

### Task 2: Implement Blender validation grid render

**Files:**
- Modify: `crates/speccade-backend-blender/python/entrypoint.py` (or create new script)
- Modify: `crates/speccade-backend-blender/src/lib.rs` (add grid render mode)

**Step 1:** Add a new Python script `validation_grid.py` that:
- Takes mesh recipe params + output path
- Sets up 6 cameras at fixed positions
- Renders each view to temp PNGs
- Composites into a single grid with labels
- Saves final grid PNG

**Camera positions (Blender coordinates, Z-up):**
```python
VIEWS = [
    ("FRONT", (0, -5, 1), (90, 0, 0)),      # Looking +Y
    ("BACK", (0, 5, 1), (90, 0, 180)),      # Looking -Y
    ("TOP", (0, 0, 5), (0, 0, 0)),          # Looking -Z
    ("LEFT", (-5, 0, 1), (90, 0, -90)),     # Looking +X
    ("RIGHT", (5, 0, 1), (90, 0, 90)),      # Looking -X
    ("ISO", (3.5, -3.5, 3), (55, 0, 45)),   # 45° azimuth, 30° elevation
]
```

**Step 2:** Add Rust wrapper that invokes the script and reads output.

**Step 3:** Test with mesh_cube.star manually.

**Step 4:** Commit:
```bash
git add crates/speccade-backend-blender/
git commit -m "feat(blender): add validation grid render mode"
```

---

### Task 3: Wire preview-grid to Blender backend

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview_grid.rs`

**Step 1:** Parse the spec to extract mesh/character/animation recipe.

**Step 2:** Call the Blender backend with "validation_grid" mode.

**Step 3:** Read the output PNG and save to the requested path.

**Step 4:** Test: `cargo run -p speccade-cli -- preview-grid --spec golden/starlark/mesh_cube.star`

**Step 5:** Commit:
```bash
git add crates/speccade-cli/src/commands/preview_grid.rs
git commit -m "feat(preview-grid): wire up Blender grid render"
```

---

### Task 4: Implement comment extraction

**Files:**
- Create: `crates/speccade-cli/src/commands/preview_grid.rs` (add function)

**Step 1:** Add `extract_validation_comments(source: &str) -> Option<String>`:
- Scan for `# [VALIDATION]` line
- Collect subsequent `#` comment lines until blank line or non-comment
- Return as structured text

**Step 2:** Test with mesh_cube.star (after adding comments).

**Step 3:** Commit:
```bash
git add crates/speccade-cli/src/commands/preview_grid.rs
git commit -m "feat(preview-grid): extract validation comments from spec"
```

---

### Task 5: Add validation comments to golden specs

**Files:**
- Modify: `golden/starlark/mesh_cube.star`
- Modify: `golden/starlark/mesh_organic_sculpt.star`
- Modify: `golden/starlark/character_humanoid.star`
- Modify: `golden/starlark/animation_walk_cycle.star`

**Step 1:** Add `[VALIDATION]` blocks to each spec with:
- SHAPE description
- PROPORTIONS
- ORIENTATION
- Per-view expectations

**Example for mesh_cube.star:**
```python
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin, aligned to world axes
# FRONT VIEW: Square face with beveled edges visible
# ISO VIEW: Three faces visible (front, top, right)
# NOTES: Bevel should create smooth edge transitions
```

**Step 2:** Commit:
```bash
git add golden/starlark/
git commit -m "docs(golden): add validation comments to key specs"
```

---

### Task 6: Create orchestration plan document

**Files:**
- Create: `docs/plans/validation-orchestration.md`

**Step 1:** Document the step-by-step process for Claude to follow:

```markdown
# Visual Validation Orchestration

For each spec to validate:

1. Read the spec file and extract [VALIDATION] comments
2. Run: `cargo run -p speccade-cli -- generate --spec $SPEC --out-root ./tmp`
3. Run: `cargo run -p speccade-cli -- preview-grid --spec $SPEC --out ./tmp/grid.png`
4. Read ./tmp/grid.png using the Read tool (vision)
5. Judge: Does the grid match the validation comments?
6. If PASS: Report success, move to next spec
7. If FAIL:
   a. Identify which views/aspects don't match
   b. Determine spec parameter changes needed
   c. Edit the spec file
   d. Increment attempt counter
   e. If attempts < 3, goto step 2
   f. If attempts >= 3, report failure and move on
8. Summarize results for all specs
```

**Step 2:** Commit:
```bash
git add docs/plans/validation-orchestration.md
git commit -m "docs: add validation orchestration guide"
```

---

## Validation Targets (Priority Order)

1. `golden/starlark/mesh_cube.star` - simplest shape
2. `golden/starlark/mesh_organic_sculpt.star` - organic/metaball
3. `golden/starlark/character_humanoid.star` - skeletal mesh
4. `golden/starlark/animation_walk_cycle.star` - animation (stretch)

---

## Success Criteria

- [ ] `preview-grid` command generates 6-view PNG for any static_mesh spec
- [ ] Validation comments exist in all 4 target golden specs
- [ ] Claude can read grid + comments and make PASS/FAIL judgment
- [ ] At least 2 specs validated end-to-end with auto-fix loop
