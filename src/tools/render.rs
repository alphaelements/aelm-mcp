//! Render tools: full circuit to SVG/PNG, and a single symbol to SVG.
//!
//! Circuit rendering goes through `aelm pipeline --stages render`, which returns
//! the image in the JSON envelope's `data.render.image_base64` field — for SVG
//! that field carries the raw XML, for PNG it is standard base64. This avoids
//! the stdin → `-o -` path, which does not stream binary cleanly.

use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use crate::cli_runner::AelmCli;

use super::{error_result, svg_result};

/// Map a caller-supplied theme to the CLI's `--theme` value, defaulting to light.
fn theme_arg(theme: Option<&str>) -> &str {
    match theme {
        Some("dark") => "dark",
        _ => "light",
    }
}

/// Extract `data.render.image_base64` from a pipeline JSON envelope.
fn render_field<'a>(env: &'a Value, key: &str) -> Option<&'a str> {
    env.get("data")?.get("render")?.get(key)?.as_str()
}

/// `render_svg` — render a circuit to SVG.
/// CLI: `aelm pipeline --stdin --stages render --render-format svg --json`
pub async fn render_svg(cli: &AelmCli, source: &str, theme: Option<&str>) -> CallToolResult {
    let args = [
        "pipeline",
        "--stdin",
        "--stages",
        "render",
        "--render-format",
        "svg",
        "--theme",
        theme_arg(theme),
        "--json",
    ];
    match cli.run_json(&args, Some(source)).await {
        Ok(env) => match render_field(&env, "image_base64") {
            Some(svg) => svg_result(svg),
            None => CallToolResult::error(vec![Content::text(
                "render produced no image; see diagnostics".to_string(),
            )]),
        },
        Err(e) => error_result(&e),
    }
}

/// `render_png` — render a circuit to PNG (base64 image content).
/// CLI: `aelm pipeline --stdin --stages render --render-format png --dpi <dpi> --json`
pub async fn render_png(
    cli: &AelmCli,
    source: &str,
    dpi: Option<u32>,
    theme: Option<&str>,
) -> CallToolResult {
    let dpi_str = dpi.unwrap_or(150).to_string();
    let args = [
        "pipeline",
        "--stdin",
        "--stages",
        "render",
        "--render-format",
        "png",
        "--dpi",
        &dpi_str,
        "--theme",
        theme_arg(theme),
        "--json",
    ];
    match cli.run_json(&args, Some(source)).await {
        Ok(env) => match render_field(&env, "image_base64") {
            // pipeline already base64-encodes PNG bytes — pass through.
            Some(b64) => {
                CallToolResult::success(vec![Content::image(b64.to_string(), "image/png")])
            }
            None => CallToolResult::error(vec![Content::text(
                "render produced no image; see diagnostics".to_string(),
            )]),
        },
        Err(e) => error_result(&e),
    }
}

/// `render_symbol` — render a single symbol template to SVG.
/// CLI: `aelm symbols render <name>` (SVG to stdout by default)
pub async fn render_symbol(cli: &AelmCli, name: &str) -> CallToolResult {
    match cli.run_binary(&["symbols", "render", name], None).await {
        Ok(bytes) => svg_result(&String::from_utf8_lossy(&bytes)),
        Err(e) => error_result(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_arg_dark() {
        assert_eq!(theme_arg(Some("dark")), "dark");
    }

    #[test]
    fn test_theme_arg_defaults_light() {
        assert_eq!(theme_arg(None), "light");
        assert_eq!(theme_arg(Some("bogus")), "light");
        assert_eq!(theme_arg(Some("light")), "light");
    }

    #[test]
    fn test_render_field_extracts_image() {
        let env = serde_json::json!({
            "data": { "render": { "image_base64": "<svg/>", "format": "svg" } }
        });
        assert_eq!(render_field(&env, "image_base64"), Some("<svg/>"));
    }

    #[test]
    fn test_render_field_missing_returns_none() {
        let env = serde_json::json!({ "data": { "check": {} } });
        assert_eq!(render_field(&env, "image_base64"), None);
    }
}
