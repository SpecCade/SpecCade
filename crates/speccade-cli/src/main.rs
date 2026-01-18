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

    /// Manage generation cache
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand)]
enum TemplateCommands {
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
enum StdlibCommands {
    /// Dump stdlib function metadata in machine-readable format
    Dump {
        /// Output format (currently only "json" is supported)
        #[arg(long, default_value = "json", value_parser = ["json"])]
        format: String,
    },
}

#[derive(Subcommand)]
enum CacheCommands {
    /// Clear all cache entries
    Clear,
    /// Show cache information (entry count, total size)
    Info,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Analyze {
            input,
            spec,
            input_dir,
            output,
            json,
            output_format,
            embeddings,
        } => commands::analyze::run(
            input.as_deref(),
            spec.as_deref(),
            input_dir.as_deref(),
            output.as_deref(),
            json,
            &output_format,
            embeddings,
        ),
        Commands::Audit {
            input_dir,
            tolerances,
            update_baselines,
            json,
        } => commands::audit::run(&input_dir, tolerances.as_deref(), update_baselines, json),
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
            preview,
            no_cache,
            profile,
        } => commands::generate::run(
            &spec,
            out_root.as_deref(),
            expand_variants,
            budget.as_deref(),
            json,
            preview,
            no_cache,
            profile,
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
        Commands::Expand {
            spec,
            output,
            pretty,
            compact,
            json,
        } => commands::expand::run(&spec, output.as_deref(), pretty && !compact, json),
        Commands::Inspect {
            spec,
            out_dir,
            json,
        } => commands::inspect::run(&spec, &out_dir, json),
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
            TemplateCommands::List { asset_type, json } => {
                commands::template::list(&asset_type, json)
            }
            TemplateCommands::Show { id, asset_type } => commands::template::show(&asset_type, &id),
            TemplateCommands::Copy { id, to, asset_type } => {
                commands::template::copy(&asset_type, &id, Path::new(&to))
            }
            TemplateCommands::Search {
                tags,
                query,
                asset_type,
                json,
            } => commands::template::search(tags, query, asset_type, json),
        },
        Commands::Stdlib { command } => match command {
            StdlibCommands::Dump { format } => {
                let dump_format = format
                    .parse::<commands::stdlib::DumpFormat>()
                    .expect("clap should have validated format");
                commands::stdlib::run_dump(dump_format)
            }
        },
        Commands::Cache { command } => match command {
            CacheCommands::Clear => commands::cache::clear(),
            CacheCommands::Info => commands::cache::info(),
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
                preview,
                no_cache,
                profile,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_root.as_deref(), Some("out"));
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
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
                preview,
                no_cache,
                profile,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert_eq!(budget.as_deref(), Some("zx-8bit"));
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
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
                preview,
                no_cache,
                profile,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_with_profile() {
        let cli = Cli::try_parse_from(["speccade", "generate", "--spec", "spec.json", "--profile"])
            .unwrap();
        match cli.command {
            Commands::Generate {
                spec,
                out_root,
                expand_variants,
                budget,
                json,
                preview,
                no_cache,
                profile,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(profile);
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_expand() {
        let cli = Cli::try_parse_from(["speccade", "expand", "--spec", "spec.json"]).unwrap();
        match cli.command {
            Commands::Expand {
                spec,
                output,
                pretty,
                compact,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(output.is_none());
                assert!(pretty); // default is true
                assert!(!compact);
                assert!(!json);
            }
            _ => panic!("expected expand command"),
        }
    }

    #[test]
    fn test_cli_parses_expand_with_output() {
        let cli = Cli::try_parse_from([
            "speccade",
            "expand",
            "--spec",
            "spec.json",
            "--output",
            "out.json",
        ])
        .unwrap();
        match cli.command {
            Commands::Expand {
                spec,
                output,
                pretty,
                compact,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(output.as_deref(), Some("out.json"));
                assert!(pretty);
                assert!(!compact);
                assert!(!json);
            }
            _ => panic!("expected expand command"),
        }
    }

    #[test]
    fn test_cli_parses_expand_with_compact() {
        let cli = Cli::try_parse_from(["speccade", "expand", "--spec", "spec.json", "--compact"])
            .unwrap();
        match cli.command {
            Commands::Expand {
                spec,
                output,
                pretty,
                compact,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(output.is_none());
                assert!(pretty); // still true, but compact overrides
                assert!(compact);
                assert!(!json);
            }
            _ => panic!("expected expand command"),
        }
    }

    #[test]
    fn test_cli_parses_expand_with_json() {
        let cli =
            Cli::try_parse_from(["speccade", "expand", "--spec", "spec.json", "--json"]).unwrap();
        match cli.command {
            Commands::Expand {
                spec,
                output,
                pretty,
                compact,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(output.is_none());
                assert!(pretty);
                assert!(!compact);
                assert!(json);
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
                TemplateCommands::List { asset_type, json } => {
                    assert_eq!(asset_type, "texture");
                    assert!(!json);
                }
                _ => panic!("expected template list"),
            },
            _ => panic!("expected template command"),
        }
    }

    #[test]
    fn test_cli_parses_template_list_json() {
        let cli = Cli::try_parse_from([
            "speccade",
            "template",
            "list",
            "--asset-type",
            "audio",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Template { command } => match command {
                TemplateCommands::List { asset_type, json } => {
                    assert_eq!(asset_type, "audio");
                    assert!(json);
                }
                _ => panic!("expected template list"),
            },
            _ => panic!("expected template command"),
        }
    }

    #[test]
    fn test_cli_parses_template_search() {
        let cli = Cli::try_parse_from([
            "speccade", "template", "search", "--tags", "kick,808", "--query", "bass", "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Template { command } => match command {
                TemplateCommands::Search {
                    tags,
                    query,
                    asset_type,
                    json,
                } => {
                    assert_eq!(tags, Some(vec!["kick".to_string(), "808".to_string()]));
                    assert_eq!(query, Some("bass".to_string()));
                    assert!(asset_type.is_none());
                    assert!(json);
                }
                _ => panic!("expected template search"),
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

    #[test]
    fn test_cli_parses_analyze_with_input() {
        let cli = Cli::try_parse_from(["speccade", "analyze", "--input", "sound.wav"]).unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert_eq!(input.as_deref(), Some("sound.wav"));
                assert!(spec.is_none());
                assert!(input_dir.is_none());
                assert!(output.is_none());
                assert!(!json);
                assert_eq!(output_format, "json");
                assert!(!embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_with_output() {
        let cli = Cli::try_parse_from([
            "speccade",
            "analyze",
            "--input",
            "sound.wav",
            "--output",
            "metrics.json",
        ])
        .unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert_eq!(input.as_deref(), Some("sound.wav"));
                assert!(spec.is_none());
                assert!(input_dir.is_none());
                assert_eq!(output.as_deref(), Some("metrics.json"));
                assert!(!json);
                assert_eq!(output_format, "json");
                assert!(!embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_with_json() {
        let cli =
            Cli::try_parse_from(["speccade", "analyze", "--input", "sound.wav", "--json"]).unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert_eq!(input.as_deref(), Some("sound.wav"));
                assert!(spec.is_none());
                assert!(input_dir.is_none());
                assert!(output.is_none());
                assert!(json);
                assert_eq!(output_format, "json");
                assert!(!embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_with_embeddings() {
        let cli = Cli::try_parse_from([
            "speccade",
            "analyze",
            "--input",
            "sound.wav",
            "--embeddings",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert_eq!(input.as_deref(), Some("sound.wav"));
                assert!(spec.is_none());
                assert!(input_dir.is_none());
                assert!(output.is_none());
                assert!(json);
                assert_eq!(output_format, "json");
                assert!(embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_with_input_dir() {
        let cli = Cli::try_parse_from([
            "speccade",
            "analyze",
            "--input-dir",
            "./assets",
            "--output-format",
            "jsonl",
        ])
        .unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert!(input.is_none());
                assert!(spec.is_none());
                assert_eq!(input_dir.as_deref(), Some("./assets"));
                assert!(output.is_none());
                assert!(!json);
                assert_eq!(output_format, "jsonl");
                assert!(!embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_analyze_with_csv_format() {
        let cli = Cli::try_parse_from([
            "speccade",
            "analyze",
            "--input-dir",
            "./test",
            "--output-format",
            "csv",
            "--embeddings",
        ])
        .unwrap();
        match cli.command {
            Commands::Analyze {
                input,
                spec,
                input_dir,
                output,
                json,
                output_format,
                embeddings,
            } => {
                assert!(input.is_none());
                assert!(spec.is_none());
                assert_eq!(input_dir.as_deref(), Some("./test"));
                assert!(output.is_none());
                assert!(!json);
                assert_eq!(output_format, "csv");
                assert!(embeddings);
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[test]
    fn test_cli_parses_audit_basic() {
        let cli = Cli::try_parse_from(["speccade", "audit", "--input-dir", "./sounds"]).unwrap();
        match cli.command {
            Commands::Audit {
                input_dir,
                tolerances,
                update_baselines,
                json,
            } => {
                assert_eq!(input_dir, "./sounds");
                assert!(tolerances.is_none());
                assert!(!update_baselines);
                assert!(!json);
            }
            _ => panic!("expected audit command"),
        }
    }

    #[test]
    fn test_cli_parses_audit_with_tolerances() {
        let cli = Cli::try_parse_from([
            "speccade",
            "audit",
            "--input-dir",
            "./sounds",
            "--tolerances",
            "config.json",
        ])
        .unwrap();
        match cli.command {
            Commands::Audit {
                input_dir,
                tolerances,
                update_baselines,
                json,
            } => {
                assert_eq!(input_dir, "./sounds");
                assert_eq!(tolerances.as_deref(), Some("config.json"));
                assert!(!update_baselines);
                assert!(!json);
            }
            _ => panic!("expected audit command"),
        }
    }

    #[test]
    fn test_cli_parses_audit_with_update_baselines() {
        let cli = Cli::try_parse_from([
            "speccade",
            "audit",
            "--input-dir",
            "./sounds",
            "--update-baselines",
        ])
        .unwrap();
        match cli.command {
            Commands::Audit {
                input_dir,
                tolerances,
                update_baselines,
                json,
            } => {
                assert_eq!(input_dir, "./sounds");
                assert!(tolerances.is_none());
                assert!(update_baselines);
                assert!(!json);
            }
            _ => panic!("expected audit command"),
        }
    }

    #[test]
    fn test_cli_parses_audit_with_json() {
        let cli = Cli::try_parse_from(["speccade", "audit", "--input-dir", "./sounds", "--json"])
            .unwrap();
        match cli.command {
            Commands::Audit {
                input_dir,
                tolerances,
                update_baselines,
                json,
            } => {
                assert_eq!(input_dir, "./sounds");
                assert!(tolerances.is_none());
                assert!(!update_baselines);
                assert!(json);
            }
            _ => panic!("expected audit command"),
        }
    }

    #[test]
    fn test_cli_requires_input_dir_for_audit() {
        let err = Cli::try_parse_from(["speccade", "audit"]).err().unwrap();
        assert!(err.to_string().contains("--input-dir"));
    }

    #[test]
    fn test_cli_parses_inspect_basic() {
        let cli = Cli::try_parse_from([
            "speccade",
            "inspect",
            "--spec",
            "spec.json",
            "--out-dir",
            "./out",
        ])
        .unwrap();
        match cli.command {
            Commands::Inspect {
                spec,
                out_dir,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_dir, "./out");
                assert!(!json);
            }
            _ => panic!("expected inspect command"),
        }
    }

    #[test]
    fn test_cli_parses_inspect_with_json() {
        let cli = Cli::try_parse_from([
            "speccade",
            "inspect",
            "--spec",
            "spec.json",
            "--out-dir",
            "./out",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Inspect {
                spec,
                out_dir,
                json,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_dir, "./out");
                assert!(json);
            }
            _ => panic!("expected inspect command"),
        }
    }

    #[test]
    fn test_cli_requires_spec_for_inspect() {
        let err = Cli::try_parse_from(["speccade", "inspect", "--out-dir", "./out"])
            .err()
            .unwrap();
        assert!(err.to_string().contains("--spec"));
    }

    #[test]
    fn test_cli_requires_out_dir_for_inspect() {
        let err = Cli::try_parse_from(["speccade", "inspect", "--spec", "spec.json"])
            .err()
            .unwrap();
        assert!(err.to_string().contains("--out-dir"));
    }
}
