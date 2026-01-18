//! Tests for procedural texture graph generation.

mod test_blend;
mod test_color;
mod test_filter;
mod test_math;
mod test_primitive;
mod test_uv;

use crate::color::Color;
use speccade_spec::recipe::texture::{TextureProceduralNode, TextureProceduralV1Params};

use super::{encode_graph_value_png, generate_graph};

fn make_params(tileable: bool, nodes: Vec<TextureProceduralNode>) -> TextureProceduralV1Params {
    TextureProceduralV1Params {
        resolution: [32, 32],
        tileable,
        nodes,
    }
}

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn color_approx_eq(a: Color, b: Color) -> bool {
    approx_eq(a.r, b.r) && approx_eq(a.g, b.g) && approx_eq(a.b, b.b) && approx_eq(a.a, b.a)
}

// Core graph tests (determinism, error handling, ordering)

use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig, TextureProceduralOp};

#[test]
fn graph_is_deterministic_for_same_seed() {
    let params = make_params(
        true,
        vec![
            TextureProceduralNode {
                id: "n".to_string(),
                op: TextureProceduralOp::Noise {
                    noise: NoiseConfig {
                        algorithm: NoiseAlgorithm::Perlin,
                        scale: 0.1,
                        octaves: 3,
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
    );

    let a = generate_graph(&params, 42).unwrap();
    let b = generate_graph(&params, 42).unwrap();

    let mask_a = a.get("mask").unwrap();
    let mask_b = b.get("mask").unwrap();

    let (bytes_a, hash_a) = encode_graph_value_png(mask_a).unwrap();
    let (bytes_b, hash_b) = encode_graph_value_png(mask_b).unwrap();
    assert_eq!(hash_a, hash_b);
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn unknown_node_reference_is_error() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "bad".to_string(),
            op: TextureProceduralOp::Invert {
                input: "missing".to_string(),
            },
        }],
    );

    let err = generate_graph(&params, 1).unwrap_err();
    assert!(
        err.to_string().contains("unknown node id")
            || err.to_string().contains("unknown node reference")
    );
}

#[test]
fn cycle_is_error() {
    let params = make_params(
        true,
        vec![
            TextureProceduralNode {
                id: "a".to_string(),
                op: TextureProceduralOp::Invert {
                    input: "b".to_string(),
                },
            },
            TextureProceduralNode {
                id: "b".to_string(),
                op: TextureProceduralOp::Invert {
                    input: "a".to_string(),
                },
            },
        ],
    );

    let err = generate_graph(&params, 1).unwrap_err();
    assert!(err.to_string().contains("cycle detected"));
}

#[test]
fn obvious_type_mismatch_is_error() {
    let params = make_params(
        true,
        vec![
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
                id: "bad".to_string(),
                op: TextureProceduralOp::Palette {
                    input: "n".to_string(),
                    palette: vec!["#000000".to_string(), "#ffffff".to_string()],
                },
            },
        ],
    );

    let err = generate_graph(&params, 1).unwrap_err();
    assert!(
        err.to_string().contains("color output") || err.to_string().contains("color was required")
    );
}

#[test]
fn reorder_does_not_change_output() {
    let n = TextureProceduralNode {
        id: "n".to_string(),
        op: TextureProceduralOp::Noise {
            noise: NoiseConfig {
                algorithm: NoiseAlgorithm::Perlin,
                scale: 0.1,
                octaves: 3,
                persistence: 0.5,
                lacunarity: 2.0,
            },
        },
    };
    let mask = TextureProceduralNode {
        id: "mask".to_string(),
        op: TextureProceduralOp::Threshold {
            input: "n".to_string(),
            threshold: 0.5,
        },
    };

    let params_a = make_params(true, vec![n.clone(), mask.clone()]);
    let params_b = make_params(true, vec![mask, n]);

    let a = generate_graph(&params_a, 123).unwrap();
    let b = generate_graph(&params_b, 123).unwrap();

    let (bytes_a, hash_a) = encode_graph_value_png(a.get("mask").unwrap()).unwrap();
    let (bytes_b, hash_b) = encode_graph_value_png(b.get("mask").unwrap()).unwrap();

    assert_eq!(hash_a, hash_b);
    assert_eq!(bytes_a, bytes_b);
}
