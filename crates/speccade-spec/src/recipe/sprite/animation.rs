//! Sprite animation clip recipe types for `sprite.animation_v1`.
//!
//! Defines animation clips that reference frames from a spritesheet,
//! with timing and loop mode configuration.

use serde::{Deserialize, Serialize};

/// Parameters for the `sprite.animation_v1` recipe.
///
/// Defines an animation clip with frame references and timing.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteAnimationV1Params {
    /// Animation clip name.
    pub name: String,

    /// Default frames per second (used when frame duration is not specified).
    #[serde(default = "default_fps")]
    pub fps: u32,

    /// Playback loop mode.
    #[serde(default)]
    pub loop_mode: AnimationLoopMode,

    /// Ordered list of animation frames.
    pub frames: Vec<AnimationFrame>,
}

fn default_fps() -> u32 {
    12
}

/// Animation playback loop mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AnimationLoopMode {
    /// Repeat from start after last frame.
    #[default]
    Loop,

    /// Play once and stop on last frame.
    Once,

    /// Alternate forward/backward (ping-pong).
    PingPong,
}

/// A frame reference in an animation clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnimationFrame {
    /// Reference to frame ID in the spritesheet.
    pub frame_id: String,

    /// Frame display duration in milliseconds.
    /// If not specified, uses 1000/fps from the parent animation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}

/// Metadata output for a sprite animation clip.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteAnimationMetadata {
    /// Animation clip name.
    pub name: String,

    /// Default frames per second.
    pub fps: u32,

    /// Playback loop mode.
    pub loop_mode: AnimationLoopMode,

    /// Total animation duration in milliseconds.
    pub total_duration_ms: u32,

    /// Ordered list of animation frames with resolved durations.
    pub frames: Vec<AnimationFrameResolved>,
}

/// A frame in the animation metadata with resolved duration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnimationFrameResolved {
    /// Reference to frame ID in the spritesheet.
    pub frame_id: String,

    /// Frame display duration in milliseconds.
    pub duration_ms: u32,
}

impl SpriteAnimationV1Params {
    /// Creates a new animation clip params with the given name.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            fps: default_fps(),
            loop_mode: AnimationLoopMode::default(),
            frames: Vec::new(),
        }
    }

    /// Sets the frames per second.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = fps;
        self
    }

    /// Sets the loop mode.
    pub fn with_loop_mode(mut self, loop_mode: AnimationLoopMode) -> Self {
        self.loop_mode = loop_mode;
        self
    }

    /// Adds a frame to the animation.
    pub fn with_frame(mut self, frame: AnimationFrame) -> Self {
        self.frames.push(frame);
        self
    }

    /// Computes the total animation duration in milliseconds.
    pub fn compute_total_duration_ms(&self) -> u32 {
        let default_duration = if self.fps > 0 { 1000 / self.fps } else { 0 };
        self.frames
            .iter()
            .map(|f| f.duration_ms.unwrap_or(default_duration))
            .sum()
    }

    /// Resolves all frame durations and produces metadata.
    pub fn to_metadata(&self) -> SpriteAnimationMetadata {
        let default_duration = if self.fps > 0 { 1000 / self.fps } else { 0 };

        let frames: Vec<AnimationFrameResolved> = self
            .frames
            .iter()
            .map(|f| AnimationFrameResolved {
                frame_id: f.frame_id.clone(),
                duration_ms: f.duration_ms.unwrap_or(default_duration),
            })
            .collect();

        let total_duration_ms = frames.iter().map(|f| f.duration_ms).sum();

        SpriteAnimationMetadata {
            name: self.name.clone(),
            fps: self.fps,
            loop_mode: self.loop_mode,
            total_duration_ms,
            frames,
        }
    }
}

impl AnimationFrame {
    /// Creates a new animation frame with default duration.
    pub fn new(frame_id: impl Into<String>) -> Self {
        Self {
            frame_id: frame_id.into(),
            duration_ms: None,
        }
    }

    /// Creates a new animation frame with explicit duration.
    pub fn with_duration(frame_id: impl Into<String>, duration_ms: u32) -> Self {
        Self {
            frame_id: frame_id.into(),
            duration_ms: Some(duration_ms),
        }
    }
}

impl std::fmt::Display for AnimationLoopMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnimationLoopMode::Loop => write!(f, "loop"),
            AnimationLoopMode::Once => write!(f, "once"),
            AnimationLoopMode::PingPong => write!(f, "ping_pong"),
        }
    }
}
