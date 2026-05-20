//! MCP protocol-level tests: spin up the server over a duplex transport,
//! connect an rmcp client, and exercise tool/prompt/resource endpoints.
//!
//! These tests cover `server.rs` (routing, `get_info`, list/read resources)
//! which has 0% coverage from unit tests alone.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use aelm_mcp::cli_runner::AelmCli;
use aelm_mcp::server::AelmMcpServer;
use rmcp::model::CallToolRequestParams;
use rmcp::{ClientHandler, ServiceExt};

fn aelm_bin() -> PathBuf {
    if let Ok(p) = std::env::var("AELM_TEST_BIN") {
        return PathBuf::from(p);
    }
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        let candidate = PathBuf::from(dir).join("debug/aelm");
        if candidate.exists() {
            return candidate;
        }
    }
    let parent_config = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.cargo/config.toml");
    if let Ok(content) = std::fs::read_to_string(&parent_config) {
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("target-dir") {
                let val = val.trim().trim_start_matches('=').trim().trim_matches('"');
                let candidate = PathBuf::from(val).join("debug/aelm");
                if candidate.exists() {
                    return candidate;
                }
            }
        }
    }
    PathBuf::from("aelm")
}

fn cli() -> AelmCli {
    AelmCli::new(aelm_bin(), vec![], None)
}

fn rc_source() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/rc_filter.aelm"
    ))
    .expect("fixture")
}

#[derive(Debug, Clone, Default)]
struct DummyClient;

impl ClientHandler for DummyClient {
    fn get_info(&self) -> rmcp::model::ClientInfo {
        rmcp::model::ClientInfo::default()
    }
}

async fn setup() -> (
    rmcp::service::RunningService<rmcp::RoleClient, DummyClient>,
    tokio::task::JoinHandle<Result<(), String>>,
) {
    let (server_transport, client_transport) = tokio::io::duplex(65536);
    let server = AelmMcpServer::new(cli());
    let server_handle = tokio::spawn(async move {
        server
            .serve(server_transport)
            .await
            .map_err(|e| format!("serve: {e}"))?
            .waiting()
            .await
            .map_err(|e| format!("waiting: {e}"))?;
        Ok(())
    });
    let client = DummyClient
        .serve(client_transport)
        .await
        .expect("client serve");
    (client, server_handle)
}

fn tool_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Server info ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_server_get_info() {
    let server = AelmMcpServer::new(cli());
    let info = rmcp::ServerHandler::get_info(&server);
    assert!(
        info.capabilities.tools.is_some(),
        "tools capability enabled"
    );
    assert!(
        info.capabilities.resources.is_some(),
        "resources capability enabled"
    );
    assert!(
        info.capabilities.prompts.is_some(),
        "prompts capability enabled"
    );
    assert!(info.instructions.is_some(), "instructions should be set");
}

// ── Tool calls via MCP protocol ────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_tools_list() {
    let (client, _server) = setup().await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert!(
        tools.len() >= 30,
        "should have 30+ tools, got {}",
        tools.len()
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_parse() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("parse").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool parse");
    assert_eq!(result.is_error, Some(false));
    let text = tool_text(&result);
    assert!(text.contains("\"success\""));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_validate() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("validate").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool validate");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_format() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("format").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool format");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_render_svg() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("render_svg").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool render_svg");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_render_png() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("render_png").with_arguments(
                serde_json::json!({ "source": rc_source(), "dpi": 72 })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool render_png");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_list_parts() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_parts")
                .with_arguments(serde_json::json!({}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool list_parts");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_search_parts() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("search_parts").with_arguments(
                serde_json::json!({ "query": "resistor" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool search_parts");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_get_part_info() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_part_info").with_arguments(
                serde_json::json!({ "name": "Resistor", "render_symbol": false })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool get_part_info");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_list_symbols() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_symbols")
                .with_arguments(serde_json::json!({}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool list_symbols");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_get_symbol_info() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_symbol_info").with_arguments(
                serde_json::json!({ "name": "resistor" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool get_symbol_info");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_list_examples() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("list_examples")
                .with_arguments(serde_json::json!({}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool list_examples");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_get_example() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_example").with_arguments(
                serde_json::json!({ "name": "hello", "render": false })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool get_example");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_get_dsl_reference() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("get_dsl_reference")
                .with_arguments(serde_json::json!({}).as_object().unwrap().clone()),
        )
        .await
        .expect("call_tool get_dsl_reference");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_analyze_project() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("analyze_project").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool analyze_project");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_apply_move() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("apply_move").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "instance": "R1",
                    "x": 2.0,
                    "y": 3.0
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool apply_move");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_scaffold_circuit() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("scaffold_circuit").with_arguments(
                serde_json::json!({
                    "module": "Test",
                    "parts": [{ "name": "R1", "type_name": "Resistor" }]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool scaffold_circuit");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_diff_circuits() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("diff_circuits").with_arguments(
                serde_json::json!({
                    "before": rc_source(),
                    "after": rc_source()
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool diff_circuits");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_suggest_placement() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("suggest_placement").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool suggest_placement");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_suggest_circuit_pattern() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("suggest_circuit_pattern").with_arguments(
                serde_json::json!({ "query": "divider" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool suggest_circuit_pattern");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_render_symbol() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("render_symbol").with_arguments(
                serde_json::json!({ "name": "resistor" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool render_symbol");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_extract_netlist() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("extract_netlist").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool extract_netlist");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_extract_bom() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("extract_bom").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool extract_bom");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_apply_rotate() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("apply_rotate").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "instance": "R1",
                    "degrees": 90
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool apply_rotate");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_apply_mirror() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("apply_mirror").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "instance": "R1",
                    "axis": "x"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool apply_mirror");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_apply_add_connection() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("apply_add_connection").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "from_pin": "R1.a",
                    "to_pin": "C1.b"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool apply_add_connection");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_apply_delete_connection() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("apply_delete_connection").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "from_pin": "R1.b",
                    "to_pin": "C1.a"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool apply_delete_connection");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_analyze_connectivity() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("analyze_connectivity").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool analyze_connectivity");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_extract_subcircuit() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("extract_subcircuit").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "instances": "R1"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool extract_subcircuit");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_render_batch() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("render_batch").with_arguments(
                serde_json::json!({ "sources": [rc_source()] })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool render_batch");
    assert!(!result.content.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_preview_style() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("preview_style").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool preview_style");
    assert!(!result.content.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_evaluate_calc() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("evaluate_calc").with_arguments(
                serde_json::json!({ "source": rc_source() })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool evaluate_calc");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_find_similar_circuits() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("find_similar_circuits").with_arguments(
                serde_json::json!({
                    "source": rc_source(),
                    "candidates": [rc_source()]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("call_tool find_similar_circuits");
    assert_eq!(result.is_error, Some(false));
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_validate_library() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("validate_library").with_arguments(
                serde_json::json!({ "path": "/nonexistent/path.alib" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool validate_library");
    assert!(
        result.is_error == Some(true),
        "validate_library with missing path should error"
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_call_compile_svg_to_symbol() {
    let (client, _server) = setup().await;
    let result = client
        .call_tool(
            CallToolRequestParams::new("compile_svg_to_symbol").with_arguments(
                serde_json::json!({ "svg": "<not-svg>" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("call_tool compile_svg_to_symbol");
    assert!(
        result.is_error == Some(true),
        "compile_svg_to_symbol with invalid svg should error"
    );
    client.cancel().await.expect("cancel");
}

// ── Remaining prompts via MCP ──────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_prompt_debug_drc() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("debug_drc").with_arguments(
                serde_json::json!({ "source": "module M {}", "error_codes": "DRC-E001" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt debug_drc");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_select_parts() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("select_parts").with_arguments(
                serde_json::json!({ "requirements": "low-noise op-amp" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt select_parts");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_explain_circuit() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("explain_circuit").with_arguments(
                serde_json::json!({ "source": "module M {}" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt explain_circuit");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

// ── Prompts via MCP protocol ───────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_prompts_list() {
    let (client, _server) = setup().await;
    let prompts = client.list_all_prompts().await.expect("list_all_prompts");
    assert!(
        prompts.len() >= 7,
        "should have 7+ prompts, got {}",
        prompts.len()
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_design_circuit() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("design_circuit").with_arguments(
                serde_json::json!({
                    "description": "a voltage divider",
                    "constraints": "5V input"
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        )
        .await
        .expect("get_prompt design_circuit");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_review_circuit() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("review_circuit").with_arguments(
                serde_json::json!({ "source": "module M {}" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt review_circuit");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_learn_aelm() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("learn_aelm").with_arguments(
                serde_json::json!({ "level": "beginner" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt learn_aelm");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_prompt_interactive_design() {
    let (client, _server) = setup().await;
    let result = client
        .get_prompt(
            rmcp::model::GetPromptRequestParams::new("interactive_design").with_arguments(
                serde_json::json!({ "goal": "LED blinker" })
                    .as_object()
                    .unwrap()
                    .clone(),
            ),
        )
        .await
        .expect("get_prompt interactive_design");
    assert!(!result.messages.is_empty());
    client.cancel().await.expect("cancel");
}

// ── Resources via MCP protocol ─────────────────────────────────────────────

#[tokio::test]
async fn test_mcp_resources_list() {
    let (client, _server) = setup().await;
    let resources = client
        .list_all_resources()
        .await
        .expect("list_all_resources");
    assert!(
        resources.len() >= 6,
        "should have 6+ resources, got {}",
        resources.len()
    );
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_resource_read_known() {
    let (client, _server) = setup().await;
    let result = client
        .read_resource(rmcp::model::ReadResourceRequestParams::new(
            "aelm://docs/dsl",
        ))
        .await
        .expect("read_resource dsl");
    assert!(!result.contents.is_empty());
    client.cancel().await.expect("cancel");
}

#[tokio::test]
async fn test_mcp_resource_read_unknown() {
    let (client, _server) = setup().await;
    let result = client
        .read_resource(rmcp::model::ReadResourceRequestParams::new(
            "aelm://docs/nonexistent",
        ))
        .await;
    assert!(result.is_err(), "unknown resource should error");
    client.cancel().await.expect("cancel");
}
