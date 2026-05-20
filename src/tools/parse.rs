//! Parse / validate / format tools.

use rmcp::model::{CallToolResult, Content};

use crate::cli_runner::AelmCli;

use super::{error_result, json_result};

/// `parse` — parse source into structured IR.
/// CLI: `aelm parse --stdin --json`
pub async fn parse(cli: &AelmCli, source: &str) -> CallToolResult {
    match cli
        .run_json(&["parse", "--stdin", "--json"], Some(source))
        .await
    {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `validate` — parse plus DRC validation.
/// CLI: `aelm check --stdin --json`
pub async fn validate(cli: &AelmCli, source: &str) -> CallToolResult {
    match cli
        .run_json(&["check", "--stdin", "--json"], Some(source))
        .await
    {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `format` — format source; returns the formatted text.
/// CLI: `aelm fmt --stdin` (writes formatted source to stdout)
pub async fn format(cli: &AelmCli, source: &str) -> CallToolResult {
    match cli.run_binary(&["fmt", "--stdin"], Some(source)).await {
        Ok(bytes) => {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            CallToolResult::success(vec![Content::text(text)])
        }
        Err(e) => error_result(&e),
    }
}
