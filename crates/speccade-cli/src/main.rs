//! SpecCade CLI - Command-line interface for declarative asset generation
//!
//! This binary provides commands for validating, generating, and managing
//! SpecCade specs and their generated assets.

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod commands;
mod dispatch;
pub mod parity_data;
pub mod parity_matrix;

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

        /// Allow execution of Python specs (UNSAFE: only use with trusted files)
        #[arg(long)]
        allow_exec_specs: bool,

        /// Audit mode: scan specs and report completeness without migrating
        #[arg(long)]
        audit: bool,

        /// Minimum completeness threshold for audit mode (0.0-1.0, default 0.90)
        #[arg(long, default_value = "0.90")]
        audit_threshold: f64,
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
        Commands::Migrate { project, allow_exec_specs, audit, audit_threshold } => {
            if audit {
                commands::migrate::run_audit(&project, allow_exec_specs, audit_threshold)
            } else {
                commands::migrate::run(&project, allow_exec_specs)
            }
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses_validate() {
        let cli = Cli::try_parse_from(["speccade", "validate", "--spec", "spec.json"]).unwrap();
        match cli.command {
            Commands::Validate { spec, artifacts } => {
                assert_eq!(spec, "spec.json");
                assert!(!artifacts);
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn test_cli_parses_validate_with_artifacts() {
        let cli = Cli::try_parse_from([
            "speccade",
            "validate",
            "--spec",
            "spec.json",
            "--artifacts",
        ])
        .unwrap();
        match cli.command {
            Commands::Validate { spec, artifacts } => {
                assert_eq!(spec, "spec.json");
                assert!(artifacts);
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate() {
        let cli = Cli::try_parse_from([
            "speccade",
            "generate",
            "--spec",
            "spec.json",
            "--out-root",
            "out",
        ])
        .unwrap();
        match cli.command {
            Commands::Generate { spec, out_root } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_root.as_deref(), Some("out"));
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_requires_spec_for_generate() {
        let err = Cli::try_parse_from(["speccade", "generate"]).err().unwrap();
        assert!(err.to_string().contains("--spec"));
    }

    #[test]
    fn test_cli_parses_migrate_basic() {
        let cli = Cli::try_parse_from([
            "speccade",
            "migrate",
            "--project",
            "/path/to/project",
        ])
        .unwrap();
        match cli.command {
            Commands::Migrate { project, allow_exec_specs, audit, audit_threshold } => {
                assert_eq!(project, "/path/to/project");
                assert!(!allow_exec_specs);
                assert!(!audit);
                assert!((audit_threshold - 0.90).abs() < 0.001);
            }
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn test_cli_parses_migrate_with_audit() {
        let cli = Cli::try_parse_from([
            "speccade",
            "migrate",
            "--project",
            "/path/to/project",
            "--audit",
        ])
        .unwrap();
        match cli.command {
            Commands::Migrate { project, allow_exec_specs, audit, audit_threshold } => {
                assert_eq!(project, "/path/to/project");
                assert!(!allow_exec_specs);
                assert!(audit);
                assert!((audit_threshold - 0.90).abs() < 0.001);
            }
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn test_cli_parses_migrate_with_audit_threshold() {
        let cli = Cli::try_parse_from([
            "speccade",
            "migrate",
            "--project",
            "/path/to/project",
            "--audit",
            "--audit-threshold",
            "0.75",
        ])
        .unwrap();
        match cli.command {
            Commands::Migrate { project, allow_exec_specs, audit, audit_threshold } => {
                assert_eq!(project, "/path/to/project");
                assert!(!allow_exec_specs);
                assert!(audit);
                assert!((audit_threshold - 0.75).abs() < 0.001);
            }
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn test_cli_parses_migrate_with_exec_specs() {
        let cli = Cli::try_parse_from([
            "speccade",
            "migrate",
            "--project",
            "/path/to/project",
            "--allow-exec-specs",
        ])
        .unwrap();
        match cli.command {
            Commands::Migrate { project, allow_exec_specs, audit, audit_threshold } => {
                assert_eq!(project, "/path/to/project");
                assert!(allow_exec_specs);
                assert!(!audit);
                assert!((audit_threshold - 0.90).abs() < 0.001);
            }
            _ => panic!("expected migrate command"),
        }
    }
}
