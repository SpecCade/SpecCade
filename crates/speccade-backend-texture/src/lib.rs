//! SpecCade Texture Generation Backend
//!
//! This crate provides deterministic procedural texture generation for SpecCade.
//! All output is byte-identical given the same seed and parameters, ensuring
//! Tier 1 determinism as specified in the SpecCade determinism policy.
//!
//! # Features
//!
//! - **Noise Primitives**: Simplex, Perlin, Worley/Voronoi, and FBM
//! - **Pattern Primitives**: Brick, checkerboard, wood grain, scratches, edge wear
//! - **Procedural Graph Ops**: Named DAG nodes producing grayscale or RGBA outputs
//! - **Deterministic PNG**: Fixed compression settings for byte-identical output
//!
//! # Example
//!
//! ```no_run
//! use speccade_backend_texture::generate::{encode_graph_value_png, generate_graph};
//! use speccade_spec::recipe::texture::{
//!     NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralOp,
//!     TextureProceduralV1Params,
//! };
//!
//! let params = TextureProceduralV1Params {
//!     resolution: [256, 256],
//!     tileable: true,
//!     nodes: vec![
//!         TextureProceduralNode {
//!             id: "n".to_string(),
//!             op: TextureProceduralOp::Noise {
//!                 noise: NoiseConfig {
//!                     algorithm: NoiseAlgorithm::Perlin,
//!                     scale: 0.08,
//!                     octaves: 3,
//!                     persistence: 0.5,
//!                     lacunarity: 2.0,
//!                 },
//!             },
//!         },
//!         TextureProceduralNode {
//!             id: "mask".to_string(),
//!             op: TextureProceduralOp::Threshold {
//!                 input: "n".to_string(),
//!                 threshold: 0.55,
//!             },
//!         },
//!     ],
//! };
//!
//! let nodes = generate_graph(&params, 42).unwrap();
//! let (bytes, _hash) = encode_graph_value_png(nodes.get("mask").unwrap()).unwrap();
//! std::fs::write("output/mask.png", bytes).unwrap();
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
pub mod trimsheet;

// Re-export main types for convenience
pub use color::{BlendMode, Color};
pub use generate::{
    encode_graph_value_png, generate_graph, generate_material_maps, generate_packed_maps,
    save_texture_result, GenerateError, GraphValue, MapResult, TextureResult,
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
pub use trimsheet::{generate_trimsheet, TrimsheetError, TrimsheetResult};

#[cfg(test)]
mod tests {
    use super::*;
    use speccade_spec::recipe::texture::{
        NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralOp,
        TextureProceduralV1Params,
    };

    #[test]
    fn crate_root_reexports_generate_graph() {
        let params = TextureProceduralV1Params {
            resolution: [16, 16],
            tileable: true,
            nodes: vec![
                TextureProceduralNode {
                    id: "n".to_string(),
                    op: TextureProceduralOp::Noise {
                        noise: NoiseConfig {
                            algorithm: NoiseAlgorithm::Perlin,
                            scale: 0.1,
                            octaves: 2,
                            persistence: 0.5,
                            lacunarity: 2.0,
                        },
                    },
                },
                TextureProceduralNode {
                    id: "mask".to_string(),
                    op: TextureProceduralOp::Threshold {
                        input: "n".to_string(),
                        threshold: 0.5,
                    },
                },
            ],
        };

        let nodes = generate_graph(&params, 42).unwrap();
        let value = nodes.get("mask").unwrap();
        let (data, _hash) = encode_graph_value_png(value).unwrap();
        assert!(!data.is_empty());
    }
}
