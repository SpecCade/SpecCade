//! Flipbook/VFX recipe types for `vfx.flipbook_v1`.
//!
//! Defines parameters for generating flipbook-style VFX animations (explosions,
//! smoke, particles, etc.) with deterministic frame generation and atlas packing.

use serde::{Deserialize, Serialize};

/// Parameters for the `vfx.flipbook_v1` recipe.
///
/// Generates a flipbook animation sequence for visual effects. Frames are
/// generated procedurally and packed into an atlas with deterministic packing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VfxFlipbookV1Params {
    /// Atlas resolution [width, height] in pixels.
    pub resolution: [u32; 2],

    /// Padding/gutter in pixels between frames (for mip-safe borders).
    #[serde(default = "default_padding")]
    pub padding: u32,

    /// Effect type to generate.
    pub effect: FlipbookEffectType,

    /// Number of frames in the animation sequence.
    #[serde(default = "default_frame_count")]
    pub frame_count: u32,

    /// Frame dimensions [width, height] in pixels.
    pub frame_size: [u32; 2],

    /// Animation playback speed in frames per second.
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Playback loop mode.
    #[serde(default)]
    pub loop_mode: FlipbookLoopMode,
}

fn default_padding() -> u32 {
    2
}

fn default_frame_count() -> u32 {
    16
}

fn default_fps() -> u32 {
    24
}

/// Effect type for flipbook generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlipbookEffectType {
    /// Expanding explosion effect with radial gradient and noise.
    Explosion,

    /// Rising smoke/steam with turbulent noise.
    Smoke,

    /// Expanding energy/magic circle effect.
    Energy,

    /// Dissolving/fading particles.
    Dissolve,
}

/// Playback loop mode for flipbook animations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FlipbookLoopMode {
    /// Play once and stop on last frame.
    #[default]
    Once,

    /// Repeat from start after last frame.
    Loop,

    /// Alternate forward/backward (ping-pong).
    PingPong,
}

/// UV rectangle for a packed flipbook frame in normalized [0, 1] coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FlipbookFrameUv {
    /// Frame index in the animation sequence.
    pub index: u32,

    /// Left edge U coordinate (0-1).
    pub u_min: f64,

    /// Top edge V coordinate (0-1).
    pub v_min: f64,

    /// Right edge U coordinate (0-1).
    pub u_max: f64,

    /// Bottom edge V coordinate (0-1).
    pub v_max: f64,

    /// Frame width in pixels.
    pub width: u32,

    /// Frame height in pixels.
    pub height: u32,
}

/// Metadata output for a packed flipbook atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VfxFlipbookMetadata {
    /// Atlas width in pixels.
    pub atlas_width: u32,

    /// Atlas height in pixels.
    pub atlas_height: u32,

    /// Padding/gutter in pixels.
    pub padding: u32,

    /// Effect type.
    pub effect: FlipbookEffectType,

    /// Total number of frames.
    pub frame_count: u32,

    /// Frame dimensions [width, height].
    pub frame_size: [u32; 2],

    /// Frames per second.
    pub fps: u32,

    /// Playback loop mode.
    pub loop_mode: FlipbookLoopMode,

    /// Total animation duration in milliseconds.
    pub total_duration_ms: u32,

    /// UV rectangles for each packed frame (in sequence order).
    pub frames: Vec<FlipbookFrameUv>,
}

impl VfxFlipbookV1Params {
    /// Creates a new flipbook params with the given resolution and effect.
    pub fn new(width: u32, height: u32, effect: FlipbookEffectType) -> Self {
        Self {
            resolution: [width, height],
            padding: default_padding(),
            effect,
            frame_count: default_frame_count(),
            frame_size: [64, 64],
            fps: default_fps(),
            loop_mode: FlipbookLoopMode::default(),
        }
    }

    /// Sets the padding/gutter between frames.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.padding = padding;
        self
    }

    /// Sets the number of frames in the animation.
    pub fn with_frame_count(mut self, frame_count: u32) -> Self {
        self.frame_count = frame_count;
        self
    }

    /// Sets the frame dimensions.
    pub fn with_frame_size(mut self, width: u32, height: u32) -> Self {
        self.frame_size = [width, height];
        self
    }

    /// Sets the frames per second.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Sets the loop mode.
    pub fn with_loop_mode(mut self, loop_mode: FlipbookLoopMode) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    /// Computes the total animation duration in milliseconds.
    pub fn compute_total_duration_ms(&self) -> u32 {
        if self.fps > 0 {
            (self.frame_count * 1000) / self.fps
        } else {
            0
        }
    }

    /// Creates metadata from these parameters with computed UV coordinates.
    pub fn to_metadata(&self, frames: Vec<FlipbookFrameUv>) -> VfxFlipbookMetadata {
        VfxFlipbookMetadata {
            atlas_width: self.resolution[0],
            atlas_height: self.resolution[1],
            padding: self.padding,
            effect: self.effect,
            frame_count: self.frame_count,
            frame_size: self.frame_size,
            fps: self.fps,
            loop_mode: self.loop_mode,
            total_duration_ms: self.compute_total_duration_ms(),
            frames,
        }
    }
}

impl std::fmt::Display for FlipbookEffectType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlipbookEffectType::Explosion => write!(f, "explosion"),
            FlipbookEffectType::Smoke => write!(f, "smoke"),
            FlipbookEffectType::Energy => write!(f, "energy"),
            FlipbookEffectType::Dissolve => write!(f, "dissolve"),
        }
    }
}

impl std::fmt::Display for FlipbookLoopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FlipbookLoopMode::Once => write!(f, "once"),
            FlipbookLoopMode::Loop => write!(f, "loop"),
            FlipbookLoopMode::PingPong => write!(f, "ping_pong"),
        }
    }
}
