//! Compose inspection helpers for music.tracker_song_compose_v1 specs.

use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

use super::super::json_output::IntermediateFile;

/// Inspect music.tracker_song_compose_v1 spec - expand to canonical params JSON
pub fn inspect_compose(
    spec: &speccade_spec::Spec,
    intermediates_dir: &Path,
    _out_dir: &str,
) -> Result<(Vec<IntermediateFile>, String, Vec<IntermediateFile>)> {
    let recipe = spec
        .recipe
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Spec has no recipe"))?;

    let params = recipe
        .as_music_tracker_song_compose()
        .with_context(|| "Invalid music.tracker_song_compose_v1 params")?;

    // Expand compose to canonical tracker params
    let expanded = speccade_backend_music::expand_compose(&params, spec.seed)
        .with_context(|| "Failed to expand compose")?;

    // Write expanded params to JSON
    let expanded_json = serde_json::to_string_pretty(&expanded)?;
    let expanded_filename = "expanded_params.json";
    let expanded_path = intermediates_dir.join(expanded_filename);
    fs::write(&expanded_path, &expanded_json).with_context(|| "Failed to write expanded params")?;

    // Calculate hash of expanded params
    let hash = blake3::hash(expanded_json.as_bytes()).to_hex().to_string();

    let rel_path = format!("intermediates/{}", expanded_filename);
    let intermediates = vec![IntermediateFile {
        id: "expanded_params".to_string(),
        format: "json".to_string(),
        path: rel_path.clone(),
        hash: Some(hash),
    }];

    // For compose specs, there are no additional final outputs at this stage
    // (the actual music file generation would happen via `generate`)
    let final_outputs = Vec::new();

    Ok((intermediates, rel_path, final_outputs))
}
