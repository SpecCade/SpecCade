use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Command;

/// Find the speccade binary
fn speccade_bin() -> PathBuf {
    if which::which("speccade").is_ok() {
        return PathBuf::from("speccade");
    }
    PathBuf::from("speccade")
}

pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub async fn run_cli(args: &[&str]) -> Result<CliOutput> {
    let bin = speccade_bin();
    let output = Command::new(&bin).args(args).output().await?;

    Ok(CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}
