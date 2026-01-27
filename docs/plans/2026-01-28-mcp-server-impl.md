# speccade-mcp Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build an MCP server crate that exposes SpecCade's asset pipeline as tools for any MCP-compatible LLM client.

**Architecture:** A standalone binary (`speccade-mcp`) using the `rmcp` crate for MCP protocol handling over stdio. Each tool wraps an existing `speccade` CLI command via subprocess execution. The editor's file watcher provides live preview for free.

**Tech Stack:** Rust, rmcp (official MCP SDK), tokio, serde, schemars (JsonSchema), clap

---

### Task 1: Scaffold the crate

**Files:**
- Create: `crates/speccade-mcp/Cargo.toml`
- Create: `crates/speccade-mcp/src/main.rs`
- Create: `crates/speccade-mcp/src/cli_runner.rs`
- Create: `crates/speccade-mcp/src/tools/mod.rs`
- Modify: `Cargo.toml` (workspace members)

**Step 1: Create `crates/speccade-mcp/Cargo.toml`**

```toml
[package]
name = "speccade-mcp"
version = "0.1.0"
edition.workspace = true
license.workspace = true

[[bin]]
name = "speccade-mcp"
path = "src/main.rs"

[dependencies]
rmcp = { version = "0.1", features = ["server", "transport-io"] }
serde.workspace = true
serde_json.workspace = true
tokio = { workspace = true, features = ["rt-multi-thread", "process", "fs"] }
clap.workspace = true
anyhow.workspace = true
schemars = "0.8"
glob = "0.3"
```

> Note: `rmcp` version/features may need adjusting after checking crates.io. The key features needed are `server` (ServerHandler) and `transport-io` (stdio transport). Check with `cargo add rmcp --features server` to see available features.

**Step 2: Create `crates/speccade-mcp/src/cli_runner.rs`**

Shared helper to run `speccade` CLI commands as subprocesses:

```rust
use anyhow::{bail, Result};
use std::path::PathBuf;
use tokio::process::Command;

/// Find the speccade binary (installed or via cargo)
fn speccade_bin() -> PathBuf {
    // Check if `speccade` is on PATH
    if which::which("speccade").is_ok() {
        return PathBuf::from("speccade");
    }
    // Fallback: assume we're in the workspace and use cargo run
    PathBuf::from("speccade")
}

pub struct CliOutput {
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

pub async fn run_cli(args: &[&str]) -> Result<CliOutput> {
    let bin = speccade_bin();
    let output = Command::new(&bin)
        .args(args)
        .output()
        .await?;

    Ok(CliOutput {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        success: output.status.success(),
    })
}

pub async fn run_cli_json(args: &[&str]) -> Result<serde_json::Value> {
    let output = run_cli(args).await?;
    if !output.success {
        bail!("speccade {} failed: {}", args.join(" "), output.stderr);
    }
    let val: serde_json::Value = serde_json::from_str(&output.stdout)?;
    Ok(val)
}
```

**Step 3: Create stub `crates/speccade-mcp/src/tools/mod.rs`**

```rust
pub mod discovery;
pub mod authoring;
pub mod generation;
pub mod analysis;
```

**Step 4: Create minimal `crates/speccade-mcp/src/main.rs`**

```rust
mod cli_runner;
mod tools;

use rmcp::ServiceExt;
use rmcp::transport::stdio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = tools::SpeccadeMcp::new()
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
```

**Step 5: Add to workspace `Cargo.toml`**

Add `"crates/speccade-mcp"` to the `members` list.

**Step 6: Verify it compiles**

Run: `cargo check -p speccade-mcp`
Expected: Compiles (tools module will have stubs/empty files initially)

**Step 7: Commit**

```bash
git add crates/speccade-mcp/ Cargo.toml
git commit -m "feat(mcp): scaffold speccade-mcp crate with rmcp + cli_runner"
```

---

### Task 2: Implement the SpeccadeMcp server struct + discovery tools

**Files:**
- Create: `crates/speccade-mcp/src/tools/discovery.rs`
- Modify: `crates/speccade-mcp/src/tools/mod.rs`

**Step 1: Implement `tools/mod.rs` with the server struct**

```rust
pub mod discovery;
pub mod authoring;
pub mod generation;
pub mod analysis;

use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, tool_router};

#[derive(Clone)]
pub struct SpeccadeMcp {
    project_dir: std::path::PathBuf,
}

impl SpeccadeMcp {
    pub fn new() -> Self {
        let project_dir = std::env::current_dir().unwrap_or_default();
        Self { project_dir }
    }
}

// Tool routing and handler will be implemented via rmcp macros.
// Each tool module adds methods to SpeccadeMcp via impl blocks.
```

> Note: The exact rmcp macro syntax may differ from docs.rs examples. The implementer should check `rmcp` 's actual API by reading its source/examples after `cargo add`. The pattern is: `#[tool_handler]` on `impl ServerHandler for SpeccadeMcp`, with `#[tool(...)]` on each async method.

**Step 2: Implement `tools/discovery.rs`**

Five tools: `stdlib_reference`, `list_templates`, `get_template`, `list_specs`, `read_spec`.

```rust
use crate::cli_runner;
use rmcp::model::{CallToolResult, Content};
use schemars::JsonSchema;
use serde::Deserialize;

// --- stdlib_reference: no params ---

pub async fn stdlib_reference() -> anyhow::Result<String> {
    let output = cli_runner::run_cli(&["stdlib", "dump", "--format", "json"]).await?;
    if !output.success {
        anyhow::bail!("stdlib dump failed: {}", output.stderr);
    }
    Ok(output.stdout)
}

// --- list_templates ---

#[derive(Deserialize, JsonSchema)]
pub struct ListTemplatesParams {
    /// Filter by asset type (e.g. "audio", "texture", "mesh"). Optional.
    pub asset_type: Option<String>,
}

pub async fn list_templates(params: ListTemplatesParams) -> anyhow::Result<String> {
    let mut args = vec!["template", "list", "--json"];
    let at;
    if let Some(ref asset_type) = params.asset_type {
        at = asset_type.clone();
        args.push("--asset-type");
        args.push(&at);
    }
    let output = cli_runner::run_cli(&args).await?;
    if !output.success {
        anyhow::bail!("template list failed: {}", output.stderr);
    }
    Ok(output.stdout)
}

// --- get_template ---

#[derive(Deserialize, JsonSchema)]
pub struct GetTemplateParams {
    /// Template ID (asset_id from list_templates)
    pub template_id: String,
    /// Asset type (required to locate the template)
    pub asset_type: Option<String>,
}

pub async fn get_template(params: GetTemplateParams) -> anyhow::Result<String> {
    let mut args = vec!["template", "show"];
    args.push(&params.template_id);
    let at;
    if let Some(ref asset_type) = params.asset_type {
        at = asset_type.clone();
        args.push("--asset-type");
        args.push(&at);
    }
    let output = cli_runner::run_cli(&args).await?;
    if !output.success {
        anyhow::bail!("template show failed: {}", output.stderr);
    }
    Ok(output.stdout)
}

// --- list_specs ---

#[derive(Deserialize, JsonSchema)]
pub struct ListSpecsParams {
    /// Directory to search for .star files. Defaults to project root.
    pub directory: Option<String>,
}

pub async fn list_specs(params: ListSpecsParams) -> anyhow::Result<String> {
    let dir = params.directory.unwrap_or_else(|| ".".to_string());
    let pattern = format!("{}/**/*.star", dir);
    let paths: Vec<String> = glob::glob(&pattern)?
        .filter_map(|p| p.ok())
        .map(|p| p.display().to_string())
        .collect();
    Ok(serde_json::to_string_pretty(&paths)?)
}

// --- read_spec ---

#[derive(Deserialize, JsonSchema)]
pub struct ReadSpecParams {
    /// Path to the .star spec file
    pub path: String,
}

pub async fn read_spec(params: ReadSpecParams) -> anyhow::Result<String> {
    let content = tokio::fs::read_to_string(&params.path).await?;
    Ok(content)
}
```

**Step 3: Wire tools into the server handler**

Update `tools/mod.rs` to register these as MCP tools using rmcp's `#[tool]` macro pattern. The exact wiring depends on rmcp's API -- the implementer should follow the `#[tool_handler]` + `#[tool]` pattern from rmcp examples.

**Step 4: Verify it compiles**

Run: `cargo check -p speccade-mcp`

**Step 5: Commit**

```bash
git add crates/speccade-mcp/src/
git commit -m "feat(mcp): add discovery tools (stdlib, templates, list/read specs)"
```

---

### Task 3: Implement authoring tools

**Files:**
- Create: `crates/speccade-mcp/src/tools/authoring.rs`

**Step 1: Implement `tools/authoring.rs`**

Two tools: `write_spec`, `eval_spec`.

```rust
use crate::cli_runner;
use schemars::JsonSchema;
use serde::Deserialize;

// --- write_spec ---

#[derive(Deserialize, JsonSchema)]
pub struct WriteSpecParams {
    /// Path where the .star file should be written
    pub path: String,
    /// Starlark source code content
    pub content: String,
}

pub async fn write_spec(params: WriteSpecParams) -> anyhow::Result<String> {
    // Ensure parent directory exists
    if let Some(parent) = std::path::Path::new(&params.path).parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(&params.path, &params.content).await?;
    Ok(format!("Written to {}", params.path))
}

// --- eval_spec ---

#[derive(Deserialize, JsonSchema)]
pub struct EvalSpecParams {
    /// Path to the .star spec file to evaluate
    pub path: String,
}

pub async fn eval_spec(params: EvalSpecParams) -> anyhow::Result<String> {
    let output = cli_runner::run_cli(&["eval", "--spec", &params.path, "--json"]).await?;
    if !output.success {
        // Return the error output as content rather than failing,
        // so the LLM can see compilation errors and fix them
        return Ok(format!("{{\"success\": false, \"errors\": [\"{}\"]}}",
            output.stderr.replace('"', "\\\"")));
    }
    Ok(output.stdout)
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p speccade-mcp`

**Step 3: Commit**

```bash
git add crates/speccade-mcp/src/tools/authoring.rs
git commit -m "feat(mcp): add authoring tools (write_spec, eval_spec)"
```

---

### Task 4: Implement generation tools

**Files:**
- Create: `crates/speccade-mcp/src/tools/generation.rs`

**Step 1: Implement `tools/generation.rs`**

Three tools: `validate_spec`, `generate_preview`, `generate_full`.

```rust
use crate::cli_runner;
use schemars::JsonSchema;
use serde::Deserialize;

// --- validate_spec ---

#[derive(Deserialize, JsonSchema)]
pub struct ValidateSpecParams {
    /// Path to the .star spec file
    pub path: String,
    /// Budget profile: "default", "strict", "zx-8bit", or "nethercore"
    pub budget: Option<String>,
}

pub async fn validate_spec(params: ValidateSpecParams) -> anyhow::Result<String> {
    let mut args = vec!["validate", "--spec", &params.path, "--json"];
    let budget;
    if let Some(ref b) = params.budget {
        budget = b.clone();
        args.push("--budget");
        args.push(&budget);
    }
    let output = cli_runner::run_cli(&args).await?;
    // Return both success and failure output -- LLM needs to see errors
    Ok(output.stdout)
}

// --- generate_preview ---

#[derive(Deserialize, JsonSchema)]
pub struct GeneratePreviewParams {
    /// Path to the .star spec file
    pub path: String,
}

pub async fn generate_preview(params: GeneratePreviewParams) -> anyhow::Result<String> {
    let output = cli_runner::run_cli(&[
        "generate", "--spec", &params.path, "--preview", "2", "--json",
    ]).await?;
    Ok(output.stdout)
}

// --- generate_full ---

#[derive(Deserialize, JsonSchema)]
pub struct GenerateFullParams {
    /// Path to the .star spec file
    pub path: String,
    /// Output directory. Defaults to current directory.
    pub out_dir: Option<String>,
}

pub async fn generate_full(params: GenerateFullParams) -> anyhow::Result<String> {
    let mut args = vec!["generate", "--spec", &params.path, "--json"];
    let out;
    if let Some(ref dir) = params.out_dir {
        out = dir.clone();
        args.push("--out-root");
        args.push(&out);
    }
    let output = cli_runner::run_cli(&args).await?;
    Ok(output.stdout)
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p speccade-mcp`

**Step 3: Commit**

```bash
git add crates/speccade-mcp/src/tools/generation.rs
git commit -m "feat(mcp): add generation tools (validate, preview, full)"
```

---

### Task 5: Implement analysis tools

**Files:**
- Create: `crates/speccade-mcp/src/tools/analysis.rs`

**Step 1: Implement `tools/analysis.rs`**

Two tools: `analyze_asset`, `compare_assets`.

```rust
use crate::cli_runner;
use schemars::JsonSchema;
use serde::Deserialize;

// --- analyze_asset ---

#[derive(Deserialize, JsonSchema)]
pub struct AnalyzeAssetParams {
    /// Path to the generated asset file (WAV, PNG, GLB)
    pub path: String,
}

pub async fn analyze_asset(params: AnalyzeAssetParams) -> anyhow::Result<String> {
    let output = cli_runner::run_cli(&[
        "analyze", "--input", &params.path, "--json",
    ]).await?;
    Ok(if output.success { output.stdout } else { output.stderr })
}

// --- compare_assets ---

#[derive(Deserialize, JsonSchema)]
pub struct CompareAssetsParams {
    /// Path to the reference asset
    pub path_a: String,
    /// Path to the comparison target
    pub path_b: String,
}

pub async fn compare_assets(params: CompareAssetsParams) -> anyhow::Result<String> {
    let output = cli_runner::run_cli(&[
        "compare", "-a", &params.path_a, "-b", &params.path_b, "--json",
    ]).await?;
    Ok(if output.success { output.stdout } else { output.stderr })
}
```

**Step 2: Verify it compiles**

Run: `cargo check -p speccade-mcp`

**Step 3: Commit**

```bash
git add crates/speccade-mcp/src/tools/analysis.rs
git commit -m "feat(mcp): add analysis tools (analyze_asset, compare_assets)"
```

---

### Task 6: Wire all tools into rmcp ServerHandler and verify end-to-end

**Files:**
- Modify: `crates/speccade-mcp/src/tools/mod.rs`
- Modify: `crates/speccade-mcp/src/main.rs`

**Step 1: Complete the ServerHandler implementation**

Wire all 12 tool functions into the rmcp `#[tool_handler]` impl. Each tool function becomes a `#[tool(name = "...", description = "...")]` method on `SpeccadeMcp`.

The exact wiring pattern follows rmcp's API:

```rust
use rmcp::{tool_handler, ServerHandler};
use rmcp::model::{ServerInfo, ServerCapabilities, CallToolResult, Content};

#[tool_handler]
impl ServerHandler for SpeccadeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "SpecCade asset pipeline tools. Use stdlib_reference to see available \
                 Starlark functions, list_templates to find starter templates, \
                 write_spec to create/edit specs, and generate_preview/generate_full \
                 to produce assets.".into()
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }

    // Each #[tool] method delegates to the functions in submodules
    #[tool(description = "Get the complete Starlark stdlib function reference (all available spec authoring functions, their parameters, and examples)")]
    async fn stdlib_reference(&self) -> Result<CallToolResult, McpError> {
        match discovery::stdlib_reference().await {
            Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(e.to_string())])),
        }
    }

    // ... same pattern for all 12 tools
}
```

**Step 2: Add clap CLI args for --project-dir**

Update `main.rs`:

```rust
use clap::Parser;

#[derive(Parser)]
struct Args {
    /// Project directory (defaults to current dir)
    #[arg(long, default_value = ".")]
    project_dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    std::env::set_current_dir(&args.project_dir)?;

    let service = SpeccadeMcp::new()
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
```

**Step 3: Build and run**

Run: `cargo build -p speccade-mcp`
Expected: Compiles successfully

**Step 4: Smoke test with stdio**

Run: `echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"test"},"protocolVersion":"2024-11-05"}}' | cargo run -p speccade-mcp --`
Expected: JSON response with server info and tool list

**Step 5: Commit**

```bash
git add crates/speccade-mcp/
git commit -m "feat(mcp): wire all 12 tools into rmcp ServerHandler"
```

---

### Task 7: Add .mcp.json and documentation

**Files:**
- Create: `.mcp.json` (project root)
- Modify: `docs/plans/2026-01-28-mcp-server-design.md` (mark status as Implemented)

**Step 1: Create `.mcp.json`**

```json
{
  "mcpServers": {
    "speccade": {
      "command": "cargo",
      "args": ["run", "-p", "speccade-mcp", "--"],
      "env": {}
    }
  }
}
```

**Step 2: Update design doc status**

Change `**Status:** Draft` to `**Status:** Implemented`.

**Step 3: Verify end-to-end with Claude Code**

Restart Claude Code in the speccade directory. It should auto-detect `.mcp.json` and load the speccade MCP server. Test by asking Claude Code to:
1. Call `stdlib_reference` -- should return Starlark function catalog
2. Call `list_templates` -- should return preset templates
3. Call `write_spec` with a simple audio spec -- should create a `.star` file

**Step 4: Commit**

```bash
git add .mcp.json docs/plans/2026-01-28-mcp-server-design.md
git commit -m "feat(mcp): add .mcp.json config and mark design as implemented"
```

---

### Important Notes for the Implementer

1. **rmcp API uncertainty:** The code examples above are based on docs.rs documentation fetched during planning. The `rmcp` crate is evolving -- after `cargo add rmcp`, check the actual available features and API surface. The feature flags might be named differently (e.g., `transport-io` vs `transport-stdio`). Run `cargo doc -p rmcp --open` to browse the actual API.

2. **Tool macro pattern:** If `rmcp` uses a different tool definition pattern than shown (e.g., a `ToolRouter` builder instead of `#[tool]` attribute macros), adapt accordingly. The key requirement is: 12 tools, each calling the subprocess helper, returning the CLI's JSON output as text content.

3. **Error handling:** Tools should return errors as `CallToolResult::error(...)` content, not as MCP protocol errors. This lets the LLM see the error message and fix the spec.

4. **`which` crate:** Already in workspace dependencies. Use it to find the `speccade` binary.

5. **Windows paths:** The project runs on Windows. Use `std::path` consistently, not string path manipulation.
