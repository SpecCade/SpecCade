//! Texture inspection helpers for texture.procedural_v1 specs.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::super::json_output::IntermediateFile;

/// Inspect texture.procedural_v1 spec - generate per-node intermediate PNGs
pub fn inspect_texture_procedural(
    spec: &speccade_spec::Spec,
    intermediates_dir: &Path,
    out_dir: &str,
) -> Result<(Vec<IntermediateFile>, Vec<IntermediateFile>)> {
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Spec has no recipe"))?;

    let params = recipe
        .as_texture_procedural()
        .with_context(|| "Invalid texture.procedural_v1 params")?;

    // Generate all nodes
    let nodes = speccade_backend_texture::generate_graph(&params, spec.seed)
        .with_context(|| "Failed to generate texture graph")?;

    // Write each node as an intermediate PNG
    let mut intermediates = Vec::new();
    let mut node_ids: Vec<_> = nodes.keys().collect();
    node_ids.sort(); // Stable ordering

    for node_id in node_ids {
        let value = &nodes[node_id];
        let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
            .with_context(|| format!("Failed to encode node '{}' as PNG", node_id))?;

        let filename = format!("{}.png", node_id);
        let path = intermediates_dir.join(&filename);
        fs::write(&path, &png_data)
            .with_context(|| format!("Failed to write intermediate: {}", path.display()))?;

        let rel_path = format!("intermediates/{}", filename);
        intermediates.push(IntermediateFile {
            id: node_id.clone(),
            format: "png".to_string(),
            path: rel_path,
            hash: Some(hash),
        });
    }

    // Generate final outputs as specified in the spec
    let mut final_outputs = Vec::new();
    for output_spec in &spec.outputs {
        if let Some(ref source) = output_spec.source {
            if let Some(value) = nodes.get(source) {
                let (png_data, hash) = speccade_backend_texture::encode_graph_value_png(value)
                    .with_context(|| format!("Failed to encode output '{}'", source))?;

                let output_path = Path::new(out_dir).join(&output_spec.path);
                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(&output_path, &png_data)
                    .with_context(|| format!("Failed to write output: {}", output_spec.path))?;

                final_outputs.push(IntermediateFile {
                    id: source.clone(),
                    format: "png".to_string(),
                    path: output_spec.path.clone(),
                    hash: Some(hash),
                });
            }
        }
    }

    Ok((intermediates, final_outputs))
}
