//! Tests for primitive operations (constant, noise, gradient, stripes, checkerboard).

use speccade_spec::recipe::texture::{
    GradientDirection, NoiseAlgorithm, NoiseConfig, StripeDirection, TextureProceduralNode,
    TextureProceduralOp,
};

use super::{approx_eq, generate_graph, make_params};

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
fn gradient_and_stripes_ops_produce_expected_patterns() {
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
fn gabor_noise_node_generates_non_uniform_field() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "g".to_string(),
            op: TextureProceduralOp::Noise {
                noise: NoiseConfig {
                    algorithm: NoiseAlgorithm::Gabor,
                    scale: 0.1,
                    octaves: 1,
                    persistence: 0.5,
                    lacunarity: 2.0,
                },
            },
        }],
    );

    let nodes = generate_graph(&params, 77).unwrap();
    let g = nodes.get("g").unwrap().as_grayscale().unwrap();
    let a = g.get(0, 0);
    let b = g.get(g.width / 2, g.height / 2);
    assert!(
        (a - b).abs() > 1e-6,
        "expected non-uniform Gabor field values"
    );
}

#[test]
fn reaction_diffusion_is_deterministic() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "rd".to_string(),
            op: TextureProceduralOp::ReactionDiffusion {
                steps: 64,
                feed: 0.055,
                kill: 0.062,
                diffuse_a: 1.0,
                diffuse_b: 0.5,
                dt: 1.0,
                seed_density: 0.03,
            },
        }],
    );

    let nodes_a = generate_graph(&params, 42).unwrap();
    let nodes_b = generate_graph(&params, 42).unwrap();
    let rd_a = nodes_a.get("rd").unwrap().as_grayscale().unwrap();
    let rd_b = nodes_b.get("rd").unwrap().as_grayscale().unwrap();
    assert_eq!(rd_a.data, rd_b.data);
}

#[test]
fn reaction_diffusion_generates_non_uniform_field() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "rd".to_string(),
            op: TextureProceduralOp::ReactionDiffusion {
                steps: 180,
                feed: 0.037,
                kill: 0.065,
                diffuse_a: 1.0,
                diffuse_b: 0.5,
                dt: 1.0,
                seed_density: 0.08,
            },
        }],
    );

    let nodes = generate_graph(&params, 123).unwrap();
    let rd = nodes.get("rd").unwrap().as_grayscale().unwrap();
    let mut min_v = f64::INFINITY;
    let mut max_v = f64::NEG_INFINITY;
    for v in &rd.data {
        min_v = min_v.min(*v);
        max_v = max_v.max(*v);
    }
    assert!(
        max_v - min_v > 1e-3,
        "expected reaction-diffusion to produce non-uniform values, min={}, max={}",
        min_v,
        max_v
    );
}

#[test]
fn reaction_diffusion_rejects_invalid_params() {
    let params = make_params(
        true,
        vec![TextureProceduralNode {
            id: "rd".to_string(),
            op: TextureProceduralOp::ReactionDiffusion {
                steps: 0,
                feed: 0.055,
                kill: 0.062,
                diffuse_a: 1.0,
                diffuse_b: 0.5,
                dt: 1.0,
                seed_density: 0.03,
            },
        }],
    );

    let err = generate_graph(&params, 42).unwrap_err();
    assert!(err.to_string().contains("reaction_diffusion.steps"));
}
