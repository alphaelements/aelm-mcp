//! Integration tests — call each tool function with a real `aelm` binary.
//!
//! Set `AELM_TEST_BIN` to point to the `aelm` binary. Falls back to the
//! default cargo-target path used in development.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use aelm_mcp::cli_runner::AelmCli;
use aelm_mcp::tools;
use aelm_mcp::tools::generate::ScaffoldPart;

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
    .expect("fixture rc_filter.aelm")
}

fn is_success(result: &rmcp::model::CallToolResult) -> bool {
    result.is_error == Some(false)
}

fn result_text(result: &rmcp::model::CallToolResult) -> String {
    result
        .content
        .iter()
        .filter_map(|c| c.raw.as_text().map(|t| t.text.clone()))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── parse ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_parse_valid_source() {
    let r = tools::parse::parse(&cli(), &rc_source()).await;
    assert!(is_success(&r), "parse should succeed");
    let text = result_text(&r);
    assert!(text.contains("\"success\""), "should contain JSON envelope");
}

#[tokio::test]
async fn test_parse_invalid_source() {
    let r = tools::parse::parse(&cli(), "this is not valid aelm").await;
    assert!(r.is_error == Some(true), "parse invalid should error");
}

#[tokio::test]
async fn test_validate_valid_source() {
    let r = tools::parse::validate(&cli(), &rc_source()).await;
    assert!(is_success(&r), "validate should succeed");
}

#[tokio::test]
async fn test_validate_invalid_source() {
    let r = tools::parse::validate(&cli(), "not valid").await;
    // The CLI outputs a JSON envelope even on failure (exit code 1), so run_json
    // recovers the envelope and wraps it as a "success" CallToolResult with the
    // diagnostics visible as text content. Verify the envelope reports failure.
    let text = result_text(&r);
    assert!(
        text.contains("\"success\": false") || r.is_error == Some(true),
        "validate invalid should report failure"
    );
}

#[tokio::test]
async fn test_format_valid_source() {
    let r = tools::parse::format(&cli(), &rc_source()).await;
    assert!(is_success(&r), "format should succeed");
    let text = result_text(&r);
    assert!(
        text.contains("module"),
        "formatted text should contain module"
    );
}

// ── render ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_render_svg_valid() {
    let r = tools::render::render_svg(&cli(), &rc_source(), None).await;
    assert!(is_success(&r), "render_svg should succeed");
    assert!(r.content.len() >= 2, "should have text + image content");
}

#[tokio::test]
async fn test_render_svg_dark_theme() {
    let r = tools::render::render_svg(&cli(), &rc_source(), Some("dark")).await;
    assert!(is_success(&r), "render_svg dark should succeed");
}

#[tokio::test]
async fn test_render_svg_invalid_source() {
    let r = tools::render::render_svg(&cli(), "invalid", None).await;
    assert!(r.is_error == Some(true), "render_svg invalid should error");
}

#[tokio::test]
async fn test_render_png_valid() {
    let r = tools::render::render_png(&cli(), &rc_source(), Some(72), None).await;
    assert!(is_success(&r), "render_png should succeed");
}

#[tokio::test]
async fn test_render_png_default_dpi() {
    let r = tools::render::render_png(&cli(), &rc_source(), None, Some("light")).await;
    assert!(is_success(&r), "render_png default dpi should succeed");
}

#[tokio::test]
async fn test_render_symbol_known() {
    let r = tools::render::render_symbol(&cli(), "resistor").await;
    assert!(is_success(&r), "render_symbol should succeed");
    let text = result_text(&r);
    assert!(text.contains("<svg"), "should contain SVG markup");
}

#[tokio::test]
async fn test_render_symbol_unknown() {
    let r = tools::render::render_symbol(&cli(), "nonexistent_symbol_xyz").await;
    assert!(
        r.is_error == Some(true),
        "render_symbol unknown should error"
    );
}

// ── parts ──────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_parts_no_filter() {
    let r = tools::parts::list_parts(&cli(), None).await;
    assert!(is_success(&r), "list_parts should succeed");
}

#[tokio::test]
async fn test_list_parts_with_category() {
    let r = tools::parts::list_parts(&cli(), Some("passive")).await;
    assert!(is_success(&r), "list_parts with category should succeed");
}

#[tokio::test]
async fn test_search_parts() {
    let r = tools::parts::search_parts(&cli(), "resistor", Some(5)).await;
    assert!(is_success(&r), "search_parts should succeed");
}

#[tokio::test]
async fn test_search_parts_default_limit() {
    let r = tools::parts::search_parts(&cli(), "cap", None).await;
    assert!(is_success(&r), "search_parts default limit should succeed");
}

#[tokio::test]
async fn test_get_part_info() {
    let r = tools::parts::get_part_info(&cli(), "Resistor", false).await;
    assert!(is_success(&r), "get_part_info should succeed");
}

#[tokio::test]
async fn test_get_part_info_with_render() {
    let r = tools::parts::get_part_info(&cli(), "Resistor", true).await;
    assert!(
        !r.content.is_empty(),
        "get_part_info with render should return content"
    );
}

// ── symbols ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_symbols_no_filter() {
    let r = tools::symbols::list_symbols(&cli(), None).await;
    assert!(is_success(&r), "list_symbols should succeed");
}

#[tokio::test]
async fn test_list_symbols_with_category() {
    let r = tools::symbols::list_symbols(&cli(), Some("passive")).await;
    assert!(is_success(&r), "list_symbols with category should succeed");
}

#[tokio::test]
async fn test_get_symbol_info() {
    let r = tools::symbols::get_symbol_info(&cli(), "resistor").await;
    assert!(is_success(&r), "get_symbol_info should succeed");
}

// ── examples ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_examples_no_filter() {
    let r = tools::examples::list_examples(&cli(), None).await;
    assert!(is_success(&r), "list_examples should succeed");
}

#[tokio::test]
async fn test_list_examples_with_category() {
    let r = tools::examples::list_examples(&cli(), Some("basic")).await;
    // May or may not find examples in that category, but shouldn't crash.
    let _ = r;
}

#[tokio::test]
async fn test_get_example() {
    let r = tools::examples::get_example(&cli(), "hello", false).await;
    assert!(is_success(&r), "get_example should succeed");
}

#[tokio::test]
async fn test_get_example_with_render() {
    let r = tools::examples::get_example(&cli(), "hello", true).await;
    assert!(is_success(&r), "get_example with render should succeed");
}

// ── project (analyze) ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_analyze_project() {
    let r = tools::project::analyze_project(&cli(), &rc_source()).await;
    assert!(is_success(&r), "analyze_project should succeed");
}

#[tokio::test]
async fn test_extract_netlist() {
    let r = tools::project::extract_netlist(&cli(), &rc_source(), None).await;
    assert!(is_success(&r), "extract_netlist should succeed");
}

#[tokio::test]
async fn test_extract_netlist_with_module() {
    let r = tools::project::extract_netlist(&cli(), &rc_source(), Some("RcFilter")).await;
    assert!(is_success(&r), "extract_netlist with module should succeed");
}

#[tokio::test]
async fn test_extract_bom() {
    let r = tools::project::extract_bom(&cli(), &rc_source(), None).await;
    assert!(is_success(&r), "extract_bom should succeed");
}

#[tokio::test]
async fn test_extract_bom_with_module() {
    let r = tools::project::extract_bom(&cli(), &rc_source(), Some("RcFilter")).await;
    assert!(is_success(&r), "extract_bom with module should succeed");
}

#[tokio::test]
async fn test_analyze_connectivity() {
    let r = tools::project::analyze_connectivity(&cli(), &rc_source(), None).await;
    assert!(is_success(&r), "analyze_connectivity should succeed");
}

#[tokio::test]
async fn test_analyze_connectivity_with_module() {
    let r = tools::project::analyze_connectivity(&cli(), &rc_source(), Some("RcFilter")).await;
    assert!(
        is_success(&r),
        "analyze_connectivity with module should succeed"
    );
}

#[tokio::test]
async fn test_extract_subcircuit() {
    let r = tools::project::extract_subcircuit(&cli(), &rc_source(), "R1", None).await;
    assert!(is_success(&r), "extract_subcircuit should succeed");
}

#[tokio::test]
async fn test_extract_subcircuit_with_module() {
    let r = tools::project::extract_subcircuit(&cli(), &rc_source(), "R1", Some("RcFilter")).await;
    assert!(
        is_success(&r),
        "extract_subcircuit with module should succeed"
    );
}

// ── edit ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_apply_move() {
    let r = tools::edit::apply_move(&cli(), &rc_source(), "R1", 2.0, 3.0).await;
    assert!(is_success(&r), "apply_move should succeed");
}

#[tokio::test]
async fn test_apply_rotate() {
    let r = tools::edit::apply_rotate(&cli(), &rc_source(), "R1", 90).await;
    assert!(is_success(&r), "apply_rotate should succeed");
}

#[tokio::test]
async fn test_apply_mirror() {
    let r = tools::edit::apply_mirror(&cli(), &rc_source(), "R1", "x").await;
    assert!(is_success(&r), "apply_mirror should succeed");
}

#[tokio::test]
async fn test_apply_add_connection() {
    let r = tools::edit::apply_add_connection(&cli(), &rc_source(), "R1.a", "C1.b").await;
    assert!(is_success(&r), "apply_add_connection should succeed");
}

#[tokio::test]
async fn test_apply_delete_connection() {
    let r = tools::edit::apply_delete_connection(&cli(), &rc_source(), "R1.b", "C1.a").await;
    assert!(is_success(&r), "apply_delete_connection should succeed");
}

// ── generate ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_scaffold_circuit() {
    let parts = vec![
        ScaffoldPart {
            name: "R1".to_string(),
            type_name: "Resistor".to_string(),
        },
        ScaffoldPart {
            name: "C1".to_string(),
            type_name: "Capacitor".to_string(),
        },
    ];
    let r = tools::generate::scaffold_circuit(&cli(), "TestModule", &parts).await;
    assert!(is_success(&r), "scaffold_circuit should succeed");
    let text = result_text(&r);
    assert!(text.contains("TestModule"), "should contain module name");
}

#[tokio::test]
async fn test_scaffold_circuit_empty_parts() {
    let r = tools::generate::scaffold_circuit(&cli(), "Empty", &[]).await;
    assert!(
        r.is_error == Some(true),
        "scaffold_circuit empty should error"
    );
}

// ── advanced ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_diff_circuits() {
    let before = rc_source();
    let after = before.replace("R1: Resistor", "R1: Capacitor");
    let r = tools::advanced::diff_circuits(&cli(), &before, &after).await;
    assert!(is_success(&r), "diff_circuits should succeed");
    let text = result_text(&r);
    assert!(
        text.contains("changed_instances"),
        "should report changed instances"
    );
}

#[tokio::test]
async fn test_render_batch_single() {
    let sources = vec![rc_source()];
    let r = tools::advanced::render_batch(&cli(), &sources).await;
    assert!(r.content.len() >= 2, "should have text + image content");
}

#[tokio::test]
async fn test_render_batch_empty() {
    let r = tools::advanced::render_batch(&cli(), &[]).await;
    assert!(r.is_error == Some(true), "render_batch empty should error");
}

#[tokio::test]
async fn test_render_batch_multiple() {
    let sources = vec![rc_source(), rc_source()];
    let r = tools::advanced::render_batch(&cli(), &sources).await;
    assert!(r.content.len() >= 4, "should have content for each circuit");
}

#[tokio::test]
async fn test_render_batch_with_invalid() {
    let sources = vec![rc_source(), "invalid source".to_string()];
    let r = tools::advanced::render_batch(&cli(), &sources).await;
    assert!(
        r.is_error == Some(true),
        "render_batch with invalid should set is_error"
    );
}

#[tokio::test]
async fn test_preview_style_default_theme() {
    let r = tools::advanced::preview_style(&cli(), &rc_source(), None).await;
    assert!(is_success(&r), "preview_style should succeed");
    assert!(
        r.content.len() >= 4,
        "should have baseline + variant images"
    );
}

#[tokio::test]
async fn test_preview_style_dark_theme() {
    let r = tools::advanced::preview_style(&cli(), &rc_source(), Some("dark")).await;
    assert!(is_success(&r), "preview_style dark should succeed");
}

#[tokio::test]
async fn test_suggest_placement() {
    let source = rc_source();
    let r = tools::advanced::suggest_placement(&cli(), &source).await;
    assert!(is_success(&r), "suggest_placement should succeed");
    let text = result_text(&r);
    assert!(
        text.contains("total_instances"),
        "should report instance count"
    );
}

#[tokio::test]
async fn test_evaluate_calc() {
    let r = tools::advanced::evaluate_calc(&cli(), &rc_source()).await;
    assert!(is_success(&r), "evaluate_calc should succeed");
}

#[tokio::test]
async fn test_find_similar_circuits() {
    let source = rc_source();
    let candidates = vec![rc_source()];
    let r = tools::advanced::find_similar_circuits(&cli(), &source, &candidates).await;
    assert!(is_success(&r), "find_similar_circuits should succeed");
    let text = result_text(&r);
    assert!(text.contains("ranked"), "should contain ranked results");
}

#[tokio::test]
async fn test_find_similar_circuits_empty_candidates() {
    let r = tools::advanced::find_similar_circuits(&cli(), &rc_source(), &[]).await;
    assert!(is_success(&r), "find_similar_circuits empty should succeed");
}

// ── library ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_validate_library_valid_path() {
    let stdlib = concat!(env!("CARGO_MANIFEST_DIR"), "/../stdlib/parts/passive.alib");
    if std::path::Path::new(stdlib).exists() {
        let r = tools::library::validate_library(&cli(), stdlib).await;
        assert!(is_success(&r), "validate_library should succeed for stdlib");
    }
}

#[tokio::test]
async fn test_validate_library_invalid_path() {
    let r = tools::library::validate_library(&cli(), "/nonexistent/path.alib").await;
    assert!(
        r.is_error == Some(true),
        "validate_library should error for missing file"
    );
}

#[tokio::test]
async fn test_compile_svg_to_symbol_invalid_svg() {
    let r = tools::library::compile_svg_to_symbol(&cli(), "<not-svg>").await;
    assert!(
        r.is_error == Some(true),
        "compile_svg_to_symbol should error for invalid SVG"
    );
}

#[tokio::test]
async fn test_compile_svg_to_symbol_no_pins() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 10 10"><rect x="0" y="0" width="10" height="10"/></svg>"#;
    let r = tools::library::compile_svg_to_symbol(&cli(), svg).await;
    assert!(
        r.is_error == Some(true),
        "compile_svg_to_symbol should error for SVG without pins"
    );
}

// ── docs (pure, no CLI) ───────────────────────────────────────────────────

#[test]
fn test_get_dsl_reference_default() {
    let r = tools::docs::get_reference(None);
    assert!(is_success(&r));
}

#[test]
fn test_get_dsl_reference_cli() {
    let r = tools::docs::get_reference(Some("cli"));
    assert!(is_success(&r));
}

#[test]
fn test_get_dsl_reference_all_topics() {
    for t in tools::docs::TOPICS {
        let r = tools::docs::get_reference(Some(t.key));
        assert!(is_success(&r), "topic {} should be available", t.key);
    }
}

// ── cli_runner edge cases ──────────────────────────────────────────────────

#[tokio::test]
async fn test_run_json_parses_envelope() {
    let cli = cli();
    let source = rc_source();
    let result = cli
        .run_json(&["parse", "--stdin", "--json"], Some(&source))
        .await;
    assert!(result.is_ok(), "run_json should parse valid JSON");
    let v = result.unwrap();
    assert_eq!(v["success"], true);
}

#[tokio::test]
async fn test_run_json_nonzero_exit_with_json_stdout() {
    let cli = cli();
    let result = cli
        .run_json(&["check", "--stdin", "--json"], Some("invalid"))
        .await;
    // check returns exit code 1 but still emits JSON on stdout.
    // run_json should recover the JSON from the NonZeroExit stdout.
    assert!(result.is_ok(), "should recover JSON from non-zero exit");
}

#[tokio::test]
async fn test_run_binary_returns_raw_bytes() {
    let cli = cli();
    let result = cli
        .run_binary(&["symbols", "render", "resistor"], None)
        .await;
    assert!(result.is_ok());
    let bytes = result.unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("<svg"), "should return SVG bytes");
}

#[tokio::test]
async fn test_run_raw_with_stdin() {
    let cli = cli();
    let source = rc_source();
    let result = cli.run_raw(&["fmt", "--stdin"], Some(&source)).await;
    assert!(result.is_ok());
    let bytes = result.unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("module"),
        "formatted output should contain module"
    );
}

#[tokio::test]
async fn test_run_raw_with_working_dir() {
    let cli = AelmCli::new(aelm_bin(), vec![], Some(PathBuf::from("/tmp")));
    let source = rc_source();
    let result = cli
        .run_json(&["parse", "--stdin", "--json"], Some(&source))
        .await;
    assert!(result.is_ok(), "working_dir should not break commands");
}

#[tokio::test]
async fn test_run_raw_with_library_dirs() {
    let cli = AelmCli::new(aelm_bin(), vec![PathBuf::from("/nonexistent/lib")], None);
    let source = rc_source();
    let result = cli
        .run_json(&["parse", "--stdin", "--json"], Some(&source))
        .await;
    // Non-existent library dirs should not prevent parsing inline-def circuits.
    assert!(result.is_ok(), "library_dirs should be appended as flags");
}
