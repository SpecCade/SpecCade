//! Animation preview rendering configuration.

use serde::{Deserialize, Serialize};

// =============================================================================
// Preview Render Configuration
// =============================================================================

fn default_preview_size() -> [u32; 2] {
    [256, 256]
}

fn default_frame_step() -> u32 {
    2
}

fn default_background() -> [f32; 4] {
    [0.2, 0.2, 0.2, 1.0]
}

/// Camera angle preset for animation preview rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreviewCameraAngle {
    /// Classic 3/4 view from front-right, slightly above.
    /// Best for general character review.
    #[default]
    ThreeQuarter,
    /// Directly in front of the character.
    /// Best for face/silhouette checks.
    Front,
    /// 90 degree side view (left side).
    /// Best for walk cycle timing verification.
    Side,
    /// Behind the character.
    /// Best for checking back details.
    Back,
    /// Above looking down.
    /// Best for footwork pattern verification.
    Top,
}

impl PreviewCameraAngle {
    /// Returns the camera position offset from character center.
    /// Format: (distance, height_offset, rotation_degrees)
    pub fn camera_params(&self) -> (f64, f64, f64) {
        match self {
            PreviewCameraAngle::ThreeQuarter => (3.0, 0.5, 45.0),
            PreviewCameraAngle::Front => (3.0, 0.3, 0.0),
            PreviewCameraAngle::Side => (3.0, 0.3, 90.0),
            PreviewCameraAngle::Back => (3.0, 0.3, 180.0),
            PreviewCameraAngle::Top => (3.0, 2.5, 0.0), // High above, looking down
        }
    }
}

/// Configuration for rendering an animation preview as a GIF.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewRender {
    /// Camera angle preset.
    #[serde(default)]
    pub camera: PreviewCameraAngle,
    /// Output frame size [width, height] in pixels.
    #[serde(default = "default_preview_size")]
    pub size: [u32; 2],
    /// Render every Nth frame (1 = all frames, 2 = every other frame).
    /// Higher values produce smaller GIFs but choppier animation.
    #[serde(default = "default_frame_step")]
    pub frame_step: u32,
    /// Background color [R, G, B, A] in 0-1 range.
    #[serde(default = "default_background")]
    pub background: [f32; 4],
    /// Reference to a skeletal mesh spec to include in the preview.
    /// If provided, the mesh will be loaded and attached to the armature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mesh: Option<String>,
}

impl Default for PreviewRender {
    fn default() -> Self {
        Self {
            camera: PreviewCameraAngle::default(),
            size: default_preview_size(),
            frame_step: default_frame_step(),
            background: default_background(),
            mesh: None,
        }
    }
}

impl PreviewRender {
    /// Creates a new preview render configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the camera preset.
    pub fn with_camera(mut self, camera: PreviewCameraAngle) -> Self {
        self.camera = camera;
        self
    }

    /// Sets the output size.
    pub fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = [width, height];
        self
    }

    /// Sets the frame step.
    pub fn with_frame_step(mut self, step: u32) -> Self {
        self.frame_step = step.max(1);
        self
    }

    /// Sets the background color.
    pub fn with_background(mut self, r: f32, g: f32, b: f32, a: f32) -> Self {
        self.background = [r, g, b, a];
        self
    }

    /// Sets the mesh reference.
    pub fn with_mesh(mut self, mesh: impl Into<String>) -> Self {
        self.mesh = Some(mesh.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preview_render_default() {
        let preview = PreviewRender::default();
        assert_eq!(preview.camera, PreviewCameraAngle::ThreeQuarter);
        assert_eq!(preview.size, [256, 256]);
        assert_eq!(preview.frame_step, 2);
        assert_eq!(preview.background, [0.2, 0.2, 0.2, 1.0]);
        assert!(preview.mesh.is_none());
    }

    #[test]
    fn test_preview_render_builder() {
        let preview = PreviewRender::new()
            .with_camera(PreviewCameraAngle::Side)
            .with_size(512, 512)
            .with_frame_step(1)
            .with_background(0.1, 0.1, 0.1, 1.0)
            .with_mesh("character.glb");

        assert_eq!(preview.camera, PreviewCameraAngle::Side);
        assert_eq!(preview.size, [512, 512]);
        assert_eq!(preview.frame_step, 1);
        assert_eq!(preview.mesh, Some("character.glb".to_string()));
    }

    #[test]
    fn test_camera_preset_serialization() {
        let preset = PreviewCameraAngle::ThreeQuarter;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"three_quarter\"");

        let preset: PreviewCameraAngle = serde_json::from_str("\"side\"").unwrap();
        assert_eq!(preset, PreviewCameraAngle::Side);
    }
}
