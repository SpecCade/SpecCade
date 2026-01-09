//! SpecCade CLI - Command-line interface for declarative asset generation
//!
//! This binary provides commands for validating, generating, and managing
//! SpecCade specs and their generated assets.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod commands;
mod dispatch;

/// SpecCade - Declarative Asset Generation System
#[derive(Parser)]
#[command(name = "speccade")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a spec file without generating assets
    Validate {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Also validate artifact references (paths, formats)
        #[arg(long)]
        artifacts: bool,
    },

    /// Generate assets from a spec file
    Generate {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,
    },

    /// Preview an asset (opens in viewer/editor)
    Preview {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Output root directory (default: current directory)
        #[arg(short, long)]
        out_root: Option<String>,
    },

    /// Check system dependencies and configuration
    Doctor,

    /// Migrate legacy .spec.py files to canonical JSON format
    Migrate {
        /// Path to the project directory containing legacy specs
        #[arg(short, long)]
        project: String,
    },

    /// Format a spec file to canonical style
    Fmt {
        /// Path to the spec JSON file
        #[arg(short, long)]
        spec: String,

        /// Output file path (default: overwrite input file)
        #[arg(short, long)]
        output: Option<String>,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Validate { spec, artifacts } => commands::validate::run(&spec, artifacts),
        Commands::Generate { spec, out_root } => commands::generate::run(&spec, out_root.as_deref()),
        Commands::Preview { spec, out_root } => commands::preview::run(&spec, out_root.as_deref()),
        Commands::Doctor => commands::doctor::run(),
        Commands::Migrate { project } => commands::migrate::run(&project),
        Commands::Fmt { spec, output } => commands::fmt::run(&spec, output.as_deref()),
    };

    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}: {}", colored::Colorize::red("error"), e);
            ExitCode::from(1)
        }
    }
}
