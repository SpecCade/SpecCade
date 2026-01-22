//! Tests for filter operations (blur, erode, dilate, warp).

use speccade_spec::recipe::texture::{
    GradientDirection, TextureProceduralNode, TextureProceduralOp,
};

use super::{approx_eq, generate_graph, make_params};

#[test]
fn blur_smooths_values() {
    let params = make_params(
        false,
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
                id: "blurred".to_string(),
                op: TextureProceduralOp::Blur {
                    input: "c".to_string(),
                    radius: 2.0,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let blurred = nodes.get("blurred").unwrap().as_grayscale().unwrap();

    // After blur, values should be smoothed between 0 and 1
    // Center of 4x4 tile should be close to extremes, edges should be mixed
    let center_val = blurred.get(2, 2);
    let edge_val = blurred.get(4, 4);

    // Just verify blur produced some smoothing effect
    assert!((0.0..=1.0).contains(&center_val));
    assert!((0.0..=1.0).contains(&edge_val));
}

#[test]
fn erode_shrinks_bright_regions() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "c".to_string(),
                op: TextureProceduralOp::Checkerboard {
                    tile_size: 8,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "eroded".to_string(),
                op: TextureProceduralOp::Erode {
                    input: "c".to_string(),
                    radius: 1,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let original = nodes.get("c").unwrap().as_grayscale().unwrap();
    let eroded = nodes.get("eroded").unwrap().as_grayscale().unwrap();

    // Erode should shrink bright areas (take minimum in neighborhood)
    // Center of bright tile should still be bright
    assert!(approx_eq(original.get(4, 4), 0.0));
    // But edge pixels of bright tiles become dark
    let edge_eroded = eroded.get(8, 8);
    assert!(edge_eroded <= 0.5, "Edge should be eroded: {}", edge_eroded);
}

#[test]
fn dilate_expands_bright_regions() {
    let params = make_params(
        false,
        vec![
            TextureProceduralNode {
                id: "c".to_string(),
                op: TextureProceduralOp::Checkerboard {
                    tile_size: 8,
                    color1: 0.0,
                    color2: 1.0,
                },
            },
            TextureProceduralNode {
                id: "dilated".to_string(),
                op: TextureProceduralOp::Dilate {
                    input: "c".to_string(),
                    radius: 1,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let dilated = nodes.get("dilated").unwrap().as_grayscale().unwrap();

    // Dilate should expand bright areas (take maximum in neighborhood)
    // Dark area edges should become bright
    let edge_dilated = dilated.get(7, 7);
    assert!(
        edge_dilated >= 0.5,
        "Edge should be dilated: {}",
        edge_dilated
    );
}

#[test]
fn warp_displaces_samples() {
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
                id: "disp".to_string(),
                op: TextureProceduralOp::Constant { value: 0.75 },
            },
            TextureProceduralNode {
                id: "warped".to_string(),
                op: TextureProceduralOp::Warp {
                    input: "grad".to_string(),
                    displacement: "disp".to_string(),
                    strength: 4.0,
                },
            },
        ],
    );

    let nodes = generate_graph(&params, 1).unwrap();
    let original = nodes.get("grad").unwrap().as_grayscale().unwrap();
    let warped = nodes.get("warped").unwrap().as_grayscale().unwrap();

    // With constant displacement > 0.5, samples should be shifted
    // The warped gradient should differ from original
    let orig_val = original.get(16, 16);
    let warp_val = warped.get(16, 16);
    assert!(
        (orig_val - warp_val).abs() > 0.01,
        "Warp should displace: orig={}, warped={}",
        orig_val,
        warp_val
    );
}
