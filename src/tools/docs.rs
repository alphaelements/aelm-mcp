//! Embedded reference documentation served as a tool and as MCP resources.
//!
//! The markdown files under `docs/` are compiled into the binary so the server
//! is self-contained — no runtime file lookups, works regardless of where the
//! `aelm-mcp` binary is installed.

use rmcp::model::{CallToolResult, Content};

/// One embedded reference document.
pub struct DocTopic {
    /// Stable topic key used in `aelm://docs/{topic}` and the tool's `topic` arg.
    pub key: &'static str,
    /// Human-readable title.
    pub title: &'static str,
    /// Full markdown body.
    pub body: &'static str,
}

/// All embedded topics. Order is the listing order.
pub const TOPICS: &[DocTopic] = &[
    DocTopic {
        key: "dsl",
        title: "DSL Reference",
        body: include_str!("../../docs/dsl-reference.md"),
    },
    DocTopic {
        key: "cli",
        title: "CLI Reference",
        body: include_str!("../../docs/cli-reference.md"),
    },
    DocTopic {
        key: "style",
        title: "Style Reference",
        body: include_str!("../../docs/style-reference.md"),
    },
    DocTopic {
        key: "patterns",
        title: "Circuit Patterns",
        body: include_str!("../../docs/circuit-patterns.md"),
    },
    DocTopic {
        key: "drc",
        title: "DRC Rules",
        body: include_str!("../../docs/drc-rules.md"),
    },
    DocTopic {
        key: "mistakes",
        title: "Common Mistakes",
        body: include_str!("../../docs/common-mistakes.md"),
    },
];

/// Look up a topic by key.
pub fn topic(key: &str) -> Option<&'static DocTopic> {
    TOPICS.iter().find(|t| t.key == key)
}

/// Comma-separated list of valid topic keys (for error messages).
pub fn topic_keys() -> String {
    TOPICS.iter().map(|t| t.key).collect::<Vec<_>>().join(", ")
}

/// `get_dsl_reference` — return an embedded reference document by topic.
pub fn get_reference(requested: Option<&str>) -> CallToolResult {
    match requested {
        None | Some("dsl") => deliver("dsl"),
        Some(key) => match topic(key) {
            Some(_) => deliver(key),
            None => CallToolResult::error(vec![Content::text(format!(
                "unknown doc topic '{key}'. Available: {}",
                topic_keys()
            ))]),
        },
    }
}

fn deliver(key: &str) -> CallToolResult {
    topic(key).map_or_else(
        || CallToolResult::error(vec![Content::text(format!("unknown doc topic '{key}'"))]),
        |t| CallToolResult::success(vec![Content::text(t.body.to_string())]),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_topics_have_nonempty_bodies() {
        for t in TOPICS {
            assert!(!t.body.is_empty(), "topic {} must have a body", t.key);
            assert!(!t.title.is_empty());
        }
    }

    #[test]
    fn test_topic_lookup_known() {
        assert!(topic("dsl").is_some());
        assert!(topic("cli").is_some());
        assert!(topic("patterns").is_some());
    }

    #[test]
    fn test_topic_lookup_unknown() {
        assert!(topic("nonexistent").is_none());
    }

    #[test]
    fn test_topic_keys_lists_all() {
        let keys = topic_keys();
        assert!(keys.contains("dsl"));
        assert!(keys.contains("mistakes"));
    }

    #[test]
    fn test_get_reference_defaults_to_dsl() {
        let r = get_reference(None);
        assert_eq!(r.is_error, Some(false));
    }

    #[test]
    fn test_get_reference_unknown_topic_is_error() {
        let r = get_reference(Some("bogus"));
        assert_eq!(r.is_error, Some(true));
    }

    #[test]
    fn test_get_reference_known_topic() {
        let r = get_reference(Some("drc"));
        assert_eq!(r.is_error, Some(false));
    }
}
