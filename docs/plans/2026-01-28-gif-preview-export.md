# GIF Preview Export Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Add `--gif` flag to `speccade preview` that generates animated GIFs from sprite sheets, VFX flipbooks, and mesh-to-sprite renders, enabling LLM-based animation validation without a game engine.

**Architecture:** The preview command reads a spec + its generated artifacts (PNG atlas + JSON metadata), extracts individual frames by cropping the atlas using UV coordinates from metadata, then encodes them as an animated GIF using the `gif` crate. Three frame-extraction paths (sprite sheet+animation, VFX flipbook, mesh-to-sprite) converge on a shared GIF assembly function.

**Tech Stack:** `gif` crate (GIF encoding), `image` crate (PNG decoding + frame cropping + color quantization to 256 colors)

---

### Task 1: Add dependencies

**Files:**
- Modify: `Cargo.toml` (workspace root, add workspace deps)
- Modify: `crates/speccade-cli/Cargo.toml` (add deps to CLI)

**Step 1: Add `image` and `gif` to workspace Cargo.toml**

Open `Cargo.toml` (workspace root) and add to `[workspace.dependencies]`:

```toml
image = { version = "0.25", default-features = false, features = ["png"] }
gif = "0.13"
```

**Step 2: Add deps to speccade-cli Cargo.toml**

In `crates/speccade-cli/Cargo.toml` at line 40 (after `png.workspace = true`), add:

```toml
image.workspace = true
gif = "0.13"
```

**Step 3: Verify it compiles**

Run: `cargo check -p speccade-cli`
Expected: compiles with no errors

**Step 4: Commit**

```bash
git add Cargo.toml crates/speccade-cli/Cargo.toml
git commit -m "deps: add image and gif crates for preview GIF export"
```

---

### Task 2: Add CLI args for --gif to preview command

**Files:**
- Modify: `crates/speccade-cli/src/main.rs:203-212` (Preview variant in Commands enum)
- Modify: `crates/speccade-cli/src/main.rs:514` (dispatch call)

**Step 1: Write a failing test for CLI parsing**

Add to the `#[cfg(test)] mod tests` block at the bottom of `main.rs`:

```rust
#[test]
fn test_cli_parses_preview_with_gif() {
    let cli = Cli::try_parse_from([
        "speccade",
        "preview",
        "--spec",
        "spec.json",
        "--gif",
    ])
    .unwrap();
    match cli.command {
        Commands::Preview { spec, out_root, gif, out, fps, scale } => {
            assert_eq!(spec, "spec.json");
            assert!(out_root.is_none());
            assert!(gif);
            assert!(out.is_none());
            assert!(fps.is_none());
            assert!(scale.is_none());
        }
        _ => panic!("expected preview command"),
    }
}

#[test]
fn test_cli_parses_preview_with_gif_options() {
    let cli = Cli::try_parse_from([
        "speccade",
        "preview",
        "--spec",
        "spec.json",
        "--gif",
        "--out",
        "preview.gif",
        "--fps",
        "24",
        "--scale",
        "2",
    ])
    .unwrap();
    match cli.command {
        Commands::Preview { spec, out_root, gif, out, fps, scale } => {
            assert_eq!(spec, "spec.json");
            assert!(gif);
            assert_eq!(out.as_deref(), Some("preview.gif"));
            assert_eq!(fps, Some(24));
            assert_eq!(scale, Some(2));
        }
        _ => panic!("expected preview command"),
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_preview_with_gif -- --nocapture`
Expected: FAIL - fields don't exist on Preview variant yet

**Step 3: Add the new fields to the Preview variant**

In `main.rs`, replace the `Preview` variant (lines 203-212) with:

```rust
    /// Preview an asset (opens in viewer/editor, or exports GIF with --gif)
    Preview {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,

        /// Export animated GIF instead of opening viewer
        #[arg(long)]
        gif: bool,

        /// Output file path for GIF (default: <asset_id>.preview.gif next to spec)
        #[arg(long)]
        out: Option<String>,

        /// Override frames per second for GIF
        #[arg(long)]
        fps: Option<u32>,

        /// Scale factor for GIF frames (default: 1)
        #[arg(long)]
        scale: Option<u32>,
    },
```

**Step 4: Update the dispatch call**

Replace line 514:
```rust
Commands::Preview { spec, out_root } => commands::preview::run(&spec, out_root.as_deref()),
```

With:
```rust
Commands::Preview { spec, out_root, gif, out, fps, scale } => {
    commands::preview::run(&spec, out_root.as_deref(), gif, out.as_deref(), fps, scale)
}
```

**Step 5: Update preview::run signature temporarily**

In `crates/speccade-cli/src/commands/preview.rs`, change the function signature to accept the new args (keep existing body for now):

```rust
pub fn run(
    spec_path: &str,
    _out_root: Option<&str>,
    _gif: bool,
    _out: Option<&str>,
    _fps: Option<u32>,
    _scale: Option<u32>,
) -> Result<ExitCode> {
```

**Step 6: Run tests to verify they pass**

Run: `cargo test -p speccade-cli --bin speccade test_cli_parses_preview -- --nocapture`
Expected: PASS

**Step 7: Commit**

```bash
git add crates/speccade-cli/src/main.rs crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): add --gif, --out, --fps, --scale CLI args"
```

---

### Task 3: Implement GIF assembly core function

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview.rs`
- Test: inline `#[cfg(test)]` in same file

**Step 1: Write a failing test for GIF assembly**

Add to preview.rs tests module:

```rust
#[test]
fn test_assemble_gif_creates_valid_file() {
    let tmp = tempfile::tempdir().unwrap();
    let out_path = tmp.path().join("test.gif");

    // Create 3 simple 4x4 RGBA frames
    let frames: Vec<Vec<u8>> = (0..3)
        .map(|i| {
            let val = (i as u8) * 80;
            vec![val, val, val, 255].repeat(16) // 4x4 pixels
        })
        .collect();

    let result = assemble_gif(
        &frames,
        4,
        4,
        &[100, 100, 100], // 100ms per frame
        true,              // loop
        out_path.to_str().unwrap(),
    );
    assert!(result.is_ok());
    assert!(out_path.exists());

    // Verify it starts with GIF magic bytes
    let data = std::fs::read(&out_path).unwrap();
    assert_eq!(&data[0..3], b"GIF");
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib test_assemble_gif -- --nocapture`
Expected: FAIL - `assemble_gif` not defined

**Step 3: Implement assemble_gif**

Add to preview.rs (above the tests module):

```rust
use gif::{Encoder, Frame as GifFrame, Repeat};
use std::io::BufWriter;

/// Assembles RGBA frames into an animated GIF file.
///
/// # Arguments
/// * `frames` - RGBA pixel data for each frame (width * height * 4 bytes each)
/// * `width` - Frame width in pixels
/// * `height` - Frame height in pixels
/// * `delays_ms` - Per-frame delay in milliseconds
/// * `loop_anim` - Whether to loop the animation
/// * `out_path` - Output file path
fn assemble_gif(
    frames: &[Vec<u8>],
    width: u16,
    height: u16,
    delays_ms: &[u32],
    loop_anim: bool,
    out_path: &str,
) -> Result<()> {
    anyhow::ensure!(!frames.is_empty(), "No frames to encode");
    anyhow::ensure!(
        frames.len() == delays_ms.len(),
        "Frame count ({}) != delay count ({})",
        frames.len(),
        delays_ms.len()
    );

    let file = fs::File::create(out_path)
        .with_context(|| format!("Failed to create GIF: {}", out_path))?;
    let mut writer = BufWriter::new(file);
    let mut encoder = Encoder::new(&mut writer, width, height, &[])
        .with_context(|| "Failed to create GIF encoder")?;

    if loop_anim {
        encoder
            .set_repeat(Repeat::Infinite)
            .with_context(|| "Failed to set GIF repeat")?;
    }

    for (i, frame_rgba) in frames.iter().enumerate() {
        // GIF delay is in centiseconds (1/100th of a second)
        let delay_cs = (delays_ms[i] / 10).max(1) as u16;

        let mut gif_frame = GifFrame::from_rgba_speed(width, height, &mut frame_rgba.clone(), 10);
        gif_frame.delay = delay_cs;
        gif_frame.dispose = gif::DisposalMethod::Background;

        encoder
            .write_frame(&gif_frame)
            .with_context(|| format!("Failed to write GIF frame {}", i))?;
    }

    Ok(())
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p speccade-cli --lib test_assemble_gif -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): implement GIF assembly from RGBA frames"
```

---

### Task 4: Implement frame extraction for VFX flipbooks

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview.rs`

This is the simplest path since flipbooks have uniform frame sizes in a grid layout.

**Step 1: Write a failing test for flipbook frame extraction**

```rust
#[test]
fn test_extract_flipbook_frames() {
    // Create a 128x64 atlas with 2 frames of 64x64 (side by side, no padding)
    let mut atlas = image::RgbaImage::new(128, 64);
    // Frame 0: red
    for y in 0..64 {
        for x in 0..64 {
            atlas.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
        }
    }
    // Frame 1: blue
    for y in 0..64 {
        for x in 64..128 {
            atlas.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
        }
    }

    let metadata_json = serde_json::json!({
        "atlas_width": 128,
        "atlas_height": 64,
        "padding": 0,
        "effect": "explosion",
        "frame_count": 2,
        "frame_size": [64, 64],
        "fps": 12,
        "loop_mode": "once",
        "total_duration_ms": 166,
        "frames": [
            { "index": 0, "u_min": 0.0, "v_min": 0.0, "u_max": 0.5, "v_max": 1.0, "width": 64, "height": 64 },
            { "index": 1, "u_min": 0.5, "v_min": 0.0, "u_max": 1.0, "v_max": 1.0, "width": 64, "height": 64 }
        ]
    });

    let metadata: speccade_spec::recipe::vfx::flipbook::VfxFlipbookMetadata =
        serde_json::from_value(metadata_json).unwrap();

    let (frames, delays, do_loop) = extract_flipbook_frames(&atlas, &metadata, None);
    assert_eq!(frames.len(), 2);
    assert_eq!(delays.len(), 2);
    assert!(!do_loop); // Once mode
    // Each frame should be 64*64*4 bytes
    assert_eq!(frames[0].len(), 64 * 64 * 4);
    // First pixel of frame 0 should be red
    assert_eq!(&frames[0][0..4], &[255, 0, 0, 255]);
    // First pixel of frame 1 should be blue
    assert_eq!(&frames[1][0..4], &[0, 0, 255, 255]);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib test_extract_flipbook -- --nocapture`
Expected: FAIL

**Step 3: Implement extract_flipbook_frames**

```rust
use image::GenericImageView;

/// Extracts frames from a VFX flipbook atlas using metadata UV coordinates.
///
/// Returns (frames_rgba, delays_ms, should_loop).
fn extract_flipbook_frames(
    atlas: &image::RgbaImage,
    metadata: &speccade_spec::recipe::vfx::flipbook::VfxFlipbookMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    let fps = fps_override.unwrap_or(metadata.fps);
    let delay_ms = if fps > 0 { 1000 / fps } else { 83 };
    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    let mut frames = Vec::with_capacity(metadata.frames.len());
    let mut delays = Vec::with_capacity(metadata.frames.len());

    for frame_uv in &metadata.frames {
        let x = (frame_uv.u_min * atlas_w as f64).round() as u32;
        let y = (frame_uv.v_min * atlas_h as f64).round() as u32;
        let w = frame_uv.width;
        let h = frame_uv.height;

        let sub = atlas.view(x, y, w, h);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for pixel in sub.pixels() {
            rgba.extend_from_slice(&pixel.2 .0);
        }
        frames.push(rgba);
        delays.push(delay_ms);
    }

    let should_loop = matches!(
        metadata.loop_mode,
        speccade_spec::recipe::vfx::flipbook::FlipbookLoopMode::Loop
    );

    // Handle PingPong by appending reversed frames (excluding first and last)
    if matches!(
        metadata.loop_mode,
        speccade_spec::recipe::vfx::flipbook::FlipbookLoopMode::PingPong
    ) && frames.len() > 2 {
        let reverse_frames: Vec<_> = frames[1..frames.len()-1].iter().rev().cloned().collect();
        let reverse_delays: Vec<_> = delays[1..delays.len()-1].iter().rev().copied().collect();
        frames.extend(reverse_frames);
        delays.extend(reverse_delays);
        // PingPong should loop
        return (frames, delays, true);
    }

    (frames, delays, should_loop)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p speccade-cli --lib test_extract_flipbook -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): implement flipbook frame extraction from atlas"
```

---

### Task 5: Implement frame extraction for sprite sheet + animation

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview.rs`

Sprite animations reference frames by `frame_id` which must match entries in the spritesheet metadata.

**Step 1: Write a failing test**

```rust
#[test]
fn test_extract_sprite_animation_frames() {
    // 128x64 atlas with 2 frames side by side
    let mut atlas = image::RgbaImage::new(128, 64);
    for y in 0..64 {
        for x in 0..64 {
            atlas.put_pixel(x, y, image::Rgba([255, 0, 0, 255]));
        }
    }
    for y in 0..64 {
        for x in 64..128 {
            atlas.put_pixel(x, y, image::Rgba([0, 255, 0, 255]));
        }
    }

    let sheet_meta: speccade_spec::recipe::sprite::sheet::SpriteSheetMetadata =
        serde_json::from_value(serde_json::json!({
            "atlas_width": 128,
            "atlas_height": 64,
            "padding": 0,
            "frames": [
                { "id": "frame_a", "u_min": 0.0, "v_min": 0.0, "u_max": 0.5, "v_max": 1.0, "width": 64, "height": 64, "pivot": [0.5, 0.5] },
                { "id": "frame_b", "u_min": 0.5, "v_min": 0.0, "u_max": 1.0, "v_max": 1.0, "width": 64, "height": 64, "pivot": [0.5, 0.5] }
            ]
        })).unwrap();

    let anim_meta: speccade_spec::recipe::sprite::animation::SpriteAnimationMetadata =
        serde_json::from_value(serde_json::json!({
            "name": "walk",
            "fps": 12,
            "loop_mode": "loop",
            "total_duration_ms": 166,
            "frames": [
                { "frame_id": "frame_a", "duration_ms": 83 },
                { "frame_id": "frame_b", "duration_ms": 83 }
            ]
        })).unwrap();

    let (frames, delays, do_loop) = extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, None);
    assert_eq!(frames.len(), 2);
    assert_eq!(delays, vec![83, 83]);
    assert!(do_loop);
    assert_eq!(&frames[0][0..4], &[255, 0, 0, 255]); // red
    assert_eq!(&frames[1][0..4], &[0, 255, 0, 255]); // green
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib test_extract_sprite_animation -- --nocapture`
Expected: FAIL

**Step 3: Implement extract_sprite_animation_frames**

```rust
/// Extracts frames from a spritesheet atlas using animation metadata.
fn extract_sprite_animation_frames(
    atlas: &image::RgbaImage,
    sheet: &speccade_spec::recipe::sprite::sheet::SpriteSheetMetadata,
    anim: &speccade_spec::recipe::sprite::animation::SpriteAnimationMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    // Build lookup from frame_id -> SpriteFrameUv
    let frame_map: std::collections::HashMap<&str, &speccade_spec::recipe::sprite::sheet::SpriteFrameUv> =
        sheet.frames.iter().map(|f| (f.id.as_str(), f)).collect();

    let mut frames = Vec::new();
    let mut delays = Vec::new();

    for anim_frame in &anim.frames {
        let Some(uv) = frame_map.get(anim_frame.frame_id.as_str()) else {
            // Skip frames not found in sheet (warn in real usage)
            continue;
        };

        let x = (uv.u_min * atlas_w as f64).round() as u32;
        let y = (uv.v_min * atlas_h as f64).round() as u32;
        let w = uv.width;
        let h = uv.height;

        let sub = atlas.view(x, y, w, h);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for pixel in sub.pixels() {
            rgba.extend_from_slice(&pixel.2 .0);
        }
        frames.push(rgba);

        let delay = if let Some(fps) = fps_override {
            if fps > 0 { 1000 / fps } else { 83 }
        } else {
            anim_frame.duration_ms
        };
        delays.push(delay);
    }

    let should_loop = matches!(
        anim.loop_mode,
        speccade_spec::recipe::sprite::animation::AnimationLoopMode::Loop
    );

    // Handle PingPong
    if matches!(
        anim.loop_mode,
        speccade_spec::recipe::sprite::animation::AnimationLoopMode::PingPong
    ) && frames.len() > 2 {
        let reverse_frames: Vec<_> = frames[1..frames.len()-1].iter().rev().cloned().collect();
        let reverse_delays: Vec<_> = delays[1..delays.len()-1].iter().rev().copied().collect();
        frames.extend(reverse_frames);
        delays.extend(reverse_delays);
        return (frames, delays, true);
    }

    (frames, delays, should_loop)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p speccade-cli --lib test_extract_sprite_animation -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): implement sprite animation frame extraction"
```

---

### Task 6: Implement frame extraction for mesh-to-sprite

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview.rs`

Mesh-to-sprite atlases have frames at rotation angles. No inherent timing, so uses `--fps` override or default 12.

**Step 1: Write a failing test**

```rust
#[test]
fn test_extract_mesh_to_sprite_frames() {
    let mut atlas = image::RgbaImage::new(128, 64);
    // Two 64x64 frames
    for y in 0..64 {
        for x in 0..64 {
            atlas.put_pixel(x, y, image::Rgba([100, 0, 0, 255]));
        }
    }
    for y in 0..64 {
        for x in 64..128 {
            atlas.put_pixel(x, y, image::Rgba([0, 100, 0, 255]));
        }
    }

    let metadata: speccade_spec::recipe::sprite::render_from_mesh::SpriteRenderFromMeshMetadata =
        serde_json::from_value(serde_json::json!({
            "atlas_dimensions": [128, 64],
            "padding": 0,
            "frame_resolution": [64, 64],
            "camera": "orthographic",
            "lighting": "three_point",
            "frames": [
                { "rotation_degrees": 0.0, "position": [0, 0], "uv": [0.0, 0.0, 0.5, 1.0] },
                { "rotation_degrees": 45.0, "position": [64, 0], "uv": [0.5, 0.0, 1.0, 1.0] }
            ]
        })).unwrap();

    let (frames, delays, do_loop) = extract_mesh_sprite_frames(&atlas, &metadata, Some(10));
    assert_eq!(frames.len(), 2);
    assert_eq!(delays, vec![100, 100]); // 1000/10 = 100ms
    assert!(do_loop); // mesh-to-sprite rotations should loop
    assert_eq!(frames[0].len(), 64 * 64 * 4);
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test -p speccade-cli --lib test_extract_mesh_to_sprite -- --nocapture`
Expected: FAIL

**Step 3: Implement extract_mesh_sprite_frames**

```rust
/// Extracts frames from a mesh-to-sprite atlas.
fn extract_mesh_sprite_frames(
    atlas: &image::RgbaImage,
    metadata: &speccade_spec::recipe::sprite::render_from_mesh::SpriteRenderFromMeshMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    let fps = fps_override.unwrap_or(12);
    let delay_ms = if fps > 0 { 1000 / fps } else { 83 };
    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    let mut frames = Vec::with_capacity(metadata.frames.len());
    let mut delays = Vec::with_capacity(metadata.frames.len());

    for frame in &metadata.frames {
        let x = (frame.uv[0] * atlas_w as f64).round() as u32;
        let y = (frame.uv[1] * atlas_h as f64).round() as u32;
        let w = metadata.frame_resolution[0];
        let h = metadata.frame_resolution[1];

        let sub = atlas.view(x, y, w, h);
        let mut rgba = Vec::with_capacity((w * h * 4) as usize);
        for pixel in sub.pixels() {
            rgba.extend_from_slice(&pixel.2 .0);
        }
        frames.push(rgba);
        delays.push(delay_ms);
    }

    // Rotation sequences naturally loop
    (frames, delays, true)
}
```

**Step 4: Run test to verify it passes**

Run: `cargo test -p speccade-cli --lib test_extract_mesh_to_sprite -- --nocapture`
Expected: PASS

**Step 5: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): implement mesh-to-sprite frame extraction"
```

---

### Task 7: Wire up the preview command's --gif path

**Files:**
- Modify: `crates/speccade-cli/src/commands/preview.rs`

This ties everything together: spec parsing -> artifact discovery -> frame extraction -> GIF assembly.

**Step 1: Implement the --gif path in preview::run**

Replace the body of `preview::run` with:

```rust
pub fn run(
    spec_path: &str,
    out_root: Option<&str>,
    gif: bool,
    out: Option<&str>,
    fps: Option<u32>,
    scale: Option<u32>,
) -> Result<ExitCode> {
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;
    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    if !gif {
        // Original stub behavior
        println!("{} {}", "Preview:".cyan().bold(), spec_path);
        println!(
            "\n{} Preview is not yet implemented for asset type '{}'",
            "INFO".yellow().bold(),
            spec.asset_type
        );
        println!(
            "{}",
            "Use --gif to export an animated GIF preview."
                .dimmed()
        );
        return Ok(ExitCode::SUCCESS);
    }

    // Determine recipe kind
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Spec has no recipe; cannot generate GIF preview"))?;

    // Find output artifacts directory
    let spec_dir = std::path::Path::new(spec_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let root = out_root
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| spec_dir.to_path_buf());

    // Find PNG atlas and JSON metadata from spec outputs
    let png_output = spec
        .outputs
        .iter()
        .find(|o| matches!(o.format, speccade_spec::output::OutputFormat::Png));
    let json_output = spec
        .outputs
        .iter()
        .find(|o| matches!(o.format, speccade_spec::output::OutputFormat::Json));

    let (frames, delays, should_loop) = match recipe.kind.as_str() {
        "vfx.flipbook_v1" => {
            let atlas_path = png_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No PNG output in spec"))?;
            let meta_path = json_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No JSON metadata output in spec"))?;

            let atlas = image::open(&atlas_path)
                .with_context(|| format!("Failed to open atlas: {}", atlas_path.display()))?
                .to_rgba8();
            let meta_str = fs::read_to_string(&meta_path)
                .with_context(|| format!("Failed to read metadata: {}", meta_path.display()))?;
            let metadata: speccade_spec::recipe::vfx::flipbook::VfxFlipbookMetadata =
                serde_json::from_str(&meta_str)?;

            extract_flipbook_frames(&atlas, &metadata, fps)
        }
        "sprite.sheet_v1" => {
            // Need both sheet metadata and animation metadata
            let atlas_path = png_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No PNG output in spec"))?;
            let meta_path = json_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No JSON metadata output in spec"))?;

            let atlas = image::open(&atlas_path)
                .with_context(|| format!("Failed to open atlas: {}", atlas_path.display()))?
                .to_rgba8();
            let meta_str = fs::read_to_string(&meta_path)?;
            let sheet_meta: speccade_spec::recipe::sprite::sheet::SpriteSheetMetadata =
                serde_json::from_str(&meta_str)?;

            // Look for animation JSON (second JSON output or .animation.json convention)
            let anim_path = spec
                .outputs
                .iter()
                .filter(|o| matches!(o.format, speccade_spec::output::OutputFormat::Json))
                .nth(1)
                .map(|o| root.join(&o.path))
                .or_else(|| {
                    // Convention: <atlas_stem>.animation.json
                    let stem = atlas_path.file_stem()?.to_str()?;
                    let anim_file = atlas_path.with_file_name(format!("{}.animation.json", stem));
                    if anim_file.exists() { Some(anim_file) } else { None }
                });

            if let Some(anim_path) = anim_path {
                let anim_str = fs::read_to_string(&anim_path)?;
                let anim_meta: speccade_spec::recipe::sprite::animation::SpriteAnimationMetadata =
                    serde_json::from_str(&anim_str)?;
                extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, fps)
            } else {
                // No animation metadata: animate through sheet frames in order
                let frame_fps = fps.unwrap_or(12);
                let delay_ms = if frame_fps > 0 { 1000 / frame_fps } else { 83 };
                let atlas_w = atlas.width();
                let atlas_h = atlas.height();
                let mut out_frames = Vec::new();
                let mut out_delays = Vec::new();
                for uv in &sheet_meta.frames {
                    let x = (uv.u_min * atlas_w as f64).round() as u32;
                    let y = (uv.v_min * atlas_h as f64).round() as u32;
                    let sub = atlas.view(x, y, uv.width, uv.height);
                    let mut rgba = Vec::with_capacity((uv.width * uv.height * 4) as usize);
                    for pixel in sub.pixels() {
                        rgba.extend_from_slice(&pixel.2 .0);
                    }
                    out_frames.push(rgba);
                    out_delays.push(delay_ms);
                }
                (out_frames, out_delays, true)
            }
        }
        "sprite.render_from_mesh_v1" => {
            let atlas_path = png_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No PNG output in spec"))?;
            let meta_path = json_output
                .map(|o| root.join(&o.path))
                .ok_or_else(|| anyhow::anyhow!("No JSON metadata output in spec"))?;

            let atlas = image::open(&atlas_path)
                .with_context(|| format!("Failed to open atlas: {}", atlas_path.display()))?
                .to_rgba8();
            let meta_str = fs::read_to_string(&meta_path)?;
            let metadata: speccade_spec::recipe::sprite::render_from_mesh::SpriteRenderFromMeshMetadata =
                serde_json::from_str(&meta_str)?;

            extract_mesh_sprite_frames(&atlas, &metadata, fps)
        }
        other => {
            anyhow::bail!(
                "GIF preview not supported for recipe kind '{}'. Supported: vfx.flipbook_v1, sprite.sheet_v1, sprite.render_from_mesh_v1",
                other
            );
        }
    };

    if frames.is_empty() {
        anyhow::bail!("No frames extracted from asset");
    }

    // Apply scale if requested
    let (final_frames, width, height) = if let Some(s) = scale {
        if s > 1 {
            let orig_w = frames[0].len() as u32 / 4;
            // Assume square root or derive from first frame
            // Actually we need to know dimensions. For simplicity, detect from frame data size
            // We'll refactor to pass dimensions through. For now, bail if scale is set.
            // TODO: implement scaling via image crate resize
            let _ = orig_w;
            let _ = s;
            eprintln!("{} --scale not yet implemented, using scale=1", "WARN".yellow().bold());
            // Detect width/height from recipe metadata
            let (w, h) = detect_frame_dimensions(&recipe.kind, &recipe.params)?;
            (frames, w as u16, h as u16)
        } else {
            let (w, h) = detect_frame_dimensions(&recipe.kind, &recipe.params)?;
            (frames, w as u16, h as u16)
        }
    } else {
        let (w, h) = detect_frame_dimensions(&recipe.kind, &recipe.params)?;
        (frames, w as u16, h as u16)
    };

    // Determine output path
    let default_out = {
        let stem = std::path::Path::new(spec_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("preview");
        let parent = std::path::Path::new(spec_path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        parent.join(format!("{}.preview.gif", stem))
    };
    let out_path = out
        .map(std::path::PathBuf::from)
        .unwrap_or(default_out);

    println!("{} {}", "GIF Preview:".cyan().bold(), spec_path);
    println!(
        "  {} frames, {}x{}, {} -> {}",
        final_frames.len(),
        width,
        height,
        recipe.kind,
        out_path.display()
    );

    assemble_gif(
        &final_frames,
        width,
        height,
        &delays,
        should_loop,
        out_path.to_str().unwrap(),
    )?;

    println!("{} {}", "OK".green().bold(), out_path.display());
    Ok(ExitCode::SUCCESS)
}

/// Detect frame dimensions from recipe params.
fn detect_frame_dimensions(kind: &str, params: &serde_json::Value) -> Result<(u32, u32)> {
    match kind {
        "vfx.flipbook_v1" => {
            let fs = params.get("frame_size")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing frame_size in flipbook params"))?;
            Ok((
                fs[0].as_u64().unwrap_or(64) as u32,
                fs[1].as_u64().unwrap_or(64) as u32,
            ))
        }
        "sprite.sheet_v1" => {
            // Use first frame dimensions from params
            let frames = params.get("frames")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing frames in sheet params"))?;
            if let Some(first) = frames.first() {
                Ok((
                    first.get("width").and_then(|v| v.as_u64()).unwrap_or(64) as u32,
                    first.get("height").and_then(|v| v.as_u64()).unwrap_or(64) as u32,
                ))
            } else {
                Ok((64, 64))
            }
        }
        "sprite.render_from_mesh_v1" => {
            let fr = params.get("frame_resolution")
                .and_then(|v| v.as_array())
                .ok_or_else(|| anyhow::anyhow!("Missing frame_resolution in mesh-to-sprite params"))?;
            Ok((
                fr[0].as_u64().unwrap_or(64) as u32,
                fr[1].as_u64().unwrap_or(64) as u32,
            ))
        }
        _ => Ok((64, 64)),
    }
}
```

**Step 2: Verify compilation**

Run: `cargo check -p speccade-cli`
Expected: compiles

**Step 3: Run all preview tests**

Run: `cargo test -p speccade-cli --lib preview -- --nocapture`
Expected: all pass

**Step 4: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "feat(preview): wire up --gif flag with full pipeline"
```

---

### Task 8: Add integration test with golden flipbook spec

**Files:**
- Test: `crates/speccade-cli/src/commands/preview.rs` (or `crates/speccade-tests/` if integration tests live there)

**Step 1: Find an existing flipbook spec in golden/

Look for an existing flipbook spec in `golden/speccade/specs/` directory. If one exists, use it. If not, create a minimal test fixture inline.

**Step 2: Write an integration-style test**

```rust
#[test]
fn test_gif_preview_end_to_end_flipbook() {
    let tmp = tempfile::tempdir().unwrap();

    // Create a minimal 4-frame flipbook atlas (2x2 grid, 32x32 frames = 64x64 atlas)
    let mut atlas = image::RgbaImage::new(64, 64);
    let colors = [
        [255, 0, 0, 255],   // red
        [0, 255, 0, 255],   // green
        [0, 0, 255, 255],   // blue
        [255, 255, 0, 255], // yellow
    ];
    for (idx, color) in colors.iter().enumerate() {
        let bx = (idx % 2) as u32 * 32;
        let by = (idx / 2) as u32 * 32;
        for y in 0..32 {
            for x in 0..32 {
                atlas.put_pixel(bx + x, by + y, image::Rgba(*color));
            }
        }
    }
    atlas.save(tmp.path().join("atlas.png")).unwrap();

    // Write metadata JSON
    let metadata = serde_json::json!({
        "atlas_width": 64,
        "atlas_height": 64,
        "padding": 0,
        "effect": "explosion",
        "frame_count": 4,
        "frame_size": [32, 32],
        "fps": 10,
        "loop_mode": "loop",
        "total_duration_ms": 400,
        "frames": [
            { "index": 0, "u_min": 0.0, "v_min": 0.0, "u_max": 0.5, "v_max": 0.5, "width": 32, "height": 32 },
            { "index": 1, "u_min": 0.5, "v_min": 0.0, "u_max": 1.0, "v_max": 0.5, "width": 32, "height": 32 },
            { "index": 2, "u_min": 0.0, "v_min": 0.5, "u_max": 0.5, "v_max": 1.0, "width": 32, "height": 32 },
            { "index": 3, "u_min": 0.5, "v_min": 0.5, "u_max": 1.0, "v_max": 1.0, "width": 32, "height": 32 }
        ]
    });
    std::fs::write(
        tmp.path().join("atlas.json"),
        serde_json::to_string_pretty(&metadata).unwrap(),
    ).unwrap();

    // Write spec
    let spec = serde_json::json!({
        "spec_version": 1,
        "asset_id": "test-flipbook",
        "asset_type": "vfx",
        "license": "CC0-1.0",
        "seed": 42,
        "outputs": [
            { "kind": "primary", "format": "png", "path": "atlas.png" },
            { "kind": "metadata", "format": "json", "path": "atlas.json" }
        ],
        "recipe": {
            "kind": "vfx.flipbook_v1",
            "params": {
                "resolution": [64, 64],
                "effect": "explosion",
                "frame_count": 4,
                "frame_size": [32, 32],
                "fps": 10,
                "loop_mode": "loop"
            }
        }
    });
    let spec_path = tmp.path().join("test.spec.json");
    std::fs::write(&spec_path, serde_json::to_string_pretty(&spec).unwrap()).unwrap();

    let gif_path = tmp.path().join("test.preview.gif");
    let code = run(
        spec_path.to_str().unwrap(),
        Some(tmp.path().to_str().unwrap()),
        true,
        Some(gif_path.to_str().unwrap()),
        None,
        None,
    ).unwrap();

    assert_eq!(code, ExitCode::SUCCESS);
    assert!(gif_path.exists());

    let data = std::fs::read(&gif_path).unwrap();
    assert_eq!(&data[0..3], b"GIF");
    assert!(data.len() > 100); // Sanity: non-trivial file
}
```

**Step 3: Run the test**

Run: `cargo test -p speccade-cli --lib test_gif_preview_end_to_end -- --nocapture`
Expected: PASS

**Step 4: Commit**

```bash
git add crates/speccade-cli/src/commands/preview.rs
git commit -m "test(preview): add end-to-end GIF preview test for flipbook"
```

---

### Task 9: Update documentation

**Files:**
- Modify: `docs/spec-reference/` (if a preview doc exists) or add a note to README

**Step 1: Add usage note to the preview section of any relevant docs**

Check if `docs/spec-reference/` has a preview section. If not, add a short usage section to `README.md` under Quick Commands:

```markdown
# GIF Preview
speccade preview --spec path/to/spec.json --gif
speccade preview --spec path/to/spec.json --gif --fps 24 --out my-preview.gif
```

**Step 2: Commit**

```bash
git add docs/ README.md
git commit -m "docs: add GIF preview usage examples"
```
