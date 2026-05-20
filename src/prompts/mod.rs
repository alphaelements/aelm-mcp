//! MCP prompt builders — reusable workflow templates that seed an AI with the
//! right Aelm context for a task.
//!
//! Each builder returns a `Vec<PromptMessage>`. The `server` module exposes them
//! through rmcp's `#[prompt_router]`; keeping the message construction here makes
//! each workflow independently testable and keeps `server.rs` declarative.

use std::fmt::Write as _;

use rmcp::model::{PromptMessage, PromptMessageRole};

use crate::tools::docs;

/// Convenience: a user-role text message.
fn user(text: impl Into<String>) -> PromptMessage {
    PromptMessage::new_text(PromptMessageRole::User, text.into())
}

/// Convenience: an assistant-role text message (used to prime the model's stance).
fn assistant(text: impl Into<String>) -> PromptMessage {
    PromptMessage::new_text(PromptMessageRole::Assistant, text.into())
}

/// Fetch an embedded doc body by topic key, or an empty string if missing.
fn doc(key: &str) -> &'static str {
    docs::topic(key).map_or("", |t| t.body)
}

/// `design_circuit` — guide the model from requirements to valid `.aelm`.
pub fn design_circuit(description: &str, constraints: Option<&str>) -> Vec<PromptMessage> {
    let mut intro = format!(
        "You are designing an electronic circuit in the Aelm DSL.\n\n\
         Requirements:\n{description}\n"
    );
    if let Some(c) = constraints {
        let _ = write!(intro, "\nConstraints:\n{c}\n");
    }
    intro.push_str(
        "\nWorkflow: (1) pick parts with `list_parts` / `search_parts`; \
         (2) define the module and place the key component with absolute \
         `place: (x, y)`, everything else pin-relative; (3) wire the \
         connections; (4) `validate` and fix any DRC issues; (5) `render_svg` \
         to confirm. Author every instance's placement explicitly — Aelm does \
         not rely on auto-layout.",
    );
    vec![
        assistant(
            "I'll design this Aelm circuit step by step, using the DSL reference \
             and the parts catalog, and validate before finishing."
                .to_string(),
        ),
        user(intro),
        user(format!("Aelm DSL reference:\n\n{}", doc("dsl"))),
        user(format!(
            "Placement and connection conventions:\n\n{}",
            doc("patterns")
        )),
    ]
}

/// `review_circuit` — review an existing circuit for issues and improvements.
pub fn review_circuit(source: &str) -> Vec<PromptMessage> {
    vec![
        assistant(
            "I'll review this circuit against DRC rules and common mistakes, then \
             suggest concrete improvements."
                .to_string(),
        ),
        user(format!(
            "Review this Aelm circuit:\n\n```aelm\n{source}\n```"
        )),
        user(format!("DRC rules:\n\n{}", doc("drc"))),
        user(format!(
            "Common mistakes to check for:\n\n{}",
            doc("mistakes")
        )),
    ]
}

/// `debug_drc` — help fix specific DRC errors.
pub fn debug_drc(source: &str, error_codes: Option<&str>) -> Vec<PromptMessage> {
    let focus = error_codes.map_or_else(
        || "Diagnose every DRC error and warning.".to_string(),
        |codes| format!("Focus on these DRC codes: {codes}."),
    );
    vec![
        assistant("I'll run `validate`, explain each DRC finding, and propose fixes.".to_string()),
        user(format!(
            "Debug the DRC issues in this circuit. {focus}\n\n```aelm\n{source}\n```"
        )),
        user(format!("Full DRC rule reference:\n\n{}", doc("drc"))),
    ]
}

/// `select_parts` — help choose parts for a requirement.
pub fn select_parts(requirements: &str) -> Vec<PromptMessage> {
    vec![
        assistant(
            "I'll search the parts catalog and recommend parts that fit, with \
             rationale."
                .to_string(),
        ),
        user(format!(
            "Recommend Aelm parts for this requirement:\n{requirements}\n\n\
             Use `list_parts` / `search_parts` to ground recommendations in the \
             actual catalog."
        )),
    ]
}

/// `learn_aelm` — structured tutorial by level.
pub fn learn_aelm(level: &str) -> Vec<PromptMessage> {
    let level = match level {
        "intermediate" | "advanced" => level,
        _ => "beginner",
    };
    vec![
        assistant(format!(
            "I'll teach Aelm at the {level} level, with runnable examples you can \
             `render_svg` to see."
        )),
        user(format!("Teach me Aelm at the {level} level.")),
        user(format!("Aelm DSL reference:\n\n{}", doc("dsl"))),
        user(format!(
            "Circuit patterns to learn from:\n\n{}",
            doc("patterns")
        )),
    ]
}

/// `explain_circuit` — step-by-step explanation of how a circuit works.
pub fn explain_circuit(source: &str) -> Vec<PromptMessage> {
    vec![
        assistant("I'll explain this circuit's topology and operation node by node.".to_string()),
        user(format!(
            "Explain how this circuit works, step by step:\n\n```aelm\n{source}\n```"
        )),
    ]
}

/// `interactive_design` — a multi-turn render→validate→iterate design loop.
pub fn interactive_design(goal: &str) -> Vec<PromptMessage> {
    vec![
        assistant(
            "I'll work iteratively: draft, `validate`, `render_svg`, inspect, and \
             refine — repeating until the design meets the goal and is DRC-clean."
                .to_string(),
        ),
        user(format!(
            "Design goal:\n{goal}\n\n\
             Loop each iteration: (1) edit the Aelm source; (2) `validate` and \
             read the diagnostics; (3) `render_svg` and check the layout; \
             (4) decide what to change next. Stop when DRC is clean and the \
             layout matches the intent."
        )),
        user(format!("Aelm DSL reference:\n\n{}", doc("dsl"))),
        user(format!("Circuit patterns:\n\n{}", doc("patterns"))),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(msgs: &[PromptMessage]) -> String {
        msgs.iter()
            .filter_map(|m| match &m.content {
                rmcp::model::PromptMessageContent::Text { text } => Some(text.clone()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn test_design_circuit_includes_requirements_and_docs() {
        let msgs = design_circuit("a 5V regulator", Some("max 100mA"));
        let body = texts(&msgs);
        assert!(body.contains("a 5V regulator"));
        assert!(body.contains("max 100mA"));
        // DSL reference must be embedded.
        assert!(body.contains("module"));
    }

    #[test]
    fn test_design_circuit_without_constraints() {
        let msgs = design_circuit("an RC filter", None);
        assert!(!msgs.is_empty());
        assert!(texts(&msgs).contains("an RC filter"));
    }

    #[test]
    fn test_review_circuit_embeds_drc_and_mistakes() {
        let msgs = review_circuit("module M {}");
        let body = texts(&msgs);
        assert!(body.contains("module M {}"));
        assert!(body.to_lowercase().contains("drc"));
    }

    #[test]
    fn test_debug_drc_with_codes() {
        let msgs = debug_drc("module M {}", Some("DRC-E001, DRC-W004"));
        assert!(texts(&msgs).contains("DRC-E001"));
    }

    #[test]
    fn test_debug_drc_without_codes() {
        let msgs = debug_drc("module M {}", None);
        assert!(texts(&msgs).to_lowercase().contains("every drc"));
    }

    #[test]
    fn test_learn_aelm_normalises_level() {
        assert!(texts(&learn_aelm("bogus")).contains("beginner"));
        assert!(texts(&learn_aelm("advanced")).contains("advanced"));
    }

    #[test]
    fn test_select_parts_includes_requirements() {
        assert!(texts(&select_parts("low-noise op-amp")).contains("low-noise op-amp"));
    }

    #[test]
    fn test_explain_circuit_includes_source() {
        assert!(texts(&explain_circuit("module X {}")).contains("module X {}"));
    }

    #[test]
    fn test_interactive_design_includes_goal_and_docs() {
        let msgs = interactive_design("a debounced button input");
        let body = texts(&msgs);
        assert!(body.contains("a debounced button input"));
        assert!(body.contains("module"));
    }

    #[test]
    fn test_learn_aelm_invalid_level_defaults_to_beginner() {
        let body = texts(&learn_aelm(""));
        assert!(
            body.contains("beginner"),
            "empty level should default to beginner"
        );
    }
}
