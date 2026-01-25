# SpecCade Roadmap Completion Plan

**Date**: 2026-01-25
**Status**: Ready for execution
**Execution Method**: `superpowers:dispatching-parallel-agents`

---

## Overview & Goals

**Purpose**: Complete all 11 remaining SpecCade roadmap tasks, bringing the project to feature-complete status for v1.

**Success Criteria**:
- All tasks marked complete in `docs/ROADMAP.md`
- All new features have tests (unit + integration where applicable)
- Tier 2 (Blender) features have golden specs in `golden/`
- Documentation updated for new capabilities
- No regressions in existing functionality

**Task Summary**:

| ID | Task | Category | Tier |
|----|------|----------|------|
| EDITOR-003 | Monaco + Starlark syntax | Editor | 1 (Rust/TS) |
| EDITOR-007 | LOD proxy generation | Editor | 1 (Rust/TS) |
| MESH-011 | Shrinkwrap workflows | Mesh | 2 (Blender) |
| MESH-012 | Boolean kitbashing | Mesh | 2 (Blender) |
| MESH-013 | Animation helper presets | Mesh | 2 (Blender) |
| ANIM-004 | Rigging parity gaps | Animation | 2 (Blender) |
| ANIM-005 | Hard constraint validation | Animation | 2 (Blender) |
| ANIM-006 | Root motion controls | Animation | 2 (Blender) |
| QA-006 | Plugin extension story | Tooling | 1 (Design) |
| MIGRATE-003 | Legacy ANIMATION mapping | Migration | 1 (Rust) |
| MIGRATE-004 | Legacy CHARACTER mapping | Migration | 1 (Rust) |

---

## Dependency Graph

**Legend**: `A → B` means A must complete before B can start.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           INDEPENDENT TRACKS                            │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  EDITOR TRACK          MESH TRACK           ANIMATION TRACK             │
│  ─────────────         ──────────           ───────────────             │
│  EDITOR-003 ─┐         MESH-011 ──┐         ANIM-004 ──┐                │
│              │                    │                    │                │
│  EDITOR-007 ─┴─► (done)  MESH-012 ─┼──► (done)  ANIM-005 ─┼──► (done)   │
│                                   │                    │                │
│                         MESH-013 ─┘         ANIM-006 ──┘                │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                         │
│  MIGRATION TRACK                  TOOLING TRACK                         │
│  ───────────────                  ─────────────                         │
│  MIGRATE-003 ──┐                  QA-006 (standalone)                   │
│                │                                                        │
│  MIGRATE-004 ──┴─► (done)                                               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

**Key Observations**:

1. **5 independent tracks** - Editor, Mesh, Animation, Migration, Tooling can all run in parallel.
2. **Within-track independence** - All tasks within each track are independent.
3. **Maximum parallelism** - All 11 tasks can theoretically run concurrently.
4. **Recommended batching** - Group by shared context to reduce agent overhead.

---

## Task Specifications

### EDITOR-003: Monaco + Starlark Syntax Highlighting

**Objective**: Integrate Monaco editor with Starlark syntax highlighting and inline validation.

**Deliverables**:
1. Monaco editor component with `.star` file support
2. Starlark syntax highlighting (keywords, strings, comments, builtins)
3. Inline parse error display (red squiggles, error messages)
4. Integration with existing `eval_spec` command for validation

**Touch Points**:
```
editor/src/components/Editor.ts       # New Monaco wrapper component
editor/src/lib/starlark-language.ts   # Language definition for Monaco
editor/src/lib/starlark-tokens.ts     # Token provider (syntax rules)
editor/package.json                   # Add monaco-editor dependency
```

**Acceptance Criteria**:
- [ ] `.star` files open with syntax highlighting
- [ ] Keywords highlighted: `def`, `if`, `for`, `return`, `True`, `False`, `None`
- [ ] SpecCade stdlib functions highlighted: `spec()`, `output()`, `synth()`, etc.
- [ ] Parse errors shown inline within 500ms of typing pause
- [ ] No flicker on rapid edits (debounced validation)

---

### EDITOR-007: LOD Proxy Generation

**Objective**: Generate low-poly mesh proxies for sub-100ms first-frame preview.

**Deliverables**:
1. Rust-side mesh decimation for preview proxy
2. Quality indicator in UI ("Preview" vs "Full Quality" badge)
3. Progressive refinement on idle or user request

**Touch Points**:
```
crates/speccade-editor/src/preview/mesh.rs  # Add decimation logic
crates/speccade-editor/src/preview/lod.rs   # New LOD generation module
editor/src/components/MeshPreview.ts        # Quality badge, refine button
```

**Acceptance Criteria**:
- [ ] Meshes >10k triangles show proxy first (<100ms)
- [ ] Proxy preserves silhouette (edge-collapse, not random decimation)
- [ ] Visual badge shows "Preview" state
- [ ] "Refine" button or auto-refine on 2s idle
- [ ] Full quality mesh replaces proxy seamlessly

---

### MESH-011: Shrinkwrap Workflows

**Objective**: Enable armor/clothing wrapping onto body meshes with stability validation.

**Deliverables**:
1. `shrinkwrap_v1` recipe type in spec schema
2. Blender backend implementation (shrinkwrap modifier)
3. Validation for self-intersection and mesh quality post-wrap
4. Golden specs for armor/clothing use cases

**Touch Points**:
```
crates/speccade-spec/src/recipe/mesh/shrinkwrap.rs    # New recipe type
crates/speccade-spec/src/recipe/mesh/mod.rs           # Register type
blender/operators/shrinkwrap.py                        # Blender operator
blender/entrypoint.py                                  # Wire up operator
golden/speccade/specs/mesh/shrinkwrap/                 # Golden specs
```

**Schema Design**:
```json
{
  "type": "mesh.shrinkwrap_v1",
  "params": {
    "base_mesh": "body_torso",
    "wrap_mesh": "armor_plate",
    "mode": "nearest_surface|project|nearest_vertex",
    "offset": 0.02,
    "smooth_iterations": 2,
    "validation": {
      "max_self_intersections": 0,
      "min_face_area": 0.0001
    }
  }
}
```

**Acceptance Criteria**:
- [ ] Shrinkwrap works for nearest_surface, project, nearest_vertex modes
- [ ] Offset parameter creates consistent gap
- [ ] Self-intersection validation fails specs with overlapping geometry
- [ ] At least 3 golden specs (armor plate, cloth drape, fitted accessory)

---

### MESH-012: Boolean Kitbashing

**Objective**: Union/difference operations with cleanup for hard-surface modeling.

**Deliverables**:
1. `boolean_kit_v1` recipe type
2. Blender boolean modifier + mesh cleanup pipeline
3. Determinism constraints (operation order, solver selection)
4. Validation for non-manifold results

**Touch Points**:
```
crates/speccade-spec/src/recipe/mesh/boolean_kit.rs   # New recipe type
blender/operators/boolean_kit.py                       # Blender operator
blender/operators/mesh_cleanup.py                      # Post-boolean cleanup
golden/speccade/specs/mesh/boolean_kit/                # Golden specs
```

**Schema Design**:
```json
{
  "type": "mesh.boolean_kit_v1",
  "params": {
    "base": "hull_body",
    "operations": [
      { "op": "union", "target": "engine_block" },
      { "op": "difference", "target": "window_cutout" },
      { "op": "intersect", "target": "trim_volume" }
    ],
    "solver": "exact|fast",
    "cleanup": {
      "merge_distance": 0.001,
      "remove_doubles": true,
      "recalc_normals": true
    }
  }
}
```

**Acceptance Criteria**:
- [ ] Union, difference, intersect operations work
- [ ] Operation order is deterministic (sequential application)
- [ ] Cleanup removes artifacts (doubles, flipped normals)
- [ ] Non-manifold results flagged in validation report
- [ ] At least 3 golden specs (vehicle hull, building cutouts, mechanical part)

---

### MESH-013: Animation Helper Presets

**Objective**: IK target and constraint presets for procedural walk/run cycles.

**Deliverables**:
1. `animation_helpers_v1` recipe type
2. Preset library: walk_cycle, run_cycle, idle_sway
3. IK pole targets and foot roll systems
4. Blender driver setup for cycle parameters

**Touch Points**:
```
crates/speccade-spec/src/recipe/animation/helpers.rs  # New recipe type
blender/operators/animation_helpers.py                 # Preset application
blender/presets/locomotion/                            # Preset definitions
golden/speccade/specs/animation/helpers/               # Golden specs
```

**Schema Design**:
```json
{
  "type": "animation.helpers_v1",
  "params": {
    "skeleton": "humanoid_rig",
    "preset": "walk_cycle|run_cycle|idle_sway",
    "settings": {
      "stride_length": 0.8,
      "cycle_frames": 24,
      "foot_roll": true,
      "arm_swing": 0.3
    },
    "ik_targets": {
      "foot_l": { "pole_angle": 90, "chain_length": 2 },
      "foot_r": { "pole_angle": 90, "chain_length": 2 }
    }
  }
}
```

**Acceptance Criteria**:
- [ ] walk_cycle, run_cycle, idle_sway presets generate valid animations
- [ ] IK targets created with correct pole angles
- [ ] Foot roll system functional (heel lift → toe off)
- [ ] Cycle is loopable (first/last frame match)
- [ ] At least 2 golden specs per preset type

---

### ANIM-004: Rigging Parity Gaps

**Objective**: Document and implement missing rigging features (IK stretch, foot roll, spine presets).

**Deliverables**:
1. Audit of current rigging capabilities vs. common game requirements
2. IK stretch settings implementation
3. Foot roll system presets
4. Basic spine rig preset (FK/IK switchable)
5. Reference documentation for all rigging options

**Touch Points**:
```
crates/speccade-spec/src/recipe/animation/rig_setup.rs  # Extend rig options
blender/operators/rigging.py                             # IK stretch, foot roll
blender/presets/rigs/spine_basic.py                      # Spine preset
docs/spec-reference/animation/rigging.md                 # Reference docs
golden/speccade/specs/skeletal_animation/rigging/        # Probe specs
```

**Schema Additions**:
```json
{
  "rig_setup": {
    "ik_settings": {
      "stretch": { "enabled": true, "limit": 1.5 },
      "soft_ik": { "enabled": true, "softness": 0.2 }
    },
    "foot_roll": {
      "enabled": true,
      "heel_pivot": "auto",
      "toe_pivot": "auto",
      "roll_range": [-30, 60]
    },
    "spine_preset": "fk_only|ik_spline|fk_ik_switch"
  }
}
```

**Acceptance Criteria**:
- [ ] IK stretch works with configurable limit
- [ ] Soft IK prevents popping at full extension
- [ ] Foot roll heel/toe pivots placed correctly
- [ ] Spine preset generates functional rig
- [ ] Docs cover all new options with examples
- [ ] At least 2 probe specs per feature

---

### ANIM-005: Hard Constraint Validation

**Objective**: Validate ball socket, planar constraints with stiffness/influence semantics.

**Deliverables**:
1. Ball socket constraint implementation + validation
2. Planar constraint implementation + validation
3. Stiffness and influence parameter semantics defined
4. Verification checks that constraints behave as specified

**Touch Points**:
```
crates/speccade-spec/src/recipe/animation/constraints.rs  # Constraint types
crates/speccade-spec/src/validation/constraints/          # Validation rules
blender/operators/constraints.py                           # Constraint setup
blender/verification/constraint_checks.py                  # Runtime checks
golden/speccade/specs/skeletal_animation/constraints/      # Probe specs
```

**Schema Design**:
```json
{
  "constraints": [
    {
      "type": "ball_socket",
      "target": "shoulder_joint",
      "pivot": [0, 0, 0],
      "limits": {
        "swing_x": [-45, 45],
        "swing_y": [-30, 90],
        "twist": [-20, 20]
      },
      "stiffness": 0.8,
      "influence": 1.0
    },
    {
      "type": "planar",
      "target": "knee_joint",
      "plane_normal": [1, 0, 0],
      "influence": 1.0
    }
  ]
}
```

**Acceptance Criteria**:
- [ ] Ball socket respects swing/twist limits
- [ ] Planar constraint locks rotation to specified plane
- [ ] Stiffness affects constraint resistance (0=loose, 1=rigid)
- [ ] Influence blends between constrained/unconstrained
- [ ] Validation catches out-of-range parameters
- [ ] At least 3 probe specs (shoulder, knee, spine joint)

---

### ANIM-006: Root Motion Controls

**Objective**: Explicit root motion extraction, locking, and validation for game export.

**Deliverables**:
1. Root motion settings in animation recipe
2. Export modes: extract, lock, bake_in_place
3. Validation that root motion matches expected displacement
4. Report section showing root motion metrics

**Touch Points**:
```
crates/speccade-spec/src/recipe/animation/root_motion.rs  # Root motion config
crates/speccade-backend-blender/src/metrics.rs            # Root motion metrics
blender/operators/root_motion.py                           # Extraction logic
blender/export/root_motion_export.py                       # Export handling
docs/spec-reference/animation/root-motion.md               # Reference docs
```

**Schema Design**:
```json
{
  "root_motion": {
    "mode": "extract|lock|bake_in_place",
    "extract_settings": {
      "axis": ["x", "z"],
      "ignore_y": true
    },
    "validation": {
      "expected_displacement": [2.5, 0, 0],
      "tolerance": 0.1
    }
  }
}
```

**Metrics Output**:
```json
{
  "root_motion_report": {
    "total_displacement": [2.48, 0.01, 0.02],
    "per_frame_velocity": [],
    "loop_error": 0.02,
    "validation_passed": true
  }
}
```

**Acceptance Criteria**:
- [ ] Extract mode moves root bone delta to separate channel
- [ ] Lock mode zeroes root translation
- [ ] Bake-in-place keeps animation but zeroes final position
- [ ] Validation compares actual vs expected displacement
- [ ] Loop error reported (mismatch between first/last frame)
- [ ] At least 2 golden specs (walk extract, idle lock)

---

### QA-006: Plugin/Backends Extension Story

**Objective**: Define how external backends can integrate with SpecCade via subprocess or WASM, with strict I/O contracts and determinism reporting.

**Deliverables**:
1. Extension architecture design document
2. I/O contract specification (JSON schema for inputs/outputs)
3. Determinism reporting requirements (hash, tier declaration)
4. Reference implementation for subprocess backend
5. WASM backend proof-of-concept (optional, can be deferred)

**Touch Points**:
```
docs/architecture/plugin-system.md                    # Architecture doc
docs/spec-reference/extensions/io-contract.md         # I/O contract spec
crates/speccade-spec/src/extension/mod.rs             # Extension types
crates/speccade-spec/src/extension/contract.rs        # Contract validation
crates/speccade-cli/src/backends/subprocess.rs        # Subprocess runner
crates/speccade-cli/src/backends/mod.rs               # Backend registry
examples/extensions/simple-subprocess/                 # Reference impl
```

**I/O Contract Schema**:
```json
{
  "extension": {
    "name": "custom-texture-gen",
    "version": "1.0.0",
    "tier": 1,
    "determinism": "byte_identical|semantic_equivalent|non_deterministic",
    "interface": "subprocess|wasm",
    "input_schema": { "$ref": "input.schema.json" },
    "output_schema": { "$ref": "output.schema.json" }
  }
}
```

**Subprocess Protocol**:
```
1. CLI spawns subprocess with: <exe> --spec <path> --out <path> --seed <u64>
2. Subprocess reads JSON spec from stdin (or file)
3. Subprocess writes outputs to --out directory
4. Subprocess writes manifest.json with:
   - output_files: [{ path, hash, size }]
   - determinism_report: { input_hash, output_hash, tier }
   - errors: [{ code, message }]
5. Exit code 0 = success, non-zero = failure
```

**Acceptance Criteria**:
- [ ] Architecture doc covers subprocess + WASM approaches
- [ ] I/O contract schema defined and validated
- [ ] Subprocess runner executes external backend
- [ ] Determinism report captured and surfaced in CLI output
- [ ] Reference subprocess extension builds and runs
- [ ] Tier declaration enforced (Tier 1 must be reproducible)
- [ ] Error handling for subprocess timeout/crash

---

### MIGRATE-003: Legacy ANIMATION Mapping

**Objective**: Map legacy `spec.py` ANIMATION dict keys to canonical `skeletal_animation.blender_rigged_v1` params.

**Deliverables**:
1. Mapping functions for ANIMATION category
2. Support for rig_setup, poses, phases, IK parameters
3. Actionable diagnostics for unknown/unsupported keys
4. Migration tests validating output against `speccade validate`

**Touch Points**:
```
crates/speccade-cli/src/commands/migrate/conversion/animation.rs  # New module
crates/speccade-cli/src/commands/migrate/conversion/mod.rs        # Register
crates/speccade-cli/src/commands/migrate/mod.rs                   # Wire up
docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md                        # Update parity
tests/migrate/fixtures/animation/                                  # Test fixtures
```

**Legacy Format** (from `spec.py`):
```python
{
  "type": "ANIMATION",
  "name": "walk_cycle",
  "rig": "humanoid",
  "poses": {
    "contact_l": { "frame": 0, "bones": {} },
    "passing_r": { "frame": 6, "bones": {} }
  },
  "phases": ["contact_l", "passing_r", "contact_r", "passing_l"],
  "ik_targets": { "foot_l": {}, "foot_r": {} },
  "loop": true
}
```

**Canonical Output**:
```json
{
  "type": "skeletal_animation.blender_rigged_v1",
  "params": {
    "skeleton_ref": "humanoid",
    "animation_name": "walk_cycle",
    "keyframes": [],
    "ik_setup": {},
    "loop": true
  }
}
```

**Mapping Rules**:

| Legacy Key | Canonical Path | Notes |
|------------|----------------|-------|
| `rig` | `params.skeleton_ref` | Direct map |
| `poses` | `params.keyframes` | Convert to keyframe array |
| `phases` | `params.keyframes[].name` | Order determines frame sequence |
| `ik_targets` | `params.ik_setup` | Restructure to canonical IK format |
| `loop` | `params.loop` | Direct map |

**Acceptance Criteria**:
- [ ] All known ANIMATION keys mapped or explicitly rejected
- [ ] Unknown keys produce warning with suggested action
- [ ] Migrated specs pass `speccade validate`
- [ ] At least 5 test fixtures (walk, run, idle, attack, jump)
- [ ] Parity matrix updated with ANIMATION row

---

### MIGRATE-004: Legacy CHARACTER Mapping

**Objective**: Map legacy `spec.py` CHARACTER dict keys to canonical `skeletal_mesh.blender_rigged_mesh_v1` params.

**Deliverables**:
1. Mapping functions for CHARACTER category
2. Support for skeleton, body_parts, skinning, export settings
3. Triangle budget preservation where specified
4. Migration tests with validation

**Touch Points**:
```
crates/speccade-cli/src/commands/migrate/conversion/character.rs  # New module
crates/speccade-cli/src/commands/migrate/conversion/mod.rs        # Register
docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md                        # Update parity
tests/migrate/fixtures/character/                                  # Test fixtures
```

**Legacy Format**:
```python
{
  "type": "CHARACTER",
  "name": "player_hero",
  "skeleton": "humanoid_rig",
  "body_parts": {
    "head": { "mesh": "head_mesh", "material": "skin" },
    "torso": { "mesh": "torso_mesh", "material": "armor" },
    "arms": { "mesh": "arms_mesh", "material": "skin" }
  },
  "skinning": {
    "method": "automatic",
    "bone_influences": 4
  },
  "export": {
    "format": "glb",
    "triangles_max": 15000
  }
}
```

**Canonical Output**:
```json
{
  "type": "skeletal_mesh.blender_rigged_mesh_v1",
  "params": {
    "name": "player_hero",
    "skeleton": { "preset": "humanoid_rig" },
    "parts": [
      { "name": "head", "mesh_ref": "head_mesh", "material": "skin" },
      { "name": "torso", "mesh_ref": "torso_mesh", "material": "armor" },
      { "name": "arms", "mesh_ref": "arms_mesh", "material": "skin" }
    ],
    "skinning": {
      "method": "automatic",
      "max_influences": 4
    },
    "output": {
      "format": "glb",
      "budget": { "max_triangles": 15000 }
    }
  }
}
```

**Mapping Rules**:

| Legacy Key | Canonical Path | Notes |
|------------|----------------|-------|
| `skeleton` | `params.skeleton.preset` | Wrap in object |
| `body_parts` | `params.parts` | Dict → array with name field |
| `skinning.method` | `params.skinning.method` | Direct map |
| `skinning.bone_influences` | `params.skinning.max_influences` | Rename |
| `export.format` | `params.output.format` | Direct map |
| `export.triangles_max` | `params.output.budget.max_triangles` | Restructure |

**Acceptance Criteria**:
- [ ] All known CHARACTER keys mapped or rejected
- [ ] Body parts dict converted to array correctly
- [ ] Triangle budgets preserved in output
- [ ] Unknown keys produce actionable warnings
- [ ] At least 5 test fixtures (humanoid, quadruped, mech, creature, simple)
- [ ] Parity matrix updated with CHARACTER row

---

## Parallelization Strategy

**Execution Model**: Using `superpowers:dispatching-parallel-agents` to run independent tasks concurrently.

### Recommended: 3-Wave Execution

```
┌─────────────────────────────────────────────────────────────────────────┐
│ WAVE 1: Foundation (4 parallel agents)                                  │
│ ─────────────────────────────────────                                   │
│                                                                         │
│   Agent A          Agent B          Agent C          Agent D            │
│   ─────────        ─────────        ─────────        ─────────          │
│   EDITOR-003       EDITOR-007       QA-006           MIGRATE-003        │
│   (Monaco)         (LOD proxy)      (Plugin arch)    (ANIMATION)        │
│                                                                         │
│   Context:         Context:         Context:         Context:           │
│   Rust/TS          Rust/TS          Design/Rust      Rust               │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│ WAVE 2: Blender Mesh (3 parallel agents)                                │
│ ────────────────────────────────────────                                │
│                                                                         │
│   Agent E          Agent F          Agent G                             │
│   ─────────        ─────────        ─────────                           │
│   MESH-011         MESH-012         MESH-013                            │
│   (Shrinkwrap)     (Boolean)        (Anim helpers)                      │
│                                                                         │
│   Context:         Context:         Context:                            │
│   Blender/Py       Blender/Py       Blender/Py                          │
│                                                                         │
├─────────────────────────────────────────────────────────────────────────┤
│ WAVE 3: Blender Animation + Migration (4 parallel agents)               │
│ ──────────────────────────────────────────────────────────              │
│                                                                         │
│   Agent H          Agent I          Agent J          Agent K            │
│   ─────────        ─────────        ─────────        ─────────          │
│   ANIM-004         ANIM-005         ANIM-006         MIGRATE-004        │
│   (Rig parity)     (Constraints)    (Root motion)    (CHARACTER)        │
│                                                                         │
│   Context:         Context:         Context:         Context:           │
│   Blender/Py       Blender/Py       Blender/Py       Rust               │
│                                                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Why 3 Waves?

1. **Context efficiency**: Agents in the same wave share similar tooling context
2. **Resource management**: 3-4 parallel agents is manageable for review checkpoints
3. **Risk isolation**: If Blender backend has issues, Wave 1 work is already complete
4. **Natural checkpoints**: Review after each wave before proceeding

### Alternative: Single Wave (Maximum Parallelism)

If speed is critical, all 11 agents can launch simultaneously:

```
SINGLE WAVE: All 11 tasks in parallel

  EDITOR-003  EDITOR-007  QA-006  MIGRATE-003  MIGRATE-004
  MESH-011    MESH-012    MESH-013
  ANIM-004    ANIM-005    ANIM-006

  Pros: Fastest wall-clock time
  Cons: Harder to review, potential merge conflicts in shared files
```

### Agent Prompt Template

Each agent receives a focused prompt:

```markdown
## Task: [TASK-ID] [Title]

**Objective**: [One sentence]

**Deliverables**:
1. [Specific file/feature]
2. [Tests]
3. [Documentation]

**Touch Points**:
- [file paths]

**Acceptance Criteria**:
- [ ] [Criterion 1]
- [ ] [Criterion 2]

**Verification Commands**:
- `cargo test -p [crate]`
- `cargo run -p speccade-cli -- validate --spec [golden]`

**Done when**:
- All acceptance criteria checked
- Tests pass
- ROADMAP.md task marked complete
```

---

## Verification & Definition of Done

### Global Definition of Done

Every task must satisfy these criteria before being marked complete:

| Criterion | Verification Method |
|-----------|---------------------|
| **Compiles** | `cargo build --all-targets` passes |
| **Tests pass** | `cargo test` passes (no new failures) |
| **Lints clean** | `cargo clippy -- -D warnings` passes |
| **Formatted** | `cargo fmt --check` passes |
| **Documented** | New public APIs have doc comments |
| **ROADMAP updated** | Task checkbox marked `[x]` with date |

### Per-Category Verification

**Editor Tasks (EDITOR-003, EDITOR-007)**:
```bash
# Build frontend
cd editor && npm run build

# Build Tauri app
cd editor/src-tauri && cargo build

# Type check
cd editor && npm run typecheck

# Manual verification: open editor, load .star file, confirm highlighting/preview
```

**Blender Tasks (MESH-011-013, ANIM-004-006)**:
```bash
# Validate golden specs
cargo run -p speccade-cli -- validate --spec golden/speccade/specs/mesh/[feature]/

# Generate and verify determinism
cargo run -p speccade-cli -- generate --spec [golden] --out-root ./out
cargo run -p speccade-cli -- verify --spec [golden] --generated ./out

# Blender script syntax check
python -m py_compile blender/operators/[new_operator].py
```

**Migration Tasks (MIGRATE-003, MIGRATE-004)**:
```bash
# Run migration tests
cargo test -p speccade-cli migrate

# Test specific fixture
cargo run -p speccade-cli -- migrate --input tests/migrate/fixtures/[type]/input.py --output /tmp/out.json
cargo run -p speccade-cli -- validate --spec /tmp/out.json
```

**Tooling Task (QA-006)**:
```bash
# Build reference extension
cd examples/extensions/simple-subprocess && cargo build

# Test subprocess protocol
cargo run -p speccade-cli -- generate --spec [test] --backend subprocess --backend-path ./target/debug/simple-subprocess
```

### Verification Checklist Template

Each agent uses this checklist before declaring done:

```markdown
## Verification Checklist for [TASK-ID]

### Build
- [ ] `cargo build --all-targets` passes
- [ ] No new warnings introduced

### Tests
- [ ] `cargo test` passes
- [ ] New tests added for feature
- [ ] Golden specs added (if Tier 2)

### Quality
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt --check` passes

### Documentation
- [ ] Doc comments on public APIs
- [ ] Reference docs updated (if applicable)

### Integration
- [ ] ROADMAP.md updated with `[x]` and date
- [ ] No merge conflicts with main

### Manual Verification (if applicable)
- [ ] Feature works as specified
- [ ] Edge cases handled
```

### Post-Wave Review

After each wave completes:

1. **Merge check**: Ensure no conflicts between parallel work
2. **Integration test**: Full `cargo test` on merged code
3. **Regression check**: Existing golden tests still pass
4. **ROADMAP audit**: Confirm all wave tasks marked complete

---

## Execution Instructions

To execute this plan using `superpowers:dispatching-parallel-agents`:

1. **Start Wave 1**:
   ```
   /superpowers:dispatching-parallel-agents

   Tasks:
   - EDITOR-003: Monaco + Starlark syntax (see Task Specifications section)
   - EDITOR-007: LOD proxy generation (see Task Specifications section)
   - QA-006: Plugin extension story (see Task Specifications section)
   - MIGRATE-003: Legacy ANIMATION mapping (see Task Specifications section)
   ```

2. **Review Wave 1 outputs**, merge, run integration tests.

3. **Start Wave 2**:
   ```
   /superpowers:dispatching-parallel-agents

   Tasks:
   - MESH-011: Shrinkwrap workflows (see Task Specifications section)
   - MESH-012: Boolean kitbashing (see Task Specifications section)
   - MESH-013: Animation helper presets (see Task Specifications section)
   ```

4. **Review Wave 2 outputs**, merge, run integration tests.

5. **Start Wave 3**:
   ```
   /superpowers:dispatching-parallel-agents

   Tasks:
   - ANIM-004: Rigging parity gaps (see Task Specifications section)
   - ANIM-005: Hard constraint validation (see Task Specifications section)
   - ANIM-006: Root motion controls (see Task Specifications section)
   - MIGRATE-004: Legacy CHARACTER mapping (see Task Specifications section)
   ```

6. **Final review**: Run full test suite, verify all ROADMAP tasks marked complete.

---

## Appendix: File Index

Files likely to be created or modified:

```
# Editor
editor/src/components/Editor.ts                    # NEW
editor/src/lib/starlark-language.ts                # NEW
editor/src/lib/starlark-tokens.ts                  # NEW
editor/src/components/MeshPreview.ts               # MODIFY
crates/speccade-editor/src/preview/lod.rs          # NEW
crates/speccade-editor/src/preview/mesh.rs         # MODIFY

# Mesh
crates/speccade-spec/src/recipe/mesh/shrinkwrap.rs    # NEW
crates/speccade-spec/src/recipe/mesh/boolean_kit.rs   # NEW
crates/speccade-spec/src/recipe/animation/helpers.rs  # NEW
blender/operators/shrinkwrap.py                        # NEW
blender/operators/boolean_kit.py                       # NEW
blender/operators/animation_helpers.py                 # NEW

# Animation
crates/speccade-spec/src/recipe/animation/rig_setup.rs    # MODIFY
crates/speccade-spec/src/recipe/animation/constraints.rs  # MODIFY
crates/speccade-spec/src/recipe/animation/root_motion.rs  # NEW
blender/operators/rigging.py                               # NEW
blender/operators/constraints.py                           # NEW
blender/operators/root_motion.py                           # NEW

# Tooling
docs/architecture/plugin-system.md                     # NEW
docs/spec-reference/extensions/io-contract.md          # NEW
crates/speccade-spec/src/extension/mod.rs              # NEW
crates/speccade-cli/src/backends/subprocess.rs         # NEW
examples/extensions/simple-subprocess/                  # NEW (directory)

# Migration
crates/speccade-cli/src/commands/migrate/conversion/animation.rs   # NEW
crates/speccade-cli/src/commands/migrate/conversion/character.rs   # NEW
tests/migrate/fixtures/animation/                                   # NEW (directory)
tests/migrate/fixtures/character/                                   # NEW (directory)
docs/legacy/PARITY_MATRIX_LEGACY_SPEC_PY.md                         # MODIFY
```
