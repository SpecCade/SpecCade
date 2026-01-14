//! Texture backend dispatch handler

use super::{write_output_bytes, DispatchError};
use speccade_spec::{OutputFormat, OutputKind, OutputResult, Spec};
use std::path::{Path, PathBuf};

/// Generate procedural texture outputs using the texture backend.
pub(super) fn generate_texture_procedural(
    spec: &Spec,
    out_root: &Path,
) -> Result<Vec<OutputResult>, DispatchError> {
    let recipe = spec.recipe.as_ref().ok_or(DispatchError::NoRecipe)?;
    let params = recipe.as_texture_procedural().map_err(|e| {
        DispatchError::BackendError(format!("Invalid texture procedural params: {}", e))
    })?;

    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed).map_err(|e| {
        DispatchError::BackendError(format!("Procedural texture generation failed: {}", e))
    })?;

    let primary_outputs: Vec<(usize, &speccade_spec::OutputSpec)> = spec
        .outputs
        .iter()
        .enumerate()
        .filter(|(_, o)| o.kind == OutputKind::Primary)
        .collect();

    if primary_outputs.is_empty() {
        return Err(DispatchError::BackendError(
            "texture.procedural_v1 requires at least one output of kind 'primary'".to_string(),
        ));
    }

    let mut outputs = Vec::with_capacity(primary_outputs.len());

    for (output_index, output_spec) in primary_outputs {
        if output_spec.format != OutputFormat::Png {
            return Err(DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must have format 'png' (outputs[{}].format)",
                output_index
            )));
        }

        let source = output_spec.source.as_ref().ok_or_else(|| {
            DispatchError::BackendError(format!(
                "texture.procedural_v1 primary outputs must set 'source' (outputs[{}].source)",
                output_index
            ))
        })?;

        let value = nodes.get(source).ok_or_else(|| {
            DispatchError::BackendError(format!(
                "outputs[{}].source '{}' does not match any node id",
                output_index, source
            ))
        })?;

        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .map_err(|e| DispatchError::BackendError(format!("PNG encoding failed: {}", e)))?;

        write_output_bytes(out_root, &output_spec.path, &png_data)?;

        outputs.push(OutputResult::tier1(
            output_spec.kind,
            OutputFormat::Png,
            PathBuf::from(&output_spec.path),
            hash,
        ));
    }

    Ok(outputs)
}
