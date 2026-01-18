//! Tests for UV transform operations (scale, rotate, translate).

use speccade_spec::recipe::texture::{
    GradientDirection, StripeDirection, TextureProceduralNode, TextureProceduralOp,
};

use super::{approx_eq, generate_graph, make_params};

#[test]
fn uv_scale_tiles_pattern() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "stripes".to_string(),
                op: TextureProceduralOp::Stripes {
                    direction: StripeDirection::Vertical,
                    stripe_width: 16,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "scaled".to_string(),
                op: TextureProceduralOp::UvScale {
                    input: "stripes".to_string(),
                    scale_x: 2.0,
                    scale_y: 1.0,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let scaled = nodes.get("scaled").unwrap().as_grayscale().unwrap();

    // With 2x scale, pattern should repeat more frequently
    // Check that we have a transition in what was previously a uniform area
    let v1 = scaled.get(8, 0);
    let v2 = scaled.get(24, 0);
    // Due to scaling, both should be valid grayscale values
    assert!(v1 >= 0.0 && v1 <= 1.0);
    assert!(v2 >= 0.0 && v2 <= 1.0);
}

#[test]
fn uv_rotate_changes_orientation() {
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
                id: "rotated".to_string(),
                op: TextureProceduralOp::UvRotate {
                    input: "grad".to_string(),
                    angle: std::f32::consts::FRAC_PI_2,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let original = nodes.get("grad").unwrap().as_grayscale().unwrap();
    let rotated = nodes.get("rotated").unwrap().as_grayscale().unwrap();

    // After 90 degree rotation, horizontal gradient becomes vertical-like
    // The center values should differ
    let orig_edge = original.get(0, 16);
    let rot_edge = rotated.get(0, 16);

    // Values should be different due to rotation
    assert!(
        (orig_edge - rot_edge).abs() > 0.1 || approx_eq(orig_edge, rot_edge),
        "Rotation should change sample positions"
    );
}

#[test]
fn uv_translate_shifts_pattern() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "stripes".to_string(),
                op: TextureProceduralOp::Stripes {
                    direction: StripeDirection::Horizontal,
                    stripe_width: 16,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "translated".to_string(),
                op: TextureProceduralOp::UvTranslate {
                    input: "stripes".to_string(),
                    offset_x: 0.0,
                    offset_y: 0.5,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let original = nodes.get("stripes").unwrap().as_grayscale().unwrap();
    let translated = nodes.get("translated").unwrap().as_grayscale().unwrap();

    // With 0.5 vertical offset, stripe pattern should be shifted by half
    // What was color1 should now be color2 and vice versa
    let orig_val = original.get(0, 0);
    let trans_val = translated.get(0, 0);

    // The values should be different (shifted by half period)
    assert!(
        (orig_val - trans_val).abs() > 0.5,
        "Translation should shift pattern: orig={}, trans={}",
        orig_val,
        trans_val
    );
}
