# Theme E: Complete Partial/Planned Work

**Date:** 2026-01-27
**Status:** Ready for implementation

This plan covers three items that are partially implemented or well-specified:

1. **E1: MESH-013** - Wire animation helpers to dispatch
2. **E2: ANIM-006** - Root motion controls
3. **E3: sprite.sheet_v1 / sprite.animation_v1** - Native sprite generation

---

## E1: Wire Animation Helpers to Dispatch (MESH-013)

### Current State

**EXISTS:**
- Spec types: `crates/speccade-spec/src/recipe/animation/helpers.rs`
  - `AnimationHelperPreset` enum (WalkCycle, RunCycle, IdleSway)
  - `SkeletonType` enum (Humanoid, Quadruped)
  - Full params struct with settings
- Python operator: `blender/operators/animation_helpers.py`
  - Foot roll bone creation
  - IK target animation
  - Preset configurations
- Golden specs: `golden/speccade/specs/animation/helpers/*.json` (6 specs)

**MISSING:**
- Backend dispatch in `crates/speccade-backend-blender/src/lib.rs`
- Handler in `blender/entrypoint.py`
- CLI dispatch in `crates/speccade-cli/src/dispatch/blender.rs`

### Implementation Steps

#### Step 1: Add Blender entrypoint handler

**File:** `blender/entrypoint.py`

```python
# Add to imports
from operators.animation_helpers import generate_animation_from_helpers

# Add handler function
def handle_animation_helpers(spec: Dict, out_root: Path, report_path: Path) -> None:
    """Handle animation.helpers_v1 generation."""
    params = spec["recipe"]["params"]

    # Get output path
    primary_output = next(o for o in spec["outputs"] if o["kind"] == "primary")
    out_path = out_root / primary_output["path"]
    out_path.parent.mkdir(parents=True, exist_ok=True)

    # Generate animation
    result = generate_animation_from_helpers(
        skeleton_type=params.get("skeleton", "humanoid"),
        preset=params["preset"],
        settings=params.get("settings", {}),
        clip_name=params.get("clip_name", "animation"),
        fps=params.get("fps", 30),
    )

    # Export to GLB
    export_gltf(str(out_path))

    # Compute metrics
    metrics = compute_animation_metrics(result["armature"], result["action"])

    # Write report
    write_report(report_path, ok=True, metrics=metrics)

# Add to choices in argparse
parser.add_argument("--mode", required=True,
    choices=[..., "animation_helpers"],  # Add this
    help="Generation mode")

# Add to handlers dict
handlers = {
    ...,
    "animation_helpers": handle_animation_helpers,
}
```

#### Step 2: Add Rust backend dispatch

**File:** `crates/speccade-backend-blender/src/lib.rs`

Add to the `generate()` match:
```rust
"skeletal_animation.helpers_v1" => {
    // Parse params
    let params: AnimationHelpersParams = serde_json::from_value(recipe.params.clone())?;

    // Run Blender with animation_helpers mode
    let result = orchestrator::run_blender(
        "animation_helpers",
        spec,
        out_root,
    )?;

    Ok(result)
}
```

**File:** `crates/speccade-backend-blender/src/animation_helpers.rs` (new)

```rust
//! Animation helpers backend module.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHelpersParams {
    pub skeleton: Option<String>,
    pub preset: String,
    pub settings: Option<AnimationHelpersSettings>,
    pub clip_name: Option<String>,
    pub fps: Option<u32>,
    pub save_blend: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationHelpersSettings {
    pub stride_length: Option<f64>,
    pub cycle_frames: Option<u32>,
    pub foot_roll: Option<bool>,
    pub arm_swing: Option<f64>,
    pub hip_sway: Option<f64>,
    pub spine_twist: Option<f64>,
    pub foot_lift: Option<f64>,
}
```

#### Step 3: Add CLI dispatch

**File:** `crates/speccade-cli/src/dispatch/blender.rs`

Add function:
```rust
pub(super) fn generate_blender_animation_helpers(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    // Similar pattern to other animation dispatch functions
}
```

**File:** `crates/speccade-cli/src/dispatch/mod.rs`

Add to recipe kind match.

### Verification

```bash
cargo test -p speccade-backend-blender
speccade generate --spec golden/speccade/specs/animation/helpers/walk_cycle_basic.json --out-root ./test-out
```

---

## E2: Root Motion Controls (ANIM-006)

### Current State

**EXISTS:**
- Validation constraint: `MaxRootMotionDelta` in `validation/constraints/mod.rs`
- Metric in reports: `root_motion_delta: Option<[f32; 3]>` in `OutputMetrics`

**MISSING:**
- Spec params for root motion extraction/lock/bake mode
- Blender implementation to extract/bake root motion
- CLI flags or spec fields to control behavior

### Implementation Steps

#### Step 1: Add spec types

**File:** `crates/speccade-spec/src/recipe/animation/root_motion.rs` (new)

```rust
//! Root motion control settings for skeletal animations.

use serde::{Deserialize, Serialize};

/// Root motion handling mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RootMotionMode {
    /// Keep root motion in the animation (default).
    #[default]
    Keep,
    /// Extract root motion to a separate curve/channel.
    Extract,
    /// Bake root motion into hip bone (remove from root).
    BakeToHip,
    /// Lock root position (zero out all root motion).
    Lock,
}

/// Root motion settings for animation export.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RootMotionSettings {
    /// How to handle root motion.
    #[serde(default)]
    pub mode: RootMotionMode,

    /// Which axes to apply root motion handling to.
    /// Default: all axes [true, true, true] for [X, Y, Z].
    #[serde(default = "default_axes")]
    pub axes: [bool; 3],

    /// Reference height for ground plane (used with BakeToHip).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ground_height: Option<f64>,
}

fn default_axes() -> [bool; 3] {
    [true, true, true]
}
```

#### Step 2: Add to animation params

**File:** `crates/speccade-spec/src/recipe/animation/clip.rs`

Add field:
```rust
/// Root motion handling settings.
#[serde(skip_serializing_if = "Option::is_none")]
pub root_motion: Option<RootMotionSettings>,
```

#### Step 3: Implement in Blender

**File:** `blender/entrypoint.py`

Add root motion processing:
```python
def apply_root_motion_settings(armature, action, settings: Dict) -> Dict:
    """Apply root motion settings to an animation."""
    mode = settings.get("mode", "keep")
    axes = settings.get("axes", [True, True, True])

    if mode == "keep":
        return {"extracted": None}

    root_bone = get_root_bone(armature)
    if not root_bone:
        return {"extracted": None}

    if mode == "extract":
        # Extract root motion to separate curves
        extracted = extract_root_motion_curves(action, root_bone, axes)
        return {"extracted": extracted}

    elif mode == "bake_to_hip":
        # Move root motion to hip bone
        hip_bone = find_hip_bone(armature)
        bake_root_to_hip(action, root_bone, hip_bone, axes)
        return {"extracted": None}

    elif mode == "lock":
        # Zero out root motion
        lock_root_motion(action, root_bone, axes)
        return {"extracted": None}

    return {"extracted": None}
```

#### Step 4: Add to metrics

**File:** `crates/speccade-spec/src/report/output.rs`

Already has `root_motion_delta`. Add:
```rust
/// Root motion mode that was applied.
#[serde(skip_serializing_if = "Option::is_none")]
pub root_motion_mode: Option<String>,

/// Extracted root motion curve data (if mode was "extract").
#[serde(skip_serializing_if = "Option::is_none")]
pub root_motion_extracted: Option<RootMotionCurves>,
```

### Verification

```bash
cargo test -p speccade-spec
# Create test spec with root_motion settings
speccade generate --spec test_root_motion.json --out-root ./test-out
# Verify report contains root_motion_mode and delta
```

---

## E3: Sprite Assets (sprite.sheet_v1 / sprite.animation_v1)

### Current State

**EXISTS:**
- RFC-0012: Full design specification (ACCEPTED)
- Shelf packing algorithm: Exists in `texture.trimsheet_v1`

**MISSING:**
- Spec types for both recipes
- Backend implementation
- Validation
- Tests and golden specs

### Implementation Steps

#### Step 1: Add spec types

**File:** `crates/speccade-spec/src/recipe/sprite/mod.rs` (new)

```rust
//! Sprite asset recipes for sheets and 2D animations.

mod sheet;
mod animation;

pub use sheet::*;
pub use animation::*;
```

**File:** `crates/speccade-spec/src/recipe/sprite/sheet.rs` (new)

```rust
use serde::{Deserialize, Serialize};

/// Parameters for sprite.sheet_v1 recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteSheetParams {
    /// Atlas resolution [width, height] in pixels.
    pub resolution: [u32; 2],

    /// Padding between frames (mip-safe gutter).
    #[serde(default = "default_padding")]
    pub padding: u32,

    /// Frame definitions.
    pub frames: Vec<SpriteFrame>,
}

fn default_padding() -> u32 { 2 }

/// A single frame in the spritesheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteFrame {
    /// Unique frame identifier.
    pub id: String,

    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,

    /// Pivot point [0-1, 0-1]. Default: [0.5, 0.5] (center).
    #[serde(default = "default_pivot")]
    pub pivot: [f64; 2],

    /// RGBA fill color.
    pub color: [f64; 4],
}

fn default_pivot() -> [f64; 2] { [0.5, 0.5] }

/// Metadata output for spritesheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteSheetMetadata {
    pub atlas_width: u32,
    pub atlas_height: u32,
    pub padding: u32,
    pub frames: Vec<SpriteFrameMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteFrameMetadata {
    pub id: String,
    pub u_min: f64,
    pub v_min: f64,
    pub u_max: f64,
    pub v_max: f64,
    pub width: u32,
    pub height: u32,
    pub pivot: [f64; 2],
}
```

**File:** `crates/speccade-spec/src/recipe/sprite/animation.rs` (new)

```rust
use serde::{Deserialize, Serialize};

/// Parameters for sprite.animation_v1 recipe.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteAnimationParams {
    /// Animation clip name.
    pub name: String,

    /// Default frames per second.
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Loop mode.
    #[serde(default)]
    pub loop_mode: LoopMode,

    /// Animation frames.
    pub frames: Vec<AnimFrame>,
}

fn default_fps() -> u32 { 12 }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LoopMode {
    #[default]
    Loop,
    Once,
    PingPong,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimFrame {
    /// Reference to frame ID in spritesheet.
    pub frame_id: String,

    /// Frame duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}

/// Metadata output for sprite animation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpriteAnimationMetadata {
    pub name: String,
    pub fps: u32,
    pub loop_mode: LoopMode,
    pub total_duration_ms: u32,
    pub frames: Vec<AnimFrameMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimFrameMetadata {
    pub frame_id: String,
    pub duration_ms: u32,
}
```

#### Step 2: Add to texture backend

**File:** `crates/speccade-backend-texture/src/sprite/mod.rs` (new)

```rust
//! Sprite generation using shelf packing from trimsheet.

mod sheet;
mod animation;

pub use sheet::generate_sprite_sheet;
pub use animation::generate_sprite_animation;
```

**File:** `crates/speccade-backend-texture/src/sprite/sheet.rs` (new)

Reuse shelf packing from trimsheet:
```rust
use crate::packing::shelf_pack;  // Existing algorithm

pub fn generate_sprite_sheet(
    params: &SpriteSheetParams,
    seed: u32,
) -> Result<(Vec<u8>, SpriteSheetMetadata), Error> {
    // 1. Sort frames by height desc, width desc, id asc
    // 2. Shelf pack into atlas
    // 3. Render solid color rectangles
    // 4. Apply mip-safe gutter
    // 5. Return PNG bytes + metadata
}
```

#### Step 3: Add validation

**File:** `crates/speccade-spec/src/validation/recipe_sprite.rs` (new)

```rust
pub fn validate_sprite_sheet(params: &SpriteSheetParams) -> Vec<ValidationError> {
    let mut errors = vec![];

    // Check resolution is power of 2 (optional warning)
    // Check frames fit in atlas
    // Check unique frame IDs
    // Check valid colors [0-1]
    // Check valid pivots [0-1]

    errors
}

pub fn validate_sprite_animation(params: &SpriteAnimationParams) -> Vec<ValidationError> {
    let mut errors = vec![];

    // Check frames not empty
    // Check valid loop_mode
    // Check fps > 0

    errors
}
```

#### Step 4: Add CLI dispatch

**File:** `crates/speccade-cli/src/dispatch/texture.rs`

Add sprite sheet generation to texture dispatch.

#### Step 5: Add golden tests

**Files:**
- `golden/starlark/sprite_sheet_basic.star`
- `golden/starlark/sprite_animation_basic.star`
- `golden/speccade/specs/sprite/sheet_basic.json`
- `golden/speccade/specs/sprite/animation_basic.json`

### Verification

```bash
cargo test -p speccade-spec
cargo test -p speccade-backend-texture
speccade generate --spec golden/starlark/sprite_sheet_basic.star --out-root ./test-out
# Verify PNG atlas and JSON metadata are created
```

---

## Task Summary

| Task | Scope | Estimated Files | Dependencies |
|------|-------|-----------------|--------------|
| E1: Animation Helpers | Wire existing code | 4 files modified | None |
| E2: Root Motion | New feature | 5 files new/modified | None |
| E3: Sprite Sheet | New recipe | 8+ files new | Trimsheet packing |
| E3: Sprite Animation | New recipe | 4+ files new | Sprite Sheet types |

## Recommended Order

1. **E1 first** - Smallest scope, just wiring existing code
2. **E3 sprite.sheet_v1** - Independent, reuses existing packing
3. **E3 sprite.animation_v1** - Depends on sheet types
4. **E2 last** - Touches animation pipeline, more testing needed

## Verification Commands

```bash
# After all implementation
cargo build --release
cargo test --release

# E1 verification
speccade generate --spec golden/speccade/specs/animation/helpers/walk_cycle_basic.json --out-root ./test-out

# E3 verification
speccade generate --spec golden/starlark/sprite_sheet_basic.star --out-root ./test-out
```
