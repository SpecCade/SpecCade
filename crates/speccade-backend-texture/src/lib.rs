//! SpecCade Texture Generation Backend
//!
//! This crate provides deterministic PBR material map generation for SpecCade.
//! All output is byte-identical given the same seed and parameters, ensuring
//! Tier 1 determinism as specified in the SpecCade determinism policy.
//!
//! # Features
//!
//! - **Noise Primitives**: Simplex, Perlin, Worley/Voronoi, and FBM
//! - **Pattern Primitives**: Brick, checkerboard, wood grain, scratches, edge wear
//! - **PBR Map Types**: Albedo, roughness, metallic, normal, AO, emissive, height
//! - **Deterministic PNG**: Fixed compression settings for byte-identical output
//!
//! # Example
//!
//! ```no_run
//! use speccade_backend_texture::generate::{generate_material_maps, save_texture_result};
//! use speccade_spec::recipe::texture::{
//!     Texture2dMaterialMapsV1Params, TextureMapType, BaseMaterial, MaterialType,
//! };
//! use std::path::Path;
//!
//! let params = Texture2dMaterialMapsV1Params {
//!     resolution: [256, 256],
//!     tileable: true,
//!     maps: vec![
//!         TextureMapType::Albedo,
//!         TextureMapType::Normal,
//!         TextureMapType::Roughness,
//!     ],
//!     base_material: Some(BaseMaterial {
//!         material_type: MaterialType::Metal,
//!         base_color: [0.8, 0.2, 0.1],
//!         roughness_range: Some([0.2, 0.5]),
//!         metallic: Some(1.0),
//!         brick_pattern: None,
//!         normal_params: None,
//!     }),
//!     layers: vec![],
//!     palette: None,
//!     color_ramp: None,
//! };
//!
//! let result = generate_material_maps(&params, 42).unwrap();
//! save_texture_result(&result, Path::new("output"), "my_material").unwrap();
//! ```
//!
//! # Determinism
//!
//! This backend guarantees Tier 1 determinism:
//!
//! - Same spec + same seed = byte-identical output
//! - PCG32 RNG is used for all random operations
//! - PNG encoding uses fixed compression settings
//! - Full file hash validation is supported
//!
//! See `docs/DETERMINISM.md` for the complete determinism policy.

pub mod color;
pub mod generate;
pub mod maps;
pub mod noise;
pub mod normal_map;
pub mod normal_map_patterns;
pub mod packing;
pub mod pattern;
pub mod png;
pub mod rng;
pub mod shared;

// Re-export main types for convenience
pub use color::{BlendMode, Color};
pub use generate::{
    generate_material_maps, save_texture_result, GenerateError, MapResult, TextureResult,
};
pub use maps::{GrayscaleBuffer, TextureBuffer};
pub use noise::{Fbm, Noise2D, PerlinNoise, SimplexNoise, WorleyNoise};
pub use normal_map::{generate_normal_map, save_normal_map, NormalMapError, NormalMapResult};
pub use packing::{
    extract_channel, pack_channels, resolve_channel_source, ChannelSource, ColorComponent,
    PackedChannels, PackingError,
};
pub use pattern::{
    BrickPattern, CheckerPattern, EdgeWearPattern, Pattern2D, ScratchesPattern, WoodGrainPattern,
};
pub use png::{PngConfig, PngError};
pub use rng::DeterministicRng;
