//! Texture recipe output validation.

use std::collections::{HashMap, HashSet};

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::recipe::Recipe;
use crate::spec::Spec;
use crate::validation::BudgetProfile;

use super::recipe_outputs::validate_primary_output_present;

/// Validates texture procedural outputs with the default budget profile.
#[allow(dead_code)]
pub(super) fn validate_texture_procedural_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
    validate_texture_procedural_outputs_with_budget(spec, recipe, &BudgetProfile::default(), result)
}

/// Validates texture procedural outputs with a specific budget profile.
pub(super) fn validate_texture_procedural_outputs_with_budget(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    budget: &BudgetProfile,
    result: &mut ValidationResult,
) {
    let max_graph_nodes = budget.texture.max_graph_nodes;

    let params = match recipe.as_texture_procedural() {
        Ok(params) => params,
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    if params.nodes.is_empty() {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            "texture.procedural_v1 requires at least one node".to_string(),
            "recipe.params.nodes",
        ));
        return;
    }

    // Check node count against budget
    if params.nodes.len() > max_graph_nodes {
        result.add_error(ValidationError::with_path(
            ErrorCode::BudgetExceeded,
            format!(
                "texture graph has {} nodes, exceeds budget limit of {} (profile: {})",
                params.nodes.len(),
                max_graph_nodes,
                budget.name
            ),
            "recipe.params.nodes",
        ));
    }

    let mut node_ids: HashSet<&str> = HashSet::new();
    for (i, node) in params.nodes.iter().enumerate() {
        if !node_ids.insert(node.id.as_str()) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("duplicate node id: '{}'", node.id),
                format!("recipe.params.nodes[{}].id", i),
            ));
        }
    }

    let validate_ref = |id: &str, path: String, result: &mut ValidationResult| {
        if !node_ids.contains(id) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("unknown node reference '{}'", id),
                path,
            ));
        }
    };

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum GraphValueType {
        Grayscale,
        Color,
    }

    impl GraphValueType {
        fn as_str(&self) -> &'static str {
            match self {
                GraphValueType::Grayscale => "grayscale",
                GraphValueType::Color => "color",
            }
        }
    }

    // Type information for each node is fixed based on op kind (this enables simple type checks).
    let mut node_types: HashMap<&str, GraphValueType> = HashMap::new();
    for node in &params.nodes {
        use crate::recipe::texture::TextureProceduralOp;

        let node_type = match &node.op {
            TextureProceduralOp::ColorRamp { .. }
            | TextureProceduralOp::Palette { .. }
            | TextureProceduralOp::ComposeRgba { .. }
            | TextureProceduralOp::NormalFromHeight { .. } => GraphValueType::Color,
            TextureProceduralOp::Constant { .. }
            | TextureProceduralOp::Noise { .. }
            | TextureProceduralOp::Gradient { .. }
            | TextureProceduralOp::Stripes { .. }
            | TextureProceduralOp::Checkerboard { .. }
            | TextureProceduralOp::Invert { .. }
            | TextureProceduralOp::Clamp { .. }
            | TextureProceduralOp::Add { .. }
            | TextureProceduralOp::Multiply { .. }
            | TextureProceduralOp::Lerp { .. }
            | TextureProceduralOp::Threshold { .. }
            | TextureProceduralOp::ToGrayscale { .. }
            | TextureProceduralOp::Blur { .. }
            | TextureProceduralOp::Erode { .. }
            | TextureProceduralOp::Dilate { .. }
            | TextureProceduralOp::Warp { .. }
            | TextureProceduralOp::BlendScreen { .. }
            | TextureProceduralOp::BlendOverlay { .. }
            | TextureProceduralOp::BlendSoftLight { .. }
            | TextureProceduralOp::BlendDifference { .. }
            | TextureProceduralOp::UvScale { .. }
            | TextureProceduralOp::UvRotate { .. }
            | TextureProceduralOp::UvTranslate { .. }
            | TextureProceduralOp::WangTiles { .. }
            | TextureProceduralOp::TextureBomb { .. } => GraphValueType::Grayscale,
        };

        node_types.insert(node.id.as_str(), node_type);
    }

    let validate_input_type =
        |expected: GraphValueType, id: &str, path: String, result: &mut ValidationResult| {
            let Some(actual) = node_types.get(id).copied() else {
                return;
            };
            if actual != expected {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "type mismatch: expected {} input from '{}' but it produces {} output",
                        expected.as_str(),
                        id,
                        actual.as_str()
                    ),
                    path,
                ));
            }
        };

    let mut deps: HashMap<&str, Vec<&str>> = HashMap::new();

    for (i, node) in params.nodes.iter().enumerate() {
        use crate::recipe::texture::TextureProceduralOp;

        match &node.op {
            TextureProceduralOp::Invert { input }
            | TextureProceduralOp::Clamp { input, .. }
            | TextureProceduralOp::Threshold { input, .. }
            | TextureProceduralOp::ToGrayscale { input }
            | TextureProceduralOp::ColorRamp { input, .. }
            | TextureProceduralOp::Palette { input, .. }
            | TextureProceduralOp::NormalFromHeight { input, .. } => {
                validate_ref(input, format!("recipe.params.nodes[{}].input", i), result);
                // Input types
                match &node.op {
                    TextureProceduralOp::ToGrayscale { .. }
                    | TextureProceduralOp::Palette { .. } => {
                        validate_input_type(
                            GraphValueType::Color,
                            input,
                            format!("recipe.params.nodes[{}].input", i),
                            result,
                        );
                    }
                    TextureProceduralOp::ColorRamp { .. }
                    | TextureProceduralOp::NormalFromHeight { .. }
                    | TextureProceduralOp::Invert { .. }
                    | TextureProceduralOp::Clamp { .. }
                    | TextureProceduralOp::Threshold { .. } => {
                        validate_input_type(
                            GraphValueType::Grayscale,
                            input,
                            format!("recipe.params.nodes[{}].input", i),
                            result,
                        );
                    }
                    _ => {}
                }

                deps.insert(node.id.as_str(), vec![input.as_str()]);
            }
            TextureProceduralOp::Add { a, b } | TextureProceduralOp::Multiply { a, b } => {
                validate_ref(a, format!("recipe.params.nodes[{}].a", i), result);
                validate_ref(b, format!("recipe.params.nodes[{}].b", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    a,
                    format!("recipe.params.nodes[{}].a", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    b,
                    format!("recipe.params.nodes[{}].b", i),
                    result,
                );

                deps.insert(node.id.as_str(), vec![a.as_str(), b.as_str()]);
            }
            TextureProceduralOp::Lerp { a, b, t } => {
                validate_ref(a, format!("recipe.params.nodes[{}].a", i), result);
                validate_ref(b, format!("recipe.params.nodes[{}].b", i), result);
                validate_ref(t, format!("recipe.params.nodes[{}].t", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    a,
                    format!("recipe.params.nodes[{}].a", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    b,
                    format!("recipe.params.nodes[{}].b", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    t,
                    format!("recipe.params.nodes[{}].t", i),
                    result,
                );

                deps.insert(node.id.as_str(), vec![a.as_str(), b.as_str(), t.as_str()]);
            }
            TextureProceduralOp::ComposeRgba { r, g, b, a } => {
                validate_ref(r, format!("recipe.params.nodes[{}].r", i), result);
                validate_ref(g, format!("recipe.params.nodes[{}].g", i), result);
                validate_ref(b, format!("recipe.params.nodes[{}].b", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    r,
                    format!("recipe.params.nodes[{}].r", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    g,
                    format!("recipe.params.nodes[{}].g", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    b,
                    format!("recipe.params.nodes[{}].b", i),
                    result,
                );

                if let Some(a) = a.as_deref() {
                    validate_ref(a, format!("recipe.params.nodes[{}].a", i), result);
                    validate_input_type(
                        GraphValueType::Grayscale,
                        a,
                        format!("recipe.params.nodes[{}].a", i),
                        result,
                    );
                }

                let mut d = vec![r.as_str(), g.as_str(), b.as_str()];
                if let Some(a) = a.as_deref() {
                    d.push(a);
                }
                deps.insert(node.id.as_str(), d);
            }
            TextureProceduralOp::Constant { .. }
            | TextureProceduralOp::Noise { .. }
            | TextureProceduralOp::Gradient { .. }
            | TextureProceduralOp::Stripes { .. }
            | TextureProceduralOp::Checkerboard { .. } => {
                deps.insert(node.id.as_str(), Vec::new());
            }
            // Single grayscale input ops
            TextureProceduralOp::Blur { input, .. }
            | TextureProceduralOp::Erode { input, .. }
            | TextureProceduralOp::Dilate { input, .. }
            | TextureProceduralOp::UvScale { input, .. }
            | TextureProceduralOp::UvRotate { input, .. }
            | TextureProceduralOp::UvTranslate { input, .. } => {
                validate_ref(input, format!("recipe.params.nodes[{}].input", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    input,
                    format!("recipe.params.nodes[{}].input", i),
                    result,
                );
                deps.insert(node.id.as_str(), vec![input.as_str()]);
            }
            // Warp: input + displacement (both grayscale)
            TextureProceduralOp::Warp {
                input,
                displacement,
                ..
            } => {
                validate_ref(input, format!("recipe.params.nodes[{}].input", i), result);
                validate_ref(
                    displacement,
                    format!("recipe.params.nodes[{}].displacement", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    input,
                    format!("recipe.params.nodes[{}].input", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    displacement,
                    format!("recipe.params.nodes[{}].displacement", i),
                    result,
                );
                deps.insert(
                    node.id.as_str(),
                    vec![input.as_str(), displacement.as_str()],
                );
            }
            // Blend modes: base + blend (both grayscale)
            TextureProceduralOp::BlendScreen { base, blend }
            | TextureProceduralOp::BlendOverlay { base, blend }
            | TextureProceduralOp::BlendSoftLight { base, blend }
            | TextureProceduralOp::BlendDifference { base, blend } => {
                validate_ref(base, format!("recipe.params.nodes[{}].base", i), result);
                validate_ref(blend, format!("recipe.params.nodes[{}].blend", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    base,
                    format!("recipe.params.nodes[{}].base", i),
                    result,
                );
                validate_input_type(
                    GraphValueType::Grayscale,
                    blend,
                    format!("recipe.params.nodes[{}].blend", i),
                    result,
                );
                deps.insert(node.id.as_str(), vec![base.as_str(), blend.as_str()]);
            }
            // Stochastic tiling: WangTiles and TextureBomb (grayscale input)
            TextureProceduralOp::WangTiles { input, .. }
            | TextureProceduralOp::TextureBomb { input, .. } => {
                validate_ref(input, format!("recipe.params.nodes[{}].input", i), result);
                validate_input_type(
                    GraphValueType::Grayscale,
                    input,
                    format!("recipe.params.nodes[{}].input", i),
                    result,
                );
                deps.insert(node.id.as_str(), vec![input.as_str()]);
            }
        }
    }

    // Cycle detection (graph must be a DAG).
    fn find_cycle<'a>(deps: &HashMap<&'a str, Vec<&'a str>>) -> Option<Vec<&'a str>> {
        fn dfs<'a>(
            node: &'a str,
            deps: &HashMap<&'a str, Vec<&'a str>>,
            state: &mut HashMap<&'a str, u8>,
            stack: &mut Vec<&'a str>,
        ) -> Option<Vec<&'a str>> {
            match state.get(node).copied().unwrap_or(0) {
                1 => {
                    if let Some(pos) = stack.iter().position(|&n| n == node) {
                        let mut cycle = stack[pos..].to_vec();
                        cycle.push(node);
                        return Some(cycle);
                    }
                    return Some(vec![node, node]);
                }
                2 => return None,
                _ => {}
            }

            state.insert(node, 1);
            stack.push(node);

            if let Some(children) = deps.get(node) {
                for &child in children {
                    if let Some(cycle) = dfs(child, deps, state, stack) {
                        return Some(cycle);
                    }
                }
            }

            stack.pop();
            state.insert(node, 2);
            None
        }

        let mut state: HashMap<&str, u8> = HashMap::new();
        let mut stack: Vec<&str> = Vec::new();
        for &node in deps.keys() {
            if state.get(node).copied().unwrap_or(0) == 0 {
                if let Some(cycle) = dfs(node, deps, &mut state, &mut stack) {
                    return Some(cycle);
                }
            }
        }
        None
    }

    if let Some(cycle) = find_cycle(&deps) {
        result.add_error(ValidationError::with_path(
            ErrorCode::InvalidRecipeParams,
            format!("cycle detected: {}", cycle.join(" -> ")),
            "recipe.params.nodes",
        ));
    }

    // Outputs: primary PNG outputs must declare source and refer to a node id.
    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind != OutputKind::Primary {
            continue;
        }

        if output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.procedural_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }

        let Some(source) = output.source.as_deref() else {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.procedural_v1 primary outputs must set 'source' to a node id",
                format!("outputs[{}].source", i),
            ));
            continue;
        };

        if !node_ids.contains(source) {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                format!(
                    "outputs[{}].source '{}' does not match any recipe.params.nodes[].id",
                    i, source
                ),
                format!("outputs[{}].source", i),
            ));
        }
    }
}

/// Validates outputs for `texture.trimsheet_v1` recipe.
pub(super) fn validate_texture_trimsheet_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    match recipe.as_texture_trimsheet() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            let mut seen_ids = HashSet::new();
            for (i, tile) in params.tiles.iter().enumerate() {
                if !seen_ids.insert(&tile.id) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("duplicate tile id: '{}'", tile.id),
                        format!("recipe.params.tiles[{}].id", i),
                    ));
                }
                if tile.width == 0 || tile.height == 0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' dimensions must be positive, got {}x{}",
                            tile.id, tile.width, tile.height
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }
                let padded_width = tile.width + params.padding * 2;
                let padded_height = tile.height + params.padding * 2;
                if padded_width > params.resolution[0] || padded_height > params.resolution[1] {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "tile '{}' ({}x{}) with padding {} is too large for atlas ({}x{})",
                            tile.id,
                            tile.width,
                            tile.height,
                            params.padding,
                            params.resolution[0],
                            params.resolution[1]
                        ),
                        format!("recipe.params.tiles[{}]", i),
                    ));
                }
            }
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
        }
    }

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
        if output.kind == OutputKind::Metadata && output.format != OutputFormat::Json {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.trimsheet_v1 metadata outputs must have format 'json'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

/// Validates outputs for `texture.decal_v1` recipe.
pub(super) fn validate_texture_decal_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_texture_decal() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    let mut has_albedo_output = false;
    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.decal_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }
                let source = output.source.as_deref().unwrap_or("");
                match source {
                    "" | "albedo" => has_albedo_output = true,
                    "normal" => {
                        if params.normal_output.is_none() {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                "texture.decal_v1 output requests normal map but recipe.params.normal_output is not set",
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    "roughness" => {
                        if params.roughness_output.is_none() {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                "texture.decal_v1 output requests roughness map but recipe.params.roughness_output is not set",
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    other => {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.decal_v1 output source '{}' is not supported (expected '', 'albedo', 'normal', or 'roughness')",
                                other
                            ),
                            format!("outputs[{}].source", i),
                        ));
                    }
                }
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.decal_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }
    if !has_albedo_output {
        result.add_error(ValidationError::with_path(
            ErrorCode::OutputValidationFailed,
            "texture.decal_v1 requires at least one albedo output (primary output with empty source or source 'albedo')",
            "outputs",
        ));
    }
}

/// Validates outputs for `texture.splat_set_v1` recipe.
pub(super) fn validate_texture_splat_set_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_texture_splat_set() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if params.layers.is_empty() {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    "layers must not be empty",
                    "recipe.params.layers",
                ));
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    let mut layer_ids = HashSet::new();
    for (i, layer) in params.layers.iter().enumerate() {
        if layer.id.is_empty() {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                "layer.id must not be empty",
                format!("recipe.params.layers[{}].id", i),
            ));
        }
        if !layer_ids.insert(layer.id.as_str()) {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("duplicate layer id: '{}'", layer.id),
                format!("recipe.params.layers[{}].id", i),
            ));
        }
    }

    let mask_count = params.layers.len().div_ceil(4);

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }
                let source = output.source.as_deref().unwrap_or("").trim();
                if source.is_empty() {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 primary outputs must set 'source' (e.g. 'grass.albedo', 'mask0', 'macro')",
                        format!("outputs[{}].source", i),
                    ));
                    continue;
                }
                if source == "macro" {
                    if !params.macro_variation {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            "texture.splat_set_v1 output requests macro texture but recipe.params.macro_variation is false",
                            format!("outputs[{}].source", i),
                        ));
                    }
                    continue;
                }
                if let Some(rest) = source.strip_prefix("mask") {
                    match rest.parse::<usize>() {
                        Ok(idx) if idx < mask_count => {}
                        Ok(_) => {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                format!(
                                    "texture.splat_set_v1 output requests '{}' but mask index is out of range (0..{})",
                                    source,
                                    mask_count.saturating_sub(1)
                                ),
                                format!("outputs[{}].source", i),
                            ));
                        }
                        Err(_) => {
                            result.add_error(ValidationError::with_path(
                                ErrorCode::OutputValidationFailed,
                                format!(
                                    "texture.splat_set_v1 output source '{}' is invalid (expected 'maskN')",
                                    source
                                ),
                                format!("outputs[{}].source", i),
                            ));
                        }
                    }
                    continue;
                }
                if let Some((layer_id, map_type)) = source.split_once('.') {
                    if !layer_ids.contains(layer_id) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.splat_set_v1 output source '{}' references unknown layer id '{}'",
                                source, layer_id
                            ),
                            format!("outputs[{}].source", i),
                        ));
                        continue;
                    }
                    if !matches!(map_type, "albedo" | "normal" | "roughness") {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::OutputValidationFailed,
                            format!(
                                "texture.splat_set_v1 output source '{}' has invalid map type '{}' (expected 'albedo', 'normal', or 'roughness')",
                                source, map_type
                            ),
                            format!("outputs[{}].source", i),
                        ));
                    }
                    continue;
                }
                result.add_error(ValidationError::with_path(
                    ErrorCode::OutputValidationFailed,
                    format!(
                        "texture.splat_set_v1 output source '{}' is invalid (expected '<layer>.<map>', 'maskN', or 'macro')",
                        source
                    ),
                    format!("outputs[{}].source", i),
                ));
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.splat_set_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }
}

pub(super) fn validate_texture_matcap_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    let _params = match recipe.as_texture_matcap() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if let Some(steps) = params.toon_steps {
                if !(2..=16).contains(&steps) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("toon_steps must be between 2 and 16, got {}", steps),
                        "recipe.params.toon_steps",
                    ));
                }
            }
            if let Some(ref outline) = params.outline {
                if !(1..=10).contains(&outline.width) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!(
                            "outline width must be between 1 and 10, got {}",
                            outline.width
                        ),
                        "recipe.params.outline.width",
                    ));
                }
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    for (i, output) in spec.outputs.iter().enumerate() {
        if output.kind == OutputKind::Primary && output.format != OutputFormat::Png {
            result.add_error(ValidationError::with_path(
                ErrorCode::OutputValidationFailed,
                "texture.matcap_v1 primary outputs must have format 'png'",
                format!("outputs[{}].format", i),
            ));
        }
    }
}

pub(super) fn validate_texture_material_preset_outputs(
    spec: &Spec,
    recipe: &Recipe,
    result: &mut ValidationResult,
) {
    let params = match recipe.as_texture_material_preset() {
        Ok(params) => {
            if params.resolution[0] == 0 || params.resolution[1] == 0 {
                result.add_error(ValidationError::with_path(
                    ErrorCode::InvalidRecipeParams,
                    format!(
                        "resolution must be positive, got [{}, {}]",
                        params.resolution[0], params.resolution[1]
                    ),
                    "recipe.params.resolution",
                ));
            }
            if let Some(ref color) = params.base_color {
                for (i, &c) in color.iter().enumerate() {
                    if !(0.0..=1.0).contains(&c) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("base_color[{}] must be in range [0, 1], got {}", i, c),
                            format!("recipe.params.base_color[{}]", i),
                        ));
                    }
                }
            }
            if let Some(ref range) = params.roughness_range {
                for (i, &r) in range.iter().enumerate() {
                    if !(0.0..=1.0).contains(&r) {
                        result.add_error(ValidationError::with_path(
                            ErrorCode::InvalidRecipeParams,
                            format!("roughness_range[{}] must be in range [0, 1], got {}", i, r),
                            format!("recipe.params.roughness_range[{}]", i),
                        ));
                    }
                }
            }
            if let Some(m) = params.metallic {
                if !(0.0..=1.0).contains(&m) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("metallic must be in range [0, 1], got {}", m),
                        "recipe.params.metallic",
                    ));
                }
            }
            if let Some(ns) = params.noise_scale {
                if ns <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("noise_scale must be positive, got {}", ns),
                        "recipe.params.noise_scale",
                    ));
                }
            }
            if let Some(ps) = params.pattern_scale {
                if ps <= 0.0 {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::InvalidRecipeParams,
                        format!("pattern_scale must be positive, got {}", ps),
                        "recipe.params.pattern_scale",
                    ));
                }
            }
            params
        }
        Err(e) => {
            result.add_error(ValidationError::with_path(
                ErrorCode::InvalidRecipeParams,
                format!("invalid params for {}: {}", recipe.kind, e),
                "recipe.params",
            ));
            return;
        }
    };

    validate_primary_output_present(spec, result);

    let valid_sources = ["albedo", "roughness", "metallic", "normal"];

    for (i, output) in spec.outputs.iter().enumerate() {
        match output.kind {
            OutputKind::Primary => {
                if output.format != OutputFormat::Png {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.material_preset_v1 primary outputs must have format 'png'",
                        format!("outputs[{}].format", i),
                    ));
                }
                let source = output.source.as_deref().unwrap_or("");
                if !valid_sources.contains(&source) {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        format!(
                            "texture.material_preset_v1 output source '{}' is not valid; expected 'albedo', 'roughness', 'metallic', or 'normal'",
                            source
                        ),
                        format!("outputs[{}].source", i),
                    ));
                }
            }
            OutputKind::Metadata => {
                if output.format != OutputFormat::Json {
                    result.add_error(ValidationError::with_path(
                        ErrorCode::OutputValidationFailed,
                        "texture.material_preset_v1 metadata outputs must have format 'json'",
                        format!("outputs[{}].format", i),
                    ));
                }
            }
            OutputKind::Preview => {}
        }
    }

    // Suppress unused variable warning
    let _ = params;
}
