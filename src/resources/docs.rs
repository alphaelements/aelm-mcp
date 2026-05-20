//! `aelm://docs/{topic}` resource provider, backed by the embedded reference
//! documents in [`crate::tools::docs`].

use rmcp::model::{AnnotateAble, RawResource, Resource, ResourceContents};

use crate::tools::docs::{topic, TOPICS};

/// URI prefix for documentation resources.
const DOCS_PREFIX: &str = "aelm://docs/";

/// Build the resource listing for every embedded doc topic.
pub fn list() -> Vec<Resource> {
    TOPICS
        .iter()
        .map(|t| {
            let uri = format!("{DOCS_PREFIX}{}", t.key);
            let mut raw = RawResource::new(uri, t.title.to_string());
            raw.mime_type = Some("text/markdown".to_string());
            raw.description = Some(format!("Aelm {} (embedded).", t.title));
            raw.no_annotation()
        })
        .collect()
}

/// Resolve a `aelm://docs/{topic}` URI to its markdown contents.
///
/// Returns `None` when the URI is not a docs URI or the topic is unknown, so the
/// caller can map that to an MCP `resource_not_found` error.
pub fn read(uri: &str) -> Option<ResourceContents> {
    let key = uri.strip_prefix(DOCS_PREFIX)?;
    let t = topic(key)?;
    Some(ResourceContents::text(t.body.to_string(), uri).with_mime_type("text/markdown"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_covers_all_topics() {
        let resources = list();
        assert_eq!(resources.len(), TOPICS.len());
        assert!(resources.iter().any(|r| r.raw.uri == "aelm://docs/dsl"));
    }

    #[test]
    fn test_read_known_topic() {
        let c = read("aelm://docs/cli");
        assert!(c.is_some());
    }

    #[test]
    fn test_read_unknown_topic_is_none() {
        assert!(read("aelm://docs/nonexistent").is_none());
    }

    #[test]
    fn test_read_non_docs_uri_is_none() {
        assert!(read("aelm://parts/Resistor").is_none());
    }
}
