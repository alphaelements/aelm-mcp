//! Error types for the aelm-mcp server.

use thiserror::Error;

/// Errors raised while invoking the `aelm` CLI or interpreting its output.
#[derive(Debug, Error)]
pub enum CliError {
    /// The `aelm` binary could not be located or spawned.
    #[error("failed to run aelm binary at {path}: {source}")]
    Spawn {
        path: String,
        #[source]
        source: std::io::Error,
    },

    /// The CLI exited with a non-zero status. `stderr` carries the human-readable
    /// diagnostic; `stdout` may still hold a JSON envelope when `--json` was used.
    #[error("aelm exited with status {code}: {stderr}")]
    NonZeroExit {
        code: i32,
        stdout: String,
        stderr: String,
    },

    /// The CLI's stdout could not be parsed as the expected JSON shape.
    #[error("failed to parse aelm JSON output: {0}")]
    JsonParse(String),

    /// I/O error while writing to the child's stdin or reading its output.
    #[error("I/O error communicating with aelm: {0}")]
    Io(#[from] std::io::Error),
}

impl CliError {
    /// Render the error as a single user-facing string suitable for an MCP
    /// tool error payload (returned via the `isError` flag, never as a
    /// protocol error).
    pub fn user_message(&self) -> String {
        match self {
            CliError::NonZeroExit { stderr, .. } if !stderr.is_empty() => stderr.clone(),
            other => other.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message_prefers_stderr_for_nonzero_exit() {
        let err = CliError::NonZeroExit {
            code: 1,
            stdout: String::new(),
            stderr: "DRC-E001: unconnected pin".to_string(),
        };
        assert_eq!(err.user_message(), "DRC-E001: unconnected pin");
    }

    #[test]
    fn test_user_message_falls_back_to_display_when_stderr_empty() {
        let err = CliError::NonZeroExit {
            code: 2,
            stdout: String::new(),
            stderr: String::new(),
        };
        assert!(err.user_message().contains("status 2"));
    }

    #[test]
    fn test_json_parse_error_message() {
        let err = CliError::JsonParse("unexpected token".to_string());
        assert!(err.to_string().contains("unexpected token"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
        let err: CliError = io.into();
        assert!(matches!(err, CliError::Io(_)));
    }
}
