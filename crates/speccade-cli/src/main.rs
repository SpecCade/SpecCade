//! SpecCade CLI - Command-line interface for declarative asset generation
//!
//! This binary provides commands for validating, generating, and managing
//! SpecCade specs and their generated assets.

use clap::{Parser, Subcommand};
use std::path::Path;
use std::process::ExitCode;

// Use modules from the library crate
use speccade_cli::commands;

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

        /// Budget profile to validate against (default, strict, zx-8bit)
        #[arg(long, value_parser = ["default", "strict", "zx-8bit"])]
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

        /// Budget profile to validate against (default, strict, zx-8bit)
        #[arg(long, value_parser = ["default", "strict", "zx-8bit"])]
        budget: Option<String>,

        /// Output machine-readable JSON diagnostics (no colored output)
        #[arg(long)]
        json: bool,
    },

    /// Generate all assets from a directory of spec files
    GenerateAll {
        /// Directory containing spec files (default: ./golden/speccade/specs)
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

    /// Expand compose specs into canonical tracker params JSON
    Expand {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,
    },

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
}

#[derive(Subcommand)]
enum TemplateCommands {
    /// List available templates
    List {
        /// Asset type to list (default: texture)
        #[arg(long, default_value = "texture")]
        asset_type: String,
    },
    /// Show details for a template
    Show {
        /// Template id (asset_id)
        id: String,
        /// Asset type scope (default: texture)
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
        /// Asset type scope (default: texture)
        #[arg(long, default_value = "texture")]
        asset_type: String,
    },
}

#[derive(Subcommand)]
enum StdlibCommands {
    /// Dump stdlib function metadata in machine-readable format
    Dump {
        /// Output format (currently only "json" is supported)
        #[arg(long, default_value = "json", value_parser = ["json"])]
        format: String,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Eval { spec, pretty, json } => commands::eval::run(&spec, pretty, json),
        Commands::Validate {
            spec,
            artifacts,
            budget,
            json,
        } => commands::validate::run(&spec, artifacts, budget.as_deref(), json),
        Commands::Generate {
            spec,
            out_root,
            expand_variants,
            budget,
            json,
        } => commands::generate::run(
            &spec,
            out_root.as_deref(),
            expand_variants,
            budget.as_deref(),
            json,
        ),
        Commands::GenerateAll {
            spec_dir,
            out_root,
            include_blender,
            verbose,
        } => commands::generate_all::run(
            spec_dir.as_deref(),
            out_root.as_deref(),
            include_blender,
            verbose,
        ),
        Commands::Preview { spec, out_root } => commands::preview::run(&spec, out_root.as_deref()),
        Commands::Doctor => commands::doctor::run(),
        Commands::Expand { spec } => commands::expand::run(&spec),
        Commands::Migrate {
            project,
            allow_exec_specs,
            audit,
            audit_threshold,
        } => {
            if audit {
                commands::migrate::run_audit(&project, allow_exec_specs, audit_threshold)
            } else {
                commands::migrate::run(&project, allow_exec_specs)
            }
        }
        Commands::Fmt { spec, output } => commands::fmt::run(&spec, output.as_deref()),
        Commands::Template { command } => match command {
            TemplateCommands::List { asset_type } => commands::template::list(&asset_type),
            TemplateCommands::Show { id, asset_type } => commands::template::show(&asset_type, &id),
            TemplateCommands::Copy { id, to, asset_type } => {
                commands::template::copy(&asset_type, &id, Path::new(&to))
            }
        },
        Commands::Stdlib { command } => match command {
            StdlibCommands::Dump { format } => {
                let dump_format = format
                    .parse::<commands::stdlib::DumpFormat>()
                    .expect("clap should have validated format");
                commands::stdlib::run_dump(dump_format)
            }
        },
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
    fn test_cli_parses_eval() {
        let cli = Cli::try_parse_from(["speccade", "eval", "--spec", "spec.star"]).unwrap();
        match cli.command {
            Commands::Eval { spec, pretty, json } => {
                assert_eq!(spec, "spec.star");
                assert!(!pretty);
                assert!(!json);
            }
            _ => panic!("expected eval command"),
        }
    }

    #[test]
    fn test_cli_parses_eval_with_pretty() {
        let cli =
            Cli::try_parse_from(["speccade", "eval", "--spec", "spec.star", "--pretty"]).unwrap();
        match cli.command {
            Commands::Eval { spec, pretty, json } => {
                assert_eq!(spec, "spec.star");
                assert!(pretty);
                assert!(!json);
            }
            _ => panic!("expected eval command"),
        }
    }

    #[test]
    fn test_cli_parses_eval_with_json() {
        let cli =
            Cli::try_parse_from(["speccade", "eval", "--spec", "spec.star", "--json"]).unwrap();
        match cli.command {
            Commands::Eval { spec, pretty, json } => {
                assert_eq!(spec, "spec.star");
                assert!(!pretty);
                assert!(json);
            }
            _ => panic!("expected eval command"),
        }
    }

    #[test]
    fn test_cli_parses_validate() {
        let cli = Cli::try_parse_from(["speccade", "validate", "--spec", "spec.json"]).unwrap();
        match cli.command {
            Commands::Validate {
                spec,
                artifacts,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(!artifacts);
                assert!(budget.is_none());
                assert!(!json);
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn test_cli_parses_validate_with_artifacts() {
        let cli =
            Cli::try_parse_from(["speccade", "validate", "--spec", "spec.json", "--artifacts"])
                .unwrap();
        match cli.command {
            Commands::Validate {
                spec,
                artifacts,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(artifacts);
                assert!(budget.is_none());
                assert!(!json);
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn test_cli_parses_validate_with_budget() {
        let cli = Cli::try_parse_from([
            "speccade",
            "validate",
            "--spec",
            "spec.json",
            "--budget",
            "strict",
        ])
        .unwrap();
        match cli.command {
            Commands::Validate {
                spec,
                artifacts,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(!artifacts);
                assert_eq!(budget.as_deref(), Some("strict"));
                assert!(!json);
            }
            _ => panic!("expected validate command"),
        }
    }

    #[test]
    fn test_cli_parses_validate_with_json() {
        let cli =
            Cli::try_parse_from(["speccade", "validate", "--spec", "spec.json", "--json"]).unwrap();
        match cli.command {
            Commands::Validate {
                spec,
                artifacts,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(!artifacts);
                assert!(budget.is_none());
                assert!(json);
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
            Commands::Generate {
                spec,
                out_root,
                expand_variants,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_root.as_deref(), Some("out"));
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_with_budget() {
        let cli = Cli::try_parse_from([
            "speccade",
            "generate",
            "--spec",
            "spec.json",
            "--budget",
            "zx-8bit",
        ])
        .unwrap();
        match cli.command {
            Commands::Generate {
                spec,
                out_root,
                expand_variants,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert_eq!(budget.as_deref(), Some("zx-8bit"));
                assert!(!json);
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_with_json() {
        let cli =
            Cli::try_parse_from(["speccade", "generate", "--spec", "spec.json", "--json"]).unwrap();
        match cli.command {
            Commands::Generate {
                spec,
                out_root,
                expand_variants,
                budget,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(json);
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_expand() {
        let cli = Cli::try_parse_from(["speccade", "expand", "--spec", "spec.json"]).unwrap();
        match cli.command {
            Commands::Expand { spec } => {
                assert_eq!(spec, "spec.json");
            }
            _ => panic!("expected expand command"),
        }
    }

    #[test]
    fn test_cli_requires_spec_for_generate() {
        let err = Cli::try_parse_from(["speccade", "generate"]).err().unwrap();
        assert!(err.to_string().contains("--spec"));
    }

    #[test]
    fn test_cli_parses_migrate_basic() {
        let cli =
            Cli::try_parse_from(["speccade", "migrate", "--project", "/path/to/project"]).unwrap();
        match cli.command {
            Commands::Migrate {
                project,
                allow_exec_specs,
                audit,
                audit_threshold,
            } => {
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
            Commands::Migrate {
                project,
                allow_exec_specs,
                audit,
                audit_threshold,
            } => {
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
            Commands::Migrate {
                project,
                allow_exec_specs,
                audit,
                audit_threshold,
            } => {
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
            Commands::Migrate {
                project,
                allow_exec_specs,
                audit,
                audit_threshold,
            } => {
                assert_eq!(project, "/path/to/project");
                assert!(allow_exec_specs);
                assert!(!audit);
                assert!((audit_threshold - 0.90).abs() < 0.001);
            }
            _ => panic!("expected migrate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_all_defaults() {
        let cli = Cli::try_parse_from(["speccade", "generate-all"]).unwrap();
        match cli.command {
            Commands::GenerateAll {
                spec_dir,
                out_root,
                include_blender,
                verbose,
            } => {
                assert!(spec_dir.is_none());
                assert!(out_root.is_none());
                assert!(!include_blender);
                assert!(!verbose);
            }
            _ => panic!("expected generate-all command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_all_with_options() {
        let cli = Cli::try_parse_from([
            "speccade",
            "generate-all",
            "--spec-dir",
            "/path/to/specs",
            "--out-root",
            "/path/to/output",
            "--include-blender",
            "--verbose",
        ])
        .unwrap();
        match cli.command {
            Commands::GenerateAll {
                spec_dir,
                out_root,
                include_blender,
                verbose,
            } => {
                assert_eq!(spec_dir.as_deref(), Some("/path/to/specs"));
                assert_eq!(out_root.as_deref(), Some("/path/to/output"));
                assert!(include_blender);
                assert!(verbose);
            }
            _ => panic!("expected generate-all command"),
        }
    }

    #[test]
    fn test_cli_parses_template_list() {
        let cli = Cli::try_parse_from(["speccade", "template", "list", "--asset-type", "texture"])
            .unwrap();
        match cli.command {
            Commands::Template { command } => match command {
                TemplateCommands::List { asset_type } => {
                    assert_eq!(asset_type, "texture");
                }
                _ => panic!("expected template list"),
            },
            _ => panic!("expected template command"),
        }
    }

    #[test]
    fn test_cli_parses_template_show() {
        let cli =
            Cli::try_parse_from(["speccade", "template", "show", "preset_texture_basic"]).unwrap();
        match cli.command {
            Commands::Template { command } => match command {
                TemplateCommands::Show { id, asset_type } => {
                    assert_eq!(id, "preset_texture_basic");
                    assert_eq!(asset_type, "texture");
                }
                _ => panic!("expected template show"),
            },
            _ => panic!("expected template command"),
        }
    }

    #[test]
    fn test_cli_parses_template_copy() {
        let cli = Cli::try_parse_from([
            "speccade",
            "template",
            "copy",
            "preset_texture_basic",
            "--to",
            "out.json",
        ])
        .unwrap();
        match cli.command {
            Commands::Template { command } => match command {
                TemplateCommands::Copy { id, to, asset_type } => {
                    assert_eq!(id, "preset_texture_basic");
                    assert_eq!(to, "out.json");
                    assert_eq!(asset_type, "texture");
                }
                _ => panic!("expected template copy"),
            },
            _ => panic!("expected template command"),
        }
    }

    #[test]
    fn test_cli_parses_stdlib_dump() {
        let cli = Cli::try_parse_from(["speccade", "stdlib", "dump"]).unwrap();
        match cli.command {
            Commands::Stdlib { command } => match command {
                StdlibCommands::Dump { format } => {
                    assert_eq!(format, "json");
                }
            },
            _ => panic!("expected stdlib command"),
        }
    }

    #[test]
    fn test_cli_parses_stdlib_dump_with_format() {
        let cli = Cli::try_parse_from(["speccade", "stdlib", "dump", "--format", "json"]).unwrap();
        match cli.command {
            Commands::Stdlib { command } => match command {
                StdlibCommands::Dump { format } => {
                    assert_eq!(format, "json");
                }
            },
            _ => panic!("expected stdlib command"),
        }
    }
}
