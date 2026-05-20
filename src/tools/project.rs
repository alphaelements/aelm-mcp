//! Project analysis tools backed by `aelm analyze *` (Issue #213 CLI).

use rmcp::model::CallToolResult;

use crate::cli_runner::AelmCli;

use super::{error_result, json_result};

/// `analyze_project` — holistic summary (module stats, parts, DRC, complexity).
/// CLI: `aelm analyze summary --stdin --json`
pub async fn analyze_project(cli: &AelmCli, source: &str) -> CallToolResult {
    match cli
        .run_json(&["analyze", "summary", "--stdin", "--json"], Some(source))
        .await
    {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `extract_netlist` — net connectivity graph.
/// CLI: `aelm analyze netlist --stdin --json [--module <m>]`
pub async fn extract_netlist(cli: &AelmCli, source: &str, module: Option<&str>) -> CallToolResult {
    let mut args = vec!["analyze", "netlist", "--stdin", "--json"];
    if let Some(m) = module {
        args.push("--module");
        args.push(m);
    }
    match cli.run_json(&args, Some(source)).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `extract_bom` — bill of materials grouped by part and value.
/// CLI: `aelm analyze bom --stdin --json [--module <m>]`
pub async fn extract_bom(cli: &AelmCli, source: &str, module: Option<&str>) -> CallToolResult {
    let mut args = vec!["analyze", "bom", "--stdin", "--json"];
    if let Some(m) = module {
        args.push("--module");
        args.push(m);
    }
    match cli.run_json(&args, Some(source)).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `analyze_connectivity` — topological analysis (isolated instances, fan-out).
/// CLI: `aelm analyze connectivity --stdin --json [--module <m>]`
pub async fn analyze_connectivity(
    cli: &AelmCli,
    source: &str,
    module: Option<&str>,
) -> CallToolResult {
    let mut args = vec!["analyze", "connectivity", "--stdin", "--json"];
    if let Some(m) = module {
        args.push("--module");
        args.push(m);
    }
    match cli.run_json(&args, Some(source)).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `extract_subcircuit` — pull a set of instances into a standalone module.
/// CLI: `aelm analyze extract --stdin --json --instances <A,B,C> [--module <m>]`
pub async fn extract_subcircuit(
    cli: &AelmCli,
    source: &str,
    instances: &str,
    module: Option<&str>,
) -> CallToolResult {
    let mut args = vec![
        "analyze",
        "extract",
        "--stdin",
        "--json",
        "--instances",
        instances,
    ];
    if let Some(m) = module {
        args.push("--module");
        args.push(m);
    }
    match cli.run_json(&args, Some(source)).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}
