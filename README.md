# aelm-mcp

**Model Context Protocol server for [Aelm](https://github.com/alphaelements/aelm)** — give AI assistants (Claude Desktop, Cursor, VS Code, …) first-class access to text-based electronic circuit design.

`aelm-mcp` is a thin, dependency-light wrapper around the `aelm` command-line
tool. It exposes Aelm's parsing, validation, rendering, parts/symbol catalog,
analysis, and editing capabilities as MCP **tools**, **resources**, and
**prompts**, so an AI can design, review, and debug circuits conversationally.

> Status: under active development. See [CHANGELOG.md](CHANGELOG.md).

## How it works

```
┌─────────────────┐   stdio (MCP)   ┌──────────────┐   subprocess    ┌──────────┐
│  AI assistant   │ ◀────────────▶ │   aelm-mcp   │ ◀────────────▶ │   aelm   │
│ (Claude, etc.)  │                 │ (this server)│   JSON / stdin  │   CLI    │
└─────────────────┘                 └──────────────┘                 └──────────┘
```

`aelm-mcp` never links Aelm's core libraries. It invokes the `aelm` binary via
`tokio::process::Command`, passing source over stdin and reading structured
`{ success, data, diagnostics }` JSON back. This keeps the server small and the
Aelm core free to evolve independently.

## Requirements

- The [`aelm`](https://github.com/alphaelements/aelm) CLI on your `PATH`
  (or pass `--aelm-path`).
- Rust 1.78+ to build from source.

## Install

```bash
cargo install --path .
```

## Configure (Claude Desktop)

```json
{
  "mcpServers": {
    "aelm": {
      "command": "aelm-mcp",
      "args": ["--aelm-path", "/usr/local/bin/aelm"]
    }
  }
}
```

### Options

| Flag | Description |
|------|-------------|
| `--aelm-path <PATH>` | Path to the `aelm` binary (default: search `PATH`). |
| `--library-dir <DIR>` | Extra user symbol/part library directory (repeatable). |
| `--working-dir <DIR>` | Project context directory for relative `use` imports. |
| `--log-level <LEVEL>` | `error` \| `warn` \| `info` \| `debug` \| `trace`. |

## Capabilities

- **Tools** — parse, validate, format, render (SVG/PNG), parts & symbol
  catalog, examples, project analysis (summary / netlist / BOM), structural
  diff, connectivity, and dry-run edit operations.
- **Resources** — `aelm://parts/`, `aelm://symbols/`, `aelm://examples/`,
  `aelm://docs/` for browsable reference material.
- **Prompts** — guided workflows: design, review, debug DRC, select parts,
  learn Aelm, explain a circuit.

## License

MIT — see [LICENSE](LICENSE).
