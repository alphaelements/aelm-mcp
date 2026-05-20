//! Async wrapper around the `aelm` command-line tool.
//!
//! Every MCP tool ultimately shells out to `aelm` via this runner. Source is
//! streamed over stdin (no temp files), and structured results come back as the
//! unified `{ success, data, diagnostics }` JSON envelope on stdout.

use std::path::PathBuf;
use std::process::Stdio;

use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

use crate::error::CliError;

/// Configuration for locating and invoking the `aelm` binary.
#[derive(Debug, Clone)]
pub struct AelmCli {
    binary_path: PathBuf,
    library_dirs: Vec<PathBuf>,
    working_dir: Option<PathBuf>,
}

impl AelmCli {
    /// Create a runner. `binary_path` may be a bare name (`"aelm"`, resolved via
    /// `PATH`) or an absolute path.
    pub fn new(
        binary_path: PathBuf,
        library_dirs: Vec<PathBuf>,
        working_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            binary_path,
            library_dirs,
            working_dir,
        }
    }

    /// Append the configured `-L <dir>` library flags to an argument vector.
    fn with_library_flags<'a>(&'a self, base: &[&'a str]) -> Vec<String> {
        let mut args: Vec<String> = base.iter().map(ToString::to_string).collect();
        for dir in &self.library_dirs {
            args.push("-L".to_string());
            args.push(dir.display().to_string());
        }
        args
    }

    fn base_command(&self, args: &[String]) -> Command {
        let mut cmd = Command::new(&self.binary_path);
        cmd.args(args);
        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd
    }

    /// Run `aelm <args>`, optionally feeding `stdin`, and return raw stdout bytes.
    ///
    /// Returns [`CliError::NonZeroExit`] when the process exits non-zero so the
    /// caller can surface stderr to the AI via the MCP `isError` flag.
    pub async fn run_raw(&self, args: &[&str], stdin: Option<&str>) -> Result<Vec<u8>, CliError> {
        let full_args = self.with_library_flags(args);
        let mut child =
            self.base_command(&full_args)
                .spawn()
                .map_err(|source| CliError::Spawn {
                    path: self.binary_path.display().to_string(),
                    source,
                })?;

        if let Some(input) = stdin {
            // Take the handle so it is dropped (EOF) after the write completes.
            let mut handle = child
                .stdin
                .take()
                .ok_or_else(|| CliError::JsonParse("child stdin unavailable".to_string()))?;
            handle.write_all(input.as_bytes()).await?;
            handle.shutdown().await?;
        }

        let output = child.wait_with_output().await?;
        if output.status.success() {
            Ok(output.stdout)
        } else {
            Err(CliError::NonZeroExit {
                code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            })
        }
    }

    /// Run a command expected to emit the JSON envelope and parse it.
    pub async fn run_json(&self, args: &[&str], stdin: Option<&str>) -> Result<Value, CliError> {
        // `--json` may already be in args; callers add it explicitly so the
        // runner stays agnostic. Parse whatever stdout we get.
        let bytes = match self.run_raw(args, stdin).await {
            Ok(b) => b,
            // The CLI prints the JSON envelope to stdout even on DRC failure
            // (data/error separation). Recover it so the AI sees diagnostics
            // rather than an opaque protocol error.
            Err(CliError::NonZeroExit { stdout, .. }) if !stdout.trim().is_empty() => {
                stdout.into_bytes()
            }
            Err(e) => return Err(e),
        };
        let text = String::from_utf8_lossy(&bytes);
        serde_json::from_str(&text).map_err(|e| CliError::JsonParse(e.to_string()))
    }

    /// Run a command expected to emit raw bytes (e.g. SVG/PNG to stdout).
    pub async fn run_binary(
        &self,
        args: &[&str],
        stdin: Option<&str>,
    ) -> Result<Vec<u8>, CliError> {
        self.run_raw(args, stdin).await
    }

    /// The configured binary path (for diagnostics / startup logging).
    pub fn binary_path(&self) -> &PathBuf {
        &self.binary_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn runner() -> AelmCli {
        AelmCli::new(
            PathBuf::from("aelm"),
            vec![PathBuf::from("/libs/a"), PathBuf::from("/libs/b")],
            None,
        )
    }

    #[test]
    fn test_with_library_flags_appends_each_dir() {
        let cli = runner();
        let args = cli.with_library_flags(&["parts", "list", "--json"]);
        assert_eq!(
            args,
            vec!["parts", "list", "--json", "-L", "/libs/a", "-L", "/libs/b"]
        );
    }

    #[test]
    fn test_with_library_flags_no_dirs() {
        let cli = AelmCli::new(PathBuf::from("aelm"), vec![], None);
        let args = cli.with_library_flags(&["check", "--json"]);
        assert_eq!(args, vec!["check", "--json"]);
    }

    #[test]
    fn test_binary_path_accessor() {
        let cli = runner();
        assert_eq!(cli.binary_path(), &PathBuf::from("aelm"));
    }

    #[tokio::test]
    async fn test_run_raw_spawn_error_for_missing_binary() {
        let cli = AelmCli::new(PathBuf::from("/nonexistent/aelm-binary-xyz"), vec![], None);
        let result = cli.run_raw(&["--version"], None).await;
        assert!(matches!(result, Err(CliError::Spawn { .. })));
    }
}
