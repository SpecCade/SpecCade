pub mod analysis;
pub mod authoring;
pub mod discovery;
pub mod generation;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, tool_router, ServerHandler};
use base64::Engine;
use std::path::{Component, Path, PathBuf};

use crate::cli_runner;

use analysis::{AnalyzeAssetParams, CompareAssetsParams};
use authoring::{EvalSpecParams, WriteSpecParams};
use discovery::{GetTemplateParams, ListSpecsParams, ListTemplatesParams, ReadSpecParams};
use generation::{GenerateFullParams, GeneratePngOutputsParams, GeneratePreviewParams, ValidateSpecParams};

#[derive(Clone)]
pub struct SpeccadeMcp {
    tool_router: ToolRouter<Self>,
}

impl SpeccadeMcp {
    /// Access the tool router for testing/introspection.
    #[allow(dead_code)]
    pub fn router(&self) -> &ToolRouter<Self> {
        &self.tool_router
    }
}

#[tool_router]
impl SpeccadeMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    // ── Discovery ──────────────────────────────────────────

    /// Return the full SpecCade stdlib reference as JSON. Lists all built-in Starlark functions, their signatures, and documentation.
    #[rmcp::tool]
    async fn stdlib_reference(&self) -> Result<CallToolResult, rmcp::ErrorData> {
        match cli_runner::run_cli(&["stdlib", "dump", "--format", "json"]).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                out.stderr
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// List available spec templates. Optionally filter by asset_type (e.g. 'sprite', 'tilemap').
    #[rmcp::tool]
    async fn list_templates(
        &self,
        Parameters(params): Parameters<ListTemplatesParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["template", "list", "--json"];
        let at;
        if let Some(ref asset_type) = params.asset_type {
            at = asset_type.clone();
            args.push("--asset-type");
            args.push(&at);
        }
        match cli_runner::run_cli(&args).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                out.stderr
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Get the source of a specific template by ID. Optionally provide asset_type for context.
    #[rmcp::tool]
    async fn get_template(
        &self,
        Parameters(params): Parameters<GetTemplateParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["template", "show"];
        let id = params.template_id.clone();
        args.push(&id);
        let at;
        if let Some(ref asset_type) = params.asset_type {
            at = asset_type.clone();
            args.push("--asset-type");
            args.push(&at);
        }
        match cli_runner::run_cli(&args).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                out.stderr
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// List all .star spec files in a directory (recursively). Returns JSON array of paths.
    #[rmcp::tool]
    async fn list_specs(
        &self,
        Parameters(params): Parameters<ListSpecsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let dir = params.directory.unwrap_or_else(|| ".".into());
        let pattern = format!("{dir}/**/*.star");
        let mut paths = Vec::new();
        match glob::glob(&pattern) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    paths.push(entry.display().to_string());
                }
                let json = serde_json::to_string_pretty(&paths).unwrap_or_default();
                Ok(CallToolResult::success(vec![Content::text(json)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Read the contents of a .star spec file.
    #[rmcp::tool]
    async fn read_spec(
        &self,
        Parameters(params): Parameters<ReadSpecParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match tokio::fs::read_to_string(&params.path).await {
            Ok(content) => Ok(CallToolResult::success(vec![Content::text(content)])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error reading {}: {e}",
                params.path
            ))])),
        }
    }

    // ── Authoring ──────────────────────────────────────────

    /// Write or overwrite a .star spec file. Creates parent directories if needed. Triggers editor hot-reload via file watcher.
    #[rmcp::tool]
    async fn write_spec(
        &self,
        Parameters(params): Parameters<WriteSpecParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let path = std::path::Path::new(&params.path);
        if let Some(parent) = path.parent() {
            if let Err(e) = tokio::fs::create_dir_all(parent).await {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error creating directories: {e}"
                ))]));
            }
        }
        match tokio::fs::write(&params.path, &params.content).await {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Wrote {} ({} bytes)",
                params.path,
                params.content.len()
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error writing {}: {e}",
                params.path
            ))])),
        }
    }

    /// Evaluate a Starlark spec and return the resulting JSON. Shows compilation errors on failure.
    #[rmcp::tool]
    async fn eval_spec(
        &self,
        Parameters(params): Parameters<EvalSpecParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match cli_runner::run_cli(&["eval", "--spec", &params.path, "--json"]).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => {
                let msg = if out.stderr.is_empty() {
                    out.stdout
                } else {
                    out.stderr
                };
                Ok(CallToolResult::success(vec![Content::text(format!(
                    "Eval failed:\n{msg}"
                ))]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    // ── Generation ─────────────────────────────────────────

    /// Validate a spec against constraints. Returns JSON with validation results (pass/fail + diagnostics).
    #[rmcp::tool]
    async fn validate_spec(
        &self,
        Parameters(params): Parameters<ValidateSpecParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["validate", "--spec", &params.path, "--json"];
        let budget;
        if let Some(ref b) = params.budget {
            budget = b.clone();
            args.push("--budget");
            args.push(&budget);
        }
        match cli_runner::run_cli(&args).await {
            Ok(out) => {
                let text = if out.stdout.is_empty() {
                    out.stderr
                } else {
                    out.stdout
                };
                Ok(CallToolResult::success(vec![Content::text(text)]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Generate a quick preview (2 variants) from a spec. Returns JSON.
    #[rmcp::tool]
    async fn generate_preview(
        &self,
        Parameters(params): Parameters<GeneratePreviewParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match cli_runner::run_cli(&[
            "generate",
            "--spec",
            &params.path,
            "--preview",
            "2",
            "--json",
        ])
        .await
        {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                if out.stderr.is_empty() {
                    out.stdout
                } else {
                    out.stderr
                }
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Generate full asset output from a spec. Optionally specify output directory.
    #[rmcp::tool]
    async fn generate_full(
        &self,
        Parameters(params): Parameters<GenerateFullParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let mut args = vec!["generate", "--spec", &params.path, "--json"];
        let out_dir;
        if let Some(ref d) = params.out_dir {
            out_dir = d.clone();
            args.push("--out-root");
            args.push(&out_dir);
        }
        match cli_runner::run_cli(&args).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                if out.stderr.is_empty() {
                    out.stdout
                } else {
                    out.stderr
                }
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Generate all declared PNG outputs for a spec and return them as base64.
    ///
    /// This is intended as a lightweight "screenshot" mechanism for LLM workflows.
    #[rmcp::tool]
    async fn generate_png_outputs(
        &self,
        Parameters(params): Parameters<GeneratePngOutputsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        const MAX_PNG_FILES: usize = 32;
        const MAX_TOTAL_BYTES: usize = 25 * 1024 * 1024;

        let tmp = match tempfile::tempdir() {
            Ok(d) => d,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: failed to create temp dir: {e}"
                ))]));
            }
        };
        let out_root = tmp.path().to_string_lossy().to_string();

        let mut args = vec!["generate", "--spec", &params.path, "--out-root", &out_root, "--json"];
        let budget;
        if let Some(ref b) = params.budget {
            budget = b.clone();
            args.push("--budget");
            args.push(&budget);
        }

        let out = match cli_runner::run_cli(&args).await {
            Ok(out) => out,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: {e}"
                ))]));
            }
        };

        if !out.success {
            let msg = if out.stderr.is_empty() {
                out.stdout
            } else {
                out.stderr
            };
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: generate failed:\n{msg}"
            ))]));
        }

        let generate_json: serde_json::Value = match serde_json::from_str(&out.stdout) {
            Ok(v) => v,
            Err(e) => {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: failed to parse generate JSON: {e}\n{}",
                    out.stdout
                ))]));
            }
        };

        let outputs = generate_json
            .get("result")
            .and_then(|r| r.get("outputs"))
            .and_then(|o| o.as_array())
            .cloned()
            .unwrap_or_default();

        let mut pngs = Vec::new();
        let mut total_bytes: usize = 0;

        for o in outputs {
            let format = o.get("format").and_then(|v| v.as_str()).unwrap_or("");
            if format != "png" {
                continue;
            }
            let path = match o.get("path").and_then(|v| v.as_str()) {
                Some(p) => p,
                None => continue,
            };

            if pngs.len() >= MAX_PNG_FILES {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: too many PNG outputs (max {MAX_PNG_FILES})"
                ))]));
            }

            let full_path = match safe_relpath_join(tmp.path(), path) {
                Ok(p) => p,
                Err(msg) => {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Error: invalid output path '{path}': {msg}"
                    ))]));
                }
            };
            let bytes = match tokio::fs::read(&full_path).await {
                Ok(b) => b,
                Err(e) => {
                    return Ok(CallToolResult::success(vec![Content::text(format!(
                        "Error: failed to read generated PNG '{}': {e}",
                        full_path.display()
                    ))]));
                }
            };

            total_bytes = total_bytes.saturating_add(bytes.len());
            if total_bytes > MAX_TOTAL_BYTES {
                return Ok(CallToolResult::success(vec![Content::text(format!(
                    "Error: PNG outputs exceed max total size ({MAX_TOTAL_BYTES} bytes)"
                ))]));
            }

            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
            pngs.push(serde_json::json!({
                "path": path,
                "mime_type": "image/png",
                "bytes": bytes.len(),
                "base64": b64,
            }));
        }

        let asset_id = generate_json
            .get("result")
            .and_then(|r| r.get("asset_id"))
            .and_then(|v| v.as_str());

        let response = serde_json::json!({
            "asset_id": asset_id,
            "pngs": pngs,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    // ── Analysis ───────────────────────────────────────────

    /// Analyze a generated asset file. Returns JSON with metrics and metadata.
    #[rmcp::tool]
    async fn analyze_asset(
        &self,
        Parameters(params): Parameters<AnalyzeAssetParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match cli_runner::run_cli(&["analyze", "--input", &params.path, "--json"]).await {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                out.stderr
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }

    /// Compare two asset files. Returns JSON diff with similarity metrics.
    #[rmcp::tool]
    async fn compare_assets(
        &self,
        Parameters(params): Parameters<CompareAssetsParams>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        match cli_runner::run_cli(&[
            "compare",
            "-a",
            &params.path_a,
            "-b",
            &params.path_b,
            "--json",
        ])
        .await
        {
            Ok(out) if out.success => Ok(CallToolResult::success(vec![Content::text(out.stdout)])),
            Ok(out) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {}",
                out.stderr
            ))])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Error: {e}"
            ))])),
        }
    }
}

fn safe_relpath_join(root: &Path, rel: &str) -> Result<PathBuf, &'static str> {
    let p = Path::new(rel);

    // Prevent path traversal or absolute paths escaping the temp out_root.
    if p.is_absolute() {
        return Err("absolute paths are not allowed");
    }

    let mut cleaned = PathBuf::new();
    for c in p.components() {
        match c {
            Component::Normal(s) => cleaned.push(s),
            Component::CurDir => {
                // ignore
            }
            Component::ParentDir => return Err("'..' path traversal is not allowed"),
            Component::Prefix(_) | Component::RootDir => {
                return Err("path prefixes/root are not allowed")
            }
        }
    }

    if cleaned.as_os_str().is_empty() {
        return Err("empty paths are not allowed");
    }

    Ok(root.join(cleaned))
}

#[tool_handler]
impl ServerHandler for SpeccadeMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "SpecCade asset pipeline tools. Create and edit game assets using \
                 declarative Starlark specs. Use stdlib_reference to see available \
                 functions, list_templates for starter templates, write_spec to \
                 create/edit specs, and generate tools to produce assets."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine;
    use rmcp::model::RawContent;

    #[test]
    fn tool_router_includes_generate_png_outputs() {
        let mcp = SpeccadeMcp::new();
        assert!(mcp.router().map.contains_key("generate_png_outputs"));
    }

    #[test]
    fn safe_relpath_join_rejects_escaping_paths() {
        let dir = tempfile::tempdir().expect("tempdir");
        let root = dir.path();

        assert!(safe_relpath_join(root, "../evil.png").is_err());
        assert!(safe_relpath_join(root, "..\\evil.png").is_err());

        if cfg!(windows) {
            assert!(safe_relpath_join(root, "C:\\evil.png").is_err());
        } else {
            assert!(safe_relpath_join(root, "/etc/passwd").is_err());
        }

        let ok = safe_relpath_join(root, "textures/out.png").expect("ok");
        assert!(ok.starts_with(root));
    }

    #[tokio::test]
    async fn generate_png_outputs_returns_base64_pngs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let spec_path = dir.path().join("test.star");

        let content = r#"# Two-output texture spec for MCP PNG extraction tests

spec(
    asset_id = "mcp-test-two-pngs",
    asset_type = "texture",
    license = "CC0-1.0",
    seed = 42,
    outputs = [
        output("textures/height.png", "png", source = "height"),
        output("textures/mask.png", "png", source = "mask")
    ],
    recipe = {
        "kind": "texture.procedural_v1",
        "params": texture_graph(
            [64, 64],
            [
                noise_node("height", "perlin", 0.1, 4, 0.5, 2.0),
                threshold_node("mask", "height", 0.5)
            ],
            True
        )
    }
)
"#;

        tokio::fs::write(&spec_path, content)
            .await
            .expect("write spec");

        let spec_path = spec_path
            .to_str()
            .expect("spec path should be valid UTF-8")
            .to_string();

        let mcp = SpeccadeMcp::new();
        let result = mcp
            .generate_png_outputs(Parameters(GeneratePngOutputsParams {
                path: spec_path,
                budget: None,
            }))
            .await
            .expect("tool call should succeed");

        assert_eq!(result.is_error, Some(false));
        assert_eq!(result.content.len(), 1);

        let text = match &result.content[0].raw {
            RawContent::Text(t) => t.text.as_str(),
            other => panic!("expected text content, got: {other:?}"),
        };

        let json: serde_json::Value = serde_json::from_str(text)
            .unwrap_or_else(|e| panic!("expected JSON response, got parse error: {e}\n{text}"));

        let pngs = json
            .get("pngs")
            .and_then(|v| v.as_array())
            .expect("expected 'pngs' array");
        assert_eq!(pngs.len(), 2, "expected 2 png outputs");

        for item in pngs {
            assert_eq!(item.get("mime_type").and_then(|v| v.as_str()), Some("image/png"));
            let b64 = item
                .get("base64")
                .and_then(|v| v.as_str())
                .expect("expected base64 field");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(b64)
                .expect("base64 should decode");
            assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }
}
