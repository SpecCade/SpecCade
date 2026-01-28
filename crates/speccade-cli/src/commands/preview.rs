//! Preview command implementation
//!
//! Opens an asset preview (stub for Blender preview).

use anyhow::{Context, Result};
use colored::Colorize;
use gif::{Encoder, Frame as GifFrame, Repeat};
use speccade_spec::Spec;
use std::fs;
use std::io::BufWriter;
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
    _out_root: Option<&str>,
    _gif: bool,
    _out: Option<&str>,
    _fps: Option<u32>,
    _scale: Option<u32>,
) -> Result<ExitCode> {
    // TODO: Implement preview for generated assets (open viewers, or launch Blender for mesh/anim).
    println!("{} {}", "Preview:".cyan().bold(), spec_path);

    // Read and parse spec to get asset type
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    let spec = Spec::from_json(&spec_content)
        .with_context(|| format!("Failed to parse spec file: {}", spec_path))?;

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
    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
