mod cli_runner;
mod tools;

use clap::Parser;
use rmcp::ServiceExt;
use tools::SpeccadeMcp;

#[derive(Parser)]
#[command(name = "speccade-mcp", about = "MCP server for SpecCade asset pipeline")]
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
        .serve(rmcp::transport::io::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
