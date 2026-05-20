# aelm-mcp

**Model Context Protocol server for [Aelm](https://github.com/alphaelements/aelm)** — give AI assistants (Claude Desktop, Claude Code, Cursor, VS Code, ...) first-class access to text-based electronic circuit design.

`aelm-mcp` is a thin, dependency-light wrapper around the `aelm` command-line
tool. It exposes Aelm's parsing, validation, rendering, parts/symbol catalog,
analysis, and editing capabilities as MCP **tools**, **resources**, and
**prompts**, so an AI can design, review, and debug circuits conversationally.

> Status: under active development. See [CHANGELOG.md](CHANGELOG.md).

## How it works

```
+-----------------+   stdio (MCP)   +--------------+   subprocess    +----------+
|  AI assistant   | <-------------> |   aelm-mcp   | <-------------> |   aelm   |
| (Claude, etc.)  |                 | (this server)|   JSON / stdin  |   CLI    |
+-----------------+                 +--------------+                 +----------+
```

`aelm-mcp` never links Aelm's core libraries. It invokes the `aelm` binary via
`tokio::process::Command`, passing source over stdin and reading structured
`{ success, data, diagnostics }` JSON back. This keeps the server small and the
Aelm core free to evolve independently.

## Requirements

| Requirement | Notes |
|---|---|
| [`aelm`](https://github.com/alphaelements/aelm) CLI | Must be the **same version** as `aelm-mcp` (lockstep release). |
| Rust 1.78+ | To build from source. |

> **Version mismatch will break things.** `aelm-mcp` calls CLI subcommands
> (`aelm parts`, `aelm analyze`, `aelm pipeline`, ...) that may not exist in
> older CLI versions. Always install matching versions.

## Install

### 1. Install the Aelm CLI

Download the `aelm` binary for your platform from the
[latest GitHub Release](https://github.com/alphaelements/aelm/releases/latest)
and place it on your `PATH`.

```bash
# Verify:
aelm --version
```

> **Note:** The `aelm` CLI binary will be included in GitHub Releases starting
> from v0.5.1. For v0.5.0, build from source with `cargo install --path
> crates/aelm-cli` in the Aelm repository.

### 2. Install the MCP server

```bash
cargo install --path .

# Verify:
aelm-mcp --version
```

Both commands should report the same version (e.g. `0.5.0`).

## Configure

### Claude Code (CLI / IDE extension)

```bash
claude mcp add aelm -- aelm-mcp --working-dir /path/to/your/project
```

Or add to `.mcp.json` at your project root:

```json
{
  "mcpServers": {
    "aelm": {
      "type": "stdio",
      "command": "aelm-mcp",
      "args": ["--working-dir", "/path/to/your/project"],
      "env": {}
    }
  }
}
```

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "aelm": {
      "command": "aelm-mcp",
      "args": ["--aelm-path", "/path/to/aelm"]
    }
  }
}
```

### Cursor / Other MCP clients

Use the stdio transport with `aelm-mcp` as the command. Pass `--aelm-path`
if the `aelm` binary is not on your `PATH`.

### Options

| Flag | Description |
|---|---|
| `--aelm-path <PATH>` | Path to the `aelm` binary (default: search `PATH`). |
| `--library-dir <DIR>` | Extra user symbol/part library directory (repeatable). |
| `--working-dir <DIR>` | Project context directory for relative `use` imports. |
| `--log-level <LEVEL>` | `error` \| `warn` \| `info` \| `debug` \| `trace`. |

## Quick test

After installing, verify the server responds to MCP initialization:

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"0.1.0"}}}' \
  | aelm-mcp --log-level error 2>/dev/null
```

You should see a JSON response with `"serverInfo"` and `"capabilities"`.

## Capabilities

### Tools (34)

| Category | Tools |
|---|---|
| **Parse & validate** | `parse`, `validate`, `format` |
| **Render** | `render_svg`, `render_png`, `render_symbol`, `render_batch`, `preview_style` |
| **Library** | `list_parts`, `search_parts`, `get_part_info`, `list_symbols`, `get_symbol_info`, `list_examples`, `get_example`, `validate_library` |
| **Analysis** | `analyze_project`, `extract_netlist`, `extract_bom`, `analyze_connectivity`, `extract_subcircuit`, `diff_circuits`, `find_similar_circuits` |
| **Edit** | `apply_move`, `apply_rotate`, `apply_mirror`, `apply_add_connection`, `apply_delete_connection` |
| **Generate** | `scaffold_circuit`, `compile_svg_to_symbol`, `suggest_placement`, `suggest_circuit_pattern` |
| **Reference** | `get_dsl_reference`, `evaluate_calc` |

### Resources (6)

| URI | Content |
|---|---|
| `aelm://docs/dsl` | DSL syntax reference |
| `aelm://docs/cli` | CLI command reference |
| `aelm://docs/style` | Stylesheet (`.astyle`) reference |
| `aelm://docs/patterns` | Common circuit patterns |
| `aelm://docs/drc` | DRC rule reference |
| `aelm://docs/mistakes` | Common authoring mistakes |

### Prompts (7)

| Prompt | Description |
|---|---|
| `design_circuit` | Guide circuit design from requirements to valid source |
| `review_circuit` | Review existing circuit for DRC issues and improvements |
| `debug_drc` | Diagnose and fix specific DRC errors |
| `select_parts` | Recommend parts for a design requirement |
| `learn_aelm` | Structured tutorial (beginner / intermediate / advanced) |
| `explain_circuit` | Step-by-step explanation of how a circuit works |
| `interactive_design` | Iterative render-validate-refine loop |

## Known limitations

- **`--stdin` and `.aproj` discovery**: When source is piped via `--stdin`
  (which is how all MCP parse/validate/render tools work), the CLI cannot
  discover a `.aproj` project file. For sources that use
  `use "stdlib/parts/..."`, the embedded stdlib resolves these automatically.
  For user-defined libraries, pass `--library-dir` to `aelm-mcp` instead.

## License

MIT — see [LICENSE](LICENSE).
