//! Parts catalog tools: list, search, info.

use rmcp::model::CallToolResult;

use crate::cli_runner::AelmCli;

use super::{error_result, json_result};

/// `list_parts` — enumerate parts from stdlib + user libraries.
/// CLI: `aelm parts list --json [--category <c>]`
pub async fn list_parts(cli: &AelmCli, category: Option<&str>) -> CallToolResult {
    let mut args = vec!["parts", "list", "--json"];
    if let Some(c) = category {
        args.push("--category");
        args.push(c);
    }
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `search_parts` — fuzzy-search the part catalog.
/// CLI: `aelm parts search <query> --json [--limit <n>]`
pub async fn search_parts(cli: &AelmCli, query: &str, limit: Option<u32>) -> CallToolResult {
    let limit_str = limit.unwrap_or(20).to_string();
    let args = vec!["parts", "search", query, "--json", "--limit", &limit_str];
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `get_part_info` — detailed part info, optionally with inline symbol SVG.
/// CLI: `aelm parts info <name> --json [--render-symbol]`
pub async fn get_part_info(cli: &AelmCli, name: &str, render_symbol: bool) -> CallToolResult {
    let mut args = vec!["parts", "info", name, "--json"];
    if render_symbol {
        args.push("--render-symbol");
    }
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}
