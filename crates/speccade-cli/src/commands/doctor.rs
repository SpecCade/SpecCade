//! Doctor command implementation
//!
//! Checks system dependencies and configuration.

use anyhow::Result;
use colored::Colorize;
use std::env;
use std::process::{Command, ExitCode};

use crate::dispatch::{get_backend_tier, is_backend_available};

/// Run the doctor command
///
/// Checks:
/// - Blender installation
/// - Output directory permissions
/// - Version information
///
/// # Returns
/// Exit code: 0 if all checks pass, 1 if any fail
pub fn run() -> Result<ExitCode> {
    println!("{}", "SpecCade Doctor".cyan().bold());
    println!("{}", "===============".cyan());
    println!();

    let mut all_ok = true;

    // Check 1: SpecCade version
    println!("{}", "Versions:".bold());
    println!(
        "  {} speccade-cli v{}",
        "->".green(),
        env!("CARGO_PKG_VERSION")
    );

    // Get Rust version
    match get_rustc_version() {
        Some(version) => {
            println!("  {} rustc {}", "->".green(), version);
        }
        None => {
            println!("  {} rustc (not found)", "->".yellow());
        }
    }

    println!();

    // Check 2: Blender installation
    println!("{}", "Dependencies:".bold());
    match check_blender() {
        BlenderStatus::Found(version) => {
            println!("  {} Blender {} (found in PATH)", "ok".green(), version);
        }
        BlenderStatus::NotFound => {
            println!("  {} Blender not found in PATH", "!!".yellow());
            println!(
                "     {}",
                "Blender is required for mesh and animation generation.".dimmed()
            );
            println!(
                "     {}",
                "Install from https://www.blender.org/download/".dimmed()
            );
            // Not a hard failure - Blender is only needed for some backends
        }
        BlenderStatus::Error(e) => {
            println!("  {} Blender check failed: {}", "!!".red(), e);
            all_ok = false;
        }
    }

    println!();

    // Check 3: Output directory permissions
    println!("{}", "Permissions:".bold());
    let current_dir = env::current_dir();
    match current_dir {
        Ok(dir) => {
            // Try to check if we can write to the directory
            let test_file = dir.join(".speccade_write_test");
            match std::fs::write(&test_file, "test") {
                Ok(_) => {
                    // Clean up test file
                    let _ = std::fs::remove_file(&test_file);
                    println!(
                        "  {} Current directory is writable ({})",
                        "ok".green(),
                        dir.display()
                    );
                }
                Err(e) => {
                    println!("  {} Cannot write to current directory: {}", "!!".red(), e);
                    all_ok = false;
                }
            }
        }
        Err(e) => {
            println!("  {} Cannot determine current directory: {}", "!!".red(), e);
            all_ok = false;
        }
    }

    println!();

    // Check 4: Available backends
    println!("{}", "Backends:".bold());
    let recipe_kinds = [
        "audio_v1",
        "music.tracker_song_v1",
        "music.tracker_song_compose_v1",
        "texture.procedural_v1",
        "static_mesh.blender_primitives_v1",
        "skeletal_mesh.armature_driven_v1",
        "skeletal_mesh.skinned_mesh_v1",
        "skeletal_animation.blender_clip_v1",
    ];
    for kind in recipe_kinds {
        let available = is_backend_available(kind);
        let tier = get_backend_tier(kind);
        let tier_str = match tier {
            Some(1) => "(Tier 1: deterministic)".dimmed(),
            Some(2) => "(Tier 2: metrics)".dimmed(),
            _ => "".dimmed(),
        };
        if available {
            println!("  {} {} {}", "ok".green(), kind, tier_str);
        } else {
            println!("  {} {} (not implemented)", "!!".yellow(), kind);
        }
    }

    println!();

    // Summary
    if all_ok {
        println!("{} All checks passed!", "SUCCESS".green().bold());
        Ok(ExitCode::SUCCESS)
    } else {
        println!(
            "{} Some checks failed. See above for details.",
            "WARNING".yellow().bold()
        );
        Ok(ExitCode::from(1))
    }
}

/// Status of Blender installation check
enum BlenderStatus {
    Found(String),
    NotFound,
    Error(String),
}

fn parse_blender_version(output: &str) -> Option<String> {
    output
        .lines()
        .next()
        .and_then(|line| line.strip_prefix("Blender "))
        .map(|v| v.trim().to_string())
}

/// Check if Blender is installed and get its version
fn check_blender() -> BlenderStatus {
    // Try to run `blender --version`
    let result = Command::new("blender").arg("--version").output();

    match result {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Parse version from output like "Blender 4.0.2\n..."
                let version =
                    parse_blender_version(&stdout).unwrap_or_else(|| "unknown".to_string());
                BlenderStatus::Found(version)
            } else {
                BlenderStatus::Error(format!("Blender exited with status: {}", output.status))
            }
        }
        Err(e) => {
            if e.kind() == std::io::ErrorKind::NotFound {
                BlenderStatus::NotFound
            } else {
                BlenderStatus::Error(e.to_string())
            }
        }
    }
}

fn parse_rustc_version(output: &str) -> Option<String> {
    // Parse "rustc 1.75.0 (..."
    output.split_whitespace().nth(1).map(|s| s.to_string())
}

/// Get the rustc version
fn get_rustc_version() -> Option<String> {
    let output = Command::new("rustc").arg("--version").output().ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        parse_rustc_version(&stdout)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blender_version() {
        let out = "Blender 4.0.2\nBuild date: ...\n";
        assert_eq!(parse_blender_version(out).as_deref(), Some("4.0.2"));
        assert_eq!(parse_blender_version("not blender\n"), None);
    }

    #[test]
    fn test_parse_rustc_version() {
        let out = "rustc 1.75.0 (82e1608df 2023-12-21)\n";
        assert_eq!(parse_rustc_version(out).as_deref(), Some("1.75.0"));
        assert_eq!(parse_rustc_version("rustc\n"), None);
    }
}
