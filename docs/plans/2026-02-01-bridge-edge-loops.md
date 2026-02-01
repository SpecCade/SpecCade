# Bridge Edge Loop Support Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add bridge edge loop functionality to `skeletal_mesh.armature_driven_v1` enabling topologically connected character meshes with smooth joint deformation.

**Architecture:** Per-bone `connect_start`/`connect_end` fields control bridging behavior. During Blender mesh construction, edge loops are tracked via vertex groups. After joining segments, bridge operations connect parent tail to child head edge loops, with weight blending at junctions.

**Tech Stack:** Rust (serde types), Python/Blender (bpy.ops.mesh.bridge_edge_loops, bmesh)

---

## Task 1: Add ConnectionMode Enum to Rust Types

**Files:**
- Modify: `crates/speccade-spec/src/recipe/character/armature_driven.rs:1-82`

**Step 1: Write the failing test**

Create test file:

```rust
// crates/speccade-spec/src/recipe/character/tests/connection_mode.rs

use super::super::*;

#[test]
fn test_connection_mode_parses_bridge() {
    let json = r#""bridge""#;
    let mode: ConnectionMode = serde_json::from_str(json).unwrap();
    assert_eq!(mode, ConnectionMode::Bridge);
}

#[test]
fn test_connection_mode_parses_segmented() {
    let json = r#""segmented""#;
    let mode: ConnectionMode = serde_json::from_str(json).unwrap();
    assert_eq!(mode, ConnectionMode::Segmented);
}

#[test]
fn test_connection_mode_default_is_none() {
    let json = r#"{"profile": "circle(8)"}"#;
    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    assert!(mesh.connect_start.is_none());
    assert!(mesh.connect_end.is_none());
}
```

**Step 2: Register the test module**

Add to `crates/speccade-spec/src/recipe/character/tests.rs`:

```rust
mod connection_mode;
```

**Step 3: Run test to verify it fails**

Run: `cargo test -p speccade-spec connection_mode`
Expected: FAIL with "cannot find type `ConnectionMode`"

**Step 4: Add ConnectionMode enum**

Add to `crates/speccade-spec/src/recipe/character/armature_driven.rs` after the imports:

```rust
/// Connection mode for bone mesh boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionMode {
    /// No topological connection (current behavior) - mesh ends are independent.
    Segmented,
    /// Bridge edge loops with adjacent bone's mesh, blend weights at junction.
    Bridge,
}

impl Default for ConnectionMode {
    fn default() -> Self {
        ConnectionMode::Segmented
    }
}
```

**Step 5: Run test to verify it passes**

Run: `cargo test -p speccade-spec connection_mode`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/speccade-spec/src/recipe/character/armature_driven.rs
git add crates/speccade-spec/src/recipe/character/tests.rs
git add crates/speccade-spec/src/recipe/character/tests/connection_mode.rs
git commit -m "feat(spec): add ConnectionMode enum for bridge edge loops"
```

---

## Task 2: Add connect_start/connect_end Fields to ArmatureDrivenBoneMesh

**Files:**
- Modify: `crates/speccade-spec/src/recipe/character/armature_driven.rs:133-178`
- Modify: `crates/speccade-spec/src/recipe/character/tests/connection_mode.rs`

**Step 1: Write the failing test**

Add to `crates/speccade-spec/src/recipe/character/tests/connection_mode.rs`:

```rust
#[test]
fn test_bone_mesh_with_bridge_connections() {
    let json = r#"{
        "profile": "circle(8)",
        "connect_start": "bridge",
        "connect_end": "segmented"
    }"#;
    let mesh: ArmatureDrivenBoneMesh = serde_json::from_str(json).unwrap();
    assert_eq!(mesh.connect_start, Some(ConnectionMode::Bridge));
    assert_eq!(mesh.connect_end, Some(ConnectionMode::Segmented));
}

#[test]
fn test_full_bone_meshes_with_bridging() {
    let json = r#"{
        "skeleton_preset": "humanoid_basic_v1",
        "bone_meshes": {
            "spine": {
                "profile": "circle(8)",
                "connect_end": "bridge"
            },
            "chest": {
                "profile": "circle(8)",
                "connect_start": "bridge",
                "connect_end": "bridge"
            },
            "neck": {
                "profile": "circle(8)",
                "connect_start": "bridge"
            }
        }
    }"#;
    let params: SkeletalMeshArmatureDrivenV1Params = serde_json::from_str(json).unwrap();

    let spine = params.bone_meshes.get("spine").unwrap();
    if let ArmatureDrivenBoneMeshDef::Mesh(m) = spine {
        assert_eq!(m.connect_end, Some(ConnectionMode::Bridge));
        assert!(m.connect_start.is_none());
    } else {
        panic!("Expected Mesh, got Mirror");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-spec connection_mode`
Expected: FAIL with "unknown field `connect_start`"

**Step 3: Add fields to ArmatureDrivenBoneMesh**

In `crates/speccade-spec/src/recipe/character/armature_driven.rs`, add to `ArmatureDrivenBoneMesh` struct after `cap_end`:

```rust
    /// How this bone's mesh start connects to parent bone's mesh end.
    /// Default: None (treated as Segmented - no topological connection).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_start: Option<ConnectionMode>,

    /// How this bone's mesh end connects to child bones' mesh starts.
    /// Default: None (treated as Segmented - no topological connection).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connect_end: Option<ConnectionMode>,
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p speccade-spec connection_mode`
Expected: PASS

**Step 5: Run full test suite to check for regressions**

Run: `cargo test -p speccade-spec`
Expected: All tests PASS

**Step 6: Commit**

```bash
git add crates/speccade-spec/src/recipe/character/armature_driven.rs
git add crates/speccade-spec/src/recipe/character/tests/connection_mode.rs
git commit -m "feat(spec): add connect_start/connect_end fields to ArmatureDrivenBoneMesh"
```

---

## Task 3: Add Python Unit Tests for Bridge Edge Loop Helpers

**Files:**
- Create: `blender/speccade/tests/test_armature_driven_bridge.py`

**Step 1: Write the failing tests**

```python
# blender/speccade/tests/test_armature_driven_bridge.py

import unittest
from pathlib import Path
import sys

# Allow `import speccade.*` from repo root.
_BLENDER_DIR = Path(__file__).resolve().parents[2]
if str(_BLENDER_DIR) not in sys.path:
    sys.path.insert(0, str(_BLENDER_DIR))


class TestBridgeEdgeLoopHelpers(unittest.TestCase):
    """Test bridge edge loop helper functions (pure Python, no Blender)."""

    def test_can_import_bridge_helpers(self) -> None:
        from speccade.armature_driven import (
            get_bridge_pairs,
            are_profiles_compatible,
        )
        self.assertTrue(callable(get_bridge_pairs))
        self.assertTrue(callable(are_profiles_compatible))

    def test_are_profiles_compatible_exact_match(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # Same segment count = compatible
        self.assertTrue(are_profiles_compatible(8, 8))
        self.assertTrue(are_profiles_compatible(12, 12))

    def test_are_profiles_compatible_double(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # One is 2x the other = compatible
        self.assertTrue(are_profiles_compatible(8, 16))
        self.assertTrue(are_profiles_compatible(16, 8))
        self.assertTrue(are_profiles_compatible(6, 12))

    def test_are_profiles_incompatible(self) -> None:
        from speccade.armature_driven import are_profiles_compatible
        # Non-matching, non-2x = incompatible
        self.assertFalse(are_profiles_compatible(8, 12))
        self.assertFalse(are_profiles_compatible(7, 9))
        self.assertFalse(are_profiles_compatible(8, 24))  # 3x, not 2x

    def test_get_bridge_pairs_empty_when_no_bridging(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)"},  # No connect_end
            "chest": {"profile": "circle(8)"},  # No connect_start
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(pairs, [])

    def test_get_bridge_pairs_finds_matching_bridges(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": ["neck"]},
            "neck": {"parent": "chest", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)", "connect_end": "bridge"},
            "chest": {"profile": "circle(8)", "connect_start": "bridge", "connect_end": "bridge"},
            "neck": {"profile": "circle(8)", "connect_start": "bridge"},
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(len(pairs), 2)
        self.assertIn(("spine", "chest"), pairs)
        self.assertIn(("chest", "neck"), pairs)

    def test_get_bridge_pairs_requires_both_sides(self) -> None:
        from speccade.armature_driven import get_bridge_pairs

        bone_hierarchy = {
            "spine": {"parent": None, "children": ["chest"]},
            "chest": {"parent": "spine", "children": []},
        }
        bone_meshes = {
            "spine": {"profile": "circle(8)", "connect_end": "bridge"},
            "chest": {"profile": "circle(8)"},  # No connect_start - asymmetric
        }
        pairs = get_bridge_pairs(bone_hierarchy, bone_meshes)
        self.assertEqual(pairs, [])  # No bridge - requires both sides


if __name__ == "__main__":
    unittest.main()
```

**Step 2: Run test to verify it fails**

Run: `python blender/speccade/tests/test_armature_driven_bridge.py`
Expected: FAIL with "cannot import name 'get_bridge_pairs'"

**Step 3: Commit test file (TDD - test first)**

```bash
git add blender/speccade/tests/test_armature_driven_bridge.py
git commit -m "test: add failing tests for bridge edge loop helpers"
```

---

## Task 4: Implement Bridge Helper Functions

**Files:**
- Modify: `blender/speccade/armature_driven.py:1-105`

**Step 1: Add helper functions**

Add after the existing helper functions (around line 105, after `_parse_profile`):

```python
def are_profiles_compatible(count_a: int, count_b: int) -> bool:
    """Check if two profile segment counts are compatible for bridging.

    Compatible means: exact match OR one is exactly 2x the other.
    """
    if count_a == count_b:
        return True
    if count_a == 2 * count_b or count_b == 2 * count_a:
        return True
    return False


def get_bridge_pairs(bone_hierarchy: dict, bone_meshes: dict) -> list[tuple[str, str]]:
    """Determine which bone pairs should have their edge loops bridged.

    A bridge is created when:
    1. Parent bone's mesh has connect_end="bridge"
    2. Child bone's mesh has connect_start="bridge"
    3. Both bones have mesh definitions (not just mirrors)

    Args:
        bone_hierarchy: Dict mapping bone_name -> {"parent": str|None, "children": [str]}
        bone_meshes: Dict mapping bone_name -> bone mesh definition dict

    Returns:
        List of (parent_bone, child_bone) tuples to bridge.
    """
    pairs = []

    for child_name, info in bone_hierarchy.items():
        parent_name = info.get("parent")
        if parent_name is None:
            continue

        parent_mesh = bone_meshes.get(parent_name)
        child_mesh = bone_meshes.get(child_name)

        if parent_mesh is None or child_mesh is None:
            continue

        # Skip mirror references - they resolve to their target's settings
        if isinstance(parent_mesh, dict) and "mirror" in parent_mesh:
            continue
        if isinstance(child_mesh, dict) and "mirror" in child_mesh:
            continue

        parent_connect_end = parent_mesh.get("connect_end")
        child_connect_start = child_mesh.get("connect_start")

        if parent_connect_end == "bridge" and child_connect_start == "bridge":
            pairs.append((parent_name, child_name))

    return pairs
```

**Step 2: Run test to verify it passes**

Run: `python blender/speccade/tests/test_armature_driven_bridge.py`
Expected: PASS

**Step 3: Commit**

```bash
git add blender/speccade/armature_driven.py
git commit -m "feat(blender): add bridge edge loop helper functions"
```

---

## Task 5: Track Edge Loop Vertices During Mesh Construction

**Files:**
- Modify: `blender/speccade/armature_driven.py:437-474` (cap handling section)

**Step 1: Write test for edge loop vertex group creation**

Add to `blender/speccade/tests/test_armature_driven_bridge.py`:

```python
class TestEdgeLoopTracking(unittest.TestCase):
    """Test edge loop vertex group tracking (requires Blender mock or skip)."""

    def test_bridge_vertex_group_names(self) -> None:
        """Test the naming convention for bridge vertex groups."""
        from speccade.armature_driven import (
            get_bridge_head_vgroup_name,
            get_bridge_tail_vgroup_name,
        )
        self.assertEqual(get_bridge_head_vgroup_name("spine"), "_bridge_head_spine")
        self.assertEqual(get_bridge_tail_vgroup_name("spine"), "_bridge_tail_spine")
```

**Step 2: Run test to verify it fails**

Run: `python blender/speccade/tests/test_armature_driven_bridge.py`
Expected: FAIL with "cannot import name 'get_bridge_head_vgroup_name'"

**Step 3: Add vertex group naming helpers**

Add to `blender/speccade/armature_driven.py` after `get_bridge_pairs`:

```python
def get_bridge_head_vgroup_name(bone_name: str) -> str:
    """Get the vertex group name for a bone's head (start) edge loop."""
    return f"_bridge_head_{bone_name}"


def get_bridge_tail_vgroup_name(bone_name: str) -> str:
    """Get the vertex group name for a bone's tail (end) edge loop."""
    return f"_bridge_tail_{bone_name}"
```

**Step 4: Run test to verify it passes**

Run: `python blender/speccade/tests/test_armature_driven_bridge.py`
Expected: PASS

**Step 5: Commit**

```bash
git add blender/speccade/armature_driven.py
git add blender/speccade/tests/test_armature_driven_bridge.py
git commit -m "feat(blender): add vertex group naming helpers for bridge tracking"
```

---

## Task 6: Modify Mesh Construction to Track Edge Loops

**Files:**
- Modify: `blender/speccade/armature_driven.py:260-475` (_build_mesh_with_steps function)

**Step 1: Add parameters to track bridging**

Modify the `_build_mesh_with_steps` function signature to accept connection mode:

Find the function definition (around line 260):
```python
def _build_mesh_with_steps(
```

Add parameters:
```python
def _build_mesh_with_steps(
    *,
    bpy_module,
    bmesh_module,
    bone_name: str,
    bone_length: float,
    head_w,
    base_q,
    profile: tuple,
    profile_radius: tuple,
    steps: list,
    cap_start: bool,
    cap_end: bool,
    select_only,
    ensure_object_mode,
    apply_scale_only,
    apply_rotation_only,
    connect_start: str | None = None,  # ADD THIS
    connect_end: str | None = None,    # ADD THIS
) -> object:
```

**Step 2: Add edge loop vertex group creation**

After mesh construction completes (around line 468, before `return obj`), add:

```python
    # Track edge loops for bridging via vertex groups
    if connect_start == "bridge" or connect_end == "bridge":
        mesh = obj.data
        bm = bmesh_module.new()
        try:
            bm.from_mesh(mesh)
            bm.verts.ensure_lookup_table()

            if bm.verts:
                z_coords = [v.co.z for v in bm.verts]
                z_min = min(z_coords)
                z_max = max(z_coords)
                z_range = z_max - z_min
                eps = max(1e-6, z_range * 0.01)

                head_verts = []
                tail_verts = []

                for v in bm.verts:
                    if abs(v.co.z - z_min) <= eps:
                        head_verts.append(v.index)
                    elif abs(v.co.z - z_max) <= eps:
                        tail_verts.append(v.index)

                bm.free()

                # Create vertex groups in object mode
                if connect_start == "bridge" and head_verts:
                    vg_name = get_bridge_head_vgroup_name(bone_name)
                    vg = obj.vertex_groups.new(name=vg_name)
                    vg.add(head_verts, 1.0, 'REPLACE')

                if connect_end == "bridge" and tail_verts:
                    vg_name = get_bridge_tail_vgroup_name(bone_name)
                    vg = obj.vertex_groups.new(name=vg_name)
                    vg.add(tail_verts, 1.0, 'REPLACE')
            else:
                bm.free()
        except Exception:
            try:
                bm.free()
            except Exception:
                pass
            raise
```

**Step 3: Update the caller to pass connection modes**

Find where `_build_mesh_with_steps` is called in `build_armature_driven_character_mesh` and add the new parameters:

```python
connect_start = bone_def.get("connect_start")
connect_end = bone_def.get("connect_end")

segment_obj = _build_mesh_with_steps(
    # ... existing params ...
    connect_start=connect_start,
    connect_end=connect_end,
)
```

**Step 4: Commit**

```bash
git add blender/speccade/armature_driven.py
git commit -m "feat(blender): track edge loop vertices for bridging"
```

---

## Task 7: Implement Bridge Operation

**Files:**
- Modify: `blender/speccade/armature_driven.py` (add after mesh joining, before modifiers)

**Step 1: Add bridge operation function**

Add new function after the helper functions:

```python
def _perform_bridge_operations(
    *,
    bpy_module,
    bmesh_module,
    mesh_obj,
    bridge_pairs: list[tuple[str, str]],
) -> None:
    """Bridge edge loops between connected bone pairs.

    Args:
        bpy_module: Blender bpy module
        bmesh_module: Blender bmesh module
        mesh_obj: The combined mesh object
        bridge_pairs: List of (parent_bone, child_bone) tuples to bridge
    """
    if not bridge_pairs:
        return

    bpy = bpy_module

    # Enter edit mode
    bpy.context.view_layer.objects.active = mesh_obj
    bpy.ops.object.mode_set(mode='EDIT')

    try:
        for parent_bone, child_bone in bridge_pairs:
            tail_vg = get_bridge_tail_vgroup_name(parent_bone)
            head_vg = get_bridge_head_vgroup_name(child_bone)

            # Check if both vertex groups exist
            if tail_vg not in mesh_obj.vertex_groups:
                continue
            if head_vg not in mesh_obj.vertex_groups:
                continue

            # Deselect all
            bpy.ops.mesh.select_all(action='DESELECT')

            # Select vertices in parent's tail group
            bpy.ops.object.mode_set(mode='OBJECT')
            tail_vg_idx = mesh_obj.vertex_groups[tail_vg].index
            for v in mesh_obj.data.vertices:
                for g in v.groups:
                    if g.group == tail_vg_idx:
                        v.select = True
                        break

            # Also select vertices in child's head group
            head_vg_idx = mesh_obj.vertex_groups[head_vg].index
            for v in mesh_obj.data.vertices:
                for g in v.groups:
                    if g.group == head_vg_idx:
                        v.select = True
                        break

            bpy.ops.object.mode_set(mode='EDIT')

            # Bridge the selected edge loops
            try:
                bpy.ops.mesh.bridge_edge_loops()
            except RuntimeError as e:
                # Bridge can fail if selection isn't valid edge loops
                import sys
                print(f"Warning: bridge_edge_loops failed for {parent_bone}->{child_bone}: {e}", file=sys.stderr)

        # Clean up: merge very close vertices at bridge points
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.mesh.remove_doubles(threshold=0.0001)

    finally:
        bpy.ops.object.mode_set(mode='OBJECT')

    # Remove temporary bridge vertex groups
    for parent_bone, child_bone in bridge_pairs:
        for vg_name in [
            get_bridge_tail_vgroup_name(parent_bone),
            get_bridge_head_vgroup_name(child_bone),
        ]:
            if vg_name in mesh_obj.vertex_groups:
                mesh_obj.vertex_groups.remove(mesh_obj.vertex_groups[vg_name])
```

**Step 2: Integrate into build_armature_driven_character_mesh**

Find where segments are joined (search for `_join_into` or `object.join`). After joining all segments, add:

```python
# Determine bridge pairs and perform bridging
bone_hierarchy = _build_bone_hierarchy(armature)  # Need to implement this
bridge_pairs = get_bridge_pairs(bone_hierarchy, resolved_bone_meshes)

if bridge_pairs:
    _perform_bridge_operations(
        bpy_module=bpy,
        bmesh_module=bmesh,
        mesh_obj=combined_mesh,
        bridge_pairs=bridge_pairs,
    )
```

**Step 3: Add bone hierarchy helper**

```python
def _build_bone_hierarchy(armature) -> dict:
    """Build bone hierarchy dict from armature.

    Returns:
        Dict mapping bone_name -> {"parent": str|None, "children": [str]}
    """
    hierarchy = {}

    for bone in armature.data.bones:
        parent_name = bone.parent.name if bone.parent else None
        hierarchy[bone.name] = {
            "parent": parent_name,
            "children": [],
        }

    # Fill in children
    for bone_name, info in hierarchy.items():
        parent_name = info["parent"]
        if parent_name and parent_name in hierarchy:
            hierarchy[parent_name]["children"].append(bone_name)

    return hierarchy
```

**Step 4: Commit**

```bash
git add blender/speccade/armature_driven.py
git commit -m "feat(blender): implement bridge edge loop operation"
```

---

## Task 8: Implement Weight Blending for Bridged Regions

**Files:**
- Modify: `blender/speccade/armature_driven.py`

**Step 1: Add weight blending function**

Add after `_perform_bridge_operations`:

```python
def _blend_bridge_weights(
    *,
    bpy_module,
    mesh_obj,
    parent_bone: str,
    child_bone: str,
    armature,
) -> None:
    """Blend skin weights for vertices in the bridge region.

    Vertices in the bridge get interpolated weights between parent and child bones
    based on their position along the bridge axis.
    """
    bpy = bpy_module

    # Get bone positions for interpolation
    parent_bone_obj = armature.data.bones.get(parent_bone)
    child_bone_obj = armature.data.bones.get(child_bone)

    if not parent_bone_obj or not child_bone_obj:
        return

    # Parent tail and child head positions in armature space
    parent_tail = armature.matrix_world @ parent_bone_obj.tail_local
    child_head = armature.matrix_world @ child_bone_obj.head_local

    # Bridge axis
    bridge_vec = child_head - parent_tail
    bridge_len = bridge_vec.length

    if bridge_len < 1e-6:
        return  # Bones are at same position

    bridge_dir = bridge_vec.normalized()

    # Get or create vertex groups for the bones
    parent_vg = mesh_obj.vertex_groups.get(parent_bone)
    child_vg = mesh_obj.vertex_groups.get(child_bone)

    if not parent_vg:
        parent_vg = mesh_obj.vertex_groups.new(name=parent_bone)
    if not child_vg:
        child_vg = mesh_obj.vertex_groups.new(name=child_bone)

    # Find vertices in bridge region (between parent tail and child head)
    mesh = mesh_obj.data
    world_matrix = mesh_obj.matrix_world

    for v in mesh.vertices:
        v_world = world_matrix @ v.co

        # Project vertex onto bridge axis
        to_vert = v_world - parent_tail
        proj_dist = to_vert.dot(bridge_dir)

        # Check if vertex is in bridge region
        if proj_dist < -0.001 or proj_dist > bridge_len + 0.001:
            continue  # Outside bridge region

        # Calculate blend factor (0 = parent, 1 = child)
        t = max(0.0, min(1.0, proj_dist / bridge_len))

        parent_weight = 1.0 - t
        child_weight = t

        # Apply blended weights
        parent_vg.add([v.index], parent_weight, 'REPLACE')
        child_vg.add([v.index], child_weight, 'REPLACE')
```

**Step 2: Call weight blending after bridge operations**

In `_perform_bridge_operations`, after the bridge loop but before removing temp vertex groups:

```python
    # Blend weights for bridged regions
    for parent_bone, child_bone in bridge_pairs:
        _blend_bridge_weights(
            bpy_module=bpy,
            mesh_obj=mesh_obj,
            parent_bone=parent_bone,
            child_bone=child_bone,
            armature=armature,
        )
```

Update the function signature to accept `armature`:

```python
def _perform_bridge_operations(
    *,
    bpy_module,
    bmesh_module,
    mesh_obj,
    bridge_pairs: list[tuple[str, str]],
    armature,  # ADD THIS
) -> None:
```

**Step 3: Commit**

```bash
git add blender/speccade/armature_driven.py
git commit -m "feat(blender): add weight blending for bridged regions"
```

---

## Task 9: Add Integration Test Spec

**Files:**
- Create: `specs/test/bridge_edge_loops_test.json`

**Step 1: Create test spec**

```json
{
  "asset_id": "test/bridge_edge_loops_character",
  "asset_type": "skeletal_mesh",
  "recipe": {
    "kind": "skeletal_mesh.armature_driven_v1",
    "params": {
      "skeleton": [
        {"name": "root", "head": [0, 0, 0], "tail": [0, 0, 0.2]},
        {"name": "spine", "head": [0, 0, 0.2], "tail": [0, 0, 0.5], "parent": "root"},
        {"name": "chest", "head": [0, 0, 0.5], "tail": [0, 0, 0.8], "parent": "spine"},
        {"name": "neck", "head": [0, 0, 0.8], "tail": [0, 0, 1.0], "parent": "chest"}
      ],
      "bone_meshes": {
        "spine": {
          "profile": "circle(8)",
          "profile_radius": 0.1,
          "extrusion_steps": [0.5, 0.5],
          "cap_start": true,
          "connect_end": "bridge"
        },
        "chest": {
          "profile": "circle(8)",
          "profile_radius": 0.12,
          "extrusion_steps": [0.5, 0.5],
          "connect_start": "bridge",
          "connect_end": "bridge"
        },
        "neck": {
          "profile": "circle(8)",
          "profile_radius": 0.06,
          "extrusion_steps": [0.5, 0.5],
          "connect_start": "bridge",
          "cap_end": true
        }
      },
      "export": {
        "include_armature": true,
        "include_skin_weights": true
      }
    }
  }
}
```

**Step 2: Commit**

```bash
git add specs/test/bridge_edge_loops_test.json
git commit -m "test: add integration test spec for bridge edge loops"
```

---

## Task 10: Update Documentation

**Files:**
- Modify: `docs/spec-reference/character.md`

**Step 1: Add documentation for new fields**

Add to the `armature_driven_v1` params table:

```markdown
| `connect_start` | string | No | Connection mode for mesh start: `"bridge"` or `"segmented"` (default) |
| `connect_end` | string | No | Connection mode for mesh end: `"bridge"` or `"segmented"` (default) |
```

Add new section after "Mirror References":

```markdown
### Bridge Edge Loops

Connect bone mesh segments topologically using bridge edge loops for smooth deformation at joints.

**Connection modes:**
- `"segmented"` (default): Mesh ends are independent, capped or uncapped per `cap_start`/`cap_end`
- `"bridge"`: Merge edge loops with adjacent bone's mesh, blend weights at junction

**Requirements for bridging:**
- Both parent's `connect_end` and child's `connect_start` must be `"bridge"`
- Profile segment counts must be compatible (exact match or 2x multiple)
- Both bones must have mesh definitions (not mirror references)

**Example - connected torso:**

```json
{
  "bone_meshes": {
    "spine": {
      "profile": "circle(8)",
      "connect_end": "bridge"
    },
    "chest": {
      "profile": "circle(8)",
      "connect_start": "bridge",
      "connect_end": "bridge"
    },
    "neck": {
      "profile": "circle(8)",
      "connect_start": "bridge"
    }
  }
}
```

**Weight blending:** Vertices in the bridge region receive interpolated weights between the parent and child bones based on their position along the bridge axis.
```

**Step 2: Commit**

```bash
git add docs/spec-reference/character.md
git commit -m "docs: add bridge edge loops documentation"
```

---

## Task 11: Run Full Integration Test

**Step 1: Build the test spec**

Run: `cargo run -p speccade-cli -- build specs/test/bridge_edge_loops_test.json -o out/test/`

Expected: GLB file generated successfully

**Step 2: Validate the output**

Run: `cargo run -p speccade-cli -- validate out/test/bridge_edge_loops_character.glb`

Expected: Validation passes

**Step 3: Visual verification (manual)**

Open `out/test/bridge_edge_loops_character.glb` in Blender and verify:
- Mesh is manifold at spine→chest→neck junctions
- Edge loops flow continuously across joints
- Weight painting shows gradient at bridge regions
- Posing bones shows smooth deformation at joints

**Step 4: Commit final state**

```bash
git add -A
git commit -m "feat: complete bridge edge loop support for armature-driven characters"
```

---

## Verification Checklist

- [ ] Rust types parse correctly (`cargo test -p speccade-spec`)
- [ ] Python unit tests pass (`python -m pytest blender/speccade/tests/test_armature_driven_bridge.py`)
- [ ] Integration test spec builds successfully
- [ ] Existing specs still work (no regressions)
- [ ] Visual verification in Blender shows connected topology
- [ ] Weight blending produces smooth gradients at bridges
