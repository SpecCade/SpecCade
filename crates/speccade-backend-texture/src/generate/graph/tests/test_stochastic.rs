//! Tests for stochastic tiling operations (Wang tiles / texture bombing).

use speccade_spec::recipe::texture::TextureProceduralNode;
use speccade_spec::recipe::texture::TextureProceduralOp;

use super::{encode_graph_value_png, generate_graph, make_params};

#[test]
fn wang_tiles_is_deterministic_for_same_seed() {
    let params = make_params(
        true,
        vec![
            TextureProceduralNode {
                id: "c".to_string(),
                op: TextureProceduralOp::Checkerboard {
                    tile_size: 4,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "wang".to_string(),
                op: TextureProceduralOp::WangTiles {
                    input: "c".to_string(),
                    tile_count: [4, 4],
                    blend_width: 0.1,
                },
            },
        ],
    );

    let a = generate_graph(&params, 123).unwrap();
    let b = generate_graph(&params, 123).unwrap();

    let (bytes_a, hash_a) = encode_graph_value_png(a.get("wang").unwrap()).unwrap();
    let (bytes_b, hash_b) = encode_graph_value_png(b.get("wang").unwrap()).unwrap();

    assert_eq!(hash_a, hash_b);
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn texture_bomb_is_deterministic_for_same_seed() {
    let params = make_params(
        true,
        vec![
            TextureProceduralNode {
                id: "c".to_string(),
                op: TextureProceduralOp::Checkerboard {
                    tile_size: 4,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "bomb".to_string(),
                op: TextureProceduralOp::TextureBomb {
                    input: "c".to_string(),
                    density: 0.35,
                    scale_variation: [0.8, 1.2],
                    rotation_variation: 45.0,
                    blend_mode: "max".to_string(),
                },
            },
        ],
    );

    let a = generate_graph(&params, 999).unwrap();
    let b = generate_graph(&params, 999).unwrap();

    let (bytes_a, hash_a) = encode_graph_value_png(a.get("bomb").unwrap()).unwrap();
    let (bytes_b, hash_b) = encode_graph_value_png(b.get("bomb").unwrap()).unwrap();

    assert_eq!(hash_a, hash_b);
    assert_eq!(bytes_a, bytes_b);
}

#[test]
fn stochastic_ops_produce_unit_range_output() {
    let params = make_params(
        true,
        vec![
            TextureProceduralNode {
                id: "c".to_string(),
                op: TextureProceduralOp::Checkerboard {
                    tile_size: 4,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "wang".to_string(),
                op: TextureProceduralOp::WangTiles {
                    input: "c".to_string(),
                    tile_count: [4, 4],
                    blend_width: 0.1,
                },
            },
            TextureProceduralNode {
                id: "bomb".to_string(),
                op: TextureProceduralOp::TextureBomb {
                    input: "c".to_string(),
                    density: 0.35,
                    scale_variation: [0.8, 1.2],
                    rotation_variation: 45.0,
                    blend_mode: "max".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    for id in ["wang", "bomb"] {
        let grid = nodes.get(id).unwrap().as_grayscale().unwrap();
        let v = grid.get(7, 9);
        assert!((0.0..=1.0).contains(&v), "{} out of range: {}", id, v);
    }
}
