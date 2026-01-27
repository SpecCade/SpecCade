pub mod analysis;
pub mod authoring;
pub mod discovery;
pub mod generation;

use rmcp::handler::server::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, ServerCapabilities, ServerInfo};
use rmcp::{tool_handler, tool_router, ServerHandler};

use crate::cli_runner;

use analysis::{AnalyzeAssetParams, CompareAssetsParams};
use authoring::{EvalSpecParams, WriteSpecParams};
use discovery::{GetTemplateParams, ListSpecsParams, ListTemplatesParams, ReadSpecParams};
use generation::{GenerateFullParams, GeneratePreviewParams, ValidateSpecParams};

#[derive(Clone)]
pub struct SpeccadeMcp {
    tool_router: ToolRouter<Self>,
}

impl SpeccadeMcp {
    /// Access the tool router for testing/introspection.
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
