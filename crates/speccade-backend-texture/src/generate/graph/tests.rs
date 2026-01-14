//! Tests for procedural texture graph generation.

use crate::color::Color;
use speccade_spec::recipe::texture::{NoiseAlgorithm, NoiseConfig, TextureProceduralNode, TextureProceduralV1Params, TextureProceduralOp};

use super::{encode_graph_value_png, generate_graph};

fn make_params(tileable: bool, nodes: Vec<TextureProceduralNode>) -> TextureProceduralV1Params {
    TextureProceduralV1Params {
        resolution: [32, 32],
        tileable,
        nodes,
    }
}

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
        err.to_string().contains("color output")
            || err.to_string().contains("color was required")
    );
}

#[test]
fn tileable_noise_matches_edges() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "n".to_string(),
            op: TextureProceduralOp::Noise {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Perlin,
                    scale: 0.12,
                    octaves: 3,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            },
        }],
    );

    let nodes = generate_graph(&params, 42).unwrap();
    let n = nodes.get("n").unwrap().as_grayscale().unwrap();

    let w = n.width;
    let h = n.height;

    for y in 0..h {
        let left = n.get(0, y);
        let right = n.get(w - 1, y);
        assert!(
            (left - right).abs() < 1e-12,
            "left/right mismatch at y={}: {} vs {}",
            y,
            left,
            right
        );
    }

    for x in 0..w {
        let top = n.get(x, 0);
        let bottom = n.get(x, h - 1);
        assert!(
            (top - bottom).abs() < 1e-12,
            "top/bottom mismatch at x={}: {} vs {}",
            x,
            top,
            bottom
        );
    }
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

fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn color_approx_eq(a: Color, b: Color) -> bool {
    approx_eq(a.r, b.r) && approx_eq(a.g, b.g) && approx_eq(a.b, b.b) && approx_eq(a.a, b.a)
}

#[test]
fn constant_outputs_fill_value() {
    let params = make_params(
        false,
        vec![TextureProceduralNode {
            id: "c".to_string(),
            op: TextureProceduralOp::Constant { value: 0.25 },
        }],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let c = nodes.get("c").unwrap().as_grayscale().unwrap();
    assert!(approx_eq(c.get(0, 0), 0.25));
    assert!(approx_eq(c.get(c.width - 1, c.height - 1), 0.25));
}

#[test]
fn gradient_and_stripes_ops_produce_expected_patterns() {
    use speccade_spec::recipe::texture::{GradientDirection, StripeDirection};

    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "grad".to_string(),
                op: TextureProceduralOp::Gradient {
                    direction: GradientDirection::Horizontal,
                    start: Some(0.0),
                    end: Some(1.0),
                    center: None,
                    inner: None,
                    outer: None,
                },
            },
            TextureProceduralNode {
                id: "stripes".to_string(),
                op: TextureProceduralOp::Stripes {
                    direction: StripeDirection::Vertical,
                    stripe_width: 4,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();

    let grad = nodes.get("grad").unwrap().as_grayscale().unwrap();
    assert!(approx_eq(grad.get(0, 0), 0.0));
    assert!(approx_eq(grad.get(grad.width - 1, 0), 1.0));

    let stripes = nodes.get("stripes").unwrap().as_grayscale().unwrap();
    assert!(approx_eq(stripes.get(0, 0), 0.0));
    assert!(approx_eq(stripes.get(3, 0), 0.0));
    assert!(approx_eq(stripes.get(4, 0), 1.0));
}

#[test]
fn add_multiply_lerp_compute_expected_values() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "a".to_string(),
                op: TextureProceduralOp::Constant { value: 0.2 },
            },
            TextureProceduralNode {
                id: "b".to_string(),
                op: TextureProceduralOp::Constant { value: 0.6 },
            },
            TextureProceduralNode {
                id: "t".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "add".to_string(),
                op: TextureProceduralOp::Add {
                    a: "a".to_string(),
                    b: "b".to_string(),
                },
            },
            TextureProceduralNode {
                id: "mul".to_string(),
                op: TextureProceduralOp::Multiply {
                    a: "a".to_string(),
                    b: "b".to_string(),
                },
            },
            TextureProceduralNode {
                id: "lerp".to_string(),
                op: TextureProceduralOp::Lerp {
                    a: "a".to_string(),
                    b: "b".to_string(),
                    t: "t".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let add = nodes.get("add").unwrap().as_grayscale().unwrap();
    let mul = nodes.get("mul").unwrap().as_grayscale().unwrap();
    let lerp = nodes.get("lerp").unwrap().as_grayscale().unwrap();
    assert!(approx_eq(add.get(0, 0), 0.8));
    assert!(approx_eq(mul.get(0, 0), 0.12));
    assert!(approx_eq(lerp.get(0, 0), 0.4));
}

#[test]
fn to_grayscale_color_ramp_palette_and_compose_rgba_work() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "g".to_string(),
                op: TextureProceduralOp::Constant { value: 0.25 },
            },
            TextureProceduralNode {
                id: "ramp".to_string(),
                op: TextureProceduralOp::ColorRamp {
                    input: "g".to_string(),
                    ramp: vec!["#000000".to_string(), "#ffffff".to_string()],
                },
            },
            TextureProceduralNode {
                id: "pal".to_string(),
                op: TextureProceduralOp::Palette {
                    input: "ramp".to_string(),
                    palette: vec!["#000000".to_string(), "#ffffff".to_string()],
                },
            },
            TextureProceduralNode {
                id: "gray".to_string(),
                op: TextureProceduralOp::ToGrayscale {
                    input: "pal".to_string(),
                },
            },
            TextureProceduralNode {
                id: "rgba".to_string(),
                op: TextureProceduralOp::ComposeRgba {
                    r: "gray".to_string(),
                    g: "g".to_string(),
                    b: "g".to_string(),
                    a: Some("g".to_string()),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();

    let ramp = nodes.get("ramp").unwrap().as_color().unwrap();
    assert!(color_approx_eq(
        ramp.get(0, 0),
        Color::rgba(0.25, 0.25, 0.25, 1.0)
    ));

    let pal = nodes.get("pal").unwrap().as_color().unwrap();
    assert!(color_approx_eq(
        pal.get(0, 0),
        Color::rgba(0.0, 0.0, 0.0, 1.0)
    ));

    let gray = nodes.get("gray").unwrap().as_grayscale().unwrap();
    assert!(approx_eq(gray.get(0, 0), 0.0));

    let rgba = nodes.get("rgba").unwrap().as_color().unwrap();
    assert!(color_approx_eq(
        rgba.get(0, 0),
        Color::rgba(0.0, 0.25, 0.25, 0.25)
    ));
}

#[test]
fn normal_from_height_constant_is_flat_normal() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "h".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "n".to_string(),
                op: TextureProceduralOp::NormalFromHeight {
                    input: "h".to_string(),
                    strength: 1.0,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let n = nodes.get("n").unwrap().as_color().unwrap();
    assert!(color_approx_eq(
        n.get(0, 0),
        Color::rgba(0.5, 0.5, 1.0, 1.0)
    ));
    assert!(color_approx_eq(
        n.get(n.width - 1, n.height - 1),
        Color::rgba(0.5, 0.5, 1.0, 1.0)
    ));
}
