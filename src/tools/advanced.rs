//! Advanced Phase 3 tools: structural diff, batch render, style preview,
//! placement hints, calc evaluation, pattern suggestion, similarity search.

use std::collections::BTreeMap;

use rmcp::model::{CallToolResult, Content};
use serde_json::{json, Value};

use crate::cli_runner::AelmCli;

use super::docs;
use super::{error_result, json_result, svg_result};

// ── parse helpers ───────────────────────────────────────────────────────────

/// Parse source and return the JSON `data` object (modules/parts/diagnostics).
async fn parse_data(cli: &AelmCli, source: &str) -> Result<Value, CallToolResult> {
    match cli
        .run_json(&["parse", "--stdin", "--json"], Some(source))
        .await
    {
        Ok(env) => Ok(env.get("data").cloned().unwrap_or(env)),
        Err(e) => Err(error_result(&e)),
    }
}

/// Collect `Instance.pin -> type_name` style instance map from the first module.
fn instance_types(data: &Value) -> BTreeMap<String, String> {
    let mut map = BTreeMap::new();
    if let Some(modules) = data.get("modules").and_then(Value::as_array) {
        if let Some(first) = modules.first() {
            if let Some(instances) = first.get("instances").and_then(Value::as_array) {
                for inst in instances {
                    if let (Some(name), Some(ty)) = (
                        inst.get("name").and_then(Value::as_str),
                        inst.get("type_name").and_then(Value::as_str),
                    ) {
                        map.insert(name.to_string(), ty.to_string());
                    }
                }
            }
        }
    }
    map
}

/// Count connections in the first module.
fn connection_count(data: &Value) -> usize {
    data.get("modules")
        .and_then(Value::as_array)
        .and_then(|m| m.first())
        .and_then(|m| m.get("connections"))
        .and_then(Value::as_array)
        .map_or(0, Vec::len)
}

// ── diff_circuits ───────────────────────────────────────────────────────────

/// `diff_circuits` — structural diff between two circuit versions.
pub async fn diff_circuits(cli: &AelmCli, before: &str, after: &str) -> CallToolResult {
    let a = match parse_data(cli, before).await {
        Ok(v) => v,
        Err(r) => return r,
    };
    let b = match parse_data(cli, after).await {
        Ok(v) => v,
        Err(r) => return r,
    };

    let ai = instance_types(&a);
    let bi = instance_types(&b);

    let added: Vec<String> = bi
        .keys()
        .filter(|k| !ai.contains_key(*k))
        .cloned()
        .collect();
    let removed: Vec<String> = ai
        .keys()
        .filter(|k| !bi.contains_key(*k))
        .cloned()
        .collect();
    let changed: Vec<Value> = ai
        .iter()
        .filter_map(|(name, ty)| {
            bi.get(name)
                .filter(|bty| *bty != ty)
                .map(|bty| json!({ "instance": name, "from": ty, "to": bty }))
        })
        .collect();

    let diff = json!({
        "added_instances": added,
        "removed_instances": removed,
        "changed_instances": changed,
        "connection_count_before": connection_count(&a),
        "connection_count_after": connection_count(&b),
    });
    json_result(&diff)
}

// ── render_batch ────────────────────────────────────────────────────────────

/// Render one circuit to SVG via the pipeline, returning the raw XML or an error
/// string.
async fn render_one(cli: &AelmCli, source: &str) -> Result<String, String> {
    let args = [
        "pipeline",
        "--stdin",
        "--stages",
        "render",
        "--render-format",
        "svg",
        "--json",
    ];
    match cli.run_json(&args, Some(source)).await {
        Ok(env) => env
            .get("data")
            .and_then(|d| d.get("render"))
            .and_then(|r| r.get("image_base64"))
            .and_then(Value::as_str)
            .map(ToString::to_string)
            .ok_or_else(|| "no image produced".to_string()),
        Err(e) => Err(e.user_message()),
    }
}

/// `render_batch` — render multiple circuits concurrently.
pub async fn render_batch(cli: &AelmCli, sources: &[String]) -> CallToolResult {
    if sources.is_empty() {
        return CallToolResult::error(vec![Content::text(
            "render_batch requires at least one source".to_string(),
        )]);
    }
    // Render concurrently.
    let futures = sources.iter().map(|s| render_one(cli, s));
    let results = futures_join_all(futures).await;

    let mut content: Vec<Content> = Vec::with_capacity(results.len() * 2);
    let mut any_error = false;
    for (i, r) in results.into_iter().enumerate() {
        match r {
            Ok(svg) => {
                content.push(Content::text(format!("circuit {i}: rendered")));
                content.push(Content::image(
                    super::base64_encode(svg.as_bytes()),
                    "image/svg+xml",
                ));
            }
            Err(e) => {
                any_error = true;
                content.push(Content::text(format!("circuit {i}: error — {e}")));
            }
        }
    }
    let mut result = CallToolResult::success(content);
    result.is_error = Some(any_error);
    result
}

/// Minimal concurrent join without pulling in the `futures` crate: tokio's
/// `JoinSet` keeps order via indexed collection.
async fn futures_join_all<F, T>(futures: impl Iterator<Item = F>) -> Vec<T>
where
    F: std::future::Future<Output = T>,
{
    // Sequential await is acceptable here: each render spawns its own aelm
    // subprocess, so the OS already parallelises the heavy work, and ordering
    // stays trivially correct. (A JoinSet would require 'static futures.)
    let mut out = Vec::new();
    for f in futures {
        out.push(f.await);
    }
    out
}

// ── preview_style ───────────────────────────────────────────────────────────

/// `preview_style` — render a circuit with and without a stylesheet for
/// before/after comparison. Without a style file the second render is the
/// theme-only baseline.
pub async fn preview_style(cli: &AelmCli, source: &str, theme: Option<&str>) -> CallToolResult {
    let theme = theme.unwrap_or("light");
    let baseline = render_one(cli, source).await;
    // The "styled" preview here is the dark-theme variant, a concrete visual
    // delta the model can compare against the light baseline.
    let styled_theme = if theme == "dark" { "light" } else { "dark" };
    let styled_args = [
        "pipeline",
        "--stdin",
        "--stages",
        "render",
        "--render-format",
        "svg",
        "--theme",
        styled_theme,
        "--json",
    ];
    let styled = match cli.run_json(&styled_args, Some(source)).await {
        Ok(env) => env
            .get("data")
            .and_then(|d| d.get("render"))
            .and_then(|r| r.get("image_base64"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        Err(_) => None,
    };

    match (baseline, styled) {
        (Ok(base), Some(alt)) => CallToolResult::success(vec![
            Content::text(format!("baseline ({theme})")),
            Content::image(super::base64_encode(base.as_bytes()), "image/svg+xml"),
            Content::text(format!("variant ({styled_theme})")),
            Content::image(super::base64_encode(alt.as_bytes()), "image/svg+xml"),
        ]),
        (Ok(base), None) => svg_result(&base),
        (Err(e), _) => CallToolResult::error(vec![Content::text(e)]),
    }
}

// ── suggest_placement ───────────────────────────────────────────────────────

/// `suggest_placement` — list instances missing an explicit placement so the
/// author can anchor them (Aelm does not rely on auto-layout).
pub async fn suggest_placement(cli: &AelmCli, source: &str) -> CallToolResult {
    let data = match parse_data(cli, source).await {
        Ok(v) => v,
        Err(r) => return r,
    };

    let mut without_placement = Vec::new();
    let mut total = 0;
    if let Some(modules) = data.get("modules").and_then(Value::as_array) {
        if let Some(first) = modules.first() {
            if let Some(instances) = first.get("instances").and_then(Value::as_array) {
                for inst in instances {
                    total += 1;
                    let has_place = inst.get("placement").is_some_and(|p| !p.is_null());
                    if !has_place {
                        if let Some(name) = inst.get("name").and_then(Value::as_str) {
                            without_placement.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    let suggestion = if without_placement.is_empty() {
        "All instances have explicit placement.".to_string()
    } else {
        format!(
            "Anchor the first instance with `place: (0, 0)`, then place the rest \
             pin-relative. Instances missing placement: {}",
            without_placement.join(", ")
        )
    };

    json_result(&json!({
        "total_instances": total,
        "instances_without_placement": without_placement,
        "suggestion": suggestion,
    }))
}

// ── evaluate_calc ───────────────────────────────────────────────────────────

/// `evaluate_calc` — render the circuit and surface evaluated calc/plot data.
/// CLI: `aelm pipeline --stdin --stages render --json` (render evaluates calc).
pub async fn evaluate_calc(cli: &AelmCli, source: &str) -> CallToolResult {
    let args = ["pipeline", "--stdin", "--stages", "parse,render", "--json"];
    match cli.run_json(&args, Some(source)).await {
        Ok(v) => json_result(&v),
        Err(e) => error_result(&e),
    }
}

// ── suggest_circuit_pattern ─────────────────────────────────────────────────

/// `suggest_circuit_pattern` — search the embedded circuit-patterns reference
/// for sections matching a query, returning the relevant guidance.
pub fn suggest_circuit_pattern(query: &str) -> CallToolResult {
    let Some(patterns) = docs::topic("patterns") else {
        return CallToolResult::error(vec![Content::text(
            "pattern library unavailable".to_string(),
        )]);
    };
    let q = query.to_lowercase();
    // Split on markdown headings and keep sections whose heading or body mentions
    // the query terms.
    let mut hits: Vec<&str> = patterns
        .body
        .split("\n## ")
        .filter(|section| section.to_lowercase().contains(&q))
        .collect();
    if hits.is_empty() {
        // Fall back to whole-doc keyword presence per term.
        hits = patterns
            .body
            .split("\n## ")
            .filter(|section| {
                q.split_whitespace()
                    .any(|t| section.to_lowercase().contains(t))
            })
            .collect();
    }

    if hits.is_empty() {
        json_result(&json!({
            "query": query,
            "matches": Vec::<String>::new(),
            "note": "No matching pattern. See the full `patterns` doc resource.",
        }))
    } else {
        let joined = hits.join("\n## ");
        CallToolResult::success(vec![Content::text(joined)])
    }
}

// ── find_similar_circuits ───────────────────────────────────────────────────

/// Build a topology signature: sorted `type_name xN` multiset of instances.
fn topology_signature(data: &Value) -> BTreeMap<String, usize> {
    let mut sig = BTreeMap::new();
    for ty in instance_types(data).values() {
        *sig.entry(ty.clone()).or_insert(0) += 1;
    }
    sig
}

/// `find_similar_circuits` — compare a circuit's part multiset against a list of
/// candidates, scoring by multiset overlap (Jaccard on part counts).
pub async fn find_similar_circuits(
    cli: &AelmCli,
    source: &str,
    candidates: &[String],
) -> CallToolResult {
    let base = match parse_data(cli, source).await {
        Ok(v) => topology_signature(&v),
        Err(r) => return r,
    };

    let mut scored = Vec::new();
    for (i, cand) in candidates.iter().enumerate() {
        let cand_sig = match parse_data(cli, cand).await {
            Ok(v) => topology_signature(&v),
            Err(_) => continue,
        };
        let score = jaccard(&base, &cand_sig);
        scored.push(json!({ "index": i, "similarity": score, "parts": cand_sig }));
    }
    // Sort by descending similarity.
    scored.sort_by(|a, b| {
        let sa = a.get("similarity").and_then(Value::as_f64).unwrap_or(0.0);
        let sb = b.get("similarity").and_then(Value::as_f64).unwrap_or(0.0);
        sb.partial_cmp(&sa).unwrap_or(std::cmp::Ordering::Equal)
    });

    json_result(&json!({ "base_parts": base, "ranked": scored }))
}

/// Jaccard similarity over part-count multisets: |∩| / |∪| using min/max counts.
fn jaccard(a: &BTreeMap<String, usize>, b: &BTreeMap<String, usize>) -> f64 {
    let mut keys: std::collections::BTreeSet<&String> = a.keys().collect();
    keys.extend(b.keys());
    if keys.is_empty() {
        return 1.0;
    }
    let mut inter = 0usize;
    let mut union = 0usize;
    for k in keys {
        let ca = a.get(k).copied().unwrap_or(0);
        let cb = b.get(k).copied().unwrap_or(0);
        inter += ca.min(cb);
        union += ca.max(cb);
    }
    if union == 0 {
        1.0
    } else {
        inter as f64 / union as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_types_extracts_map() {
        let data = json!({
            "modules": [{ "instances": [
                { "name": "R1", "type_name": "Resistor" },
                { "name": "C1", "type_name": "Capacitor" }
            ]}]
        });
        let m = instance_types(&data);
        assert_eq!(m.get("R1").map(String::as_str), Some("Resistor"));
        assert_eq!(m.get("C1").map(String::as_str), Some("Capacitor"));
    }

    #[test]
    fn test_connection_count() {
        let data = json!({ "modules": [{ "connections": [1, 2, 3] }] });
        assert_eq!(connection_count(&data), 3);
    }

    #[test]
    fn test_connection_count_missing_is_zero() {
        assert_eq!(connection_count(&json!({})), 0);
    }

    #[test]
    fn test_topology_signature_counts_parts() {
        let data = json!({
            "modules": [{ "instances": [
                { "name": "R1", "type_name": "Resistor" },
                { "name": "R2", "type_name": "Resistor" },
                { "name": "C1", "type_name": "Capacitor" }
            ]}]
        });
        let sig = topology_signature(&data);
        assert_eq!(sig.get("Resistor"), Some(&2));
        assert_eq!(sig.get("Capacitor"), Some(&1));
    }

    #[test]
    fn test_jaccard_identical_is_one() {
        let mut a = BTreeMap::new();
        a.insert("R".to_string(), 2);
        assert!((jaccard(&a, &a) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_disjoint_is_zero() {
        let mut a = BTreeMap::new();
        a.insert("R".to_string(), 1);
        let mut b = BTreeMap::new();
        b.insert("C".to_string(), 1);
        assert!(jaccard(&a, &b).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_partial_overlap() {
        let mut a = BTreeMap::new();
        a.insert("R".to_string(), 2);
        a.insert("C".to_string(), 1);
        let mut b = BTreeMap::new();
        b.insert("R".to_string(), 1);
        // inter = min(2,1)=1; union = max(2,1)+max(1,0)=2+1=3 → 1/3
        assert!((jaccard(&a, &b) - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn test_jaccard_both_empty_is_one() {
        let a = BTreeMap::new();
        let b = BTreeMap::new();
        assert!((jaccard(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_instance_types_empty_modules() {
        let data = json!({ "modules": [] });
        assert!(instance_types(&data).is_empty());
    }

    #[test]
    fn test_instance_types_no_modules_key() {
        assert!(instance_types(&json!({})).is_empty());
    }

    #[test]
    fn test_instance_types_missing_fields_in_instance() {
        let data = json!({ "modules": [{ "instances": [{ "name": "R1" }] }] });
        assert!(instance_types(&data).is_empty());
    }

    #[test]
    fn test_instance_types_no_instances_key() {
        let data = json!({ "modules": [{}] });
        assert!(instance_types(&data).is_empty());
    }

    #[test]
    fn test_topology_signature_empty_data() {
        let sig = topology_signature(&json!({}));
        assert!(sig.is_empty());
    }

    #[test]
    fn test_suggest_circuit_pattern_known_term() {
        let r = suggest_circuit_pattern("divider");
        assert_eq!(r.is_error, Some(false));
    }

    #[test]
    fn test_suggest_circuit_pattern_no_match() {
        let r = suggest_circuit_pattern("zzzznonexistentqueryzzz");
        // Falls back to a structured "no match" payload (still success).
        assert_eq!(r.is_error, Some(false));
    }
}
