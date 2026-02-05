//! Build script for speccade-cli.
//!
//! This script embeds git metadata into the binary.

use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // Get the manifest directory (where Cargo.toml lives)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let repo_root = manifest_path.join("..").join("..");

    embed_git_metadata(&repo_root);
}

fn embed_git_metadata(repo_root: &Path) {
    let git_dir = repo_root.join(".git");
    if git_dir.exists() {
        let head_path = git_dir.join("HEAD");
        if head_path.exists() {
            println!("cargo:rerun-if-changed={}", head_path.display());
        }
        let index_path = git_dir.join("index");
        if index_path.exists() {
            println!("cargo:rerun-if-changed={}", index_path.display());
        }
    }

    let sha = git(repo_root, &["rev-parse", "HEAD"]);
    if let Some(sha) = sha.as_deref().filter(|s| !s.trim().is_empty()) {
        println!("cargo:rustc-env=SPECCADE_GIT_SHA={}", sha.trim());
        if let Some(status) = git(repo_root, &["status", "--porcelain"]) {
            let dirty = !status.trim().is_empty();
            println!(
                "cargo:rustc-env=SPECCADE_GIT_DIRTY={}",
                if dirty { 1 } else { 0 }
            );
        }
    }
}

fn git(repo_root: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    String::from_utf8(output.stdout).ok()
}
