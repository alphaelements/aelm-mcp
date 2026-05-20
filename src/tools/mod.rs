//! MCP tool implementations. Each tool shells out to `aelm` via [`AelmCli`]
//! and converts the result into an MCP [`CallToolResult`].
//!
//! The `server` module wires these into rmcp's `#[tool_router]`; keeping the
//! logic here lets each capability stay small and independently testable.

pub mod advanced;
pub mod docs;
pub mod edit;
pub mod examples;
pub mod generate;
pub mod library;
pub mod parse;
pub mod parts;
pub mod project;
pub mod render;
pub mod symbols;

use rmcp::model::{CallToolResult, Content};
use serde_json::Value;

use crate::error::CliError;

/// Wrap arbitrary JSON as a single text content tool result.
///
/// MCP clients render tool text as the model-visible payload; pretty-printing
/// keeps the JSON readable when an AI inspects it.
pub fn json_result(value: &Value) -> CallToolResult {
    let text = serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string());
    CallToolResult::success(vec![Content::text(text)])
}

/// Convert a [`CliError`] into a tool error result (isError = true), never a
/// protocol error — per the MCP spec, tool failures stay in-band so the model
/// can read and react to them.
pub fn error_result(err: &CliError) -> CallToolResult {
    CallToolResult::error(vec![Content::text(err.user_message())])
}

/// Build an SVG tool result: the raw SVG as text (so the model can read/edit it)
/// plus an inline image content for clients that render it.
pub fn svg_result(svg: &str) -> CallToolResult {
    let encoded = base64_encode(svg.as_bytes());
    CallToolResult::success(vec![
        Content::text(svg.to_string()),
        Content::image(encoded, "image/svg+xml"),
    ])
}

/// Standard base64 (RFC 4648) encoder. Mirrors the CLI's dependency-free
/// encoder so image bytes embed in MCP content without an extra crate.
pub fn base64_encode(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let n = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);
        out.push(TABLE[((n >> 18) & 0x3f) as usize] as char);
        out.push(TABLE[((n >> 12) & 0x3f) as usize] as char);
        if chunk.len() > 1 {
            out.push(TABLE[((n >> 6) & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(TABLE[(n & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn test_json_result_is_success_with_text() {
        let v = serde_json::json!({"ok": true});
        let r = json_result(&v);
        assert_eq!(r.is_error, Some(false));
        assert_eq!(r.content.len(), 1);
    }

    #[test]
    fn test_error_result_sets_is_error() {
        let err = CliError::JsonParse("bad".to_string());
        let r = error_result(&err);
        assert_eq!(r.is_error, Some(true));
    }

    #[test]
    fn test_svg_result_has_text_and_image() {
        let r = svg_result("<svg></svg>");
        assert_eq!(r.is_error, Some(false));
        assert_eq!(r.content.len(), 2, "text + image content");
    }
}
