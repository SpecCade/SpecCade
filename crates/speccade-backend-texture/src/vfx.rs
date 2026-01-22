//! VFX flipbook generation with deterministic frame synthesis and atlas packing.
//!
//! This module implements a deterministic flipbook generator for visual effects
//! (explosions, smoke, energy, particles). Frames are generated procedurally using
//! noise and gradient techniques, then packed into an atlas with deterministic
//! shelf packing.

use speccade_spec::recipe::vfx::{
    FlipbookEffectType, FlipbookFrameUv, VfxFlipbookMetadata, VfxFlipbookV1Params,
};
use thiserror::Error;

use crate::color::Color;
use crate::maps::TextureBuffer;
use crate::noise::{Fbm, Noise2D, SimplexNoise};
use crate::png::{write_rgba_to_vec_with_hash, PngConfig};

/// Errors that can occur during VFX flipbook generation.
#[derive(Debug, Error)]
pub enum VfxFlipbookError {
    /// Frame is too large to fit in the atlas.
    #[error("Frame {0} ({1}x{2}) with padding is too large for atlas ({3}x{4})")]
    FrameTooLarge(u32, u32, u32, u32, u32),

    /// Not enough space to pack all frames.
    #[error(
        "Cannot fit all frames into atlas. Consider increasing resolution or reducing frame sizes"
    )]
    PackingFailed,

    /// PNG encoding error.
    #[error("PNG encoding error: {0}")]
    PngError(#[from] crate::png::PngError),

    /// Invalid parameters.
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

/// Result of VFX flipbook generation.
#[derive(Debug)]
pub struct VfxFlipbookResult {
    /// PNG-encoded atlas image data.
    pub png_data: Vec<u8>,
    /// BLAKE3 hash of the PNG data.
    pub hash: String,
    /// Metadata with UV coordinates and animation parameters.
    pub metadata: VfxFlipbookMetadata,
}

/// Internal representation of a packed frame placement.
#[derive(Debug, Clone)]
struct PackedFrame {
    /// Frame index.
    index: u32,
    /// X position in atlas (excluding padding).
    x: u32,
    /// Y position in atlas (excluding padding).
    y: u32,
    /// Frame width (without padding).
    width: u32,
    /// Frame height (without padding).
    height: u32,
}

/// Shelf for shelf packing algorithm.
#[derive(Debug)]
struct Shelf {
    /// Y position of this shelf.
    y: u32,
    /// Height of this shelf (tallest frame + padding).
    height: u32,
    /// Current X position (next free spot).
    current_x: u32,
}

/// Generate a VFX flipbook atlas from parameters.
///
/// # Arguments
/// * `params` - Flipbook parameters including effect type, frame count, and dimensions
/// * `seed` - Deterministic seed for procedural frame generation
///
/// # Returns
/// A `VfxFlipbookResult` containing the PNG data, hash, and frame metadata.
pub fn generate_vfx_flipbook(
    params: &VfxFlipbookV1Params,
    seed: u32,
) -> Result<VfxFlipbookResult, VfxFlipbookError> {
    let width = params.resolution[0];
    let height = params.resolution[1];
    let padding = params.padding;

    // Validate parameters
    if width == 0 || height == 0 {
        return Err(VfxFlipbookError::InvalidParameter(
            "Atlas resolution must be non-zero".to_string(),
        ));
    }

    if params.frame_count == 0 {
        return Err(VfxFlipbookError::InvalidParameter(
            "Frame count must be non-zero".to_string(),
        ));
    }

    if params.frame_size[0] == 0 || params.frame_size[1] == 0 {
        return Err(VfxFlipbookError::InvalidParameter(
            "Frame size must be non-zero".to_string(),
        ));
    }

    // Validate frame fits in atlas
    let padded_width = params.frame_size[0] + padding * 2;
    let padded_height = params.frame_size[1] + padding * 2;
    if padded_width > width || padded_height > height {
        return Err(VfxFlipbookError::FrameTooLarge(
            0,
            params.frame_size[0],
            params.frame_size[1],
            width,
            height,
        ));
    }

    // Pack frames using shelf algorithm
    let packed = pack_frames_shelf(
        params.frame_count,
        params.frame_size,
        width,
        height,
        padding,
    )?;

    // Create atlas buffer
    let mut atlas = TextureBuffer::new(width, height, Color::rgba(0.0, 0.0, 0.0, 0.0));

    // Render each frame into the atlas
    for placement in &packed {
        render_effect_frame(
            &mut atlas,
            params.effect,
            placement,
            params.frame_count,
            padding,
            seed,
        );
    }

    // Encode to PNG
    let config = PngConfig::default();
    let (png_data, hash) = write_rgba_to_vec_with_hash(&atlas, &config)?;

    // Build metadata
    let frame_uvs: Vec<FlipbookFrameUv> = packed
        .iter()
        .map(|p| FlipbookFrameUv {
            index: p.index,
            u_min: p.x as f64 / width as f64,
            v_min: p.y as f64 / height as f64,
            u_max: (p.x + p.width) as f64 / width as f64,
            v_max: (p.y + p.height) as f64 / height as f64,
            width: p.width,
            height: p.height,
        })
        .collect();

    let metadata = params.to_metadata(frame_uvs);

    Ok(VfxFlipbookResult {
        png_data,
        hash,
        metadata,
    })
}

/// Pack frames using a deterministic shelf packing algorithm.
///
/// All frames are the same size for flipbook animations, so packing is
/// straightforward and deterministic.
fn pack_frames_shelf(
    frame_count: u32,
    frame_size: [u32; 2],
    atlas_width: u32,
    atlas_height: u32,
    padding: u32,
) -> Result<Vec<PackedFrame>, VfxFlipbookError> {
    let mut placements = Vec::with_capacity(frame_count as usize);
    let mut shelves: Vec<Shelf> = Vec::new();

    let padded_width = frame_size[0] + padding * 2;
    let padded_height = frame_size[1] + padding * 2;

    for index in 0..frame_count {
        // Try to find a shelf that can accommodate this frame
        let mut placed = false;
        for shelf in &mut shelves {
            if shelf.current_x + padded_width <= atlas_width {
                // Place on this shelf
                placements.push(PackedFrame {
                    index,
                    x: shelf.current_x + padding,
                    y: shelf.y + padding,
                    width: frame_size[0],
                    height: frame_size[1],
                });
                shelf.current_x += padded_width;
                placed = true;
                break;
            }
        }

        if !placed {
            // Create a new shelf
            let shelf_y = shelves.last().map_or(0, |s| s.y + s.height);

            if shelf_y + padded_height > atlas_height {
                return Err(VfxFlipbookError::PackingFailed);
            }

            let new_shelf = Shelf {
                y: shelf_y,
                height: padded_height,
                current_x: padded_width,
            };

            placements.push(PackedFrame {
                index,
                x: padding,
                y: shelf_y + padding,
                width: frame_size[0],
                height: frame_size[1],
            });

            shelves.push(new_shelf);
        }
    }

    Ok(placements)
}

/// Fill mip-safe gutters around a frame by replicating edge pixels.
///
/// Gutters prevent texture filtering artifacts at frame boundaries by duplicating
/// the edge pixels into the padding area (edges and corners).
fn fill_mip_gutters(
    atlas: &mut TextureBuffer,
    frame: &TextureBuffer,
    placement: &PackedFrame,
    padding: u32,
) {
    // Fill edge gutters (top, bottom, left, right)
    for gy in 0..padding {
        for x in 0..frame.width {
            // Top gutter
            let edge_color = frame.get(x, 0);
            let gutter_x = placement.x + x;
            let gutter_y = placement.y.saturating_sub(padding) + gy;
            if gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, edge_color);
            }

            // Bottom gutter
            let edge_color = frame.get(x, frame.height - 1);
            let gutter_y = placement.y + frame.height + gy;
            if gutter_y < atlas.height {
                atlas.set(gutter_x, gutter_y, edge_color);
            }
        }
    }

    for y in 0..frame.height {
        for gx in 0..padding {
            // Left gutter
            let edge_color = frame.get(0, y);
            let gutter_x = placement.x.saturating_sub(padding) + gx;
            let gutter_y = placement.y + y;
            if gutter_x < atlas.width {
                atlas.set(gutter_x, gutter_y, edge_color);
            }

            // Right gutter
            let edge_color = frame.get(frame.width - 1, y);
            let gutter_x = placement.x + frame.width + gx;
            if gutter_x < atlas.width {
                atlas.set(gutter_x, gutter_y, edge_color);
            }
        }
    }

    // Fill corner gutters
    let corners = [
        (frame.get(0, 0), 0, 0),                           // top-left
        (frame.get(frame.width - 1, 0), frame.width, 0),   // top-right
        (frame.get(0, frame.height - 1), 0, frame.height), // bottom-left
        (
            frame.get(frame.width - 1, frame.height - 1),
            frame.width,
            frame.height,
        ), // bottom-right
    ];

    for (color, x_offset, y_offset) in corners {
        for gy in 0..padding {
            for gx in 0..padding {
                let gutter_x = placement.x.saturating_sub(padding) + gx + x_offset;
                let gutter_y = placement.y.saturating_sub(padding) + gy + y_offset;
                if gutter_x < atlas.width && gutter_y < atlas.height {
                    atlas.set(gutter_x, gutter_y, color);
                }
            }
        }
    }
}

/// Render an effect frame into the atlas with mip-safe gutter.
///
/// Generates procedural content based on the effect type and frame index.
fn render_effect_frame(
    atlas: &mut TextureBuffer,
    effect: FlipbookEffectType,
    placement: &PackedFrame,
    total_frames: u32,
    padding: u32,
    seed: u32,
) {
    // Create frame buffer
    let mut frame = TextureBuffer::new(
        placement.width,
        placement.height,
        Color::rgba(0.0, 0.0, 0.0, 0.0),
    );

    // Compute normalized time [0, 1] for this frame
    let t = if total_frames > 1 {
        placement.index as f64 / (total_frames - 1) as f64
    } else {
        0.0
    };

    // Generate effect based on type
    match effect {
        FlipbookEffectType::Explosion => render_explosion(&mut frame, t, seed),
        FlipbookEffectType::Smoke => render_smoke(&mut frame, t, seed),
        FlipbookEffectType::Energy => render_energy(&mut frame, t, seed),
        FlipbookEffectType::Dissolve => render_dissolve(&mut frame, t, seed),
    }

    // Blit frame into atlas (content area)
    for y in 0..frame.height {
        for x in 0..frame.width {
            let color = frame.get(x, y);
            atlas.set(placement.x + x, placement.y + y, color);
        }
    }

    // Fill gutters by replicating edge pixels for mip-safe filtering
    if padding > 0 {
        fill_mip_gutters(atlas, &frame, placement, padding);
    }
}

/// Render an explosion effect frame.
///
/// Creates an expanding radial gradient with noise-based distortion.
fn render_explosion(frame: &mut TextureBuffer, t: f64, seed: u32) {
    let center_x = frame.width as f64 / 2.0;
    let center_y = frame.height as f64 / 2.0;
    let max_radius = (center_x * center_x + center_y * center_y).sqrt();

    // Create noise for distortion
    let noise = SimplexNoise::new(seed);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();

            // Add noise-based distortion
            let noise_val = noise.sample(x as f64 * 0.05 + t * 10.0, y as f64 * 0.05 + t * 10.0);
            let distorted_dist = dist + noise_val * 8.0;

            // Expanding radius based on time
            let expand_radius = max_radius * t * 1.5;

            // Radial gradient with falloff
            let radial_intensity = if distorted_dist < expand_radius {
                1.0 - (distorted_dist / expand_radius).min(1.0)
            } else {
                0.0
            };

            // Color gradient: yellow -> orange -> red -> black
            let r = (radial_intensity * 1.2).min(1.0);
            let g = (radial_intensity * 0.7).min(1.0);
            let b = (radial_intensity * 0.2).min(1.0);
            let a = (radial_intensity * (1.0 - t * 0.3)).min(1.0);

            frame.set(x, y, Color::rgba(r, g, b, a));
        }
    }
}

/// Render a smoke effect frame.
///
/// Creates rising turbulent smoke with noise-based patterns.
fn render_smoke(frame: &mut TextureBuffer, t: f64, seed: u32) {
    let fbm = Fbm::new(SimplexNoise::new(seed))
        .with_octaves(4)
        .with_persistence(0.5)
        .with_lacunarity(2.0);

    for y in 0..frame.height {
        for x in 0..frame.width {
            // Normalized coordinates
            let nx = x as f64 / frame.width as f64;
            let ny = y as f64 / frame.height as f64;

            // Rising motion (smoke moves upward over time)
            let rise_offset = t * 1.5;
            let sample_y = ny * 2.0 + rise_offset;

            // Turbulent noise pattern
            let noise_val = fbm.sample(nx * 4.0, sample_y * 4.0);

            // Vertical gradient (denser at bottom)
            let vertical_falloff = 1.0 - ny;

            // Combine noise and vertical falloff
            let intensity = (noise_val * 0.5 + 0.5) * vertical_falloff * (1.0 - t * 0.5);

            // Grayscale smoke with alpha
            let v = (intensity * 0.8).min(1.0);
            let a = (intensity * (1.0 - t * 0.6)).min(1.0);

            frame.set(x, y, Color::rgba(v, v, v, a));
        }
    }
}

/// Render an energy effect frame.
///
/// Creates an expanding magic circle/energy ring with animated patterns.
fn render_energy(frame: &mut TextureBuffer, t: f64, seed: u32) {
    let center_x = frame.width as f64 / 2.0;
    let center_y = frame.height as f64 / 2.0;
    let max_radius = (center_x * center_x + center_y * center_y).sqrt();

    let noise = SimplexNoise::new(seed);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let dx = x as f64 - center_x;
            let dy = y as f64 - center_y;
            let dist = (dx * dx + dy * dy).sqrt();
            let angle = dy.atan2(dx);

            // Expanding ring
            let ring_radius = max_radius * t * 0.8;
            let ring_width = max_radius * 0.2;

            // Ring intensity (bright at ring edge, fades out)
            let ring_diff = (dist - ring_radius).abs();
            let ring_intensity = if ring_diff < ring_width {
                1.0 - (ring_diff / ring_width)
            } else {
                0.0
            };

            // Add rotating pattern noise
            let pattern_noise = noise.sample(angle * 8.0 + t * 20.0, dist * 0.1);

            let final_intensity = ring_intensity * (pattern_noise * 0.3 + 0.7);

            // Cyan/blue energy color
            let r = (final_intensity * 0.3).min(1.0);
            let g = (final_intensity * 0.8).min(1.0);
            let b = final_intensity.min(1.0);
            let a = (final_intensity * (1.0 - t * 0.4)).min(1.0);

            frame.set(x, y, Color::rgba(r, g, b, a));
        }
    }
}

/// Render a dissolve effect frame.
///
/// Creates a particle dissolve/fade-out effect with noise-based patterns.
fn render_dissolve(frame: &mut TextureBuffer, t: f64, seed: u32) {
    let noise = SimplexNoise::new(seed);

    for y in 0..frame.height {
        for x in 0..frame.width {
            let nx = x as f64 / frame.width as f64;
            let ny = y as f64 / frame.height as f64;

            // Multi-scale noise for particle pattern
            let noise_val = noise.sample(nx * 10.0, ny * 10.0) * 0.5
                + noise.sample(nx * 20.0, ny * 20.0) * 0.3
                + noise.sample(nx * 40.0, ny * 40.0) * 0.2;

            // Threshold based on time (dissolve progresses over time)
            let dissolve_threshold = t;
            let particle_intensity = if noise_val > dissolve_threshold - 0.3 {
                ((noise_val - (dissolve_threshold - 0.3)) / 0.3).min(1.0)
            } else {
                0.0
            };

            // White particles that fade out
            let v = particle_intensity;
            let a = particle_intensity * (1.0 - t);

            frame.set(x, y, Color::rgba(v, v, v, a));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_flipbook_rejected() {
        let params =
            VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Explosion).with_frame_count(0);

        let err = generate_vfx_flipbook(&params, 42).unwrap_err();
        assert!(matches!(err, VfxFlipbookError::InvalidParameter(_)));
    }

    #[test]
    fn test_single_frame_explosion() {
        let params = VfxFlipbookV1Params::new(256, 256, FlipbookEffectType::Explosion)
            .with_frame_count(1)
            .with_frame_size(64, 64);

        let result = generate_vfx_flipbook(&params, 42).unwrap();
        assert_eq!(result.metadata.frame_count, 1);
        assert_eq!(result.metadata.frames.len(), 1);
        assert!(!result.png_data.is_empty());
    }

    #[test]
    fn test_multiple_frames_smoke() {
        let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Smoke)
            .with_frame_count(16)
            .with_frame_size(64, 64);

        let result = generate_vfx_flipbook(&params, 42).unwrap();
        assert_eq!(result.metadata.frame_count, 16);
        assert_eq!(result.metadata.frames.len(), 16);

        // Verify frames are in sequence order
        for (i, frame) in result.metadata.frames.iter().enumerate() {
            assert_eq!(frame.index, i as u32);
        }
    }

    #[test]
    fn test_determinism() {
        let params = VfxFlipbookV1Params::new(256, 256, FlipbookEffectType::Energy)
            .with_frame_count(8)
            .with_frame_size(48, 48);

        let result1 = generate_vfx_flipbook(&params, 42).unwrap();
        let result2 = generate_vfx_flipbook(&params, 42).unwrap();

        // PNG data should be byte-identical
        assert_eq!(result1.png_data, result2.png_data);
        assert_eq!(result1.hash, result2.hash);

        // Metadata should be identical
        assert_eq!(result1.metadata.frames.len(), result2.metadata.frames.len());
        for (f1, f2) in result1
            .metadata
            .frames
            .iter()
            .zip(result2.metadata.frames.iter())
        {
            assert_eq!(f1.index, f2.index);
            assert!((f1.u_min - f2.u_min).abs() < 1e-10);
            assert!((f1.v_min - f2.v_min).abs() < 1e-10);
        }
    }

    #[test]
    fn test_frame_too_large() {
        let params = VfxFlipbookV1Params::new(64, 64, FlipbookEffectType::Dissolve)
            .with_frame_count(4)
            .with_frame_size(128, 128);

        let err = generate_vfx_flipbook(&params, 42).unwrap_err();
        assert!(matches!(err, VfxFlipbookError::FrameTooLarge(..)));
    }

    #[test]
    fn test_packing_failed() {
        // Try to pack too many frames in a small atlas
        let params = VfxFlipbookV1Params::new(128, 128, FlipbookEffectType::Explosion)
            .with_frame_count(100)
            .with_frame_size(32, 32)
            .with_padding(0);

        let err = generate_vfx_flipbook(&params, 42).unwrap_err();
        assert!(matches!(err, VfxFlipbookError::PackingFailed));
    }

    #[test]
    fn test_all_effect_types() {
        let effects = vec![
            FlipbookEffectType::Explosion,
            FlipbookEffectType::Smoke,
            FlipbookEffectType::Energy,
            FlipbookEffectType::Dissolve,
        ];

        for effect in effects {
            let params = VfxFlipbookV1Params::new(256, 256, effect)
                .with_frame_count(4)
                .with_frame_size(48, 48);

            let result = generate_vfx_flipbook(&params, 42);
            assert!(result.is_ok(), "Effect {:?} failed", effect);
        }
    }

    #[test]
    fn test_metadata_duration() {
        let params = VfxFlipbookV1Params::new(512, 512, FlipbookEffectType::Energy)
            .with_frame_count(24)
            .with_frame_size(48, 48)
            .with_fps(24);

        let result = generate_vfx_flipbook(&params, 42).unwrap();
        assert_eq!(result.metadata.total_duration_ms, 1000); // 24 frames at 24 fps = 1 second
    }

    #[test]
    fn test_uv_coordinates_normalized() {
        let params = VfxFlipbookV1Params::new(256, 256, FlipbookEffectType::Smoke)
            .with_frame_count(1)
            .with_frame_size(128, 128)
            .with_padding(0);

        let result = generate_vfx_flipbook(&params, 42).unwrap();
        let uv = &result.metadata.frames[0];

        // UVs should be in [0, 1] range
        assert!(uv.u_min >= 0.0 && uv.u_min <= 1.0);
        assert!(uv.v_min >= 0.0 && uv.v_min <= 1.0);
        assert!(uv.u_max >= 0.0 && uv.u_max <= 1.0);
        assert!(uv.v_max >= 0.0 && uv.v_max <= 1.0);

        // For 128x128 frame in 256x256 atlas, UV range should be 0.5
        assert!((uv.u_max - uv.u_min - 0.5).abs() < 1e-10);
        assert!((uv.v_max - uv.v_min - 0.5).abs() < 1e-10);
    }
}
