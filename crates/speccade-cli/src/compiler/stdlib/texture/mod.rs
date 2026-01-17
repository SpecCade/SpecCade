//! Texture stdlib functions for procedural texture generation.
//!
//! Provides helper functions for creating texture graph nodes and graphs.

mod graph;
mod nodes;

use starlark::environment::GlobalsBuilder;

/// Registers texture stdlib functions into a GlobalsBuilder.
pub fn register(builder: &mut GlobalsBuilder) {
    nodes::register(builder);
    graph::register(builder);
}

#[cfg(test)]
mod tests {
    use super::super::tests::eval_to_json;

    // ========================================================================
    // noise_node() tests
    // ========================================================================

    #[test]
    fn test_noise_node_defaults() {
        let result = eval_to_json("noise_node(\"n1\")").unwrap();
        assert_eq!(result["id"], "n1");
        assert_eq!(result["type"], "noise");
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "perlin");
        assert_eq!(noise["scale"], 0.1);
        assert_eq!(noise["octaves"], 4);
    }

    #[test]
    fn test_noise_node_custom() {
        let result = eval_to_json("noise_node(\"height\", \"simplex\", 0.05, 6)").unwrap();
        let noise = &result["noise"];
        assert_eq!(noise["algorithm"], "simplex");
        assert_eq!(noise["scale"], 0.05);
        assert_eq!(noise["octaves"], 6);
    }

    #[test]
    fn test_noise_node_invalid_algorithm() {
        let result = eval_to_json("noise_node(\"n\", \"fractal\")");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S104"));
        assert!(err.contains("algorithm"));
    }

    // ========================================================================
    // gradient_node() tests
    // ========================================================================

    #[test]
    fn test_gradient_node_defaults() {
        let result = eval_to_json("gradient_node(\"g1\")").unwrap();
        assert_eq!(result["id"], "g1");
        assert_eq!(result["type"], "gradient");
        assert_eq!(result["direction"], "horizontal");
        assert_eq!(result["start"], 0.0);
        assert_eq!(result["end"], 1.0);
    }

    #[test]
    fn test_gradient_node_vertical() {
        let result = eval_to_json("gradient_node(\"g2\", \"vertical\", 0.2, 0.8)").unwrap();
        assert_eq!(result["direction"], "vertical");
        assert_eq!(result["start"], 0.2);
        assert_eq!(result["end"], 0.8);
    }

    // ========================================================================
    // constant_node() tests
    // ========================================================================

    #[test]
    fn test_constant_node() {
        let result = eval_to_json("constant_node(\"white\", 1.0)").unwrap();
        assert_eq!(result["id"], "white");
        assert_eq!(result["type"], "constant");
        assert_eq!(result["value"], 1.0);
    }

    // ========================================================================
    // threshold_node() tests
    // ========================================================================

    #[test]
    fn test_threshold_node_default() {
        let result = eval_to_json("threshold_node(\"mask\", \"noise\")").unwrap();
        assert_eq!(result["id"], "mask");
        assert_eq!(result["type"], "threshold");
        assert_eq!(result["input"], "noise");
        assert_eq!(result["threshold"], 0.5);
    }

    #[test]
    fn test_threshold_node_custom() {
        let result = eval_to_json("threshold_node(\"binary\", \"height\", 0.7)").unwrap();
        assert_eq!(result["threshold"], 0.7);
    }

    // ========================================================================
    // invert_node() tests
    // ========================================================================

    #[test]
    fn test_invert_node() {
        let result = eval_to_json("invert_node(\"inverted\", \"noise\")").unwrap();
        assert_eq!(result["id"], "inverted");
        assert_eq!(result["type"], "invert");
        assert_eq!(result["input"], "noise");
    }

    // ========================================================================
    // color_ramp_node() tests
    // ========================================================================

    #[test]
    fn test_color_ramp_node() {
        let result = eval_to_json("color_ramp_node(\"colored\", \"noise\", [\"#000000\", \"#ffffff\"])").unwrap();
        assert_eq!(result["id"], "colored");
        assert_eq!(result["type"], "color_ramp");
        assert_eq!(result["input"], "noise");
        assert!(result["ramp"].is_array());
        assert_eq!(result["ramp"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_color_ramp_node_too_few_colors() {
        let result = eval_to_json("color_ramp_node(\"c\", \"n\", [\"#000000\"])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
        assert!(err.contains("at least 2 colors"));
    }

    // ========================================================================
    // texture_graph() tests
    // ========================================================================

    #[test]
    fn test_texture_graph_basic() {
        let result = eval_to_json("texture_graph([64, 64], [noise_node(\"n\")])").unwrap();

        assert!(result["resolution"].is_array());
        let res = result["resolution"].as_array().unwrap();
        assert_eq!(res.len(), 2);
        assert_eq!(res[0], 64);
        assert_eq!(res[1], 64);
        assert_eq!(result["tileable"], true);
        assert!(result["nodes"].is_array());
        assert_eq!(result["nodes"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_texture_graph_not_tileable() {
        let result = eval_to_json("texture_graph([128, 128], [], False)").unwrap();
        assert_eq!(result["tileable"], false);
    }

    #[test]
    fn test_texture_graph_invalid_resolution() {
        let result = eval_to_json("texture_graph([64], [])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S101"));
    }

    #[test]
    fn test_texture_graph_negative_resolution() {
        let result = eval_to_json("texture_graph([-64, 64], [])");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("S103"));
    }

    // ========================================================================
    // gradient_node() radial params tests
    // ========================================================================

    #[test]
    fn test_gradient_node_radial_with_params() {
        let result = eval_to_json("gradient_node(\"g3\", \"radial\", 1.0, 0.0, [0.5, 0.5], 0.0, 1.0)").unwrap();
        assert_eq!(result["direction"], "radial");
        assert!(result["center"].is_array());
        let center = result["center"].as_array().unwrap();
        assert_eq!(center[0], 0.5);
        assert_eq!(center[1], 0.5);
        assert_eq!(result["inner"], 0.0);
        assert_eq!(result["outer"], 1.0);
    }

    // ========================================================================
    // add_node() tests
    // ========================================================================

    #[test]
    fn test_add_node() {
        let result = eval_to_json("add_node(\"combined\", \"noise1\", \"noise2\")").unwrap();
        assert_eq!(result["id"], "combined");
        assert_eq!(result["type"], "add");
        assert_eq!(result["a"], "noise1");
        assert_eq!(result["b"], "noise2");
    }

    // ========================================================================
    // multiply_node() tests
    // ========================================================================

    #[test]
    fn test_multiply_node() {
        let result = eval_to_json("multiply_node(\"masked\", \"noise\", \"gradient\")").unwrap();
        assert_eq!(result["id"], "masked");
        assert_eq!(result["type"], "multiply");
        assert_eq!(result["a"], "noise");
        assert_eq!(result["b"], "gradient");
    }

    // ========================================================================
    // lerp_node() tests
    // ========================================================================

    #[test]
    fn test_lerp_node() {
        let result = eval_to_json("lerp_node(\"blended\", \"noise1\", \"noise2\", \"mask\")").unwrap();
        assert_eq!(result["id"], "blended");
        assert_eq!(result["type"], "lerp");
        assert_eq!(result["a"], "noise1");
        assert_eq!(result["b"], "noise2");
        assert_eq!(result["t"], "mask");
    }

    // ========================================================================
    // clamp_node() tests
    // ========================================================================

    #[test]
    fn test_clamp_node_defaults() {
        let result = eval_to_json("clamp_node(\"clamped\", \"noise\")").unwrap();
        assert_eq!(result["id"], "clamped");
        assert_eq!(result["type"], "clamp");
        assert_eq!(result["input"], "noise");
        assert_eq!(result["min"], 0.0);
        assert_eq!(result["max"], 1.0);
    }

    #[test]
    fn test_clamp_node_custom() {
        let result = eval_to_json("clamp_node(\"clamped\", \"noise\", 0.2, 0.8)").unwrap();
        assert_eq!(result["min"], 0.2);
        assert_eq!(result["max"], 0.8);
    }

    // ========================================================================
    // stripes_node() tests
    // ========================================================================

    #[test]
    fn test_stripes_node() {
        let result = eval_to_json("stripes_node(\"lines\", \"horizontal\", 4)").unwrap();
        assert_eq!(result["id"], "lines");
        assert_eq!(result["type"], "stripes");
        assert_eq!(result["direction"], "horizontal");
        assert_eq!(result["stripe_width"], 4);
        assert_eq!(result["color1"], 0.0);
        assert_eq!(result["color2"], 1.0);
    }

    #[test]
    fn test_stripes_node_vertical() {
        let result = eval_to_json("stripes_node(\"vlines\", \"vertical\", 8, 0.3, 0.7)").unwrap();
        assert_eq!(result["direction"], "vertical");
        assert_eq!(result["stripe_width"], 8);
        assert_eq!(result["color1"], 0.3);
        assert_eq!(result["color2"], 0.7);
    }

    // ========================================================================
    // checkerboard_node() tests
    // ========================================================================

    #[test]
    fn test_checkerboard_node() {
        let result = eval_to_json("checkerboard_node(\"checker\", 8)").unwrap();
        assert_eq!(result["id"], "checker");
        assert_eq!(result["type"], "checkerboard");
        assert_eq!(result["tile_size"], 8);
        assert_eq!(result["color1"], 0.0);
        assert_eq!(result["color2"], 1.0);
    }

    // ========================================================================
    // grayscale_node() tests
    // ========================================================================

    #[test]
    fn test_grayscale_node() {
        let result = eval_to_json("grayscale_node(\"gray\", \"colored\")").unwrap();
        assert_eq!(result["id"], "gray");
        assert_eq!(result["type"], "to_grayscale");
        assert_eq!(result["input"], "colored");
    }

    // ========================================================================
    // palette_node() tests
    // ========================================================================

    #[test]
    fn test_palette_node() {
        let result = eval_to_json("palette_node(\"retro\", \"colored\", [\"#000000\", \"#ff0000\", \"#00ff00\"])").unwrap();
        assert_eq!(result["id"], "retro");
        assert_eq!(result["type"], "palette");
        assert_eq!(result["input"], "colored");
        assert!(result["palette"].is_array());
        assert_eq!(result["palette"].as_array().unwrap().len(), 3);
    }

    // ========================================================================
    // compose_rgba_node() tests
    // ========================================================================

    #[test]
    fn test_compose_rgba_node() {
        let result = eval_to_json("compose_rgba_node(\"color\", \"r\", \"g\", \"b\")").unwrap();
        assert_eq!(result["id"], "color");
        assert_eq!(result["type"], "compose_rgba");
        assert_eq!(result["r"], "r");
        assert_eq!(result["g"], "g");
        assert_eq!(result["b"], "b");
        assert!(result.get("a").is_none() || result["a"].is_null());
    }

    #[test]
    fn test_compose_rgba_node_with_alpha() {
        let result = eval_to_json("compose_rgba_node(\"color\", \"r\", \"g\", \"b\", \"alpha\")").unwrap();
        assert_eq!(result["a"], "alpha");
    }

    // ========================================================================
    // normal_from_height_node() tests
    // ========================================================================

    #[test]
    fn test_normal_from_height_node() {
        let result = eval_to_json("normal_from_height_node(\"normals\", \"height\")").unwrap();
        assert_eq!(result["id"], "normals");
        assert_eq!(result["type"], "normal_from_height");
        assert_eq!(result["input"], "height");
        assert_eq!(result["strength"], 1.0);
    }

    #[test]
    fn test_normal_from_height_node_custom() {
        let result = eval_to_json("normal_from_height_node(\"normals\", \"height\", 2.5)").unwrap();
        assert_eq!(result["strength"], 2.5);
    }
}
