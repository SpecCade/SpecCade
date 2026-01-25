//! Render-to-sprite bridge recipe types for `sprite.render_from_mesh_v1`.
//!
//! This recipe renders a `static_mesh` from multiple angles with configurable
//! camera and lighting presets, then packs the resulting frames into a sprite
//! atlas with metadata.

use serde::{Deserialize, Serialize};

use super::super::mesh::StaticMeshBlenderPrimitivesV1Params;

/// Camera preset for mesh-to-sprite rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CameraPreset {
    /// Orthographic projection (no perspective distortion).
    #[default]
    Orthographic,
    /// Perspective projection with standard FOV.
    Perspective,
    /// Isometric camera angle (common for 2D games).
    Isometric,
}

impl CameraPreset {
    /// Returns the camera preset as a string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            CameraPreset::Orthographic => "orthographic",
            CameraPreset::Perspective => "perspective",
            CameraPreset::Isometric => "isometric",
        }
    }
}

/// Lighting preset for mesh-to-sprite rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LightingPreset {
    /// Classic three-point lighting (key, fill, back).
    #[default]
    ThreePoint,
    /// Rim lighting (highlights edges).
    Rim,
    /// Flat lighting (minimal shadows, uniform).
    Flat,
    /// Dramatic lighting (strong shadows, high contrast).
    Dramatic,
    /// Studio lighting (soft, balanced).
    Studio,
}

impl LightingPreset {
    /// Returns the lighting preset as a string identifier.
    pub fn as_str(&self) -> &'static str {
        match self {
            LightingPreset::ThreePoint => "three_point",
            LightingPreset::Rim => "rim",
            LightingPreset::Flat => "flat",
            LightingPreset::Dramatic => "dramatic",
            LightingPreset::Studio => "studio",
        }
    }
}

/// Parameters for the `sprite.render_from_mesh_v1` recipe.
///
/// Renders a 3D mesh from multiple rotation angles and packs the resulting
/// frames into a sprite atlas with metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpriteRenderFromMeshV1Params {
    /// Mesh to render (inline static_mesh params).
    pub mesh: StaticMeshBlenderPrimitivesV1Params,

    /// Camera preset for rendering.
    #[serde(default)]
    pub camera: CameraPreset,

    /// Lighting preset for rendering.
    #[serde(default)]
    pub lighting: LightingPreset,

    /// Render resolution per frame [width, height] in pixels.
    pub frame_resolution: [u32; 2],

    /// Rotation angles to render (degrees around Y axis).
    /// For example, [0, 45, 90, 135, 180, 225, 270, 315] for 8-directional.
    pub rotation_angles: Vec<f64>,

    /// Atlas padding in pixels between frames (default: 2).
    #[serde(default = "default_atlas_padding")]
    pub atlas_padding: u32,

    /// Background color for rendered frames [R, G, B, A] in 0.0-1.0 range.
    /// Default is transparent [0, 0, 0, 0].
    #[serde(default = "default_background_color")]
    pub background_color: [f64; 4],

    /// Camera distance multiplier (relative to mesh bounding box).
    /// Default is 2.0 (camera is placed 2x the mesh size away).
    #[serde(default = "default_camera_distance")]
    pub camera_distance: f64,

    /// Camera elevation angle in degrees (0 = horizontal, 90 = top-down).
    /// Default is 30 degrees for isometric-style view.
    #[serde(default = "default_camera_elevation")]
    pub camera_elevation: f64,
}

fn default_atlas_padding() -> u32 {
    2
}

fn default_background_color() -> [f64; 4] {
    [0.0, 0.0, 0.0, 0.0]
}

fn default_camera_distance() -> f64 {
    2.0
}

fn default_camera_elevation() -> f64 {
    30.0
}

/// Frame metadata for a rendered sprite in the atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteRenderFrame {
    /// Frame identifier (e.g., "angle_0", "angle_45").
    pub id: String,

    /// Rotation angle in degrees that this frame was rendered at.
    pub angle: f64,

    /// Position in atlas [x, y] in pixels.
    pub position: [u32; 2],

    /// Frame dimensions [width, height] in pixels.
    pub dimensions: [u32; 2],

    /// UV coordinates for this frame [u_min, v_min, u_max, v_max] in 0.0-1.0 range.
    pub uv: [f64; 4],
}

/// Metadata output for a rendered sprite atlas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpriteRenderFromMeshMetadata {
    /// Atlas dimensions [width, height] in pixels.
    pub atlas_dimensions: [u32; 2],

    /// Padding between frames in pixels.
    pub padding: u32,

    /// Frame resolution [width, height] in pixels.
    pub frame_resolution: [u32; 2],

    /// Camera preset used.
    pub camera: String,

    /// Lighting preset used.
    pub lighting: String,

    /// List of rendered frames with positions and UVs.
    pub frames: Vec<SpriteRenderFrame>,
}

impl SpriteRenderFromMeshV1Params {
    /// Creates a new render-from-mesh params with required fields.
    pub fn new(
        mesh: StaticMeshBlenderPrimitivesV1Params,
        frame_resolution: [u32; 2],
        rotation_angles: Vec<f64>,
    ) -> Self {
        Self {
            mesh,
            camera: CameraPreset::default(),
            lighting: LightingPreset::default(),
            frame_resolution,
            rotation_angles,
            atlas_padding: default_atlas_padding(),
            background_color: default_background_color(),
            camera_distance: default_camera_distance(),
            camera_elevation: default_camera_elevation(),
        }
    }

    /// Sets the camera preset.
    pub fn with_camera(mut self, camera: CameraPreset) -> Self {
        self.camera = camera;
        self
    }

    /// Sets the lighting preset.
    pub fn with_lighting(mut self, lighting: LightingPreset) -> Self {
        self.lighting = lighting;
        self
    }

    /// Sets the atlas padding.
    pub fn with_padding(mut self, padding: u32) -> Self {
        self.atlas_padding = padding;
        self
    }

    /// Sets the background color.
    pub fn with_background_color(mut self, color: [f64; 4]) -> Self {
        self.background_color = color;
        self
    }

    /// Sets the camera distance multiplier.
    pub fn with_camera_distance(mut self, distance: f64) -> Self {
        self.camera_distance = distance;
        self
    }

    /// Sets the camera elevation angle.
    pub fn with_camera_elevation(mut self, elevation: f64) -> Self {
        self.camera_elevation = elevation;
        self
    }

    /// Calculates the required atlas dimensions for the frames.
    pub fn calculate_atlas_dimensions(&self) -> [u32; 2] {
        let frame_count = self.rotation_angles.len() as u32;
        if frame_count == 0 {
            return [0, 0];
        }

        let frame_w = self.frame_resolution[0] + self.atlas_padding * 2;
        let frame_h = self.frame_resolution[1] + self.atlas_padding * 2;

        // Calculate grid dimensions (prefer square-ish atlas)
        let cols = (frame_count as f64).sqrt().ceil() as u32;
        let rows = frame_count.div_ceil(cols);

        [cols * frame_w, rows * frame_h]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::recipe::mesh::MeshPrimitive;

    fn create_test_mesh_params() -> StaticMeshBlenderPrimitivesV1Params {
        StaticMeshBlenderPrimitivesV1Params {
            base_primitive: MeshPrimitive::Cube,
            dimensions: [1.0, 1.0, 1.0],
            modifiers: vec![],
            uv_projection: None,
            normals: None,
            material_slots: vec![],
            export: None,
            constraints: None,
            lod_chain: None,
            collision_mesh: None,
            navmesh: None,
            baking: None,
        }
    }

    #[test]
    fn test_camera_preset_serde() {
        let preset = CameraPreset::Isometric;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"isometric\"");

        let parsed: CameraPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, preset);
    }

    #[test]
    fn test_lighting_preset_serde() {
        let preset = LightingPreset::Dramatic;
        let json = serde_json::to_string(&preset).unwrap();
        assert_eq!(json, "\"dramatic\"");

        let parsed: LightingPreset = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, preset);
    }

    #[test]
    fn test_params_basic() {
        let mesh = create_test_mesh_params();
        let params =
            SpriteRenderFromMeshV1Params::new(mesh, [64, 64], vec![0.0, 90.0, 180.0, 270.0]);

        assert_eq!(params.frame_resolution, [64, 64]);
        assert_eq!(params.rotation_angles.len(), 4);
        assert_eq!(params.camera, CameraPreset::Orthographic);
        assert_eq!(params.lighting, LightingPreset::ThreePoint);
        assert_eq!(params.atlas_padding, 2);
    }

    #[test]
    fn test_params_serde() {
        let mesh = create_test_mesh_params();
        let params = SpriteRenderFromMeshV1Params::new(
            mesh,
            [128, 128],
            vec![0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0],
        )
        .with_camera(CameraPreset::Isometric)
        .with_lighting(LightingPreset::Studio)
        .with_padding(4);

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("\"camera\":\"isometric\""));
        assert!(json.contains("\"lighting\":\"studio\""));
        assert!(json.contains("\"atlas_padding\":4"));

        let parsed: SpriteRenderFromMeshV1Params = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.camera, CameraPreset::Isometric);
        assert_eq!(parsed.lighting, LightingPreset::Studio);
        assert_eq!(parsed.atlas_padding, 4);
        assert_eq!(parsed.rotation_angles.len(), 8);
    }

    #[test]
    fn test_calculate_atlas_dimensions() {
        let mesh = create_test_mesh_params();

        // 4 frames at 64x64 with 2px padding = 2x2 grid
        // Each cell is 68x68 (64 + 2*2), so atlas is 136x136
        let params = SpriteRenderFromMeshV1Params::new(mesh.clone(), [64, 64], vec![0.0; 4]);
        assert_eq!(params.calculate_atlas_dimensions(), [136, 136]);

        // 8 frames at 64x64 with 2px padding = 3x3 grid (9 cells, 1 empty)
        // Each cell is 68x68, so atlas is 204x204
        let params = SpriteRenderFromMeshV1Params::new(mesh.clone(), [64, 64], vec![0.0; 8]);
        assert_eq!(params.calculate_atlas_dimensions(), [204, 204]);

        // 1 frame
        let params = SpriteRenderFromMeshV1Params::new(mesh.clone(), [64, 64], vec![0.0]);
        assert_eq!(params.calculate_atlas_dimensions(), [68, 68]);

        // 0 frames
        let params = SpriteRenderFromMeshV1Params::new(mesh, [64, 64], vec![]);
        assert_eq!(params.calculate_atlas_dimensions(), [0, 0]);
    }

    #[test]
    fn test_params_with_builders() {
        let mesh = create_test_mesh_params();
        let params = SpriteRenderFromMeshV1Params::new(mesh, [64, 64], vec![0.0])
            .with_camera(CameraPreset::Perspective)
            .with_lighting(LightingPreset::Rim)
            .with_padding(8)
            .with_background_color([1.0, 0.0, 0.0, 1.0])
            .with_camera_distance(3.0)
            .with_camera_elevation(45.0);

        assert_eq!(params.camera, CameraPreset::Perspective);
        assert_eq!(params.lighting, LightingPreset::Rim);
        assert_eq!(params.atlas_padding, 8);
        assert_eq!(params.background_color, [1.0, 0.0, 0.0, 1.0]);
        assert_eq!(params.camera_distance, 3.0);
        assert_eq!(params.camera_elevation, 45.0);
    }

    #[test]
    fn test_frame_metadata() {
        let frame = SpriteRenderFrame {
            id: "angle_0".to_string(),
            angle: 0.0,
            position: [0, 0],
            dimensions: [64, 64],
            uv: [0.0, 0.0, 0.5, 0.5],
        };

        let json = serde_json::to_string(&frame).unwrap();
        let parsed: SpriteRenderFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "angle_0");
        assert_eq!(parsed.angle, 0.0);
    }

    #[test]
    fn test_atlas_metadata() {
        let metadata = SpriteRenderFromMeshMetadata {
            atlas_dimensions: [256, 256],
            padding: 2,
            frame_resolution: [64, 64],
            camera: "orthographic".to_string(),
            lighting: "three_point".to_string(),
            frames: vec![SpriteRenderFrame {
                id: "angle_0".to_string(),
                angle: 0.0,
                position: [2, 2],
                dimensions: [64, 64],
                uv: [0.0078, 0.0078, 0.2578, 0.2578],
            }],
        };

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"atlas_dimensions\":[256,256]"));
        assert!(json.contains("\"frames\""));
    }
}
