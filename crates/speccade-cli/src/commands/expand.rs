//! Expand command implementation
//!
//! Expands Pattern IR compose specs into canonical tracker params JSON.

use anyhow::{bail, Context, Result};
use speccade_spec::Spec;
use std::fs;
use std::process::ExitCode;

/// Run the expand command.
pub fn run(spec_path: &str) -> Result<ExitCode> {
    // Read spec file
    let spec_content = fs::read_to_string(spec_path)
        .with_context(|| format!("Failed to read spec file: {}", spec_path))?;

    // Parse spec
    let spec =
        Spec::from_json(&spec_content).with_context(|| format!("Failed to parse {}", spec_path))?;

    let recipe = spec
        .recipe
        .as_ref()
        .with_context(|| "Spec is missing recipe".to_string())?;

    match recipe.kind.as_str() {
        "music.tracker_song_compose_v1" => {
            let params = recipe
                .as_music_tracker_song_compose()
                .with_context(|| format!("Invalid compose params for {}", recipe.kind))?;
            let expanded = speccade_backend_music::expand_compose(&params, spec.seed)
                .with_context(|| "Compose expansion failed".to_string())?;
            let json = serde_json::to_string_pretty(&expanded)?;
            println!("{}", json);
            Ok(ExitCode::SUCCESS)
        }
        _ => bail!(
            "expand is only supported for music.tracker_song_compose_v1 (got {})",
            recipe.kind
        ),
    }
}
