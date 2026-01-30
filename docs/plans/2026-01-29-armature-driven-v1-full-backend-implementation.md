# Armature-Driven Skeletal Mesh (v1) Full Backend Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make `skeletal_mesh.armature_driven_v1` fully implemented end-to-end so every field in `SkeletalMeshArmatureDrivenV1Params` affects generation output (and export settings are honored) to produce game-ready GLB characters.

**Architecture:** Keep the Rust spec contract as-is. Implement the missing behavior in the Blender backend by factoring armature-driven mesh construction into a dedicated module used by `handlers_skeletal.py`. Add Rust-side validation for referential integrity (mirrors, bool targets, material indices) and add Blender-gated integration tests.

**Tech Stack:** Rust (speccade-spec validation + tests), Blender Python (`bpy`, `bmesh`, `mathutils`), existing SpecCade Blender modules (`scene.py`, `materials.py`, `export.py`, `metrics.py`), Rust integration tests (`crates/speccade-tests`, ignored Blender tests).

---

## Definition Of Done (Acceptance)

- A spec using every field below generates successfully with Blender and produces a valid GLB:
  - `bone_meshes.*`: `profile`, `profile_radius` (relative/relative2/absolute), `taper`, `translate`, `rotate`, `bulge`, `twist`, `cap_start`, `cap_end`, `modifiers` (bevel/subdivide/bool), `material_index`, `attachments` (primitive/extrude/asset), and `mirror` refs.
  - `bool_shapes.*`: primitives, dimensions, position, optional `bone`, and `mirror` refs.
  - `material_slots` are created; `material_index` routes faces to correct slot.
  - `export`: `include_armature`, `include_normals`, `include_uvs`, `triangulate`, `include_skin_weights`, `save_blend` are honored.
- Rust validation fails fast with actionable errors for invalid references:
  - unknown bones referenced by `bone_meshes`, `bool_shapes.*.bone`
  - `mirror` points to missing key
  - boolean modifier references missing `bool_shapes` target
  - `material_index` out of range
  - invalid `profile` strings / invalid bulge control points
- Tests:
  - Rust unit tests cover new validation paths.
  - Blender-gated Rust e2e test(s) generate a “full features” armature-driven spec and assert metrics (materials, UVs, weights, triangles) are plausible.

---

## Repo Touchpoints (SSOT)

- Spec types: `crates/speccade-spec/src/recipe/character/armature_driven.rs`
- Validation: `crates/speccade-spec/src/validation/recipe_outputs.rs`
- Blender skeletal generation: `blender/speccade/handlers_skeletal.py`
- Blender export: `blender/speccade/export.py`
- Blender primitives/materials/metrics:
  - `blender/speccade/scene.py`
  - `blender/speccade/materials.py`
  - `blender/speccade/metrics.py`
- Rust Blender e2e tests (ignored): `crates/speccade-tests/tests/e2e_generation.rs`
- Docs: `docs/spec-reference/character.md`

---

## Execution Notes

- Do this work in a dedicated git worktree/branch.
- Prefer deterministic iteration order in Blender code (sort dict keys) even though Tier 2 isn’t byte-identical.
- Keep boolean cutter objects out of export (delete after apply, or hide + ensure exporter doesn’t include them).
- Keep each step tiny; commit frequently.

---

### Task 1: Add A Blender-Gated “Full Features” E2E Test Skeleton

**Files:**
- Modify: `crates/speccade-tests/tests/e2e_generation.rs`

**Step 1: Write failing (ignored) Blender test**

Add a new ignored test that:
1) builds a `Spec` with `skeletal_mesh.armature_driven_v1` and uses many fields,
2) runs `speccade_backend_blender::skeletal_mesh::generate`,
3) asserts the GLB exists and is valid,
4) asserts key metrics are present.

Suggested test (keep it minimal at first):

```rust
#[test]
#[ignore] // Run with SPECCADE_RUN_BLENDER_TESTS=1
fn test_generate_skeletal_mesh_armature_driven_full_features_smoke() {
    if !should_run_blender_tests() {
        println!("Blender tests not enabled, skipping");
        return;
    }
    if !is_blender_available() {
        println!("Blender not available, skipping");
        return;
    }

    use speccade_spec::{AssetType, OutputFormat, OutputSpec, Spec};
    use speccade_spec::recipe::Recipe;
    use crate::harness::{TestHarness, validate_glb_file};

    let harness = TestHarness::new();

    let spec = Spec::builder("armature-driven-full-features", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(123)
        .output(OutputSpec::primary(OutputFormat::Glb, "characters/armature_driven_full_features.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.armature_driven_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "material_slots": [
                    {"name":"skin","base_color":[0.9,0.75,0.65,1.0],"metallic":0.0,"roughness":0.6},
                    {"name":"armor","base_color":[0.2,0.2,0.25,1.0],"metallic":0.7,"roughness":0.3}
                ],
                "bool_shapes": {
                    "eye_socket_L": {"primitive":"sphere","dimensions":[0.06,0.08,0.06],"position":[0.05,0.15,0.6],"bone":"head"},
                    "eye_socket_R": {"mirror":"eye_socket_L"}
                },
                "bone_meshes": {
                    "spine": {
                        "profile":"hexagon(8)",
                        "profile_radius": 0.15,
                        "taper": 0.9,
                        "bulge": [{"at":0.0,"scale":0.9},{"at":0.5,"scale":1.2},{"at":1.0,"scale":0.85}],
                        "twist": 15,
                        "cap_start": true,
                        "cap_end": true,
                        "material_index": 0
                    },
                    "head": {
                        "profile":"circle(16)",
                        "profile_radius": {"absolute": 0.12},
                        "modifiers": [
                            {"bevel": {"width": 0.02, "segments": 2}},
                            {"subdivide": {"cuts": 1}},
                            {"bool": {"operation": "subtract", "target": "eye_socket_L"}}
                        ],
                        "material_index": 0,
                        "attachments": [
                            {"primitive":"sphere","dimensions":[0.08,0.06,0.08],"offset":[0.15,0.05,0.3],"rotation":[0,0,15],"material_index":1}
                        ]
                    }
                },
                "export": {
                    "include_armature": true,
                    "include_normals": true,
                    "include_uvs": true,
                    "triangulate": true,
                    "include_skin_weights": true,
                    "save_blend": false
                },
                "constraints": {"max_triangles": 20000, "max_bones": 64, "max_materials": 8}
            }),
        ))
        .build();

    let result = speccade_backend_blender::skeletal_mesh::generate(&spec, harness.path())
        .expect("generation failed");

    validate_glb_file(&result.output_path).expect("invalid glb");
    assert!(result.metrics.bone_count.is_some());
    assert!(result.metrics.material_slot_count.is_some());
    assert!(result.metrics.uv_layer_count.is_some());
    assert!(result.metrics.uv_coverage.is_some());
}
```

**Step 2: Run to verify it fails (expected until implementation is complete)**

Run:

```bash
SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --test e2e_generation --ignored
```

Expected: FAIL with a Blender-side error like “Unhandled / missing behavior” or metrics assertions failing.

**Step 3: Commit**

```bash
git add crates/speccade-tests/tests/e2e_generation.rs
git commit -m "test(blender): add armature_driven_v1 full-features smoke test"
```

---

### Task 2: Factor Armature-Driven Generation Into A Dedicated Blender Module

**Files:**
- Create: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/handlers_skeletal.py`

**Step 1: Write minimal failing import/usage test (Python)**

Create a small pure-Python unittest ensuring the module is importable (even if it requires bpy at runtime, the test can be skipped when bpy is missing):

Create: `blender/speccade/tests/test_armature_driven_module.py`

```python
import unittest


class TestArmatureDrivenModule(unittest.TestCase):
    def test_import(self) -> None:
        import speccade.armature_driven  # noqa: F401


if __name__ == "__main__":
    unittest.main()
```

**Step 2: Run test**

Run:

```bash
python -m unittest blender/speccade/tests/test_armature_driven_module.py
```

Expected: PASS.

**Step 3: Implement `blender/speccade/armature_driven.py` skeleton**

Add these entrypoints (implementation will evolve in later tasks):

```python
def build_armature_driven_character_mesh(*, armature, params: dict, out_root) -> "bpy.types.Object":
    """Return a mesh object named 'Character' with vertex groups for rigid skinning."""
    raise NotImplementedError
```

**Step 4: Wire handler to call new module (still failing)**

In `blender/speccade/handlers_skeletal.py`, replace the ad-hoc armature-driven mesh loop with:

```python
from .armature_driven import build_armature_driven_character_mesh
...
combined_mesh = build_armature_driven_character_mesh(armature=armature, params=params, out_root=out_root)
```

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py blender/speccade/handlers_skeletal.py blender/speccade/tests/test_armature_driven_module.py
git commit -m "refactor(blender): scaffold armature_driven builder module"
```

---

### Task 3: Implement `bone_meshes` Mirror Resolution (Backend)

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add a unit test for mirror resolution (pure Python, no bpy)**

Create: `blender/speccade/tests/test_armature_driven_mirror.py`

```python
import unittest


class TestArmatureDrivenMirror(unittest.TestCase):
    def test_resolve_mirror(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "arm_upper_L": {"profile": "circle(8)", "profile_radius": 0.2},
            "arm_upper_R": {"mirror": "arm_upper_L"},
        }
        resolved = _resolve_mirrors(defs)
        self.assertEqual(resolved["arm_upper_R"]["profile"], "circle(8)")
        self.assertEqual(resolved["arm_upper_R"]["profile_radius"], 0.2)
```

**Step 2: Run test (expected FAIL)**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_mirror.py
```

**Step 3: Implement `_resolve_mirrors()`**

In `blender/speccade/armature_driven.py` implement a deterministic resolver:
- Input: dict mapping bone_name -> either `{ "mirror": "other" }` or a mesh dict.
- Output: dict mapping bone_name -> concrete mesh dict (no mirror refs).
- Rules:
  - Sort keys for determinism.
  - Detect missing mirror targets (raise `ValueError`).
  - Detect cycles (raise `ValueError`).
  - Deep-copy resolved dict.

**Step 4: Run test (expected PASS)**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_mirror.py
```

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py blender/speccade/tests/test_armature_driven_mirror.py
git commit -m "feat(blender): resolve bone_meshes mirror references"
```

---

### Task 4: Implement `bool_shapes` Mirror Resolution (Backend)

**Files:**
- Modify: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/skeleton.py` (optional reuse of mirror helpers)

**Step 1: Add unit test for bool shape mirror resolution**

Create: `blender/speccade/tests/test_armature_driven_bool_shapes.py`

```python
import unittest


class TestArmatureDrivenBoolShapes(unittest.TestCase):
    def test_resolve_bool_shape_mirror(self) -> None:
        from speccade.armature_driven import _resolve_mirrors

        defs = {
            "eye_socket_L": {"primitive": "sphere", "dimensions": [0.1, 0.2, 0.1], "position": [0.1, 0.2, 0.3]},
            "eye_socket_R": {"mirror": "eye_socket_L"},
        }
        resolved = _resolve_mirrors(defs)
        self.assertEqual(resolved["eye_socket_R"]["primitive"], "sphere")
```

**Step 2: Run test**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_bool_shapes.py
```

Expected: PASS by reusing the same resolver.

**Step 3: Commit**

```bash
git add blender/speccade/tests/test_armature_driven_bool_shapes.py
git commit -m "test(blender): cover bool_shapes mirror resolution"
```

---

### Task 5: Implement Bone-Relative Length Decoding (Backend)

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add unit tests for length decoding**

Create: `blender/speccade/tests/test_armature_driven_lengths.py`

```python
import unittest


class TestArmatureDrivenLengths(unittest.TestCase):
    def test_relative_scalar(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length
        self.assertAlmostEqual(_resolve_bone_relative_length(0.25, bone_length=2.0), 0.5)

    def test_relative2_ellipse_returns_xy(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length
        rx, ry = _resolve_bone_relative_length([0.2, 0.1], bone_length=3.0)
        self.assertAlmostEqual(rx, 0.6)
        self.assertAlmostEqual(ry, 0.3)

    def test_absolute(self) -> None:
        from speccade.armature_driven import _resolve_bone_relative_length
        self.assertEqual(_resolve_bone_relative_length({"absolute": 0.05}, bone_length=999.0), 0.05)
```

**Step 2: Run tests (expected FAIL)**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_lengths.py
```

**Step 3: Implement `_resolve_bone_relative_length()`**

Rules:
- `float|int` => `float(v) * bone_length`
- `[x,y]` => `(x*bone_length, y*bone_length)`
- `{ "absolute": a }` => `float(a)`
- Validate positivity; raise `ValueError` on invalid.

**Step 4: Run tests (expected PASS)**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_lengths.py
```

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py blender/speccade/tests/test_armature_driven_lengths.py
git commit -m "feat(blender): decode BoneRelativeLength (relative/relative2/absolute)"
```

---

### Task 6: Implement Profile Parsing (`profile`) (Backend)

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add unit tests**

Create: `blender/speccade/tests/test_armature_driven_profile.py`

```python
import unittest


class TestArmatureDrivenProfile(unittest.TestCase):
    def test_circle_vertices(self) -> None:
        from speccade.armature_driven import _parse_profile
        self.assertEqual(_parse_profile("circle(12)"), ("circle", 12))

    def test_hexagon_vertices(self) -> None:
        from speccade.armature_driven import _parse_profile
        self.assertEqual(_parse_profile("hexagon(6)"), ("circle", 6))

    def test_square(self) -> None:
        from speccade.armature_driven import _parse_profile
        self.assertEqual(_parse_profile("square"), ("square", 4))
```

**Step 2: Run tests (expected FAIL)**

```bash
python -m unittest blender/speccade/tests/test_armature_driven_profile.py
```

**Step 3: Implement `_parse_profile()`**

Rules:
- default: `(circle, 12)`
- `circle(N)` => `(circle, N)`
- `hexagon(N)` => treat as `(circle, N)` (just vertex count)
- `square` => `(square, 4)`
- `rectangle` => `(rectangle, 4)`
- Validate `N>=3`.

**Step 4: Run tests (expected PASS)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py blender/speccade/tests/test_armature_driven_profile.py
git commit -m "feat(blender): parse armature-driven profile strings"
```

---

### Task 7: Implement Bone Mesh Primitive Construction (Cylinder/Square/Rectangle)

**Files:**
- Modify: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/scene.py` (optional: add helper for cube-with-uv)

**Step 1: Add Blender-gated Rust test for “produces UVs + weights”**

In `crates/speccade-tests/tests/e2e_generation.rs`, extend the smoke test assertions:
- `uv_layer_count > 0`
- `unweighted_vertex_count == 0` (rigid weights)
- `max_bone_influences == 1`

**Step 2: Run Blender-gated tests (expected FAIL)**

```bash
SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --test e2e_generation --ignored
```

**Step 3: Implement `build_armature_driven_character_mesh()` minimally**

Implement a first working version:
- resolve `bone_meshes` mirrors to concrete mesh dicts
- for each `(bone_name, mesh_spec)` sorted:
  - locate bone in `armature.data.bones`
  - compute bone length
  - create a mesh object:
    - circle-ish: `bpy.ops.mesh.primitive_cylinder_add(radius=..., depth=..., vertices=..., location=mid)`
    - square/rectangle: `bpy.ops.mesh.primitive_cube_add(size=1.0, location=mid)` then scale XY based on radius/ellipse and Z to length
  - orient object along bone axis (same approach as current handler)
  - apply transforms
  - create vertex group `bone_name`, assign all verts weight 1.0
- join all into a single object named `Character`

Leave taper/bulge/twist/caps/modifiers/attachments/bools for later tasks.

**Step 4: Run Blender-gated tests (expected PASS for basic metrics)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): generate basic armature-driven segments via builder"
```

---

### Task 8: Implement `translate` + `rotate` (Bone-Local Transforms)

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add Blender-gated test that uses translate/rotate without crashing**

Extend the e2e spec JSON for one bone to include:
- `translate: [0.1, 0.0, 0.0]`
- `rotate: [0.0, 0.0, 30.0]`

Assert: generation succeeds and produces GLB.

**Step 2: Run (expected FAIL or no-op)**

**Step 3: Implement bone-local transform application**

Rules:
- Values are bone-relative: multiply translation vector by `bone_length`.
- Rotation is degrees about bone-local axes.
- Apply these after the segment is oriented to the bone axis, before transform_apply.

**Step 4: Run (expected PASS)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): apply armature-driven translate/rotate in bone-local space"
```

---

### Task 9: Implement `cap_start` / `cap_end`

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add Blender-gated test that toggles caps**

Create a spec where one segment sets `cap_start=false`, `cap_end=true`.

Assert: generation succeeds; triangle count differs from same spec with both caps true.

**Step 2: Run (expected FAIL/no change)**

**Step 3: Implement cap removal via bmesh**

Implementation approach:
- ensure object is in local coordinates where its axis is +Z after orientation
- use `bmesh` to delete end faces close to min Z (start) and/or max Z (end)
- apply the mesh changes

**Step 4: Run (expected PASS)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): support cap_start/cap_end for armature-driven segments"
```

---

### Task 10: Implement `taper` + `bulge` + `twist`

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add Blender-gated test that asserts triangle/vertex count unchanged but mesh differs**

Because geometry deltas are hard to assert, use a proxy:
- generate two specs: one with no deformations, one with taper/bulge/twist
- assert both generate, and bounds differ (bbox size changes) or UV coverage differs slightly.

**Step 2: Run (expected FAIL/no change)**

**Step 3: Implement deformation in bmesh**

Algorithm (in segment local space, Z along bone):
- compute `t = (z - z_min) / (z_max - z_min)` per vertex
- base radial scale:
  - start_scale = 1.0
  - end_scale = `taper` (default 1.0)
  - scale = lerp(start_scale, end_scale, t)
- bulge:
  - input list of `{at, scale}` control points
  - clamp/sort by `at`
  - linearly interpolate bulge_scale(t)
  - scale *= bulge_scale
- twist:
  - angle_deg = twist * t
  - rotate vertex XY around Z by angle
- apply per-vertex edits; keep Z the same.

**Step 4: Run (expected PASS)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): implement taper/bulge/twist deformation"
```

---

### Task 11: Implement `bool_shapes` Creation + `modifiers: bool`

**Files:**
- Modify: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/scene.py` (reuse `create_primitive`)

**Step 1: Add Blender-gated test that uses bool subtraction**

Ensure the spec includes:
- a bool shape attached to a bone (e.g. `head`)
- a bone mesh modifier `{ "bool": {"operation":"subtract","target":"eye_socket_L"} }`

Assert:
- generation succeeds
- triangle count increases or decreases vs baseline (choose one stable comparison)

**Step 2: Run (expected FAIL)**

**Step 3: Implement bool shape objects**

Implementation:
- resolve bool_shapes mirrors to concrete dicts
- for each bool shape key sorted:
  - compute world transform:
    - if `bone` set: interpret `position` and `dimensions` as bone-relative and place in that bone’s local frame
    - else: interpret as armature-local absolute units (documented fallback)
  - create primitive via `create_primitive()`
  - set location/rotation, apply transforms
  - mark as hidden (and/or delete after booleans)

**Step 4: Implement bool modifier application**

For each bone segment’s modifiers list (in order):
- create a BOOLEAN modifier with `operation` mapping:
  - `subtract` -> `DIFFERENCE`
  - `union` -> `UNION`
  - `intersect` -> `INTERSECT`
- set solver to `EXACT` when available
- apply modifier immediately

After generating the full character mesh:
- delete all bool shape objects from the scene to avoid export.

**Step 5: Run (expected PASS) and Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): support bool_shapes and boolean modifiers in armature-driven meshes"
```

---

### Task 12: Implement `modifiers: bevel` + `modifiers: subdivide`

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add Blender-gated tests**

Add 2 small specs inside the e2e test:
- bevel only
- subdivide only

Assert: generation succeeds and triangle count increases for subdivide.

**Step 2: Run (expected FAIL)**

**Step 3: Implement bevel**

Implementation:
- add BEVEL modifier with width scaled by `bone_length` (bone-relative)
- set segments
- apply

**Step 4: Implement subdivide**

Implementation:
- enter EDIT mode, select all
- use bmesh subdivide with `cuts`
- return to OBJECT mode

**Step 5: Run + Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): support bevel/subdivide modifiers for armature-driven meshes"
```

---

### Task 13: Implement `attachments` (primitive/extrude/asset)

**Files:**
- Modify: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/scene.py` (reuse `create_primitive`)

**Step 1: Add Blender-gated test that uses each attachment type**

In the e2e test spec:
- primitive attachment (sphere)
- extrude attachment (start/end)
- asset attachment: create a tiny GLB fixture on the fly is hard; instead:
  - add this as a separate test that imports a known existing file under `golden/` OR
  - gate it behind “fixture exists” and skip if missing.

**Step 2: Run (expected FAIL)**

**Step 3: Implement primitive attachments**

Rules:
- dimensions are bone-relative => scale by `bone_length`
- optional offset/rotation are bone-local; offset is bone-relative
- assign vertices to the owning bone’s vertex group at weight 1.0
- honor `material_index`
- join attachment geometry into the bone segment object before joining into Character

**Step 4: Implement extrude attachments**

Implementation:
- interpret `start`/`end` as bone-local coordinates in bone-relative units
- construct a cylinder/cube between those points (like segment construction)
- honor `profile`, `profile_radius`, `taper`
- weight to owning bone

**Step 5: Implement asset attachments**

Implementation:
- resolve path relative to `out_root` if not absolute
- `bpy.ops.import_scene.gltf(filepath=...)`
- collect imported mesh objects, join into one, apply scale/rotation/offset
- weight to owning bone, join into segment

Commit after each attachment type if possible.

---

### Task 14: Implement `material_index` Face Assignment

**Files:**
- Modify: `blender/speccade/armature_driven.py`
- Modify: `blender/speccade/materials.py` (optional helper)

**Step 1: Add Blender-gated test**

Spec:
- 2 material slots
- one bone segment `material_index=0`
- one attachment `material_index=1`

Assert:
- `material_slot_count == 2`
- `materials_used_count >= 2` (if available in metrics) OR just ensure export doesn’t crash

**Step 2: Run (expected FAIL/no effect)**

**Step 3: Implement material assignment**

Implementation:
- ensure `apply_materials()` is called early enough so the mesh has material slots
- set `poly.material_index = material_index` for all polygons in the target geometry
- for joined objects, ensure indices remain

**Step 4: Run (expected PASS)**

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): honor material_index for armature-driven bone meshes and attachments"
```

---

### Task 15: Honor Skeletal Mesh Export Settings (`export.*`)

**Files:**
- Modify: `blender/speccade/export.py`
- Modify: `blender/speccade/handlers_skeletal.py`

**Step 1: Add Blender-gated tests for export toggles**

Add at least 2 cases:
- `include_uvs=false` => exported GLB should have `uv_layer_count == 0` (or `uv_coverage == 0.0`)
- `triangulate=true` => `quad_count == 0` (or quad_percentage == 0)

Note: if GLB analysis doesn’t perfectly reflect exporter toggles across Blender versions, keep assertions tolerant and focus on “doesn’t include texcoords / has no UV layers”.

**Step 2: Run (expected FAIL)**

**Step 3: Update `export_glb()` to accept options**

Change signature to:

```python
def export_glb(
    output_path: Path,
    *,
    include_armature: bool = True,
    include_animation: bool = False,
    include_normals: bool = True,
    include_uvs: bool = True,
    include_skin_weights: bool = True,
    triangulate: bool = True,
    export_tangents: bool = False,
) -> None:
    ...
```

Implementation:
- map to glTF exporter kwargs (normalize via `_normalize_operator_kwargs`):
  - `export_normals`
  - `export_texcoords`
  - `export_skins` (for skin weights)
  - `export_armatures` / equivalent when available
- if `triangulate` is true: apply a triangulate modifier on the character mesh object before calling exporter (do not rely on exporter).

**Step 4: Wire skeletal handler to pass export params**

In `handlers_skeletal.py`, read `params.get("export", {})` and pass the flags into `export_glb()`.

**Step 5: Run (expected PASS) and Commit**

```bash
git add blender/speccade/export.py blender/speccade/handlers_skeletal.py crates/speccade-tests/tests/e2e_generation.rs
git commit -m "feat(blender): honor skeletal export settings in GLB export"
```

---

### Task 16: Harden Rust Validation For Armature-Driven (Referential Integrity)

**Files:**
- Modify: `crates/speccade-spec/src/validation/recipe_outputs.rs`
- Modify: `crates/speccade-spec/src/validation/tests/mesh_tests.rs`

**Step 1: Add failing validation tests**

Add tests for:
- bone_mesh refers to unknown bone name
- mirror refers to missing bone_mesh key
- bool modifier refers to missing bool_shape key
- material_index >= material_slots.len()
- bulge.at out of [0,1]
- invalid profile string

Example pattern (one test):

```rust
#[test]
fn test_armature_driven_rejects_unknown_bone_mesh_key() {
    let spec = crate::spec::Spec::builder("bad", AssetType::SkeletalMesh)
        .license("CC0-1.0")
        .seed(1)
        .output(OutputSpec::primary(OutputFormat::Glb, "character.glb"))
        .recipe(Recipe::new(
            "skeletal_mesh.armature_driven_v1",
            serde_json::json!({
                "skeleton_preset": "humanoid_basic_v1",
                "bone_meshes": { "not_a_bone": {"profile":"circle(8)","profile_radius":0.2} }
            }),
        ))
        .build();
    let result = validate_for_generate(&spec);
    assert!(!result.is_ok());
}
```

**Step 2: Run tests (expected FAIL)**

```bash
cargo test -p speccade-spec
```

**Step 3: Implement validation additions**

In `validate_skeletal_mesh_armature_driven()`:
- Build an allowed-bone-name set from:
  - `skeleton_preset` bone list (known)
  - `skeleton` custom list bone names
- Validate `bone_meshes` keys are subset of allowed bones.
- Validate mirror targets exist.
- Validate bool modifier `target` exists in `bool_shapes`.
- Validate `material_index` bounds.
- Validate `bulge` constraints and `profile` parse.

**Step 4: Run tests (expected PASS)**

**Step 5: Commit**

```bash
git add crates/speccade-spec/src/validation/recipe_outputs.rs crates/speccade-spec/src/validation/tests/mesh_tests.rs
git commit -m "feat(spec): validate armature_driven_v1 references and ranges"
```

---

### Task 17: Update Docs + Add A Golden Spec That Exercises Full Feature Surface

**Files:**
- Modify: `docs/spec-reference/character.md`
- Create: `golden/speccade/specs/skeletal_mesh/armature_driven_full_features.spec.json`

**Step 1: Add golden spec file**

Create a spec similar to the e2e test JSON, but include at least one example of each field (including `attachments.extrude` and `bone_meshes.*.mirror`).

**Step 2: Validate spec locally**

Run:

```bash
cargo run -p speccade-cli -- validate --spec golden/speccade/specs/skeletal_mesh/armature_driven_full_features.spec.json --budget strict
```

Expected: OK.

**Step 3: Document semantics**

In `docs/spec-reference/character.md`:
- clarify units (bone-relative where applicable)
- list supported boolean operations
- note how `mirror` behaves for offsets/rotations
- state export settings are honored.

**Step 4: Commit**

```bash
git add docs/spec-reference/character.md golden/speccade/specs/skeletal_mesh/armature_driven_full_features.spec.json
git commit -m "docs(golden): add full-feature armature-driven skeletal mesh example"
```

---

### Task 18 (Optional But Recommended): Update Preview Path To Match Generation

**Why:** The editor preview helpers currently implement only a simplified cylinder preview for skeletal_mesh recipes; if we want preview to be WYSIWYG for armature-driven features, reuse the same builder.

**Files:**
- Modify: `blender/speccade/handlers_render.py`

**Steps:**
- Switch skeletal mesh preview path to call `build_armature_driven_character_mesh()`.
- Keep it tolerant (skip asset attachments if missing).
- Add a Blender-gated test if desired.

---

## Final Verification Checklist (Run Before Calling It Done)

- Rust validation/tests:

```bash
cargo test -p speccade-spec
```

- Python unit tests (non-Blender):

```bash
python -m unittest blender/speccade/tests/test_skeletal_mesh_rework.py
python -m unittest blender/speccade/tests/test_armature_driven_module.py
python -m unittest blender/speccade/tests/test_armature_driven_mirror.py
python -m unittest blender/speccade/tests/test_armature_driven_lengths.py
python -m unittest blender/speccade/tests/test_armature_driven_profile.py
```

- Blender-gated integration tests:

```bash
SPECCADE_RUN_BLENDER_TESTS=1 cargo test -p speccade-tests --test e2e_generation --ignored
```
