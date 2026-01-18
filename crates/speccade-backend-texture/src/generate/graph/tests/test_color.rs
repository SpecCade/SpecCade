//! Tests for color operations (to_grayscale, color_ramp, palette, compose_rgba, normal_from_height).

use crate::color::Color;
use speccade_spec::recipe::texture::{TextureProceduralNode, TextureProceduralOp};

use super::{approx_eq, color_approx_eq, generate_graph, make_params};

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
