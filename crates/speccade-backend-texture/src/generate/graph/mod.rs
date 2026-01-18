//! Map-agnostic procedural texture generator.

use std::collections::HashMap;

use speccade_spec::recipe::texture::TextureProceduralV1Params;

use crate::maps::{GrayscaleBuffer, TextureBuffer};

use super::GenerateError;

mod encoding;
mod filters;
mod helpers;
mod operations;
mod ops_color;
mod ops_filter;
mod ops_math;
mod ops_primitive;

#[cfg(test)]
mod tests;

// Re-export public API
pub use encoding::encode_graph_value_png;

/// A graph node's evaluated value.
#[derive(Debug, Clone)]
pub enum GraphValue {
    Grayscale(GrayscaleBuffer),
    Color(TextureBuffer),
}

impl GraphValue {
    pub fn as_grayscale(&self) -> Option<&GrayscaleBuffer> {
        match self {
            GraphValue::Grayscale(v) => Some(v),
            GraphValue::Color(_) => None,
        }
    }

    pub fn as_color(&self) -> Option<&TextureBuffer> {
        match self {
            GraphValue::Color(v) => Some(v),
            GraphValue::Grayscale(_) => None,
        }
    }
}

/// Generate all nodes for a `texture.procedural_v1` recipe.
pub fn generate_graph(
    params: &TextureProceduralV1Params,
    seed: u32,
) -> Result<HashMap<String, GraphValue>, GenerateError> {
    use super::helpers::validate_resolution;
    use operations::eval_node;
    use std::collections::HashSet;

    let [width, height] = params.resolution;
    validate_resolution(width, height)?;

    if params.nodes.is_empty() {
        return Err(GenerateError::InvalidParameter(
            "texture.procedural_v1 requires at least 1 node".to_string(),
        ));
    }

    let mut nodes_by_id: HashMap<&str, &speccade_spec::recipe::texture::TextureProceduralNode> =
        HashMap::new();
    for node in &params.nodes {
        if nodes_by_id.insert(node.id.as_str(), node).is_some() {
            return Err(GenerateError::InvalidParameter(format!(
                "duplicate node id: '{}'",
                node.id
            )));
        }
    }

    let mut cache: HashMap<&str, GraphValue> = HashMap::new();
    let mut visiting: HashSet<&str> = HashSet::new();

    // Evaluate everything (small graphs; keeps output binding simple).
    let node_ids: Vec<&str> = nodes_by_id.keys().copied().collect();
    for node_id in node_ids {
        eval_node(
            node_id,
            &nodes_by_id,
            &mut cache,
            &mut visiting,
            width,
            height,
            params.tileable,
            seed,
        )?;
    }

    Ok(cache.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}
