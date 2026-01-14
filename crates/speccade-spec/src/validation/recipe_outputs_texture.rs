//! Texture recipe output validation.

use std::collections::{HashMap, HashSet};

use crate::error::{ErrorCode, ValidationError, ValidationResult};
use crate::output::{OutputFormat, OutputKind};
use crate::spec::Spec;

use super::recipe_outputs::validate_primary_output_present;

// Legacy texture recipe validations removed in favor of texture.procedural_v1.
pub(super) fn validate_texture_procedural_outputs(
    spec: &Spec,
    recipe: &crate::recipe::Recipe,
    result: &mut ValidationResult,
) {
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
            | TextureProceduralOp::ToGrayscale { .. } => GraphValueType::Grayscale,
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
