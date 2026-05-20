//! Edit tools: dry-run `aelm apply-*` operations that return modified source.
//!
//! `aelm apply-*` reads a file path (it has no `--stdin`), so each tool writes
//! the incoming source to a private temp file and runs the command with
//! `--dry-run --json`. `--dry-run` guarantees the file is never mutated; the
//! modified source and a line diff come back in the JSON envelope's
//! `data.{source,changes}`. The temp file is removed when it drops.

use std::io::Write;

use rmcp::model::CallToolResult;
use tempfile::Builder;

use crate::cli_runner::AelmCli;
use crate::error::CliError;

use super::{error_result, json_result};

/// Write `source` to a temp `.aelm` file and run `aelm <subcommand> <file>
/// <extra...> --dry-run --json`, returning the parsed envelope.
async fn run_apply(
    cli: &AelmCli,
    subcommand: &str,
    source: &str,
    extra: &[&str],
) -> CallToolResult {
    let file = match Builder::new().suffix(".aelm").tempfile() {
        Ok(mut f) => {
            if let Err(e) = f.write_all(source.as_bytes()).and_then(|()| f.flush()) {
                return error_result(&CliError::Io(e));
            }
            f
        }
        Err(e) => return error_result(&CliError::Io(e)),
    };

    let path = file.path().to_string_lossy().into_owned();
    let mut args: Vec<&str> = vec![subcommand, &path];
    args.extend_from_slice(extra);
    args.push("--dry-run");
    args.push("--json");

    match cli.run_json(&args, None).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
    // `file` drops here, removing the temp file.
}

/// `apply_move` — move an instance to `(x, y)` (grid-snapped by the CLI).
pub async fn apply_move(
    cli: &AelmCli,
    source: &str,
    instance: &str,
    x: f64,
    y: f64,
) -> CallToolResult {
    let (xs, ys) = (x.to_string(), y.to_string());
    run_apply(cli, "apply-move", source, &[instance, &xs, &ys]).await
}

/// `apply_rotate` — rotate an instance by `degrees`.
pub async fn apply_rotate(
    cli: &AelmCli,
    source: &str,
    instance: &str,
    degrees: i64,
) -> CallToolResult {
    let d = degrees.to_string();
    run_apply(cli, "apply-rotate", source, &[instance, &d]).await
}

/// `apply_mirror` — mirror an instance across an axis (`x` or `y`).
pub async fn apply_mirror(
    cli: &AelmCli,
    source: &str,
    instance: &str,
    axis: &str,
) -> CallToolResult {
    run_apply(cli, "apply-mirror", source, &[instance, axis]).await
}

/// `apply_add_connection` — add a connection between two pins.
pub async fn apply_add_connection(
    cli: &AelmCli,
    source: &str,
    from_pin: &str,
    to_pin: &str,
) -> CallToolResult {
    run_apply(cli, "apply-add-connection", source, &[from_pin, to_pin]).await
}

/// `apply_delete_connection` — remove a connection between two pins.
pub async fn apply_delete_connection(
    cli: &AelmCli,
    source: &str,
    from_pin: &str,
    to_pin: &str,
) -> CallToolResult {
    run_apply(cli, "apply-delete-connection", source, &[from_pin, to_pin]).await
}
