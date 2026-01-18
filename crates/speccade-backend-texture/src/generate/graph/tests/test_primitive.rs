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
