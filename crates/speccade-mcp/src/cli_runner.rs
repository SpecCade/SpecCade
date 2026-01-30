use anyhow::{bail, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;

enum SpeccadeInvocation {
    Direct(PathBuf),
    CargoRun,
}

/// Best-effort resolver for a usable `speccade` invocation.
fn speccade_invocation() -> SpeccadeInvocation {
    if let Ok(bin) = std::env::var("SPECCADE_BIN") {
        return SpeccadeInvocation::Direct(PathBuf::from(bin));
    }

    if which::which("speccade").is_ok() {
        return SpeccadeInvocation::Direct(PathBuf::from("speccade"));
    }

    // Development convenience: if we're in the speccade repo, use the built binary directly.
    let exe = if cfg!(windows) {
        "speccade.exe"
    } else {
        "speccade"
    };

    for profile in ["debug", "release"] {
        let candidate = Path::new("target").join(profile).join(exe);
        if candidate.exists() {
            return SpeccadeInvocation::Direct(candidate);
        }
    }

    SpeccadeInvocation::CargoRun
}

pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub async fn run_cli(args: &[&str]) -> Result<CliOutput> {
    let mut cmd = match speccade_invocation() {
        SpeccadeInvocation::Direct(bin) => {
            let mut cmd = Command::new(&bin);
            cmd.args(args);
            cmd
        }
        SpeccadeInvocation::CargoRun => {
            if which::which("cargo").is_err() {
                bail!("Could not find 'speccade' on PATH and 'cargo' is unavailable");
            }
            // Use `-q` to avoid `Running ...` output polluting JSON.
            let mut cmd = Command::new("cargo");
            cmd.args(["run", "-q", "-p", "speccade-cli", "--"]);
            cmd.args(args);
            cmd
        }
    };

    let output = cmd.output().await?;

    Ok(CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}
