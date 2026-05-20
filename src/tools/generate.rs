//! Scaffold generation: build a valid `.aelm` skeleton from a part list.
//!
//! This is a pure template engine (no CLI) followed by a `validate` pass so the
//! returned skeleton is guaranteed to parse. Placement follows the Aelm
//! convention: the first instance is anchored at an absolute `place: (0, 0)` and
//! the rest are laid out on a row relative to it.

use std::fmt::Write as _;

use rmcp::model::{CallToolResult, Content};
use serde_json::json;

use crate::cli_runner::AelmCli;

use super::error_result;

/// One instance request for the scaffold.
#[derive(Debug, Clone)]
pub struct ScaffoldPart {
    pub name: String,
    pub type_name: String,
}

/// Build the `.aelm` skeleton text (no validation).
pub fn build_skeleton(module: &str, parts: &[ScaffoldPart]) -> String {
    let mut out = String::new();
    out.push_str("# aelm-version: 0.5.0\n");
    let _ = writeln!(out, "module {module} {{");
    out.push_str("    instances {\n");
    for (i, p) in parts.iter().enumerate() {
        if i == 0 {
            let _ = writeln!(out, "        {}: {} place: (0, 0)", p.name, p.type_name);
        } else {
            // Lay out subsequent parts in a row, relative to the first.
            let dx = i as i32 * 4;
            let _ = writeln!(
                out,
                "        {}: {} place: {}({dx}, 0)",
                p.name, p.type_name, parts[0].name
            );
        }
    }
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}

/// `scaffold_circuit` — generate a skeleton and validate it.
///
/// Returns the generated source plus the validation envelope so the model can
/// see immediately whether the skeleton is DRC-clean and what to wire next.
pub async fn scaffold_circuit(
    cli: &AelmCli,
    module: &str,
    parts: &[ScaffoldPart],
) -> CallToolResult {
    if parts.is_empty() {
        return CallToolResult::error(vec![Content::text(
            "scaffold_circuit requires at least one part".to_string(),
        )]);
    }
    let source = build_skeleton(module, parts);

    match cli
        .run_json(&["check", "--stdin", "--json"], Some(&source))
        .await
    {
        Ok(validation) => {
            let payload = json!({
                "source": source,
                "validation": validation,
            });
            let text = serde_json::to_string_pretty(&payload).unwrap_or_else(|_| source.clone());
            CallToolResult::success(vec![Content::text(text)])
        }
        Err(e) => error_result(&e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> Vec<ScaffoldPart> {
        vec![
            ScaffoldPart {
                name: "R1".to_string(),
                type_name: "Resistor".to_string(),
            },
            ScaffoldPart {
                name: "C1".to_string(),
                type_name: "Capacitor".to_string(),
            },
        ]
    }

    #[test]
    fn test_skeleton_has_version_header() {
        let s = build_skeleton("M", &parts());
        assert!(s.starts_with("# aelm-version:"));
    }

    #[test]
    fn test_skeleton_first_part_absolute_placement() {
        let s = build_skeleton("M", &parts());
        assert!(s.contains("R1: Resistor place: (0, 0)"));
    }

    #[test]
    fn test_skeleton_subsequent_part_relative_placement() {
        let s = build_skeleton("M", &parts());
        assert!(s.contains("C1: Capacitor place: R1(4, 0)"));
    }

    #[test]
    fn test_skeleton_wraps_in_module() {
        let s = build_skeleton("MyModule", &parts());
        assert!(s.contains("module MyModule {"));
        assert!(s.contains("instances {"));
    }

    #[test]
    fn test_skeleton_single_part() {
        let single = vec![ScaffoldPart {
            name: "U1".to_string(),
            type_name: "OpAmp".to_string(),
        }];
        let s = build_skeleton("Amp", &single);
        assert!(s.contains("U1: OpAmp place: (0, 0)"));
        assert!(!s.contains("place: U1"));
    }

    #[test]
    fn test_skeleton_empty_parts_produces_empty_instances() {
        let s = build_skeleton("Empty", &[]);
        assert!(s.contains("module Empty {"));
        assert!(s.contains("instances {"));
        let instances_block = s.split("instances {").nth(1).unwrap();
        assert!(
            !instances_block.contains("place:"),
            "no instances should be generated"
        );
    }
}
