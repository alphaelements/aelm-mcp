//! Symbol template tools: list, info.

use rmcp::model::CallToolResult;

use crate::cli_runner::AelmCli;

use super::{error_result, json_result};

/// `list_symbols` — enumerate built-in + user-defined symbol templates.
/// CLI: `aelm symbols list --json [--category <c>]`
pub async fn list_symbols(cli: &AelmCli, category: Option<&str>) -> CallToolResult {
    let mut args = vec!["symbols", "list", "--json"];
    if let Some(c) = category {
        args.push("--category");
        args.push(c);
    }
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `get_symbol_info` — detailed symbol info (pins, bounds, draw commands).
/// CLI: `aelm symbols info <name> --json`
pub async fn get_symbol_info(cli: &AelmCli, name: &str) -> CallToolResult {
    match cli
        .run_json(&["symbols", "info", name, "--json"], None)
        .await
    {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}
