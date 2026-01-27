//! Generate command implementation
//!
//! Generates assets from a spec file using the appropriate backend.

mod human;
mod json;
pub mod quality;
mod variations;

#[cfg(test)]
mod tests;

use anyhow::Result;
use std::process::ExitCode;

pub use quality::QualityConstraints;

/// Run the generate command
///
/// # Arguments
/// * `spec_path` - Path to the spec file (JSON or Starlark)
/// * `out_root` - Output root directory (default: current directory)
/// * `expand_variants` - Whether to expand `spec.variants[]` during generation
/// * `budget_name` - Optional budget profile name (default, strict, zx-8bit)
/// * `json_output` - Whether to output machine-readable JSON diagnostics
/// * `preview_duration` - Optional preview duration in seconds (truncates audio generation)
/// * `no_cache` - Whether to bypass cache (default: false, cache enabled)
/// * `profile` - Whether to include per-stage timing in the report
/// * `variations` - Optional number of SFX variations to generate
/// * `max_peak_db` - Optional maximum peak level in dB for variation quality gating
/// * `max_dc_offset` - Optional maximum DC offset for variation quality gating
/// * `save_blend` - Force saving .blend files alongside GLB output
///
/// # Returns
/// Exit code: 0 success, 1 spec error, 2 generation error
#[allow(clippy::too_many_arguments)]
pub fn run(
    spec_path: &str,
    out_root: Option<&str>,
    expand_variants: bool,
    budget_name: Option<&str>,
    json_output: bool,
    preview_duration: Option<f64>,
    no_cache: bool,
    profile: bool,
    variations: Option<u32>,
    max_peak_db: Option<f64>,
    max_dc_offset: Option<f64>,
    save_blend: bool,
) -> Result<ExitCode> {
    let constraints = QualityConstraints::from_options(max_peak_db, max_dc_offset);

    if json_output {
        json::run_json(
            spec_path,
            out_root,
            expand_variants,
            budget_name,
            preview_duration,
            no_cache,
            profile,
            variations,
            constraints,
            save_blend,
        )
    } else {
        human::run_human(
            spec_path,
            out_root,
            expand_variants,
            budget_name,
            preview_duration,
            no_cache,
            profile,
            variations,
            constraints,
            save_blend,
        )
    }
}
