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
                param!("algorithm", "string", opt, "perlin", enum: &["perlin", "simplex", "worley", "value", "gabor", "fbm"]),
                param!("scale", "float", opt, 0.1, range: Some(0.0), None),
                param!("octaves", "int", opt, 4, range: Some(1.0), None),
                param!("persistence", "float", opt, 0.5),
                param!("lacunarity", "float", opt, 2.0),
            ],
            "A texture node dict.",
            r#"noise_node("height", "perlin", 0.1, 4)"#
        ),
        func!(
            "reaction_diffusion_node",
            "texture.nodes",
            "Creates a Gray-Scott reaction-diffusion texture node.",
            vec![
                param!("id", "string", req),
                param!("steps", "int", opt, 120, range: Some(1.0), Some(2000.0)),
                param!("feed", "float", opt, 0.055, range: Some(0.0), Some(1.0)),
                param!("kill", "float", opt, 0.062, range: Some(0.0), Some(1.0)),
                param!("diffuse_a", "float", opt, 1.0, range: Some(0.0), Some(2.0)),
                param!("diffuse_b", "float", opt, 0.5, range: Some(0.0), Some(2.0)),
                param!("dt", "float", opt, 1.0, range: Some(0.0), Some(2.0)),
                param!("seed_density", "float", opt, 0.03, range: Some(0.0), Some(0.5)),
            ],
            "A texture node dict.",
            r#"reaction_diffusion_node("rd", 180, 0.054, 0.064, 1.0, 0.5, 1.0, 0.04)"#
        ),
        func!(
            "reaction_diffusion_preset",
            "texture.nodes",
            "Returns tuned parameters for a reaction-diffusion preset.",
            vec![param!("preset", "string", opt, "mitosis", enum: &["mitosis", "worms", "spots"]),],
            "A dict with reaction_diffusion_node-compatible parameters.",
            r#"reaction_diffusion_preset("worms")"#
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
