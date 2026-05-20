//! aelm-mcp — Model Context Protocol server for the Aelm circuit-CAD toolchain.
//!
//! Speaks MCP over stdio and fulfils every request by shelling out to the
//! `aelm` CLI, so the server stays small and the Aelm core stays independent.

use std::path::PathBuf;

use aelm_mcp::cli_runner::AelmCli;
use aelm_mcp::server::AelmMcpServer;
use clap::Parser;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

/// Command-line configuration for the MCP server.
#[derive(Parser, Debug)]
#[command(
    name = "aelm-mcp",
    version,
    about = "MCP server for the Aelm circuit-CAD toolchain."
)]
struct Args {
    /// Path to the `aelm` binary (default: search `PATH`).
    #[arg(long, value_name = "PATH", default_value = "aelm")]
    aelm_path: PathBuf,

    /// Extra user library directory passed to `aelm` as `-L`. Repeatable.
    #[arg(long = "library-dir", value_name = "DIR")]
    library_dir: Vec<PathBuf>,

    /// Working directory for relative `use` imports in circuit sources.
    #[arg(long, value_name = "DIR")]
    working_dir: Option<PathBuf>,

    /// Log level: error | warn | info | debug | trace.
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow_lite::Result {
    let args = Args::parse();

    // Logs go to stderr; stdout is reserved for the MCP stdio transport.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new(format!("aelm_mcp={}", args.log_level))),
        )
        .with_writer(std::io::stderr)
        .init();

    let cli = AelmCli::new(args.aelm_path, args.library_dir, args.working_dir);
    tracing::info!(binary = %cli.binary_path().display(), "starting aelm-mcp server");

    let service = AelmMcpServer::new(cli)
        .serve((tokio::io::stdin(), tokio::io::stdout()))
        .await
        .map_err(|e| anyhow_lite::Error(format!("failed to start MCP service: {e}")))?;

    service
        .waiting()
        .await
        .map_err(|e| anyhow_lite::Error(format!("MCP service error: {e}")))?;
    Ok(())
}

/// Minimal error wrapper so `main` can use `?` without pulling in `anyhow`.
mod anyhow_lite {
    pub struct Error(pub String);

    impl std::fmt::Debug for Error {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    pub type Result = std::result::Result<(), Error>;
}
