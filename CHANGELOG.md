# aelm-mcp Changelog

Format: [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
Versioning: [SemVer 2.0.0](https://semver.org/spec/v2.0.0.html).

## [0.5.0] - 2026-05-20

### Added

- **Advanced tools (Phase 3).**
  - `analyze_connectivity` (isolated instances, net fan-out) and
    `extract_subcircuit` (pull instances + mutual connections into a standalone
    module), backed by new `aelm analyze connectivity` / `extract` CLI commands.
  - `diff_circuits`: structural diff (added / removed / changed instances) via
    two parses compared MCP-side.
  - `render_batch`: render several circuits concurrently.
  - `preview_style`: render light vs dark for visual comparison.
  - `suggest_placement`: flag instances missing an explicit `place:` hint.
  - `evaluate_calc`: render and surface evaluated calc/plot data.
  - `suggest_circuit_pattern`: search the embedded pattern library by query.
  - `find_similar_circuits`: rank candidates by part-multiset (Jaccard) similarity.
  - `validate_library` and `compile_svg_to_symbol` (SVG → symbol block).
  - `interactive_design` prompt: a render → validate → refine loop.
- **Prompts, analysis, edit & scaffold tools (Phase 2).**
  - Six workflow prompts: `design_circuit`, `review_circuit`, `debug_drc`,
    `select_parts`, `learn_aelm`, `explain_circuit` — each seeded with the
    relevant embedded reference docs.
  - Analysis tools: `analyze_project`, `extract_netlist`, `extract_bom`
    (backed by `aelm analyze *`).
  - Edit tools (dry-run, source-in → modified-source-out): `apply_move`,
    `apply_rotate`, `apply_mirror`, `apply_add_connection`,
    `apply_delete_connection`.
  - `scaffold_circuit`: generate a valid `.aelm` skeleton from a part list and
    validate it in one call.
- **MCP server skeleton (Phase 1).** stdio transport over `rmcp`, invoking the
  `aelm` CLI via subprocess (`tokio::process`). 14 tools:
  - Parse & validate: `parse`, `validate`, `format`.
  - Render: `render_svg`, `render_png`, `render_symbol` (returns SVG text plus
    an inline image, or a base64 PNG image).
  - Catalog: `list_parts`, `search_parts`, `get_part_info`, `list_symbols`,
    `get_symbol_info`, `list_examples`, `get_example`.
  - Docs: `get_dsl_reference` (embedded reference by topic).
- **Resources.** `aelm://docs/{topic}` for the six embedded reference documents
  (dsl, cli, style, patterns, drc, mistakes).
- **Configuration.** `--aelm-path`, `--library-dir` (repeatable), `--working-dir`,
  `--log-level`.
- Tool failures are reported in-band via the MCP `isError` flag, never as
  protocol errors, so the model can read and react to CLI diagnostics.
- Initial repository scaffold (README, LICENSE, CHANGELOG) and project-wide
  quality gates mirrored via lefthook.
