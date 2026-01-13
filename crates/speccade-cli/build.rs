//! Build script for speccade-cli.
//!
//! This script parses the legacy parity matrix at build time and generates
//! Rust code containing parity data that can be included in the binary.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

/// Represents the implementation status of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum KeyStatus {
    Implemented,
    Partial,
    NotImplemented,
    Deprecated,
}

impl KeyStatus {
    fn parse_cell(s: &str) -> Option<Self> {
        let s = s.trim();
        // Check for the actual Unicode characters used in the markdown
        if s.contains('\u{2713}') || s.contains('\u{2714}') || s == "checkmark" {
            Some(KeyStatus::Implemented)
        } else if s.contains('~') {
            Some(KeyStatus::Partial)
        } else if s.contains('\u{2717}') || s.contains('\u{2718}') || s.contains('\u{2715}') {
            Some(KeyStatus::NotImplemented)
        } else if s == "-" || s.is_empty() {
            Some(KeyStatus::Deprecated)
        } else {
            None
        }
    }

    fn as_code_str(self) -> &'static str {
        match self {
            KeyStatus::Implemented => "KeyStatus::Implemented",
            KeyStatus::Partial => "KeyStatus::Partial",
            KeyStatus::NotImplemented => "KeyStatus::NotImplemented",
            KeyStatus::Deprecated => "KeyStatus::Deprecated",
        }
    }
}

/// A parsed key entry from the parity matrix.
#[derive(Debug, Clone)]
struct ParsedKey {
    section: String,
    table: String,
    key: String,
    required: bool,
    status: KeyStatus,
}

/// Column indices for a table with a Key column.
#[derive(Debug, Clone)]
struct TableColumns {
    key_idx: usize,
    required_idx: Option<usize>,
    status_idx: Option<usize>,
}

fn main() {
    // Get the manifest directory (where Cargo.toml lives)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let manifest_path = PathBuf::from(&manifest_dir);
    let repo_root = manifest_path.join("..").join("..");

    embed_git_metadata(&repo_root);

    // Build the path to the legacy parity matrix (relative to crates/speccade-cli)
    let parity_matrix_path = manifest_path
        .join("..")
        .join("..")
        .join("docs")
        .join("legacy")
        .join("PARITY_MATRIX_LEGACY_SPEC_PY.md");
    let parity_matrix_path = parity_matrix_path.canonicalize().unwrap_or_else(|e| {
        panic!(
            "Failed to canonicalize legacy parity matrix path {:?}: {}",
            parity_matrix_path, e
        )
    });

    // Tell Cargo to rerun if the file changes
    println!("cargo:rerun-if-changed={}", parity_matrix_path.display());

    // Read the parity matrix file
    let content = fs::read_to_string(&parity_matrix_path).unwrap_or_else(|e| {
        panic!(
            "Failed to read legacy parity matrix at {:?}: {}",
            parity_matrix_path, e
        )
    });

    // Parse the matrix
    let keys = parse_parity_matrix(&content);

    // Generate Rust code
    let rust_code = generate_rust_code(&keys);

    // Write to OUT_DIR
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let out_path = PathBuf::from(&out_dir).join("parity_data.rs");
    fs::write(&out_path, rust_code)
        .unwrap_or_else(|e| panic!("Failed to write parity_data.rs to {:?}: {}", out_path, e));

    println!(
        "cargo:warning=Generated parity_data.rs with {} keys",
        keys.len()
    );
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

/// Parse a parity matrix markdown string.
fn parse_parity_matrix(content: &str) -> Vec<ParsedKey> {
    let mut result = Vec::new();
    let mut current_section = String::new();
    let mut current_table = String::new();
    let mut in_table = false;
    let mut header_indices: Option<TableColumns> = None;

    for line in content.lines() {
        let trimmed = line.trim();

        // Track section headings (## ...)
        if let Some(stripped) = trimmed.strip_prefix("## ") {
            current_section = stripped.trim().to_string();
            current_table.clear();
            in_table = false;
            header_indices = None;
            continue;
        }

        // Track table headings (### ...)
        if let Some(stripped) = trimmed.strip_prefix("### ") {
            current_table = stripped.trim().to_string();
            in_table = false;
            header_indices = None;
            continue;
        }

        // Skip empty lines or non-table lines
        if !trimmed.starts_with('|') {
            in_table = false;
            header_indices = None;
            continue;
        }

        // Parse table rows
        let cells: Vec<&str> = trimmed
            .split('|')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();

        if cells.is_empty() {
            continue;
        }

        // Check if this is a header row (contains "Key")
        if !in_table {
            if let Some(indices) = find_table_columns(&cells) {
                header_indices = Some(indices);
                in_table = true;
            }
            continue;
        }

        // Skip separator rows (contain only dashes and colons)
        if cells
            .iter()
            .all(|c| c.chars().all(|ch| ch == '-' || ch == ':'))
        {
            continue;
        }

        // Parse data row
        if let Some(ref indices) = header_indices {
            if let Some(parsed_key) =
                parse_table_row(&cells, indices, &current_section, &current_table)
            {
                result.push(parsed_key);
            }
        }
    }

    result
}

/// Find the column indices for Key, Required, and Status columns.
fn find_table_columns(header_cells: &[&str]) -> Option<TableColumns> {
    let mut key_idx = None;
    let mut required_idx = None;
    let mut status_idx = None;

    for (i, cell) in header_cells.iter().enumerate() {
        let cell_lower = cell.to_lowercase();
        if cell_lower == "key" {
            key_idx = Some(i);
        } else if cell_lower == "required" {
            required_idx = Some(i);
        } else if cell_lower == "status" {
            status_idx = Some(i);
        }
    }

    key_idx.map(|key_idx| TableColumns {
        key_idx,
        required_idx,
        status_idx,
    })
}

/// Parse a single table row into a ParsedKey.
fn parse_table_row(
    cells: &[&str],
    indices: &TableColumns,
    section: &str,
    table: &str,
) -> Option<ParsedKey> {
    // Get the key name
    let key_raw = cells.get(indices.key_idx)?;
    let key = strip_backticks(key_raw);

    // Skip if key is empty or looks like a header/separator
    if key.is_empty() || key == "Key" || key.chars().all(|c| c == '-' || c == ':') {
        return None;
    }

    // Get the required status
    let required = indices
        .required_idx
        .and_then(|i| cells.get(i))
        .map(|s| s.trim_start().to_lowercase().starts_with("yes"))
        .unwrap_or(false);

    // Get the implementation status
    let status = indices
        .status_idx
        .and_then(|i| cells.get(i))
        .and_then(|s| KeyStatus::parse_cell(s))
        .unwrap_or(KeyStatus::NotImplemented);

    Some(ParsedKey {
        section: section.to_string(),
        table: table.to_string(),
        key,
        required,
        status,
    })
}

/// Strip backticks from a key name.
///
/// Handles strings like "`key_name`" and returns "key_name".
fn strip_backticks(s: &str) -> String {
    s.trim().trim_matches('`').to_string()
}

/// Generate Rust code for the parsed matrix.
fn generate_rust_code(keys: &[ParsedKey]) -> String {
    let mut code = String::new();

    // Header comment
    code.push_str("// Auto-generated by build.rs from PARITY_MATRIX.md\n");
    code.push_str("// Do not edit manually!\n\n");

    // KeyStatus enum
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("pub enum KeyStatus {\n");
    code.push_str("    Implemented,\n");
    code.push_str("    Partial,\n");
    code.push_str("    NotImplemented,\n");
    code.push_str("    Deprecated,\n");
    code.push_str("}\n\n");

    // ParityKey struct
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]\n");
    code.push_str("pub struct ParityKey {\n");
    code.push_str("    pub section: &'static str,\n");
    code.push_str("    pub table: &'static str,\n");
    code.push_str("    pub key: &'static str,\n");
    code.push_str("}\n\n");

    // KeyInfo struct
    code.push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq)]\n");
    code.push_str("pub struct KeyInfo {\n");
    code.push_str("    pub key: ParityKey,\n");
    code.push_str("    pub required: bool,\n");
    code.push_str("    pub status: KeyStatus,\n");
    code.push_str("}\n\n");

    // ALL_KEYS static array
    code.push_str("pub static ALL_KEYS: &[KeyInfo] = &[\n");
    for key in keys {
        code.push_str("    KeyInfo {\n");
        code.push_str("        key: ParityKey {\n");
        code.push_str(&format!("            section: {:?},\n", key.section));
        code.push_str(&format!("            table: {:?},\n", key.table));
        code.push_str(&format!("            key: {:?},\n", key.key));
        code.push_str("        },\n");
        code.push_str(&format!("        required: {},\n", key.required));
        code.push_str(&format!("        status: {},\n", key.status.as_code_str()));
        code.push_str("    },\n");
    }
    code.push_str("];\n\n");

    // find() helper function
    code.push_str("/// Find a key by section, table, and key name.\n");
    code.push_str(
        "pub fn find(section: &str, table: &str, key: &str) -> Option<&'static KeyInfo> {\n",
    );
    code.push_str("    ALL_KEYS.iter().find(|info| {\n");
    code.push_str(
        "        info.key.section == section && info.key.table == table && info.key.key == key\n",
    );
    code.push_str("    })\n");
    code.push_str("}\n");

    code
}
