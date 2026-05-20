//! Library tools: validate a `.alib`, and compile an SVG into a symbol block.

use std::io::{Read, Write};

use rmcp::model::{CallToolResult, Content};
use tempfile::Builder;

use crate::cli_runner::AelmCli;
use crate::error::CliError;

use super::{error_result, json_result};

/// `validate_library` — validate a `.alib` file or directory.
/// CLI: `aelm lib validate <path> --json`
pub async fn validate_library(cli: &AelmCli, path: &str) -> CallToolResult {
    match cli
        .run_json(&["lib", "validate", path, "--json"], None)
        .await
    {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

/// `compile_svg_to_symbol` — convert SVG artwork into an Aelm `symbol {}` block.
///
/// The SVG is written to a temp file and compiled with `--to-aelm`; the result
/// is read back from a temp output file. Both temp files drop afterwards.
pub async fn compile_svg_to_symbol(cli: &AelmCli, svg: &str) -> CallToolResult {
    let src = match Builder::new().suffix(".svg").tempfile() {
        Ok(mut f) => {
            if let Err(e) = f.write_all(svg.as_bytes()).and_then(|()| f.flush()) {
                return error_result(&CliError::Io(e));
            }
            f
        }
        Err(e) => return error_result(&CliError::Io(e)),
    };
    let out = match Builder::new().suffix(".alib").tempfile() {
        Ok(f) => f,
        Err(e) => return error_result(&CliError::Io(e)),
    };

    let src_path = src.path().to_string_lossy().into_owned();
    let out_path = out.path().to_string_lossy().into_owned();

    match cli
        .run_binary(
            &["lib", "compile", &src_path, "-o", &out_path, "--to-aelm"],
            None,
        )
        .await
    {
        Ok(_) => {
            let mut contents = String::new();
            match std::fs::File::open(&out_path).and_then(|mut f| f.read_to_string(&mut contents)) {
                Ok(_) => CallToolResult::success(vec![Content::text(contents)]),
                Err(e) => error_result(&CliError::Io(e)),
            }
        }
        Err(e) => error_result(&e),
    }
}
