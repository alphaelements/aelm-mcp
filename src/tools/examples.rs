//! Example circuit tools: list, get.

use rmcp::model::CallToolResult;

use crate::cli_runner::AelmCli;

use super::{error_result, json_result};

/// `list_examples` — enumerate bundled example circuits.
/// CLI: `aelm examples list --json [--category <c>]`
pub async fn list_examples(cli: &AelmCli, category: Option<&str>) -> CallToolResult {
    let mut args = vec!["examples", "list", "--json"];
    if let Some(c) = category {
        args.push("--category");
        args.push(c);
    }
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `get_example` — fetch an example's source, optionally with a rendered SVG.
/// CLI: `aelm examples get <name> --json [--render]`
pub async fn get_example(cli: &AelmCli, name: &str, render: bool) -> CallToolResult {
    let mut args = vec!["examples", "get", name, "--json"];
    if render {
        args.push("--render");
    }
    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}
