//! Tests for mathematical operations (add, multiply, lerp, threshold).

use speccade_spec::recipe::texture::{TextureProceduralNode, TextureProceduralOp};

use super::{approx_eq, generate_graph, make_params};

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
