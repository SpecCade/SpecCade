//! CLI argument definitions for the SpecCade command-line interface.
//!
//! All `#[derive(Parser)]` and `#[derive(Subcommand)]` types are defined here,
//! keeping `main.rs` focused on dispatch logic.

use clap::{Parser, Subcommand};

/// SpecCade - Declarative Asset Generation System
#[derive(Parser)]
#[command(name = "speccade")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Analyze a generated asset file and output quality metrics
    Analyze {
        /// Path to the input file to analyze (WAV or PNG)
        #[arg(short, long)]
        input: Option<String>,

        /// Path to spec file (generate then analyze)
        #[arg(short, long)]
        spec: Option<String>,

        /// Directory to recursively scan for .wav and .png files (batch mode)
        #[arg(long)]
        input_dir: Option<String>,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,

        /// Output format for batch mode (json, jsonl, csv)
        #[arg(long, default_value = "json", value_parser = ["json", "jsonl", "csv"])]
        output_format: String,

        /// Include fixed-dimension feature embedding for similarity search
        #[arg(long)]
        embeddings: bool,

        /// Start WebSocket analysis server on the specified port (default: 9123)
        #[cfg(feature = "serve")]
        #[arg(long)]
        serve: Option<Option<u16>>,
    },

    /// Compare two asset files and output perceptual difference metrics
    Compare {
        /// Path to the first file (reference)
        #[arg(short, long)]
        a: String,

        /// Path to the second file (comparison target)
        #[arg(short, long)]
        b: String,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Audit audio files for quality regressions against baselines
    Audit {
        /// Directory to scan for .wav files
        #[arg(long)]
        input_dir: String,

        /// Path to tolerances config file (JSON)
        #[arg(long)]
        tolerances: Option<String>,

        /// Update or create baseline files for each audio file
        #[arg(long)]
        update_baselines: bool,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Evaluate a spec file and print canonical IR JSON to stdout
    Eval {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Pretty-print the output JSON
        #[arg(short, long)]
        pretty: bool,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Validate a spec file without generating assets
    Validate {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Also validate artifact references (paths, formats)
        #[arg(long)]
        artifacts: bool,

        /// Budget profile to validate against (default, strict, zx-8bit, nethercore)
        #[arg(long, value_parser = ["default", "strict", "zx-8bit", "nethercore"])]
        budget: Option<String>,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Generate assets from a spec file
    Generate {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,

        /// Expand `variants[]` into separate generation runs under `{out_root}/variants/{variant_id}/`
        #[arg(long)]
        expand_variants: bool,

        /// Budget profile to validate against (default, strict, zx-8bit, nethercore)
        #[arg(long, value_parser = ["default", "strict", "zx-8bit", "nethercore"])]
        budget: Option<String>,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,

        /// Generate preview of specified duration in seconds (for fast iteration)
        #[arg(long)]
        preview: Option<f64>,

        /// Disable content-addressed caching (force regeneration)
        #[arg(long)]
        no_cache: bool,

        /// Enable per-stage timing profiling (timings included in report)
        #[arg(long)]
        profile: bool,

        /// Generate N SFX variations by incrementing seed (outputs variations.json manifest)
        #[arg(long)]
        variations: Option<u32>,

        /// Maximum peak level in dB; reject variations exceeding this (e.g., 0.0 for no clipping)
        #[arg(long, allow_hyphen_values = true)]
        max_peak_db: Option<f64>,

        /// Maximum DC offset (absolute value); reject variations exceeding this threshold
        #[arg(long)]
        max_dc_offset: Option<f64>,

        /// Force saving .blend files alongside GLB output (Blender mesh pipelines only)
        #[arg(long)]
        save_blend: bool,
    },

    /// Generate all assets from a directory of spec files
    GenerateAll {
        /// Directory containing spec files (default: ./specs)
        #[arg(short, long)]
        spec_dir: Option<String>,

        /// Output root directory (default: ./test-outputs)
        #[arg(short, long)]
        out_root: Option<String>,

        /// Include Blender-based assets (static_mesh, skeletal_mesh, skeletal_animation)
        #[arg(long)]
        include_blender: bool,

        /// Show verbose output
        #[arg(short, long)]
        verbose: bool,

        /// Force regeneration of all specs (skip freshness check)
        #[arg(short = 'f', long)]
        force: bool,
    },

    /// Preview an asset (opens in viewer/editor, or exports GIF with --gif)
    Preview {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,

        /// Export animated GIF instead of opening viewer
        #[arg(long)]
        gif: bool,

        /// Output file path for GIF (default: <asset_id>.preview.gif next to spec)
        #[arg(long)]
        out: Option<String>,

        /// Override frames per second for GIF
        #[arg(long)]
        fps: Option<u32>,

        /// Scale factor for GIF frames (default: 1)
        #[arg(long)]
        scale: Option<u32>,
    },

    /// Generate a 6-view validation grid PNG for 3D assets (FRONT, BACK, TOP, LEFT, RIGHT, ISO)
    PreviewGrid {
        /// Path to the spec file (.star or .json)
        #[arg(short, long)]
        spec: String,

        /// Output PNG path (default: test-outputs/{asset_type}/<spec_stem>.grid.png)
        #[arg(short, long)]
        out: Option<String>,

        /// Panel size in pixels for each view (default: 256, grid is 3x2 panels)
        #[arg(long, default_value = "256")]
        panel_size: u32,
    },

    /// Check system dependencies and configuration
    Doctor,

    /// Expand compose specs into canonical tracker params JSON
    Expand {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Output file path (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Pretty-print the output JSON (default: true)
        #[arg(long, default_value = "true")]
        pretty: bool,

        /// Compact output (minified JSON, overrides --pretty)
        #[arg(long)]
        compact: bool,

        /// Output machine-readable JSON envelope
        #[arg(long)]
        json: bool,
    },

    /// Inspect intermediate build artifacts (texture nodes, expanded params)
    Inspect {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Output directory for intermediate artifacts
        #[arg(short, long)]
        out_dir: String,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Format a spec file to canonical style
    Fmt {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Output file path (default: overwrite input file)
        #[arg(short, long)]
        output: Option<String>,
    },

    /// Manage built-in templates (texture kits)
    Template {
        #[command(subcommand)]
        command: TemplateCommands,
    },

    /// Inspect Starlark stdlib functions and metadata
    Stdlib {
        #[command(subcommand)]
        command: StdlibCommands,
    },

    /// Manage generation cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },

    /// Feature coverage tracking and enforcement
    Coverage {
        #[command(subcommand)]
        command: CoverageCommands,
    },

    /// Verify generated assets against constraints
    Verify {
        /// Path to the report file (*.report.json)
        #[arg(long)]
        report: String,

        /// Path to the constraints file (*.constraints.json)
        #[arg(long)]
        constraints: String,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Lint a generated asset file for semantic quality issues
    Lint {
        /// Path to the asset file to lint (WAV, PNG, GLB, XM, etc.)
        #[arg(short, long)]
        input: String,

        /// Path to the original spec file (for spec_path context in issues)
        #[arg(short, long)]
        spec: Option<String>,

        /// Treat warnings as errors (fail if any warnings)
        #[arg(long)]
        strict: bool,

        /// Disable specific lint rules (can be repeated)
        #[arg(long = "disable-rule", value_name = "RULE_ID")]
        disable_rules: Vec<String>,

        /// Only run these rules (comma-separated list)
        #[arg(long = "only-rules", value_name = "RULE_IDS")]
        only_rules: Option<String>,

        /// Output format (text or json)
        #[arg(long, default_value = "text", value_parser = ["text", "json"])]
        format: String,
    },

    /// Validate a single asset through full pipeline
    ValidateAsset {
        /// Path to spec file
        #[arg(short, long)]
        spec: String,
        /// Output directory for validation artifacts
        #[arg(short, long)]
        out_root: Option<String>,
        /// Generate full validation report
        #[arg(long)]
        full_report: bool,
    },

    /// Batch validate multiple assets
    BatchValidate {
        /// Glob pattern for spec files (e.g., "specs/character/*.star")
        #[arg(short, long)]
        specs: String,
        /// Output directory for all validation artifacts
        #[arg(short, long)]
        out_root: Option<String>,
        /// Output format for batch report
        #[arg(short, long, default_value = "json")]
        format: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum TemplateCommands {
    /// List available templates
    List {
        /// Asset type to list (texture, audio, music)
        #[arg(long, default_value = "texture")]
        asset_type: String,
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
    /// Show details for a template
    Show {
        /// Template id (asset_id)
        id: String,
        /// Asset type scope (texture, audio, music)
        #[arg(long, default_value = "texture")]
        asset_type: String,
    },
    /// Copy a template spec to a destination path
    Copy {
        /// Template id (asset_id)
        id: String,
        /// Destination path to write the template JSON
        #[arg(long)]
        to: String,
        /// Asset type scope (texture, audio, music)
        #[arg(long, default_value = "texture")]
        asset_type: String,
    },
    /// Search templates by tags or keywords
    Search {
        /// Filter by tags (comma-separated)
        #[arg(long, value_delimiter = ',')]
        tags: Option<Vec<String>>,
        /// Filter by description/asset_id keyword
        #[arg(long)]
        query: Option<String>,
        /// Filter by asset type (texture, audio, music)
        #[arg(long)]
        asset_type: Option<String>,
        /// Output machine-readable JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand)]
pub(crate) enum StdlibCommands {
    /// Dump stdlib function metadata in machine-readable format
    Dump {
        /// Output format (currently only "json" is supported)
        #[arg(long, default_value = "json", value_parser = ["json"])]
        format: String,
    },
}

#[derive(Subcommand)]
pub(crate) enum CacheCommands {
    /// Clear all cache entries
    Clear,
    /// Show cache information (entry count, total size)
    Info,
}

/// Subcommands for feature coverage tracking
#[derive(Subcommand, Debug)]
pub(crate) enum CoverageCommands {
    /// Generate coverage report (writes YAML)
    Generate {
        /// Fail if coverage < 100% (CI mode)
        #[arg(long)]
        strict: bool,

        /// Output YAML path (default: docs/coverage/feature-coverage.yaml)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Print coverage summary to stdout
    Report,
}
