//! Tests for blend mode operations.

use speccade_spec::recipe::texture::{TextureProceduralNode, TextureProceduralOp};

use super::{approx_eq, generate_graph, make_params};

#[test]
fn blend_screen_lightens() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "a".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "b".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "screen".to_string(),
                op: TextureProceduralOp::BlendScreen {
                    base: "a".to_string(),
                    blend: "b".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let screen = nodes.get("screen").unwrap().as_grayscale().unwrap();

    // Screen: 1 - (1 - 0.5) * (1 - 0.5) = 1 - 0.25 = 0.75
    assert!(approx_eq(screen.get(0, 0), 0.75));
}

#[test]
fn blend_overlay_contrasts() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "dark".to_string(),
                op: TextureProceduralOp::Constant { value: 0.25 },
            },
            TextureProceduralNode {
                id: "blend".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "overlay".to_string(),
                op: TextureProceduralOp::BlendOverlay {
                    base: "dark".to_string(),
                    blend: "blend".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let overlay = nodes.get("overlay").unwrap().as_grayscale().unwrap();

    // Overlay with base < 0.5: 2 * 0.25 * 0.5 = 0.25
    assert!(approx_eq(overlay.get(0, 0), 0.25));
}

#[test]
fn blend_soft_light_subtle() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "a".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "b".to_string(),
                op: TextureProceduralOp::Constant { value: 0.5 },
            },
            TextureProceduralNode {
                id: "soft".to_string(),
                op: TextureProceduralOp::BlendSoftLight {
                    base: "a".to_string(),
                    blend: "b".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let soft = nodes.get("soft").unwrap().as_grayscale().unwrap();

    // Soft light (Pegtop): (1 - 2*0.5) * 0.25 + 2 * 0.5 * 0.5 = 0 + 0.5 = 0.5
    assert!(approx_eq(soft.get(0, 0), 0.5));
}

#[test]
fn blend_difference_computes_abs() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "a".to_string(),
                op: TextureProceduralOp::Constant { value: 0.8 },
            },
            TextureProceduralNode {
                id: "b".to_string(),
                op: TextureProceduralOp::Constant { value: 0.3 },
            },
            TextureProceduralNode {
                id: "diff".to_string(),
                op: TextureProceduralOp::BlendDifference {
                    base: "a".to_string(),
                    blend: "b".to_string(),
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let diff = nodes.get("diff").unwrap().as_grayscale().unwrap();

    // Difference: |0.8 - 0.3| = 0.5
    assert!(approx_eq(diff.get(0, 0), 0.5));
}
