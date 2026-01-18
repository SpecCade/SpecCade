//! Texture node graph functions

use super::{func, param, FunctionInfo};

pub(super) fn register_functions() -> Vec<FunctionInfo> {
    vec![
        // === TEXTURE NODES ===
        func!(
            "noise_node",
            "texture.nodes",
            "Creates a noise texture node.",
            vec![
                param!("id", "string", req),
                param!("algorithm", "string", opt, "perlin", enum: &["perlin", "simplex", "worley", "value", "fbm"]),
                param!("scale", "float", opt, 0.1, range: Some(0.0), None),
                param!("octaves", "int", opt, 4, range: Some(1.0), None),
                param!("persistence", "float", opt, 0.5),
                param!("lacunarity", "float", opt, 2.0),
            ],
            "A texture node dict.",
            r#"noise_node("height", "perlin", 0.1, 4)"#
        ),
        func!(
            "gradient_node",
            "texture.nodes",
            "Creates a gradient texture node.",
            vec![
                param!("id", "string", req),
                param!("direction", "string", opt, "horizontal", enum: &["horizontal", "vertical", "radial"]),
                param!("start", "float", opt, 0.0),
                param!("end", "float", opt, 1.0),
            ],
            "A texture node dict.",
            r#"gradient_node("grad", "horizontal")"#
        ),
        func!(
            "color_ramp_node",
            "texture.nodes",
            "Creates a color ramp mapping node.",
            vec![
                param!("id", "string", req),
                param!("input", "string", req),
                param!("ramp", "list", req),
            ],
            "A texture node dict.",
            r##"color_ramp_node("colored", "noise", ["#000000", "#ffffff"])"##
        ),
        func!(
            "threshold_node",
            "texture.nodes",
            "Creates a threshold operation node.",
            vec![
                param!("id", "string", req),
                param!("input", "string", req),
                param!("threshold", "float", opt, 0.5),
            ],
            "A texture node dict.",
            r#"threshold_node("mask", "noise", 0.5)"#
        ),
        // === TEXTURE GRAPH ===
        func!(
            "texture_graph",
            "texture.graph",
            "Creates a complete texture graph recipe params.",
            vec![
                param!("resolution", "list", req),
                param!("nodes", "list", req),
                param!("tileable", "bool", opt, true),
            ],
            "A texture graph params dict.",
            r#"texture_graph([64, 64], [noise_node("base", "perlin")])"#
        ),
    ]
}
