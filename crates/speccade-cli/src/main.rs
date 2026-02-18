//! SpecCade CLI - Command-line interface for declarative asset generation
//!
//! This binary provides commands for validating, generating, and managing
//! SpecCade specs and their generated assets.

mod cli_args;

use clap::Parser;
use std::path::Path;
use std::process::ExitCode;

use cli_args::*;

// Use modules from the library crate
use speccade_cli::commands;

fn main() -> ExitCode {
    let cli = Cli::parse();

    let result = match cli.command {
        #[cfg(feature = "serve")]
        Commands::Analyze {
            input,
            spec,
            input_dir,
            output,
            json,
            output_format,
            embeddings,
            serve,
        } => {
            // If --serve flag is provided, start the WebSocket server
            if let Some(port_opt) = serve {
                let port = port_opt.unwrap_or(commands::serve::DEFAULT_PORT);
                commands::serve::run(port)
            } else {
                commands::analyze::run(
                    input.as_deref(),
                    spec.as_deref(),
                    input_dir.as_deref(),
                    output.as_deref(),
                    json,
                    &output_format,
                    embeddings,
                )
            }
        }
        #[cfg(not(feature = "serve"))]
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
        Commands::Compare { a, b, json } => commands::compare::run(&a, &b, json),
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
            variations,
            max_peak_db,
            max_dc_offset,
            save_blend,
        } => commands::generate::run(
            &spec,
            out_root.as_deref(),
            expand_variants,
            budget.as_deref(),
            json,
            preview,
            no_cache,
            profile,
            variations,
            max_peak_db,
            max_dc_offset,
            save_blend,
        ),
        Commands::GenerateAll {
            spec_dir,
            out_root,
            include_blender,
            verbose,
            force,
        } => commands::generate_all::run(
            spec_dir.as_deref(),
            out_root.as_deref(),
            include_blender,
            verbose,
            force,
        ),
        Commands::Preview {
            spec,
            out_root,
            gif,
            out,
            fps,
            scale,
        } => commands::preview::run(&spec, out_root.as_deref(), gif, out.as_deref(), fps, scale),
        Commands::PreviewGrid {
            spec,
            out,
            panel_size,
        } => commands::preview_grid::run(&spec, out.as_deref(), panel_size),
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
        Commands::Coverage { command } => match command {
            CoverageCommands::Generate { strict, output } => {
                commands::coverage::run_generate(strict, output.as_deref())
            }
            CoverageCommands::Report => commands::coverage::run_report(),
        },
        Commands::Verify {
            report,
            constraints,
            json,
        } => commands::verify::run(&report, &constraints, json),
        Commands::Lint {
            input,
            spec,
            strict,
            disable_rules,
            only_rules,
            format,
        } => {
            let output_format = format
                .parse::<commands::lint::OutputFormat>()
                .expect("clap should have validated format");
            commands::lint::run(
                &input,
                spec.as_deref(),
                strict,
                &disable_rules,
                only_rules.as_deref(),
                output_format,
            )
        }
        Commands::ValidateAsset {
            spec,
            out_root,
            full_report,
        } => commands::validate_asset::run(&spec, out_root.as_deref(), full_report),
        Commands::BatchValidate {
            specs,
            out_root,
            format,
        } => commands::batch_validate::run(&specs, out_root.as_deref(), &format),
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
    fn test_cli_parses_compare() {
        let cli = Cli::try_parse_from([
            "speccade",
            "compare",
            "--a",
            "file1.wav",
            "--b",
            "file2.wav",
        ])
        .unwrap();
        match cli.command {
            Commands::Compare { a, b, json } => {
                assert_eq!(a, "file1.wav");
                assert_eq!(b, "file2.wav");
                assert!(!json);
            }
            _ => panic!("expected compare command"),
        }
    }

    #[test]
    fn test_cli_parses_compare_with_json() {
        let cli = Cli::try_parse_from([
            "speccade",
            "compare",
            "--a",
            "file1.png",
            "--b",
            "file2.png",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Compare { a, b, json } => {
                assert_eq!(a, "file1.png");
                assert_eq!(b, "file2.png");
                assert!(json);
            }
            _ => panic!("expected compare command"),
        }
    }

    #[test]
    fn test_cli_requires_a_and_b_for_compare() {
        let err = Cli::try_parse_from(["speccade", "compare", "--a", "file.wav"])
            .err()
            .unwrap();
        assert!(err.to_string().contains("-b"));

        let err = Cli::try_parse_from(["speccade", "compare", "--b", "file.wav"])
            .err()
            .unwrap();
        assert!(err.to_string().contains("-a"));
    }

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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert_eq!(out_root.as_deref(), Some("out"));
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
                assert!(variations.is_none());
                assert!(max_peak_db.is_none());
                assert!(max_dc_offset.is_none());
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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert_eq!(budget.as_deref(), Some("zx-8bit"));
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
                assert!(variations.is_none());
                assert!(max_peak_db.is_none());
                assert!(max_dc_offset.is_none());
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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
                assert!(variations.is_none());
                assert!(max_peak_db.is_none());
                assert!(max_dc_offset.is_none());
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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(profile);
                assert!(variations.is_none());
                assert!(max_peak_db.is_none());
                assert!(max_dc_offset.is_none());
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_with_variations() {
        let cli = Cli::try_parse_from([
            "speccade",
            "generate",
            "--spec",
            "spec.json",
            "--variations",
            "5",
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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
                assert_eq!(variations, Some(5));
                assert!(max_peak_db.is_none());
                assert!(max_dc_offset.is_none());
            }
            _ => panic!("expected generate command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_with_quality_constraints() {
        let cli = Cli::try_parse_from([
            "speccade",
            "generate",
            "--spec",
            "spec.json",
            "--variations",
            "10",
            "--max-peak-db",
            "-3.0",
            "--max-dc-offset",
            "0.01",
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
                variations,
                max_peak_db,
                max_dc_offset,
                save_blend: _,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(!expand_variants);
                assert!(budget.is_none());
                assert!(!json);
                assert!(preview.is_none());
                assert!(!no_cache);
                assert!(!profile);
                assert_eq!(variations, Some(10));
                assert!((max_peak_db.unwrap() - (-3.0)).abs() < 0.001);
                assert!((max_dc_offset.unwrap() - 0.01).abs() < 0.0001);
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
    fn test_cli_parses_generate_all_defaults() {
        let cli = Cli::try_parse_from(["speccade", "generate-all"]).unwrap();
        match cli.command {
            Commands::GenerateAll {
                spec_dir,
                out_root,
                include_blender,
                verbose,
                force,
            } => {
                assert!(spec_dir.is_none());
                assert!(out_root.is_none());
                assert!(!include_blender);
                assert!(!verbose);
                assert!(!force);
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
                force,
            } => {
                assert_eq!(spec_dir.as_deref(), Some("/path/to/specs"));
                assert_eq!(out_root.as_deref(), Some("/path/to/output"));
                assert!(include_blender);
                assert!(verbose);
                assert!(!force);
            }
            _ => panic!("expected generate-all command"),
        }
    }

    #[test]
    fn test_cli_parses_generate_all_with_force() {
        let cli = Cli::try_parse_from(["speccade", "generate-all", "--force"]).unwrap();
        match cli.command {
            Commands::GenerateAll {
                spec_dir,
                out_root,
                include_blender,
                verbose,
                force,
            } => {
                assert!(spec_dir.is_none());
                assert!(out_root.is_none());
                assert!(!include_blender);
                assert!(!verbose);
                assert!(force);
            }
            _ => panic!("expected generate-all command"),
        }

        // Also test short flag
        let cli = Cli::try_parse_from(["speccade", "generate-all", "-f"]).unwrap();
        match cli.command {
            Commands::GenerateAll { force, .. } => {
                assert!(force);
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
                ..
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
                ..
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
                ..
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
                ..
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
                ..
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
                ..
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

    #[cfg(feature = "serve")]
    #[test]
    fn test_cli_parses_analyze_with_serve_default_port() {
        let cli = Cli::try_parse_from(["speccade", "analyze", "--serve"]).unwrap();
        match cli.command {
            Commands::Analyze { serve, .. } => {
                // --serve with no value should be Some(None)
                assert!(serve.is_some());
                assert!(serve.unwrap().is_none());
            }
            _ => panic!("expected analyze command"),
        }
    }

    #[cfg(feature = "serve")]
    #[test]
    fn test_cli_parses_analyze_with_serve_custom_port() {
        let cli = Cli::try_parse_from(["speccade", "analyze", "--serve", "8080"]).unwrap();
        match cli.command {
            Commands::Analyze { serve, .. } => {
                assert!(serve.is_some());
                assert_eq!(serve.unwrap(), Some(8080));
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

    #[test]
    fn test_cli_parses_verify() {
        let cli = Cli::try_parse_from([
            "speccade",
            "verify",
            "--report",
            "test.report.json",
            "--constraints",
            "test.constraints.json",
        ])
        .unwrap();
        match cli.command {
            Commands::Verify {
                report,
                constraints,
                json,
            } => {
                assert_eq!(report, "test.report.json");
                assert_eq!(constraints, "test.constraints.json");
                assert!(!json);
            }
            _ => panic!("expected verify command"),
        }
    }

    #[test]
    fn test_cli_parses_verify_with_json() {
        let cli = Cli::try_parse_from([
            "speccade",
            "verify",
            "--report",
            "test.report.json",
            "--constraints",
            "test.constraints.json",
            "--json",
        ])
        .unwrap();
        match cli.command {
            Commands::Verify {
                report,
                constraints,
                json,
            } => {
                assert_eq!(report, "test.report.json");
                assert_eq!(constraints, "test.constraints.json");
                assert!(json);
            }
            _ => panic!("expected verify command"),
        }
    }

    #[test]
    fn test_cli_requires_report_for_verify() {
        let err = Cli::try_parse_from([
            "speccade",
            "verify",
            "--constraints",
            "test.constraints.json",
        ])
        .err()
        .unwrap();
        assert!(err.to_string().contains("--report"));
    }

    #[test]
    fn test_cli_requires_constraints_for_verify() {
        let err = Cli::try_parse_from(["speccade", "verify", "--report", "test.report.json"])
            .err()
            .unwrap();
        assert!(err.to_string().contains("--constraints"));
    }

    #[test]
    fn test_cli_parses_lint() {
        let cli = Cli::try_parse_from(["speccade", "lint", "--input", "sound.wav"]).unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert!(spec.is_none());
                assert!(!strict);
                assert!(disable_rules.is_empty());
                assert!(only_rules.is_none());
                assert_eq!(format, "text");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_parses_lint_with_spec() {
        let cli = Cli::try_parse_from([
            "speccade",
            "lint",
            "--input",
            "sound.wav",
            "--spec",
            "spec.star",
        ])
        .unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert_eq!(spec.as_deref(), Some("spec.star"));
                assert!(!strict);
                assert!(disable_rules.is_empty());
                assert!(only_rules.is_none());
                assert_eq!(format, "text");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_parses_lint_with_strict() {
        let cli =
            Cli::try_parse_from(["speccade", "lint", "--input", "sound.wav", "--strict"]).unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert!(spec.is_none());
                assert!(strict);
                assert!(disable_rules.is_empty());
                assert!(only_rules.is_none());
                assert_eq!(format, "text");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_parses_lint_with_disable_rule() {
        let cli = Cli::try_parse_from([
            "speccade",
            "lint",
            "--input",
            "sound.wav",
            "--disable-rule",
            "audio/clipping",
            "--disable-rule",
            "audio/silence",
        ])
        .unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert!(spec.is_none());
                assert!(!strict);
                assert_eq!(disable_rules, vec!["audio/clipping", "audio/silence"]);
                assert!(only_rules.is_none());
                assert_eq!(format, "text");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_parses_lint_with_only_rules() {
        let cli = Cli::try_parse_from([
            "speccade",
            "lint",
            "--input",
            "sound.wav",
            "--only-rules",
            "audio/clipping,audio/silence",
        ])
        .unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert!(spec.is_none());
                assert!(!strict);
                assert!(disable_rules.is_empty());
                assert_eq!(only_rules, Some("audio/clipping,audio/silence".to_string()));
                assert_eq!(format, "text");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_parses_lint_with_json_format() {
        let cli = Cli::try_parse_from([
            "speccade",
            "lint",
            "--input",
            "sound.wav",
            "--format",
            "json",
        ])
        .unwrap();
        match cli.command {
            Commands::Lint {
                input,
                spec,
                strict,
                disable_rules,
                only_rules,
                format,
            } => {
                assert_eq!(input, "sound.wav");
                assert!(spec.is_none());
                assert!(!strict);
                assert!(disable_rules.is_empty());
                assert!(only_rules.is_none());
                assert_eq!(format, "json");
            }
            _ => panic!("expected lint command"),
        }
    }

    #[test]
    fn test_cli_requires_input_for_lint() {
        let err = Cli::try_parse_from(["speccade", "lint"]).err().unwrap();
        assert!(err.to_string().contains("--input"));
    }

    #[test]
    fn test_cli_parses_preview_with_gif() {
        let cli =
            Cli::try_parse_from(["speccade", "preview", "--spec", "spec.json", "--gif"]).unwrap();
        match cli.command {
            Commands::Preview {
                spec,
                out_root,
                gif,
                out,
                fps,
                scale,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(gif);
                assert!(out.is_none());
                assert!(fps.is_none());
                assert!(scale.is_none());
            }
            _ => panic!("expected preview command"),
        }
    }

    #[test]
    fn test_cli_parses_preview_with_gif_options() {
        let cli = Cli::try_parse_from([
            "speccade",
            "preview",
            "--spec",
            "spec.json",
            "--gif",
            "--out",
            "preview.gif",
            "--fps",
            "24",
            "--scale",
            "2",
        ])
        .unwrap();
        match cli.command {
            Commands::Preview {
                spec,
                out_root,
                gif,
                out,
                fps,
                scale,
            } => {
                assert_eq!(spec, "spec.json");
                assert!(out_root.is_none());
                assert!(gif);
                assert_eq!(out.as_deref(), Some("preview.gif"));
                assert_eq!(fps, Some(24));
                assert_eq!(scale, Some(2));
            }
            _ => panic!("expected preview command"),
        }
    }

    #[test]
    fn test_cli_parses_preview_grid() {
        let cli = Cli::try_parse_from(["speccade", "preview-grid", "--spec", "mesh.star"]).unwrap();
        match cli.command {
            Commands::PreviewGrid {
                spec,
                out,
                panel_size,
            } => {
                assert_eq!(spec, "mesh.star");
                assert!(out.is_none());
                assert_eq!(panel_size, 256);
            }
            _ => panic!("expected preview-grid command"),
        }
    }

    #[test]
    fn test_cli_parses_preview_grid_with_options() {
        let cli = Cli::try_parse_from([
            "speccade",
            "preview-grid",
            "--spec",
            "mesh.star",
            "--out",
            "grid.png",
            "--panel-size",
            "128",
        ])
        .unwrap();
        match cli.command {
            Commands::PreviewGrid {
                spec,
                out,
                panel_size,
            } => {
                assert_eq!(spec, "mesh.star");
                assert_eq!(out.as_deref(), Some("grid.png"));
                assert_eq!(panel_size, 128);
            }
            _ => panic!("expected preview-grid command"),
        }
    }

    #[test]
    fn test_cli_parses_coverage_generate() {
        let cli = Cli::try_parse_from(["speccade", "coverage", "generate"]).unwrap();
        match cli.command {
            Commands::Coverage { command } => match command {
                CoverageCommands::Generate { strict, output } => {
                    assert!(!strict);
                    assert!(output.is_none());
                }
                _ => panic!("expected generate subcommand"),
            },
            _ => panic!("expected coverage command"),
        }
    }

    #[test]
    fn test_cli_parses_coverage_generate_strict() {
        let cli = Cli::try_parse_from([
            "speccade",
            "coverage",
            "generate",
            "--strict",
            "--output",
            "custom.yaml",
        ])
        .unwrap();
        match cli.command {
            Commands::Coverage { command } => match command {
                CoverageCommands::Generate { strict, output } => {
                    assert!(strict);
                    assert_eq!(output.as_deref(), Some("custom.yaml"));
                }
                _ => panic!("expected generate subcommand"),
            },
            _ => panic!("expected coverage command"),
        }
    }

    #[test]
    fn test_cli_parses_coverage_report() {
        let cli = Cli::try_parse_from(["speccade", "coverage", "report"]).unwrap();
        match cli.command {
            Commands::Coverage { command } => match command {
                CoverageCommands::Report => {}
                _ => panic!("expected report subcommand"),
            },
            _ => panic!("expected coverage command"),
        }
    }
}
