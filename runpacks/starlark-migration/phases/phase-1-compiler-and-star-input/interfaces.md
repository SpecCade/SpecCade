# Phase 1 Interfaces

This document defines the new types, functions, and CLI commands introduced in Phase 1.

---

## 1. Input Module (`crates/speccade-cli/src/input.rs`)

### Source Kind Enum

```rust
use serde::{Deserialize, Serialize};

/// Identifies the source format of a spec file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// JSON spec file (existing format)
    Json,
    /// Starlark spec file (new format)
    Starlark,
}

impl SourceKind {
    /// Returns the string representation for reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            SourceKind::Json => "json",
            SourceKind::Starlark => "starlark",
        }
    }
}
```

### Compile Warning

```rust
/// A warning generated during Starlark compilation.
#[derive(Debug, Clone)]
pub struct CompileWarning {
    /// Warning message
    pub message: String,
    /// Source location (line:column) if available
    pub location: Option<String>,
}
```

### Load Result

```rust
use speccade_spec::Spec;

/// Result of loading and compiling a spec from any supported format.
#[derive(Debug)]
pub struct LoadResult {
    /// The canonical spec IR
    pub spec: Spec,
    /// Source format
    pub source_kind: SourceKind,
    /// BLAKE3 hash of the source file content (hex string)
    pub source_hash: String,
    /// Warnings from compilation (Starlark only; empty for JSON)
    pub warnings: Vec<CompileWarning>,
}
```

### Input Error

```rust
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur during spec loading.
#[derive(Debug, Error)]
pub enum InputError {
    /// File could not be read
    #[error("failed to read file: {path}")]
    FileRead {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Unknown file extension
    #[error("unknown file extension: {extension:?} (expected .json or .star)")]
    UnknownExtension { extension: Option<String> },

    /// JSON parsing failed
    #[error("JSON parse error: {message}")]
    JsonParse { message: String },

    /// Starlark compilation failed
    #[error("Starlark error: {message}")]
    StarlarkCompile { message: String },

    /// Starlark evaluation timed out
    #[error("Starlark evaluation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Starlark output is not a valid spec
    #[error("Starlark output is not a valid spec: {message}")]
    InvalidSpec { message: String },
}
```

### Load Function

```rust
use std::path::Path;

/// Load a spec from a file path, dispatching by extension.
///
/// # Arguments
/// * `path` - Path to the spec file (.json or .star)
///
/// # Returns
/// * `Ok(LoadResult)` - Successfully loaded and parsed spec
/// * `Err(InputError)` - File read, parse, or compilation error
///
/// # Example
/// ```no_run
/// use std::path::Path;
/// use speccade_cli::input::load_spec;
///
/// let result = load_spec(Path::new("spec.star"))?;
/// println!("Loaded {} spec", result.source_kind.as_str());
/// ```
pub fn load_spec(path: &Path) -> Result<LoadResult, InputError>;
```

---

## 2. Compiler Module (`crates/speccade-cli/src/compiler/`)

### Compiler Configuration

```rust
/// Configuration for the Starlark compiler.
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Maximum evaluation time in seconds (default: 30)
    pub timeout_seconds: u64,
    /// Whether to enable Starlark `load()` statements (default: false)
    pub enable_load: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 30,
            enable_load: false,
        }
    }
}
```

### Compile Result

```rust
/// Result of Starlark compilation.
#[derive(Debug)]
pub struct CompileResult {
    /// The parsed spec
    pub spec: Spec,
    /// Compilation warnings
    pub warnings: Vec<CompileWarning>,
}
```

### Compile Error

```rust
use thiserror::Error;

/// Errors from Starlark compilation.
#[derive(Debug, Error)]
pub enum CompileError {
    /// Syntax error in Starlark source
    #[error("syntax error at {location}: {message}")]
    Syntax { location: String, message: String },

    /// Runtime error during evaluation
    #[error("runtime error at {location}: {message}")]
    Runtime { location: String, message: String },

    /// Evaluation timed out
    #[error("evaluation timed out after {seconds}s")]
    Timeout { seconds: u64 },

    /// Output value is not a dict
    #[error("spec must return a dict, got {type_name}")]
    NotADict { type_name: String },

    /// Output cannot be converted to JSON
    #[error("cannot convert to JSON: {message}")]
    JsonConversion { message: String },

    /// Resulting JSON is not a valid Spec
    #[error("invalid spec: {message}")]
    InvalidSpec { message: String },
}
```

### Compile Function

```rust
/// Compile Starlark source to a canonical Spec.
///
/// # Arguments
/// * `filename` - Filename for error messages
/// * `source` - Starlark source code
/// * `config` - Compiler configuration
///
/// # Returns
/// * `Ok(CompileResult)` - Successfully compiled spec
/// * `Err(CompileError)` - Compilation failed
pub fn compile(
    filename: &str,
    source: &str,
    config: &CompilerConfig,
) -> Result<CompileResult, CompileError>;
```

---

## 3. CLI Commands

### Eval Command (New)

```
speccade eval [OPTIONS] --spec <SPEC>

Evaluate a spec file and print canonical IR JSON to stdout.

Arguments:
  -s, --spec <SPEC>    Path to the spec file (JSON or Starlark)

Options:
  -p, --pretty         Pretty-print the output JSON (default: false)
  -h, --help           Print help
```

**Rust definition:**

```rust
#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Evaluate a spec file and print canonical IR JSON
    Eval {
        /// Path to the spec file (JSON or Starlark)
        #[arg(short, long)]
        spec: String,

        /// Pretty-print the output JSON
        #[arg(short, long)]
        pretty: bool,
    },
}
```

**Run function:**

```rust
// crates/speccade-cli/src/commands/eval.rs

use crate::input::load_spec;
use anyhow::Result;
use std::path::Path;
use std::process::ExitCode;

/// Run the eval command.
///
/// Prints canonical IR JSON to stdout.
/// Returns exit code 0 on success, 1 on error.
pub fn run(spec_path: &str, pretty: bool) -> Result<ExitCode> {
    let path = Path::new(spec_path);
    let result = load_spec(path)?;

    let json = if pretty {
        result.spec.to_json_pretty()?
    } else {
        result.spec.to_json()?
    };

    println!("{}", json);
    Ok(ExitCode::SUCCESS)
}
```

### Updated Validate Command

```
speccade validate [OPTIONS] --spec <SPEC>

Validate a spec file without generating assets.

Arguments:
  -s, --spec <SPEC>    Path to the spec file (JSON or Starlark)

Options:
      --artifacts      Also validate artifact references (paths, formats)
  -h, --help           Print help
```

No signature change; internal implementation uses `load_spec()`.

### Updated Generate Command

```
speccade generate [OPTIONS] --spec <SPEC>

Generate assets from a spec file.

Arguments:
  -s, --spec <SPEC>    Path to the spec file (JSON or Starlark)

Options:
  -o, --out-root <OUT_ROOT>    Output root directory (default: current directory)
      --expand-variants        Expand variants into separate generation runs
  -h, --help                   Print help
```

No signature change; internal implementation uses `load_spec()`.

---

## 4. Report Additions

### New Report Fields

```rust
// In crates/speccade-spec/src/report/mod.rs

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    // ... existing fields ...

    /// Source file format ("json" or "starlark")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,

    /// BLAKE3 hash of the source file content (before compilation)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,

    /// Starlark stdlib version (for Starlark sources; cache invalidation key)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdlib_version: Option<String>,
}
```

### New Builder Methods

```rust
// In crates/speccade-spec/src/report/builder.rs

impl ReportBuilder {
    // ... existing methods ...

    /// Sets the source provenance information.
    pub fn source_provenance(
        mut self,
        kind: impl Into<String>,
        hash: impl Into<String>,
    ) -> Self {
        self.source_kind = Some(kind.into());
        self.source_hash = Some(hash.into());
        self
    }

    /// Sets the Starlark stdlib version (for Starlark sources).
    pub fn stdlib_version(mut self, version: impl Into<String>) -> Self {
        self.stdlib_version = Some(version.into());
        self
    }
}
```

### Example Report (Starlark source)

```json
{
  "report_version": 1,
  "spec_hash": "a1b2c3d4...",
  "source_kind": "starlark",
  "source_hash": "e5f6g7h8...",
  "stdlib_version": "0.1.0",
  "asset_id": "laser-blast-01",
  "asset_type": "audio",
  "ok": true,
  "errors": [],
  "warnings": [],
  "outputs": [...],
  "duration_ms": 123,
  "backend_version": "speccade-cli v0.1.0",
  "target_triple": "x86_64-pc-windows-msvc"
}
```

### Example Report (JSON source)

```json
{
  "report_version": 1,
  "spec_hash": "a1b2c3d4...",
  "source_kind": "json",
  "source_hash": "a1b2c3d4...",
  "asset_id": "laser-blast-01",
  "asset_type": "audio",
  "ok": true,
  "errors": [],
  "warnings": [],
  "outputs": [...],
  "duration_ms": 45,
  "backend_version": "speccade-cli v0.1.0",
  "target_triple": "x86_64-pc-windows-msvc"
}
```

Note: For JSON sources, `source_hash` equals `spec_hash` (no transformation).

---

## 5. Error Types

### Error Hierarchy

```
InputError (user-facing, in commands)
├── FileRead
├── UnknownExtension
├── JsonParse
├── StarlarkCompile (wraps CompileError)
├── Timeout
└── InvalidSpec

CompileError (internal, in compiler module)
├── Syntax
├── Runtime
├── Timeout
├── NotADict
├── JsonConversion
└── InvalidSpec
```

### Error Code Mapping

| Error Type | Report Code | Description |
|------------|-------------|-------------|
| `InputError::FileRead` | E001 | File read failure |
| `InputError::UnknownExtension` | E002 | Unknown file extension |
| `InputError::JsonParse` | E003 | JSON parse error |
| `InputError::StarlarkCompile` | E004 | Starlark compilation error |
| `InputError::Timeout` | E005 | Starlark timeout |
| `InputError::InvalidSpec` | E006 | Invalid spec structure |

Note: These are new error codes for the input layer; existing validation error codes (E010-E099) remain unchanged.

---

## 6. Feature Gate

### Compile-time Configuration

```rust
// In crates/speccade-cli/src/input.rs

#[cfg(feature = "starlark")]
use crate::compiler;

pub fn load_spec(path: &Path) -> Result<LoadResult, InputError> {
    let extension = path.extension().and_then(|e| e.to_str());

    match extension {
        Some("json") => load_json_spec(path),
        #[cfg(feature = "starlark")]
        Some("star") | Some("bzl") => load_starlark_spec(path),
        #[cfg(not(feature = "starlark"))]
        Some("star") | Some("bzl") => Err(InputError::StarlarkNotEnabled),
        _ => Err(InputError::UnknownExtension {
            extension: extension.map(String::from),
        }),
    }
}
```

### CLI Help (without Starlark)

When built without the `starlark` feature:
```
speccade validate --spec <SPEC>

Arguments:
  -s, --spec <SPEC>    Path to the spec JSON file

Note: Starlark (.star) support is disabled. Rebuild with --features starlark.
```

---

## 7. Constants

### Stdlib Version

```rust
// In crates/speccade-cli/src/compiler/mod.rs

/// Current Starlark stdlib version.
/// Increment when stdlib changes affect output.
pub const STDLIB_VERSION: &str = "0.1.0";
```

### Timeout Default

```rust
// In crates/speccade-cli/src/compiler/mod.rs

/// Default Starlark evaluation timeout in seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 30;
```

### Supported Extensions

```rust
// In crates/speccade-cli/src/input.rs

/// Recognized JSON extensions
pub const JSON_EXTENSIONS: &[&str] = &["json"];

/// Recognized Starlark extensions
pub const STARLARK_EXTENSIONS: &[&str] = &["star", "bzl"];
```
