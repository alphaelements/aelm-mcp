# AI aelm CLI Reference

**Last updated**: 2026-04-08
**Japanese version**: [AI-CLI-Reference](AI-CLI-Reference)

---

## §1 Overview

The `aelm` CLI is Aelm's **native command-line tool**. It enables circuit validation, formatting, and export without the VSCode extension.

**Primary use cases**:
- Syntax checking and DRC validation of circuit files
- Schematic export to SVG/PNG formats
- Automated export in CI/CD pipelines
- JSON dump of layout results (for debugging)

---

## §2 Build

```bash
# Development build
cargo build -p aelm-cli

# Release build
cargo build -p aelm-cli --release

# Install
cargo install --path crates/aelm-cli
```

The resulting binary name is `aelm`.

---

## §3 Subcommands

### §3.1 export-svg — SVG Export

```
aelm export-svg <INPUT> [-o <OUTPUT>] [--grid] [--module <NAME>] [--style <ASTYLE>]
```

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `INPUT` | path | (required) | Input `.aelm` file |
| `-o, --output` | path | `<INPUT>.svg` | Output file path |
| `--grid` | flag | false | Include grid overlay |
| `--module` | string | first module | Target module name |
| `-s, --style` | path | auto-detect | `.astyle` stylesheet (see §7.2) |

**Examples**:
```bash
aelm export-svg circuit.aelm                        # → circuit.svg
aelm export-svg circuit.aelm -o out.svg --grid       # With grid
aelm export-svg circuit.aelm --module MyModule       # Specific module
aelm export-svg circuit.aelm --style dark.astyle     # Custom style
```

### §3.2 export-png — PNG Export

```
aelm export-png <INPUT> [-o <OUTPUT>] [--dpi <DPI>] [--grid] [--module <NAME>] [--style <ASTYLE>]
```

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `INPUT` | path | (required) | Input `.aelm` file |
| `-o, --output` | path | `<INPUT>.png` | Output file path |
| `--dpi` | integer | 300 | Resolution (DPI) |
| `--grid` | flag | false | Include grid overlay |
| `--module` | string | first module | Target module name |
| `-s, --style` | path | auto-detect | `.astyle` stylesheet (see §7.2) |

**Note**: Requires `png-export` feature (enabled by default). Returns exit code 2 if unavailable.

**Examples**:
```bash
aelm export-png circuit.aelm --dpi 600               # High-resolution PNG
aelm export-png circuit.aelm -o preview.png --grid    # Preview with grid
aelm export-png circuit.aelm --style dark.astyle      # Custom style
```

### §3.3 check — Syntax Check + DRC

```
aelm check <INPUT>
```

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `INPUT` | path | (required) | Input `.aelm` file |

**Output**: rustc-style diagnostics on stderr + summary line.

**Examples**:
```bash
aelm check circuit.aelm
# Output example:
# 0 error(s), 0 warning(s)

aelm check bad_circuit.aelm
# Output example:
# error[E001]: unknown part type 'Resistr'
#  --> bad_circuit.aelm:3:13
# 1 error(s), 0 warning(s)
```

### §3.4 fmt — Source Formatting

```
aelm fmt <INPUT> [--check]
```

| Argument | Type | Default | Description |
|----------|------|---------|-------------|
| `INPUT` | path | (required) | Input `.aelm` file |
| `--check` | flag | false | Check only (no file modification) |

**Examples**:
```bash
aelm fmt circuit.aelm            # Format and overwrite
aelm fmt circuit.aelm --check    # Check only (for CI)
```

### §3.5 parse — Parse Result Dump

```
aelm parse <INPUT>
```

JSON output to stdout:
```json
{
  "parts": [...],
  "modules": [...],
  "diagnostics": [...]
}
```

**Examples**:
```bash
aelm parse circuit.aelm | jq '.modules[0].instances'
```

### §3.6 layout — Layout Result Dump

```
aelm layout <INPUT> [--module <NAME>]
```

JSON output to stdout:
```json
{
  "layout": { "components": [...], "bounds": {...} },
  "routing": { "wires": [...], "labels": [...], "junctions": [...] },
  "diagnostics": [...]
}
```

**Examples**:
```bash
aelm layout circuit.aelm | jq '.layout.components[] | {name: .instance_name, pos: .position}'
aelm layout circuit.aelm --module MyModule | jq '.routing.wires | length'
```

---

## §4 Global Options

### Verbosity (-v)

| Flag | Level | Output |
|------|-------|--------|
| (none) | warn | Warnings and errors only |
| `-v` | info | Add informational messages |
| `-vv` | debug | Add debug information |
| `-vvv` | trace | Full trace logging |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `AELM_LOG` | Log level setting (env_logger format) |
| `AELM_STDLIB_PATH` | Explicit stdlib path specification |

---

## §5 Exit Codes

| Code | Meaning | Trigger |
|------|---------|---------|
| 0 | Success | Command completed normally |
| 1 | Input error | Parse error, DRC error, module not found |
| 2 | Argument error | Invalid CLI arguments, feature not enabled |
| 3 | I/O error | File not found, write failure |

---

## §6 Typical Workflows

### Circuit Development

```bash
# 1. Write circuit
vim circuit.aelm

# 2. Syntax check
aelm check circuit.aelm

# 3. Format
aelm fmt circuit.aelm

# 4. SVG preview
aelm export-svg circuit.aelm --grid -o preview.svg
```

### Layout Debugging

```bash
# Check component placement
aelm layout circuit.aelm | jq '.layout.components[] | {name: .instance_name, pos: .position, rot: .rotation}'

# Check routing
aelm layout circuit.aelm | jq '.routing.wires[] | {net: .net_name, segments: (.segments | length)}'

# Check diagnostics
aelm layout circuit.aelm | jq '.diagnostics'
```

### CI/CD Integration

```bash
# CI pipeline example
aelm check circuit.aelm || exit 1
aelm fmt circuit.aelm --check || exit 1
aelm export-svg circuit.aelm -o output.svg
```

### Reverse Sync Debugging (Phase 2)

```bash
# Move component R1 (grid 2, grid 4) — ~(5.08mm, 10.16mm)
aelm apply-move circuit.aelm R1 5.08 10.16
# → Updates circuit.aelm inline hint: R1: Resistor place: (2, 4)

# Change lock level of U1
aelm apply-lock circuit.aelm U1 layout_locked
# → Updates circuit.aelm inline hint: U1: OpAmp lock: layout_locked

# Rotate Q1 by 90 degrees
aelm apply-rotate circuit.aelm Q1 90
# → Updates circuit.aelm inline hint: Q1: NPN rot: 90 (...)

# Mirror U2 horizontally
aelm apply-mirror circuit.aelm U2 horizontal
# → Updates circuit.aelm inline hint: U2: OpAmp mirror: horizontal (...)

# Add a new connection
aelm apply-add-connection circuit.aelm R1.a U1.in
# → Appends to connections block: R1.a -> U1.in

# Reconnect an existing wire endpoint
aelm apply-wire-reconnect circuit.aelm R1.p1 C1.p1
# → Changes R1.p1 to C1.p1 in the connections block

# Delete a connection
aelm apply-delete-connection circuit.aelm R1.p1 R2.p2
# → Removes "R1.p1 -> R2.p2" from the connections block

# Verify changes
aelm check circuit.aelm
aelm layout circuit.aelm | jq '.layout.components[] | select(.instance_name == "R1") | .position'
```

---

## §6.5 Reverse Sync Debug Commands (Phase 2)

### apply-move — Component Move

```
aelm apply-move <INPUT> <INSTANCE> <X> <Y>
```

| Argument | Type | Description |
|----------|------|-------------|
| `INPUT` | path | Input `.aelm` file |
| `INSTANCE` | string | Instance name (e.g., `R1`) |
| `X` | float | X coordinate in mm (grid-snapped to 2.54mm) |
| `Y` | float | Y coordinate in mm (grid-snapped to 2.54mm) |

Updates the instance's `place:` inline hint in `<INPUT>` (#133). When the
instance has no existing `place:` hint, a new one is inserted; otherwise
the existing hint is updated. The co-located `.astyle` file is not
modified.

### apply-lock — Lock Level Change

```
aelm apply-lock <INPUT> <INSTANCE> <LEVEL>
```

| Argument | Type | Description |
|----------|------|-------------|
| `LEVEL` | string | `unlocked`, `group_locked`, `layout_locked`, `route_locked`, `full_locked` |

Updates the instance's `lock:` inline hint in `<INPUT>` (#133). `unlocked`
removes the hint. The co-located `.astyle` file is not modified.

### apply-rotate — Component Rotation

```
aelm apply-rotate <INPUT> <INSTANCE> <DEGREES>
```

| Argument | Type | Description |
|----------|------|-------------|
| `DEGREES` | integer | `0`, `90`, `180`, `270` |

Updates the `.aelm` file's `rot:` inline hint via CST edit.

### apply-mirror — Component Mirror

```
aelm apply-mirror <INPUT> <INSTANCE> <AXIS>
```

| Argument | Type | Description |
|----------|------|-------------|
| `AXIS` | string | `horizontal` or `vertical` |

Updates the `.aelm` file's `mirror:` inline hint via CST edit.

### apply-wire-reconnect — Reconnect Wire Endpoint

```
aelm apply-wire-reconnect <INPUT> <OLD_PIN> <NEW_PIN>
```

| Argument | Type | Description |
|----------|------|-------------|
| `INPUT` | path | Input `.aelm` file |
| `OLD_PIN` | string | Existing pin reference in `instance.pin` format (e.g., `R1.a`) |
| `NEW_PIN` | string | Replacement pin reference in `instance.pin` format (e.g., `C1.p`) |

Replaces `OLD_PIN` with `NEW_PIN` in the connections block. Returns an error if `OLD_PIN` is not found.

**Examples**:
```bash
aelm apply-wire-reconnect circuit.aelm R1.p1 C1.p1
# → Changes: "R1.p1 -> R2.p2" to "C1.p1 -> R2.p2"
```

### apply-add-connection — Add New Wire Connection

```
aelm apply-add-connection <INPUT> <FROM_PIN> <TO_PIN>
```

| Argument | Type | Description |
|----------|------|-------------|
| `INPUT` | path | Input `.aelm` file |
| `FROM_PIN` | string | Source pin reference in `instance.pin` format (e.g., `R1.a`) |
| `TO_PIN` | string | Target pin reference in `instance.pin` format (e.g., `C1.p`) |

Appends a new `FROM_PIN -> TO_PIN` connection line to the connections block without modifying existing connections.

**Examples**:
```bash
aelm apply-add-connection circuit.aelm R1.a U1.in
# → Appends: "R1.a -> U1.in" to the connections block
```

### apply-delete-connection — Delete Wire Connection

```
aelm apply-delete-connection <INPUT> <FROM_PIN> <TO_PIN>
```

| Argument | Type | Description |
|----------|------|-------------|
| `INPUT` | path | Input `.aelm` file |
| `FROM_PIN` | string | Source pin reference in `instance.pin` format (e.g., `R1.p1`) |
| `TO_PIN` | string | Target pin reference in `instance.pin` format (e.g., `C1.p2`) |

Removes the line `FROM_PIN -> TO_PIN` from the connections block in the `.aelm` file. Returns an error if the connection is not found.

**Examples**:
```bash
aelm apply-delete-connection circuit.aelm R1.p1 R2.p2
# → Removes: "R1.p1 -> R2.p2" from the connections block
```

---

## §7 Project Configuration

### .aproj Auto-Detection

Walks parent directories of the input file upward to find the first `.aproj` file and loads it as project configuration.

```
project/
├── project.aproj      ← Auto-detected
├── stdlib/
│   └── parts/
└── circuits/
    └── amplifier.aelm  ← Input file
```

### §7.2 .astyle Stylesheet Loading Order

When `--style` is not specified for `export-svg` / `export-png`, stylesheets are auto-detected:

1. Co-located `.astyle` file with the same name (e.g., `circuit.aelm` → `circuit.astyle`)
2. `stdlib/themes/default.astyle` (resolved via stdlib path)
3. If neither found, no stylesheet (hardcoded defaults)

```bash
# Explicit (highest priority)
aelm export-png circuit.aelm --style mytheme.astyle

# Auto-detect (uses circuit.astyle if in same directory)
aelm export-png circuit.aelm

# See AI-Style-Reference for stylesheet syntax details
```

> For stylesheet syntax, selectors, and properties, see [AI-Style-Reference.en](AI-Style-Reference.en).

### stdlib Path Resolution Order

1. If `.aproj` found: `<aproj_dir>/stdlib/parts/`
2. `AELM_STDLIB_PATH` environment variable
3. Binary-relative: `<binary_dir>/../stdlib/parts/`
4. Fallback: `./stdlib/parts/`
