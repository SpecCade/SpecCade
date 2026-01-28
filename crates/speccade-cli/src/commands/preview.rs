//! Preview command implementation
//!
//! Opens an asset preview (stub for Blender preview).

use anyhow::{Context, Result};
use colored::Colorize;
use gif::{Encoder, Frame as GifFrame, Repeat};
use speccade_spec::Spec;
use std::fs;
use std::io::BufWriter;
use std::path::PathBuf;
use std::process::ExitCode;

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
        anyhow::ensure!(
            frame_rgba.len() == (width as usize) * (height as usize) * 4,
            "Frame {} has unexpected byte length: {}",
            i,
            frame_rgba.len()
        );

        // GIF delay is in centiseconds (1/100th of a second)
        let delay_cs_u32 = ((delays_ms[i] + 9) / 10).max(1);
        let delay_cs = delay_cs_u32.min(u32::from(u16::MAX)) as u16;

        let mut rgba = frame_rgba.clone();
        let mut gif_frame = GifFrame::from_rgba_speed(width, height, &mut rgba, 10);
        gif_frame.delay = delay_cs;
        gif_frame.dispose = gif::DisposalMethod::Background;

        encoder
            .write_frame(&gif_frame)
            .with_context(|| format!("Failed to write GIF frame {}", i))?;
    }

    Ok(())
}

/// Extracts frames from a VFX flipbook atlas using metadata UV coordinates.
///
/// Returns (frames_rgba, delays_ms, should_loop).
fn extract_flipbook_frames(
    atlas: &image::RgbaImage,
    metadata: &speccade_spec::recipe::vfx::VfxFlipbookMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    use image::GenericImageView;

    let fps = fps_override.unwrap_or(metadata.fps);
    let delay_ms = if fps == 0 { 83 } else { (1000 / fps).max(1) };

    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    let mut frames = Vec::with_capacity(metadata.frames.len());
    let mut delays = Vec::with_capacity(metadata.frames.len());

    for frame_uv in &metadata.frames {
        let w = frame_uv.width;
        let h = frame_uv.height;

        if w == 0 || h == 0 {
            continue;
        }
        if w > atlas_w || h > atlas_h {
            continue;
        }

        let max_x = atlas_w - w;
        let max_y = atlas_h - h;

        let x = (frame_uv.u_min * atlas_w as f64)
            .round()
            .max(0.0)
            .min(max_x as f64) as u32;
        let y = (frame_uv.v_min * atlas_h as f64)
            .round()
            .max(0.0)
            .min(max_y as f64) as u32;

        let rgba = atlas.view(x, y, w, h).to_image().into_raw();
        frames.push(rgba);
        delays.push(delay_ms);
    }

    let mut should_loop = matches!(
        metadata.loop_mode,
        speccade_spec::recipe::vfx::FlipbookLoopMode::Loop
    );

    if matches!(
        metadata.loop_mode,
        speccade_spec::recipe::vfx::FlipbookLoopMode::PingPong
    ) {
        // PingPong always loops.
        should_loop = true;

        // Append reversed frames excluding the first and last so the endpoints
        // are not duplicated.
        if frames.len() > 2 {
            let base_len = frames.len();
            let extra = base_len - 2;
            frames.reserve(extra);
            delays.reserve(extra);

            for i in (1..base_len - 1).rev() {
                frames.push(frames[i].clone());
                delays.push(delays[i]);
            }
        }
    }

    (frames, delays, should_loop)
}

/// Extracts frames from a spritesheet atlas using sheet + animation metadata.
///
/// Returns (frames_rgba, delays_ms, should_loop).
fn extract_sprite_animation_frames(
    atlas: &image::RgbaImage,
    sheet_meta: &speccade_spec::recipe::sprite::SpriteSheetMetadata,
    anim_meta: &speccade_spec::recipe::sprite::SpriteAnimationMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    use image::GenericImageView;
    use speccade_spec::recipe::sprite::AnimationLoopMode;
    use std::collections::HashMap;

    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    let mut uv_by_id = HashMap::with_capacity(sheet_meta.frames.len());
    for uv in &sheet_meta.frames {
        uv_by_id.insert(uv.id.as_str(), uv);
    }

    let override_delay_ms = fps_override.map(|fps| if fps == 0 { 83 } else { (1000 / fps).max(1) });

    let default_delay_ms = if anim_meta.fps > 0 {
        (1000 / anim_meta.fps).max(1)
    } else {
        83
    };

    let mut frames = Vec::with_capacity(anim_meta.frames.len());
    let mut delays = Vec::with_capacity(anim_meta.frames.len());

    for anim_frame in &anim_meta.frames {
        let Some(frame_uv) = uv_by_id.get(anim_frame.frame_id.as_str()) else {
            continue;
        };

        let w = frame_uv.width;
        let h = frame_uv.height;

        if w == 0 || h == 0 {
            continue;
        }
        if w > atlas_w || h > atlas_h {
            continue;
        }
        if !frame_uv.u_min.is_finite() || !frame_uv.v_min.is_finite() {
            continue;
        }

        let max_x = atlas_w - w;
        let max_y = atlas_h - h;

        let x = (frame_uv.u_min * atlas_w as f64)
            .round()
            .max(0.0)
            .min(max_x as f64) as u32;
        let y = (frame_uv.v_min * atlas_h as f64)
            .round()
            .max(0.0)
            .min(max_y as f64) as u32;

        let rgba = atlas.view(x, y, w, h).to_image().into_raw();
        frames.push(rgba);

        let delay_ms = match override_delay_ms {
            Some(ms) => ms,
            None => {
                if anim_frame.duration_ms == 0 {
                    default_delay_ms
                } else {
                    anim_frame.duration_ms
                }
            }
        };
        delays.push(delay_ms);
    }

    let mut should_loop = matches!(anim_meta.loop_mode, AnimationLoopMode::Loop);

    if matches!(anim_meta.loop_mode, AnimationLoopMode::PingPong) {
        // PingPong always loops.
        should_loop = true;

        // Append reversed frames excluding the first and last so the endpoints
        // are not duplicated.
        if frames.len() > 2 {
            let base_len = frames.len();
            let extra = base_len - 2;
            frames.reserve(extra);
            delays.reserve(extra);

            for i in (1..base_len - 1).rev() {
                frames.push(frames[i].clone());
                delays.push(delays[i]);
            }
        }
    }

    (frames, delays, should_loop)
}

/// Extracts frames from a mesh-rendered sprite atlas using UV coordinates.
///
/// Returns (frames_rgba, delays_ms, should_loop).
fn extract_mesh_sprite_frames(
    atlas: &image::RgbaImage,
    metadata: &speccade_spec::recipe::sprite::SpriteRenderFromMeshMetadata,
    fps_override: Option<u32>,
) -> (Vec<Vec<u8>>, Vec<u32>, bool) {
    use image::GenericImageView;

    let fps = fps_override.unwrap_or(12);
    let delay_ms = if fps == 0 { 83 } else { (1000 / fps).max(1) };

    let atlas_w = atlas.width();
    let atlas_h = atlas.height();

    let w = metadata.frame_resolution[0];
    let h = metadata.frame_resolution[1];

    if w == 0 || h == 0 {
        return (Vec::new(), Vec::new(), true);
    }
    if w > atlas_w || h > atlas_h {
        return (Vec::new(), Vec::new(), true);
    }

    let max_x = atlas_w - w;
    let max_y = atlas_h - h;

    let mut frames = Vec::with_capacity(metadata.frames.len());
    let mut delays = Vec::with_capacity(metadata.frames.len());

    for frame in &metadata.frames {
        let u_min = frame.uv[0];
        let v_min = frame.uv[1];

        if !u_min.is_finite() || !v_min.is_finite() {
            continue;
        }

        let x = (u_min * atlas_w as f64).round().max(0.0).min(max_x as f64) as u32;
        let y = (v_min * atlas_h as f64).round().max(0.0).min(max_y as f64) as u32;

        let rgba = atlas.view(x, y, w, h).to_image().into_raw();
        frames.push(rgba);
        delays.push(delay_ms);
    }

    (frames, delays, true)
}

/// Run the preview command
///
/// # Arguments
/// * `spec_path` - Path to the spec JSON file
/// * `out_root` - Output root directory (default: current directory)
///
/// # Returns
/// Exit code: 0 success, 1 error
pub fn run(
    spec_path: &str,
    out_root: Option<&str>,
    gif: bool,
    out: Option<&str>,
    fps: Option<u32>,
    scale: Option<u32>,
) -> Result<ExitCode> {
    // Read and parse spec to get asset type / recipe kind
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

    if !gif {
        // TODO: Implement preview for generated assets (open viewers, or launch Blender for mesh/anim).
        println!("{} {}", "Preview:".cyan().bold(), spec_path);

        // Preview is currently only planned for Blender-based assets
        println!(
            "\n{} Preview is not yet implemented for asset type '{}'",
            "INFO".yellow().bold(),
            spec.asset_type
        );
        println!(
            "{}",
            "Preview functionality will be available in a future release for mesh and animation assets."
                .dimmed()
        );

        // Return success since this is expected behavior for now
        return Ok(ExitCode::SUCCESS);
    }

    let recipe = spec
        .recipe
        .as_ref()
        .with_context(|| "Spec has no recipe; cannot export GIF preview")?;

    // Artifact discovery
    let spec_path_pb = PathBuf::from(spec_path);
    let spec_dir = spec_path_pb
        .parent()
        .with_context(|| format!("Spec path has no parent directory: {}", spec_path))?;

    let root = out_root
        .map(PathBuf::from)
        .unwrap_or_else(|| spec_dir.to_path_buf());

    let png_output = spec
        .outputs
        .iter()
        .find(|o| matches!(o.format, speccade_spec::output::OutputFormat::Png))
        .with_context(|| "Spec has no PNG output (atlas) in outputs[]")?;

    let json_output = spec
        .outputs
        .iter()
        .find(|o| matches!(o.format, speccade_spec::output::OutputFormat::Json))
        .with_context(|| "Spec has no JSON output (metadata) in outputs[]")?;

    let atlas_path = root.join(&png_output.path);
    let metadata_path = root.join(&json_output.path);

    let atlas = image::open(&atlas_path)
        .with_context(|| format!("Failed to open atlas PNG: {}", atlas_path.display()))?
        .to_rgba8();

    let metadata_str = fs::read_to_string(&metadata_path)
        .with_context(|| format!("Failed to read metadata JSON: {}", metadata_path.display()))?;
    let metadata_json: serde_json::Value = serde_json::from_str(&metadata_str)
        .with_context(|| format!("Failed to parse metadata JSON: {}", metadata_path.display()))?;

    let (mut frames, delays_ms, should_loop, frame_w, frame_h) = match recipe.kind.as_str() {
        "vfx.flipbook_v1" => {
            let meta: speccade_spec::recipe::vfx::VfxFlipbookMetadata =
                serde_json::from_value(metadata_json).with_context(|| {
                    format!(
                        "Metadata JSON did not match VfxFlipbookMetadata: {}",
                        metadata_path.display()
                    )
                })?;

            let (frames, delays, should_loop) = extract_flipbook_frames(&atlas, &meta, fps);
            (
                frames,
                delays,
                should_loop,
                meta.frame_size[0],
                meta.frame_size[1],
            )
        }
        "sprite.sheet_v1" => {
            let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
                serde_json::from_value(metadata_json).with_context(|| {
                    format!(
                        "Metadata JSON did not match SpriteSheetMetadata: {}",
                        metadata_path.display()
                    )
                })?;

            // Optional: look for a separate JSON output in outputs[] that parses as SpriteAnimationMetadata.
            let mut anim_meta: Option<speccade_spec::recipe::sprite::SpriteAnimationMetadata> =
                None;
            for extra in spec.outputs.iter().filter(|o| {
                matches!(o.format, speccade_spec::output::OutputFormat::Json)
                    && o.path != json_output.path
            }) {
                let path = root.join(&extra.path);
                let s = fs::read_to_string(&path).with_context(|| {
                    format!("Failed to read animation metadata JSON: {}", path.display())
                })?;
                match serde_json::from_str::<speccade_spec::recipe::sprite::SpriteAnimationMetadata>(
                    &s,
                ) {
                    Ok(parsed) => {
                        anim_meta = Some(parsed);
                        break;
                    }
                    Err(_) => continue,
                }
            }

            if let Some(anim_meta) = anim_meta {
                let (w, h) = sheet_meta
                    .frames
                    .iter()
                    .filter_map(|uv| {
                        anim_meta
                            .frames
                            .iter()
                            .any(|f| f.frame_id == uv.id)
                            .then(|| (uv.width, uv.height))
                    })
                    .next()
                    .with_context(|| {
                        "SpriteAnimationMetadata referenced no frames present in SpriteSheetMetadata"
                    })?;

                let (frames, delays, should_loop) =
                    extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, fps);

                (frames, delays, should_loop, w, h)
            } else {
                // No animation metadata: animate through sheet frames in order.
                use image::GenericImageView;

                let delay_ms = match fps.unwrap_or(12) {
                    0 => 83,
                    f => (1000 / f).max(1),
                };

                let atlas_w = atlas.width();
                let atlas_h = atlas.height();

                let mut out_frames: Vec<Vec<u8>> = Vec::with_capacity(sheet_meta.frames.len());
                let mut out_delays: Vec<u32> = Vec::with_capacity(sheet_meta.frames.len());

                let mut expected_w: Option<u32> = None;
                let mut expected_h: Option<u32> = None;

                for uv in &sheet_meta.frames {
                    let w = uv.width;
                    let h = uv.height;
                    if w == 0 || h == 0 {
                        continue;
                    }
                    if w > atlas_w || h > atlas_h {
                        continue;
                    }
                    if !uv.u_min.is_finite() || !uv.v_min.is_finite() {
                        continue;
                    }

                    if let (Some(ew), Some(eh)) = (expected_w, expected_h) {
                        anyhow::ensure!(
                            ew == w && eh == h,
                            "SpriteSheetMetadata has mixed frame sizes ({}x{} vs {}x{}) which cannot be encoded into a single GIF",
                            ew,
                            eh,
                            w,
                            h
                        );
                    } else {
                        expected_w = Some(w);
                        expected_h = Some(h);
                    }

                    let max_x = atlas_w - w;
                    let max_y = atlas_h - h;

                    let x = (uv.u_min * atlas_w as f64)
                        .round()
                        .max(0.0)
                        .min(max_x as f64) as u32;
                    let y = (uv.v_min * atlas_h as f64)
                        .round()
                        .max(0.0)
                        .min(max_y as f64) as u32;

                    let rgba = atlas.view(x, y, w, h).to_image().into_raw();
                    out_frames.push(rgba);
                    out_delays.push(delay_ms);
                }

                let w = expected_w.unwrap_or(0);
                let h = expected_h.unwrap_or(0);
                (out_frames, out_delays, true, w, h)
            }
        }
        "sprite.render_from_mesh_v1" => {
            let meta: speccade_spec::recipe::sprite::SpriteRenderFromMeshMetadata =
                serde_json::from_value(metadata_json).with_context(|| {
                    format!(
                        "Metadata JSON did not match SpriteRenderFromMeshMetadata: {}",
                        metadata_path.display()
                    )
                })?;

            let (frames, delays, should_loop) = extract_mesh_sprite_frames(&atlas, &meta, fps);
            (
                frames,
                delays,
                should_loop,
                meta.frame_resolution[0],
                meta.frame_resolution[1],
            )
        }
        other => {
            anyhow::bail!(
                "GIF preview not supported for recipe kind '{}'; supported: vfx.flipbook_v1, sprite.sheet_v1, sprite.render_from_mesh_v1",
                other
            );
        }
    };

    anyhow::ensure!(!frames.is_empty(), "Frame extraction produced no frames");
    anyhow::ensure!(
        frame_w > 0 && frame_h > 0,
        "Frame dimensions are invalid: {}x{}",
        frame_w,
        frame_h
    );

    let expected_len = (frame_w as usize)
        .checked_mul(frame_h as usize)
        .and_then(|px| px.checked_mul(4))
        .with_context(|| format!("Frame dimensions overflow: {}x{}", frame_w, frame_h))?;

    for (i, f) in frames.iter().enumerate() {
        anyhow::ensure!(
            f.len() == expected_len,
            "Extracted frame {} has unexpected byte length {} (expected {})",
            i,
            f.len(),
            expected_len
        );
    }

    // Scaling (pixel-art safe)
    let scale_factor = scale.unwrap_or(1);
    let (out_frames, out_w, out_h) = if scale_factor <= 1 {
        anyhow::ensure!(
            frame_w <= u32::from(u16::MAX) && frame_h <= u32::from(u16::MAX),
            "Frame dimensions too large for GIF encoder: {}x{}",
            frame_w,
            frame_h
        );

        (std::mem::take(&mut frames), frame_w as u16, frame_h as u16)
    } else {
        use image::imageops::FilterType;

        let new_w = frame_w
            .checked_mul(scale_factor)
            .with_context(|| "Scaled width overflow")?;
        let new_h = frame_h
            .checked_mul(scale_factor)
            .with_context(|| "Scaled height overflow")?;

        anyhow::ensure!(
            new_w <= u32::from(u16::MAX) && new_h <= u32::from(u16::MAX),
            "Scaled frame dimensions too large for GIF encoder: {}x{}",
            new_w,
            new_h
        );

        let mut scaled_frames = Vec::with_capacity(frames.len());
        for (i, rgba) in frames.into_iter().enumerate() {
            let img = image::RgbaImage::from_raw(frame_w, frame_h, rgba)
                .with_context(|| format!("Failed to construct image from frame {}", i))?;
            let resized = image::imageops::resize(&img, new_w, new_h, FilterType::Nearest);
            scaled_frames.push(resized.into_raw());
        }

        (scaled_frames, new_w as u16, new_h as u16)
    };

    // Output path
    let out_path = if let Some(out) = out {
        PathBuf::from(out)
    } else {
        let stem = spec_path_pb
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("preview");
        let parent = spec_dir;
        parent.join(format!("{}.preview.gif", stem))
    };

    if let Some(parent) = out_path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create output directory: {}", parent.display())
            })?;
        }
    }

    let out_path_str = out_path
        .to_str()
        .with_context(|| format!("Output path is not valid UTF-8: {}", out_path.display()))?;

    assemble_gif(
        &out_frames,
        out_w,
        out_h,
        &delays_ms,
        should_loop,
        out_path_str,
    )?;

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn test_preview_stub_returns_success() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.json");
        std::fs::write(
            &spec_path,
            r#"{
	  "spec_version": 1,
	  "asset_id": "test-asset-01",
	  "asset_type": "audio",
	  "license": "CC0-1.0",
	  "seed": 42,
	  "outputs": [{"kind": "primary", "format": "wav", "path": "sounds/test.wav"}]
	}"#,
        )
        .unwrap();

        let code = run(spec_path.to_str().unwrap(), None, false, None, None, None).unwrap();
        assert_eq!(code, ExitCode::SUCCESS);
    }

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
            true,             // loop
            out_path.to_str().unwrap(),
        );
        assert!(result.is_ok());
        assert!(out_path.exists());

        // Verify it starts with GIF magic bytes
        let data = std::fs::read(&out_path).unwrap();
        assert_eq!(&data[0..3], b"GIF");
    }

    #[test]
    fn test_extract_flipbook_frames() {
        // Create a 128x64 atlas with 2 frames of 64x64 (side by side, no padding)
        let mut atlas = RgbaImage::new(128, 64);

        // Frame 0: red
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        // Frame 1: blue
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 0, 255, 255]));
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

        let metadata: speccade_spec::recipe::vfx::VfxFlipbookMetadata =
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

    #[test]
    fn test_extract_flipbook_frames_skips_out_of_bounds() {
        let atlas = RgbaImage::new(16, 16);

        let metadata_json = serde_json::json!({
            "atlas_width": 16,
            "atlas_height": 16,
            "padding": 0,
            "effect": "explosion",
            "frame_count": 1,
            "frame_size": [32, 32],
            "fps": 12,
            "loop_mode": "once",
            "total_duration_ms": 0,
            "frames": [
                { "index": 0, "u_min": 0.0, "v_min": 0.0, "u_max": 2.0, "v_max": 2.0, "width": 32, "height": 32 }
            ]
        });

        let metadata: speccade_spec::recipe::vfx::VfxFlipbookMetadata =
            serde_json::from_value(metadata_json).unwrap();

        let (frames, delays, do_loop) = extract_flipbook_frames(&atlas, &metadata, None);
        assert!(frames.is_empty());
        assert!(delays.is_empty());
        assert!(!do_loop);
    }

    #[test]
    fn test_extract_sprite_animation_frames() {
        // Create a 128x64 atlas with 2 frames of 64x64 (side by side, no padding)
        let mut atlas = RgbaImage::new(128, 64);

        // Frame A: red
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        // Frame B: green
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        let sheet_meta_json = serde_json::json!({
            "atlas_width": 128,
            "atlas_height": 64,
            "padding": 0,
            "frames": [
                {
                    "id": "frame_a",
                    "u_min": 0.0,
                    "v_min": 0.0,
                    "u_max": 0.5,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                },
                {
                    "id": "frame_b",
                    "u_min": 0.5,
                    "v_min": 0.0,
                    "u_max": 1.0,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                }
            ]
        });

        let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
            serde_json::from_value(sheet_meta_json).unwrap();

        let anim_meta_json = serde_json::json!({
            "name": "test_anim",
            "fps": 12,
            "loop_mode": "loop",
            "total_duration_ms": 166,
            "frames": [
                { "frame_id": "frame_a", "duration_ms": 83 },
                { "frame_id": "frame_b", "duration_ms": 83 }
            ]
        });

        let anim_meta: speccade_spec::recipe::sprite::SpriteAnimationMetadata =
            serde_json::from_value(anim_meta_json).unwrap();

        let (frames, delays, do_loop) =
            extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, None);

        assert_eq!(frames.len(), 2);
        assert_eq!(delays, vec![83, 83]);
        assert!(do_loop);

        // First pixel of frame A should be red
        assert_eq!(&frames[0][0..4], &[255, 0, 0, 255]);
        // First pixel of frame B should be green
        assert_eq!(&frames[1][0..4], &[0, 255, 0, 255]);
    }

    #[test]
    fn test_extract_sprite_animation_frames_fps_override_zero_uses_83ms() {
        let mut atlas = RgbaImage::new(128, 64);
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        let sheet_meta_json = serde_json::json!({
            "atlas_width": 128,
            "atlas_height": 64,
            "padding": 0,
            "frames": [
                {
                    "id": "frame_a",
                    "u_min": 0.0,
                    "v_min": 0.0,
                    "u_max": 0.5,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                },
                {
                    "id": "frame_b",
                    "u_min": 0.5,
                    "v_min": 0.0,
                    "u_max": 1.0,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                }
            ]
        });

        let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
            serde_json::from_value(sheet_meta_json).unwrap();

        let anim_meta_json = serde_json::json!({
            "name": "test_anim",
            "fps": 12,
            "loop_mode": "loop",
            "total_duration_ms": 166,
            "frames": [
                { "frame_id": "frame_a", "duration_ms": 1 },
                { "frame_id": "frame_b", "duration_ms": 1 }
            ]
        });

        let anim_meta: speccade_spec::recipe::sprite::SpriteAnimationMetadata =
            serde_json::from_value(anim_meta_json).unwrap();

        let (_frames, delays, _do_loop) =
            extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, Some(0));

        assert_eq!(delays, vec![83, 83]);
    }

    #[test]
    fn test_extract_sprite_animation_frames_duration_zero_uses_fps_fallback() {
        let mut atlas = RgbaImage::new(128, 64);
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let sheet_meta_json = serde_json::json!({
            "atlas_width": 128,
            "atlas_height": 64,
            "padding": 0,
            "frames": [
                {
                    "id": "frame_a",
                    "u_min": 0.0,
                    "v_min": 0.0,
                    "u_max": 0.5,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                }
            ]
        });

        let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
            serde_json::from_value(sheet_meta_json).unwrap();

        let anim_meta_json = serde_json::json!({
            "name": "test_anim",
            "fps": 10,
            "loop_mode": "loop",
            "total_duration_ms": 0,
            "frames": [
                { "frame_id": "frame_a", "duration_ms": 0 }
            ]
        });

        let anim_meta: speccade_spec::recipe::sprite::SpriteAnimationMetadata =
            serde_json::from_value(anim_meta_json).unwrap();

        let (_frames, delays, _do_loop) =
            extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, None);

        assert_eq!(delays, vec![100]);
    }

    #[test]
    fn test_extract_sprite_animation_frames_duration_zero_fps_zero_uses_83ms() {
        let mut atlas = RgbaImage::new(128, 64);
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }

        let sheet_meta_json = serde_json::json!({
            "atlas_width": 128,
            "atlas_height": 64,
            "padding": 0,
            "frames": [
                {
                    "id": "frame_a",
                    "u_min": 0.0,
                    "v_min": 0.0,
                    "u_max": 0.5,
                    "v_max": 1.0,
                    "width": 64,
                    "height": 64,
                    "pivot": [0.5, 0.5]
                }
            ]
        });

        let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
            serde_json::from_value(sheet_meta_json).unwrap();

        let anim_meta_json = serde_json::json!({
            "name": "test_anim",
            "fps": 0,
            "loop_mode": "loop",
            "total_duration_ms": 0,
            "frames": [
                { "frame_id": "frame_a", "duration_ms": 0 }
            ]
        });

        let anim_meta: speccade_spec::recipe::sprite::SpriteAnimationMetadata =
            serde_json::from_value(anim_meta_json).unwrap();

        let (_frames, delays, _do_loop) =
            extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, None);

        assert_eq!(delays, vec![83]);
    }

    #[test]
    fn test_extract_sprite_animation_frames_skips_out_of_bounds() {
        let atlas = RgbaImage::new(16, 16);

        let sheet_meta_json = serde_json::json!({
            "atlas_width": 16,
            "atlas_height": 16,
            "padding": 0,
            "frames": [
                {
                    "id": "frame_oob",
                    "u_min": 0.0,
                    "v_min": 0.0,
                    "u_max": 2.0,
                    "v_max": 2.0,
                    "width": 32,
                    "height": 32,
                    "pivot": [0.5, 0.5]
                }
            ]
        });

        let sheet_meta: speccade_spec::recipe::sprite::SpriteSheetMetadata =
            serde_json::from_value(sheet_meta_json).unwrap();

        let anim_meta_json = serde_json::json!({
            "name": "test_anim",
            "fps": 12,
            "loop_mode": "once",
            "total_duration_ms": 0,
            "frames": [
                { "frame_id": "frame_oob", "duration_ms": 83 }
            ]
        });

        let anim_meta: speccade_spec::recipe::sprite::SpriteAnimationMetadata =
            serde_json::from_value(anim_meta_json).unwrap();

        let (frames, delays, do_loop) =
            extract_sprite_animation_frames(&atlas, &sheet_meta, &anim_meta, None);

        assert!(frames.is_empty());
        assert!(delays.is_empty());
        assert!(!do_loop);
    }

    #[test]
    fn test_extract_flipbook_frames_fps_override_zero_uses_83ms() {
        let mut atlas = RgbaImage::new(128, 64);
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([255, 0, 0, 255]));
            }
        }
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 0, 255, 255]));
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

        let metadata: speccade_spec::recipe::vfx::VfxFlipbookMetadata =
            serde_json::from_value(metadata_json).unwrap();

        let (_frames, delays, _do_loop) = extract_flipbook_frames(&atlas, &metadata, Some(0));
        assert_eq!(delays, vec![83, 83]);
    }

    #[test]
    fn test_extract_mesh_to_sprite_frames() {
        // Create a 128x64 atlas with 2 frames of 64x64 (side by side, no padding)
        let mut atlas = RgbaImage::new(128, 64);

        // Frame 0: [100,0,0,255]
        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([100, 0, 0, 255]));
            }
        }

        // Frame 1: [0,100,0,255]
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 100, 0, 255]));
            }
        }

        let metadata_json = serde_json::json!({
            "atlas_dimensions": [128, 64],
            "padding": 0,
            "frame_resolution": [64, 64],
            "camera": "ortho",
            "lighting": "default",
            "frames": [
                {
                    "id": "angle_0",
                    "angle": 0.0,
                    "position": [0, 0],
                    "dimensions": [64, 64],
                    "uv": [0.0, 0.0, 0.5, 1.0]
                },
                {
                    "id": "angle_90",
                    "angle": 90.0,
                    "position": [64, 0],
                    "dimensions": [64, 64],
                    "uv": [0.5, 0.0, 1.0, 1.0]
                }
            ]
        });

        let metadata: speccade_spec::recipe::sprite::SpriteRenderFromMeshMetadata =
            serde_json::from_value(metadata_json).unwrap();

        let (frames, delays, do_loop) = extract_mesh_sprite_frames(&atlas, &metadata, Some(10));
        assert_eq!(frames.len(), 2);
        assert_eq!(delays, vec![100, 100]);
        assert!(do_loop);
        assert_eq!(frames[0].len(), 64 * 64 * 4);

        // Pixel correctness: first pixel in each extracted frame.
        assert_eq!(&frames[0][0..4], &[100, 0, 0, 255]);
        assert_eq!(&frames[1][0..4], &[0, 100, 0, 255]);
    }

    #[test]
    fn test_extract_mesh_to_sprite_frames_skips_out_of_bounds() {
        let atlas = RgbaImage::new(16, 16);

        let metadata_json = serde_json::json!({
            "atlas_dimensions": [16, 16],
            "padding": 0,
            "frame_resolution": [32, 32],
            "camera": "ortho",
            "lighting": "default",
            "frames": [
                {
                    "id": "angle_0",
                    "angle": 0.0,
                    "position": [0, 0],
                    "dimensions": [32, 32],
                    "uv": [0.0, 0.0, 1.0, 1.0]
                }
            ]
        });

        let metadata: speccade_spec::recipe::sprite::SpriteRenderFromMeshMetadata =
            serde_json::from_value(metadata_json).unwrap();

        let (frames, delays, do_loop) = extract_mesh_sprite_frames(&atlas, &metadata, None);
        assert!(frames.is_empty());
        assert!(delays.is_empty());
        assert!(do_loop);
    }

    #[test]
    fn test_extract_mesh_to_sprite_frames_fps_override_zero_uses_83ms() {
        let mut atlas = RgbaImage::new(128, 64);

        for y in 0..64 {
            for x in 0..64 {
                atlas.put_pixel(x, y, Rgba([100, 0, 0, 255]));
            }
        }
        for y in 0..64 {
            for x in 64..128 {
                atlas.put_pixel(x, y, Rgba([0, 100, 0, 255]));
            }
        }

        let metadata_json = serde_json::json!({
            "atlas_dimensions": [128, 64],
            "padding": 0,
            "frame_resolution": [64, 64],
            "camera": "ortho",
            "lighting": "default",
            "frames": [
                {
                    "id": "angle_0",
                    "angle": 0.0,
                    "position": [0, 0],
                    "dimensions": [64, 64],
                    "uv": [0.0, 0.0, 0.5, 1.0]
                },
                {
                    "id": "angle_90",
                    "angle": 90.0,
                    "position": [64, 0],
                    "dimensions": [64, 64],
                    "uv": [0.5, 0.0, 1.0, 1.0]
                }
            ]
        });

        let metadata: speccade_spec::recipe::sprite::SpriteRenderFromMeshMetadata =
            serde_json::from_value(metadata_json).unwrap();

        let (_frames, delays, _do_loop) = extract_mesh_sprite_frames(&atlas, &metadata, Some(0));
        assert_eq!(delays, vec![83, 83]);
    }
}
