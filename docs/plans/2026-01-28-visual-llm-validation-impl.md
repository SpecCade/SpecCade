# Visual LLM Validation Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add a `preview-grid` CLI command that generates 6-view PNG grids for 3D assets (static meshes, skeletal meshes, animations), enabling Claude's vision to validate correctness against embedded spec comments.

**Architecture:** Extends the existing Blender backend with a new "validation_grid" mode. The CLI parses specs, invokes Blender to render 6 fixed camera angles, composites them into a labeled grid, and saves a single PNG. Validation comments are extracted from `# [VALIDATION]` blocks in `.star` source files.

**Tech Stack:** Rust CLI (clap, image crate for compositing), Python/Blender (bpy for rendering), existing `speccade-backend-blender` orchestrator pattern.

**Reference Design:** `docs/plans/2026-01-28-visual-llm-validation-design.md`

---

### Task 1: Add `preview-grid` command stub to CLI

**Files:**
- Modify: `crates/speccade-cli/src/main.rs` (add PreviewGrid variant to Commands enum)
- Create: `crates/speccade-cli/src/commands/preview_grid.rs`
- Modify: `crates/speccade-cli/src/commands/mod.rs` (add module declaration)

**Step 1: Write a failing test for CLI parsing**

Add to the `#[cfg(test)] mod tests` block at the bottom of `main.rs`:

```rust
#[test]
fn test_cli_parses_preview_grid() {
    let cli = Cli::try_parse_from([
        "speccade",
        "preview-grid",
        "--spec",
        "mesh.star",
    ])
    .unwrap();
    match cli.command {
        Commands::PreviewGrid { spec, out, panel_size } => {
            assert_eq!(spec, "mesh.star");
            assert!(out.is_none());
            assert_eq!(panel_size, 256);
        }
        _ => panic!("expected preview-grid command"),
    }
}

#[test]
fn test_cli_parses_preview_grid_with_options() {
    let cli = Cli::try_parse_from([
        "speccade",
        "preview-grid",
        "--spec",
        "mesh.star",
        "--out",
        "grid.png",
        "--panel-size",
        "128",
    ])
    .unwrap();
    match cli.command {
        Commands::PreviewGrid { spec, out, panel_size } => {
            assert_eq!(spec, "mesh.star");
            assert_eq!(out.as_deref(), Some("grid.png"));
            assert_eq!(panel_size, 128);
        }
        _ => panic!("expected preview-grid command"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_preview_grid -- --nocapture`
Expected: FAIL - PreviewGrid variant not defined

**Step 3: Add the PreviewGrid variant to Commands enum**

In `main.rs`, find the `Commands` enum (around line 100-300) and add after the `Preview` variant:

```rust
    /// Generate a multi-angle validation grid PNG for 3D assets
    PreviewGrid {
        /// Path to the spec file (.star or .json)
        #[arg(short, long)]
        spec: String,

        /// Output PNG path (default: <spec_stem>.grid.png next to spec)
        #[arg(short, long)]
        out: Option<String>,

        /// Panel size in pixels (default: 256)
        #[arg(long, default_value = "256")]
        panel_size: u32,
    },
```

**Step 4: Add module declaration in mod.rs**

In `crates/speccade-cli/src/commands/mod.rs`, add after `pub mod preview;`:

```rust
pub mod preview_grid;
```

Also add to the test block:

```rust
let _ = preview_grid::run;
```

**Step 5: Create preview_grid.rs with stub**

Create `crates/speccade-cli/src/commands/preview_grid.rs`:

```rust
//! Preview grid command implementation
//!
//! Generates a multi-angle validation grid PNG for 3D assets.

use anyhow::Result;
use std::process::ExitCode;

/// Run the preview-grid command
///
/// # Arguments
/// * `spec_path` - Path to the spec file
/// * `out` - Output PNG path (optional)
/// * `panel_size` - Size of each panel in pixels
pub fn run(spec_path: &str, out: Option<&str>, panel_size: u32) -> Result<ExitCode> {
    println!(
        "preview-grid: spec={}, out={:?}, panel_size={}",
        spec_path, out, panel_size
    );
    println!("Not yet implemented");
    Ok(ExitCode::SUCCESS)
}
```

**Step 6: Add dispatch in main.rs**

Find the match block for `Commands` (around line 500+) and add after `Commands::Preview { ... }`:

```rust
        Commands::PreviewGrid { spec, out, panel_size } => {
            commands::preview_grid::run(&spec, out.as_deref(), panel_size)
        }
```

**Step 7: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_preview_grid -- --nocapture`
Expected: PASS

**Step 8: Verify CLI help shows the new command**

Run: `cargo run -p speccade-cli -- preview-grid --help`
Expected: Shows help text for preview-grid

**Step 9: Commit**

```bash
git add crates/speccade-cli/src/main.rs crates/speccade-cli/src/commands/preview_grid.rs crates/speccade-cli/src/commands/mod.rs
git commit -m "feat(cli): add preview-grid command stub"
```

---

### Task 2: Add validation_grid mode to Blender orchestrator

**Files:**
- Modify: `crates/speccade-backend-blender/src/orchestrator.rs`

**Step 1: Add ValidationGrid to GenerationMode enum**

In `orchestrator.rs`, find the `GenerationMode` enum and add:

```rust
    /// Validation grid rendering (6-view PNG for LLM verification).
    ValidationGrid,
```

**Step 2: Add as_str match arm**

In the `impl GenerationMode` block, add to `as_str`:

```rust
            GenerationMode::ValidationGrid => "validation_grid",
```

**Step 3: Add mode_from_recipe_kind mapping**

This mode is not recipe-driven, so skip this step. The CLI will invoke it directly with a synthesized mode.

**Step 4: Update tests**

Add to the test block in `orchestrator.rs`:

```rust
#[test]
fn test_validation_grid_mode() {
    assert_eq!(GenerationMode::ValidationGrid.as_str(), "validation_grid");
}
```

**Step 5: Run tests**

Run: `cargo test -p speccade-backend-blender test_validation_grid -- --nocapture`
Expected: PASS

**Step 6: Commit**

```bash
git add crates/speccade-backend-blender/src/orchestrator.rs
git commit -m "feat(blender): add ValidationGrid generation mode"
```

---

### Task 3: Add validation_grid handler to Blender entrypoint

**Files:**
- Modify: `blender/entrypoint.py`

**Step 1: Add handle_validation_grid function**

Add before the `if __name__ == "__main__":` block:

```python
# =============================================================================
# Validation Grid Handler
# =============================================================================

VALIDATION_GRID_VIEWS = [
    # (label, azimuth_deg, elevation_deg)
    ("FRONT", 0.0, 30.0),
    ("BACK", 180.0, 30.0),
    ("TOP", 0.0, 90.0),
    ("LEFT", 90.0, 30.0),
    ("RIGHT", 270.0, 30.0),
    ("ISO", 45.0, 35.264),  # Isometric angle
]


def handle_validation_grid(spec: Dict, out_root: Path, report_path: Path) -> None:
    """
    Generate a 6-view validation grid PNG for LLM-based visual verification.

    Grid layout (2 rows x 3 columns):
        FRONT | BACK  | TOP
        LEFT  | RIGHT | ISO
    """
    start_time = time.time()

    try:
        recipe = spec.get("recipe", {})
        params = recipe.get("params", {})

        # Get panel size from spec (default 256)
        panel_size = params.get("panel_size", 256)
        grid_padding = 4

        # Create the mesh based on recipe kind
        recipe_kind = recipe.get("kind", "")

        if recipe_kind.startswith("static_mesh."):
            obj = create_mesh_from_recipe(recipe_kind, params)
        elif recipe_kind.startswith("skeletal_mesh."):
            obj = create_skeletal_mesh_from_recipe(recipe_kind, params)
        else:
            # Try to extract mesh params directly
            mesh_params = params.get("mesh", params)
            primitive = mesh_params.get("base_primitive", "cube")
            dimensions = mesh_params.get("dimensions", [1, 1, 1])
            obj = create_primitive(primitive, dimensions)

            modifiers = mesh_params.get("modifiers", [])
            for mod_spec in modifiers:
                apply_modifier(obj, mod_spec)

        # Compute mesh bounds for camera placement
        mesh_bounds = get_mesh_bounds(obj)
        mesh_center = [
            (mesh_bounds[0][i] + mesh_bounds[1][i]) / 2
            for i in range(3)
        ]
        mesh_size = max(
            mesh_bounds[1][i] - mesh_bounds[0][i]
            for i in range(3)
        )

        # Camera distance scaled to mesh size
        cam_dist = mesh_size * 2.5

        # Set up orthographic camera
        camera_data = bpy.data.cameras.new(name="ValidationCamera")
        camera_data.type = 'ORTHO'
        camera_data.ortho_scale = mesh_size * 1.5
        camera = bpy.data.objects.new("ValidationCamera", camera_data)
        bpy.context.collection.objects.link(camera)
        bpy.context.scene.camera = camera

        # Set up three-point lighting
        setup_lighting("three_point", mesh_center, mesh_size)

        # Configure render settings
        bpy.context.scene.render.resolution_x = panel_size
        bpy.context.scene.render.resolution_y = panel_size
        bpy.context.scene.render.film_transparent = True

        # Create temp directory for individual frames
        temp_frames_dir = Path(tempfile.mkdtemp(prefix="speccade_validation_grid_"))
        frame_paths = []

        # Render each view
        for i, (label, azimuth, elevation) in enumerate(VALIDATION_GRID_VIEWS):
            azimuth_rad = math.radians(azimuth)
            elev_rad = math.radians(elevation)

            # Position camera
            cam_x = mesh_center[0] + cam_dist * math.sin(azimuth_rad) * math.cos(elev_rad)
            cam_y = mesh_center[1] - cam_dist * math.cos(azimuth_rad) * math.cos(elev_rad)
            cam_z = mesh_center[2] + cam_dist * math.sin(elev_rad)

            camera.location = Vector((cam_x, cam_y, cam_z))

            # Point camera at mesh center
            direction = Vector(mesh_center) - camera.location
            rot_quat = direction.to_track_quat('-Z', 'Y')
            camera.rotation_euler = rot_quat.to_euler()

            # Render frame
            frame_path = temp_frames_dir / f"view_{i}_{label}.png"
            bpy.context.scene.render.filepath = str(frame_path)
            bpy.ops.render.render(write_still=True)
            frame_paths.append((frame_path, label))

        # Composite into grid (3 cols x 2 rows)
        grid_width = panel_size * 3 + grid_padding * 4
        grid_height = panel_size * 2 + grid_padding * 3

        # Use PIL if available, otherwise use Blender's compositor
        try:
            from PIL import Image, ImageDraw, ImageFont

            grid_img = Image.new('RGBA', (grid_width, grid_height), (0, 0, 0, 0))

            for i, (frame_path, label) in enumerate(frame_paths):
                col = i % 3
                row = i // 3
                x = grid_padding + col * (panel_size + grid_padding)
                y = grid_padding + row * (panel_size + grid_padding)

                frame_img = Image.open(frame_path)
                grid_img.paste(frame_img, (x, y))

                # Draw label
                draw = ImageDraw.Draw(grid_img)
                try:
                    font = ImageFont.truetype("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 14)
                except:
                    font = ImageFont.load_default()

                # Label background
                text_bbox = draw.textbbox((x + 4, y + 4), label, font=font)
                draw.rectangle([text_bbox[0] - 2, text_bbox[1] - 2, text_bbox[2] + 2, text_bbox[3] + 2], fill=(0, 0, 0, 180))
                draw.text((x + 4, y + 4), label, fill=(255, 255, 255, 255), font=font)

            # Get output path
            outputs = spec.get("outputs", [])
            primary_output = next((o for o in outputs if o.get("kind") == "primary"), None)

            if primary_output:
                output_rel_path = primary_output.get("path", "validation_grid.png")
            else:
                output_rel_path = "validation_grid.png"

            output_path = out_root / output_rel_path
            output_path.parent.mkdir(parents=True, exist_ok=True)

            grid_img.save(str(output_path))

        except ImportError:
            # Fallback: save individual frames, let Rust composite
            output_path = out_root / "validation_grid_frames"
            output_path.mkdir(parents=True, exist_ok=True)
            for frame_path, label in frame_paths:
                import shutil
                shutil.copy(frame_path, output_path / f"{label}.png")

        # Clean up temp files
        import shutil
        shutil.rmtree(temp_frames_dir)

        duration_ms = int((time.time() - start_time) * 1000)

        write_report(report_path, ok=True,
                     output_path=str(output_path),
                     duration_ms=duration_ms)

    except Exception as e:
        write_report(report_path, ok=False, error=str(e))
        raise
```

**Step 2: Add mode dispatch in main block**

Find the mode dispatch section (around the end of the file) and add:

```python
    elif args.mode == "validation_grid":
        handle_validation_grid(spec, out_root, report_path)
```

**Step 3: Test manually with a mesh spec**

Run: `blender --background --factory-startup --python blender/entrypoint.py -- --mode validation_grid --spec golden/starlark/mesh_cube.star --out-root ./tmp --report ./tmp/report.json`

Expected: Creates a 6-view grid PNG

**Step 4: Commit**

```bash
git add blender/entrypoint.py
git commit -m "feat(blender): add validation_grid handler for 6-view rendering"
```

---

### Task 4: Wire preview-grid command to Blender backend

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview_grid.rs`

**Step 1: Add imports and implement run function**

Replace the stub with:

```rust
//! Preview grid command implementation
//!
//! Generates a multi-angle validation grid PNG for 3D assets.

use anyhow::{Context, Result};
use colored::Colorize;
use speccade_backend_blender::{GenerationMode, Orchestrator, OrchestratorConfig};
use speccade_spec::Spec;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

/// Run the preview-grid command
///
/// # Arguments
/// * `spec_path` - Path to the spec file
/// * `out` - Output PNG path (optional)
/// * `panel_size` - Size of each panel in pixels
pub fn run(spec_path: &str, out: Option<&str>, panel_size: u32) -> Result<ExitCode> {
    println!("{} {}", "Preview Grid:".cyan().bold(), spec_path);

    // Read and parse spec
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    let mut spec = Spec::from_json(&spec_content)
        .or_else(|_| {
            // Try evaluating as Starlark
            crate::compiler::eval_starlark_to_spec(spec_path)
        })
        .with_context(|| format!("Failed to parse spec: {}", spec_path))?;

    // Verify it's a 3D asset type
    match spec.asset_type.as_str() {
        "static_mesh" | "skeletal_mesh" | "skeletal_animation" | "sprite" => {}
        other => {
            anyhow::bail!(
                "preview-grid only supports 3D assets (static_mesh, skeletal_mesh, skeletal_animation, sprite), got: {}",
                other
            );
        }
    }

    // Set up output path
    let spec_path_pb = PathBuf::from(spec_path);
    let spec_dir = spec_path_pb.parent().unwrap_or_else(|| std::path::Path::new("."));

    let out_path = if let Some(out) = out {
        PathBuf::from(out)
    } else {
        let stem = spec_path_pb.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("preview");
        spec_dir.join(format!("{}.grid.png", stem))
    };

    // Inject panel_size into recipe params for Blender
    if let Some(ref mut recipe) = spec.recipe {
        if let Some(params) = recipe.params.as_object_mut() {
            params.insert("panel_size".to_string(), serde_json::json!(panel_size));
        }
    }

    // Update spec outputs to point to our grid path
    let out_rel = out_path.file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("validation_grid.png");

    spec.outputs = vec![speccade_spec::output::Output {
        kind: speccade_spec::output::OutputKind::Primary,
        format: speccade_spec::output::OutputFormat::Png,
        path: out_rel.to_string(),
    }];

    // Create output directory
    let out_root = out_path.parent().unwrap_or_else(|| std::path::Path::new("."));
    fs::create_dir_all(out_root)
        .with_context(|| format!("Failed to create output directory: {}", out_root.display()))?;

    // Write spec to temp file for Blender
    let temp_dir = tempfile::tempdir()?;
    let temp_spec_path = temp_dir.path().join("spec.json");
    let temp_report_path = temp_dir.path().join("report.json");

    let spec_json = serde_json::to_string_pretty(&spec)?;
    fs::write(&temp_spec_path, &spec_json)?;

    // Run Blender
    let orchestrator = Orchestrator::new();
    let report = orchestrator.run(
        GenerationMode::ValidationGrid,
        &temp_spec_path,
        out_root,
        &temp_report_path,
    )?;

    if !report.ok {
        let error = report.error.unwrap_or_else(|| "Unknown error".to_string());
        anyhow::bail!("Blender validation grid generation failed: {}", error);
    }

    println!(
        "{} Grid saved to: {}",
        "OK".green().bold(),
        out_path.display()
    );

    if let Some(duration_ms) = report.duration_ms {
        println!("  Rendered in {}ms", duration_ms);
    }

    Ok(ExitCode::SUCCESS)
}
```

**Step 2: Add tempfile to CLI dependencies if not present**

Check `crates/speccade-cli/Cargo.toml` and add if missing:

```toml
tempfile = "3"
```

**Step 3: Verify compilation**

Run: `cargo check -p speccade-cli`
Expected: Compiles

**Step 4: Test with mesh_cube.star**

Run: `cargo run -p speccade-cli -- preview-grid --spec golden/starlark/mesh_cube.star`
Expected: Creates `golden/starlark/mesh_cube.grid.png`

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview_grid.rs crates/speccade-cli/Cargo.toml
git commit -m "feat(preview-grid): wire up Blender validation grid rendering"
```

---

### Task 5: Implement validation comment extraction

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview_grid.rs`

**Step 1: Write failing test for comment extraction**

Add to the bottom of `preview_grid.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_validation_comments() {
        let source = r#"
# Simple cube mesh
#
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin
# FRONT VIEW: Square face visible
# NOTES: Bevel creates smooth edges

spec(
    asset_id = "mesh-cube-01",
    ...
)
"#;
        let comments = extract_validation_comments(source);
        assert!(comments.is_some());
        let text = comments.unwrap();
        assert!(text.contains("SHAPE:"));
        assert!(text.contains("beveled cube"));
        assert!(text.contains("FRONT VIEW:"));
    }

    #[test]
    fn test_extract_validation_comments_none() {
        let source = r#"
# Simple cube mesh - no validation block
spec(
    asset_id = "mesh-cube-01",
)
"#;
        let comments = extract_validation_comments(source);
        assert!(comments.is_none());
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib extract_validation -- --nocapture`
Expected: FAIL - function not defined

**Step 3: Implement extract_validation_comments**

Add above the tests module:

```rust
/// Extracts the [VALIDATION] comment block from a Starlark source file.
///
/// Looks for `# [VALIDATION]` and collects all subsequent `#` lines
/// until a blank line or non-comment line is encountered.
pub fn extract_validation_comments(source: &str) -> Option<String> {
    let mut in_validation_block = false;
    let mut lines = Vec::new();

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed == "# [VALIDATION]" {
            in_validation_block = true;
            continue;
        }

        if in_validation_block {
            if trimmed.starts_with('#') {
                // Strip the leading `# ` and collect
                let content = trimmed.strip_prefix("# ").unwrap_or(
                    trimmed.strip_prefix('#').unwrap_or(trimmed)
                );
                lines.push(content.to_string());
            } else if trimmed.is_empty() {
                // Empty line ends the block
                break;
            } else {
                // Non-comment line ends the block
                break;
            }
        }
    }

    if lines.is_empty() {
        None
    } else {
        Some(lines.join("\n"))
    }
}
```

**Step 4: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --lib extract_validation -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview_grid.rs
git commit -m "feat(preview-grid): add validation comment extraction from .star files"
```

---

### Task 6: Add validation comments to golden specs

**Files:**
- Modify: `golden/starlark/mesh_cube.star`
- Modify: `golden/starlark/mesh_organic_sculpt.star`
- Modify: `golden/starlark/character_humanoid.star`
- Modify: `golden/starlark/animation_walk_cycle.star`

**Step 1: Add validation block to mesh_cube.star**

At the top of the file, replace existing comments with:

```python
# Simple cube mesh - demonstrates mesh stdlib
#
# [VALIDATION]
# SHAPE: A beveled cube with smooth subdivision
# PROPORTIONS: Equal 1.0 unit dimensions on all axes
# ORIENTATION: Cube centered at origin, aligned to world axes (XYZ)
# FRONT VIEW: Square face with beveled edges visible, centered
# BACK VIEW: Identical to front (cube symmetry)
# LEFT VIEW: Square face with beveled edges, centered
# RIGHT VIEW: Identical to left (cube symmetry)
# TOP VIEW: Square face looking down, centered
# ISO VIEW: Three faces visible (front, top, right corner)
# NOTES: Bevel modifier (0.02, 2 segments) should create smooth edge transitions, subdivision (level 2) adds smoothness

spec(
    ...
```

**Step 2: Add validation block to mesh_organic_sculpt.star**

```python
# Organic sculpt mesh - demonstrates metaball-based organic mesh generation
#
# [VALIDATION]
# SHAPE: Blob-like organic form made from merged metaballs - main body sphere with 3 protrusions
# PROPORTIONS: Main body ~1.0 radius, extensions ~0.5-0.6 radius, small bump ~0.35 radius
# ORIENTATION: Main body centered at origin, extensions asymmetrically placed
# FRONT VIEW: Irregular blob silhouette, not perfectly round
# BACK VIEW: Different silhouette than front due to asymmetric extensions
# LEFT VIEW: Shows extension at position (-0.4, 0.3, 0.1)
# RIGHT VIEW: Shows extension at position (0.6, 0.0, 0.2)
# TOP VIEW: Shows small bump at (0, -0.4, 0.5) and overall irregular shape
# ISO VIEW: 3D organic form clearly visible, not a simple sphere
# NOTES: Displacement noise (0.05 strength) adds surface detail, should look organic not mechanical

spec(
    ...
```

**Step 3: Add validation block to character_humanoid.star**

```python
# Simple humanoid character - demonstrates character stdlib
#
# [VALIDATION]
# SHAPE: Humanoid figure with cylindrical torso, spherical head, cylindrical limbs
# PROPORTIONS: Head ~0.15 units, torso segments ~0.3-0.4 units, arms ~0.25 units, legs ~0.35 units
# ORIENTATION: Standing upright (+Z up), facing +Y forward, T-pose with arms extended sideways
# FRONT VIEW: Symmetric humanoid shape - head on top, arms extended left/right, legs below
# BACK VIEW: Mirror of front, spine area visible
# LEFT VIEW: Profile view - head, single arm cylinder, single leg cylinder
# RIGHT VIEW: Mirror of left view
# TOP VIEW: Looking down at head sphere, torso below, arms extending sideways
# ISO VIEW: Full 3D humanoid form clearly visible in T-pose
# NOTES: Body uses skin-tone material (0.8, 0.6, 0.5), head slightly lighter (0.9, 0.7, 0.6)

skeletal_mesh_spec(
    ...
```

**Step 4: Add validation block to animation_walk_cycle.star**

```python
# Simple walk cycle animation - demonstrates animation stdlib
#
# [VALIDATION]
# SHAPE: Humanoid skeleton performing walk cycle
# MOTION: Legs swing forward/back alternately, arms counter-swing, spine subtle tilt
# FRAME 0: Contact pose - left leg forward (25°), right leg back (-25°), left arm back, right arm forward
# FRAME 12 (0.5s): Passing pose - legs swapped, right leg forward, left leg back
# FRAME 24 (1.0s): Return to contact pose (loop point)
# ORIENTATION: Character facing +Y, walking in place (no root motion)
# FRONT VIEW (frame 0): Left leg in front, right arm in front
# FRONT VIEW (frame 12): Right leg in front, left arm in front
# ISO VIEW: Full body visible, leg swing angle clearly visible (~25 degrees)
# NOTES: Linear interpolation, smooth transitions, loop=true so frame 24 matches frame 0

skeletal_animation_spec(
    ...
```

**Step 5: Verify specs still parse**

Run: `cargo run -p speccade-cli -- eval --spec golden/starlark/mesh_cube.star --pretty | head -20`
Expected: Valid JSON output

**Step 6: Commit**

```bash
git add golden/starlark/mesh_cube.star golden/starlark/mesh_organic_sculpt.star golden/starlark/character_humanoid.star golden/starlark/animation_walk_cycle.star
git commit -m "docs(golden): add validation comment blocks to key specs"
```

---

### Task 7: Create validation orchestration guide

**Files:**
- Create: `docs/plans/validation-orchestration.md`

**Step 1: Create the orchestration document**

```markdown
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
```

**Step 2: Commit**

```bash
git add docs/plans/validation-orchestration.md
git commit -m "docs: add visual validation orchestration guide for Claude"
```

---

### Task 8: End-to-end integration test

**Files:**
- Add test to: `crates/speccade-cli/src/commands/preview_grid.rs`

**Step 1: Add integration test**

Add to the tests module:

```rust
#[test]
#[ignore = "requires Blender"]
fn test_preview_grid_end_to_end() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("cube.grid.png");

    // This test requires Blender to be installed
    let result = run(
        "golden/starlark/mesh_cube.star",
        Some(out_path.to_str().unwrap()),
        128,
    );

    // If Blender is not available, this will fail with a specific error
    if let Err(ref e) = result {
        if e.to_string().contains("Blender not found") {
            eprintln!("Skipping test: Blender not available");
            return;
        }
    }

    assert!(result.is_ok());
    assert!(out_path.exists());

    // Verify it's a valid PNG
    let data = std::fs::read(&out_path).unwrap();
    assert!(data.starts_with(&[0x89, 0x50, 0x4E, 0x47])); // PNG magic
}
```

**Step 2: Run the test (if Blender available)**

Run: `cargo test -p speccade-cli --lib test_preview_grid_end_to_end -- --ignored --nocapture`
Expected: PASS (if Blender installed) or skip message

**Step 3: Commit**

```bash
git add crates/speccade-cli/src/commands/preview_grid.rs
git commit -m "test(preview-grid): add end-to-end integration test"
```

---

## Success Criteria

After completing all tasks:

- [ ] `cargo run -p speccade-cli -- preview-grid --help` shows command help
- [ ] `cargo run -p speccade-cli -- preview-grid --spec golden/starlark/mesh_cube.star` generates `mesh_cube.grid.png`
- [ ] Grid PNG contains 6 labeled views (FRONT, BACK, TOP, LEFT, RIGHT, ISO)
- [ ] `extract_validation_comments()` correctly parses `[VALIDATION]` blocks
- [ ] All 4 target golden specs have validation comments
- [ ] `docs/plans/validation-orchestration.md` documents the Claude workflow
- [ ] All unit tests pass: `cargo test -p speccade-cli`
