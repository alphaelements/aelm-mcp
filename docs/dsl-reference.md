# AI .aelm DSL Reference

**Last updated**: 2026-03-24
**Japanese version**: [AI-DSL-Reference](AI-DSL-Reference)

---

## §1 Overview

This document is a reference guide for AI (LLM) to write circuit schematics using the `.aelm` DSL.
It covers DSL syntax, stdlib components, placement hints, drafting rules, and typical circuit patterns.

The Aelm DSL is a **declarative** circuit description language — you describe "what to connect" and the layout engine automatically handles placement and routing.

---

## §2 File Structure

Basic structure of a `.aelm` file:

```aelm
// 1. Imports (optional)
use "stdlib/parts/passive.alib"
use "stdlib/parts/semiconductor.alib"

// 2. Part definitions (local or imported)
part PartName {
    pins { ... }
    symbol: symbol_name
}

// 3. Module definitions (circuits)
module ModuleName {
    ports { ... }
    instances { ... }
    connections { ... }
}
```

### File Types

| Extension | Purpose | Description |
|-----------|---------|-------------|
| `.aelm` | Circuit definition | Contains part + module definitions |
| `.alib` | Parts library | Collection of part definitions (stdlib, etc.) |
| `.astyle` | Stylesheet | Layout customization |
| `.aproj` | Project config | Grid settings, routing config, etc. |

### use Statements (Imports)

```aelm
use "stdlib/parts/passive.alib"          // Import from stdlib
use "my_parts/custom_transistor.alib"    // Project-relative path
```

You can also define `part` directly in the file without `use` statements.

---

## §3 Part Definition

```aelm
part PartName {
    params {                          // Optional: parameter definitions
        value: Ohm = 10k
    }
    pins {                            // Required: pin definitions
        pin_name: electrical_type @number
    }
    symbol: built_in_symbol_name      // Required: symbol reference
    footprints { ... }                // Optional: for .alib
    metadata { ... }                  // Optional: for .alib
    units { ... }                     // Optional: multi-unit parts
}
```

### units Block (Multi-Unit Parts)

Splits a single physical package into multiple symbols (units). Used for parts such as the LM358 dual op-amp.

```aelm
units {
    Suffix {                          // Identifier (A, B, ...)
        pins: {                       // Explicit symbol-pin -> part-pin map
            symbol_pin_1: part_pin_1
            symbol_pin_2: part_pin_2
        }
        symbol: built_in_symbol_name
    }
}
```

The binding order must mirror the symbol template's own pin order. An instance `U1: Part` is drawn as `U1A`, `U1B`, .... Every part pin must belong to exactly one unit; mapping the same part pin under multiple units is rejected with `E-UNITS-DUPLICATE-PIN-ASSIGNMENT`. Shared rails (VCC/GND) are expressed via a dedicated power-block unit (#138).

The legacy `pins: [p1, p2, ...]` array form is still accepted for backward compatibility but emits a `W-UNITS-LEGACY` warning. Prefer the mapping form above for new code.

### electrical_type

| Type | Meaning | Example Usage |
|------|---------|---------------|
| `input` | Input pin | Transistor B (base), op-amp IN |
| `output` | Output pin | Transistor C (collector), op-amp OUT |
| `passive` | Passive pin | Resistor/capacitor terminals, transistor E |
| `power` | Power pin | VCC, GND, op-amp power pins |

### Built-in Symbols

| Symbol Name | Component Type | Pins | Description |
|------------|---------------|------|-------------|
| `resistor` | Resistor etc. | 2 | Generic 2-terminal symbol (zigzag) |
| `capacitor` | Capacitor | 2 | Parallel lines symbol |
| `inductor` | Inductor | 2 | Coil symbol |
| `diode` | Diode | 2 | Triangle + line symbol |
| `led` | LED | 2 | Diode + arrows |
| `zener` | Zener | 2 | Diode + Z-line |
| `transistor_npn` | NPN | 3 | BJT NPN symbol |
| `transistor_pnp` | PNP | 3 | BJT PNP symbol |
| `transistor_nmos` | NMOS | 3 | N-ch MOSFET symbol |
| `transistor_pmos` | PMOS | 3 | P-ch MOSFET symbol |
| `op_amp` | Op-amp | 5 | Triangle (INP/INM/OUT/VCC/VEE) |
| `op_amp_signal` | Op-amp (signal-only) | 3 | Triangle without supply stubs. For `units{}` split (#138) |
| `op_amp_power` | Op-amp power block | 2 | Rectangle, VCC top / VEE bottom. For `units{}` split (#138) |
| `crystal` | Crystal | 2 | Crystal symbol |

---

## §4 Module Definition

```aelm
module ModuleName {
    ports {
        port_name: electrical_type direction
    }
    instances {
        InstanceName: PartType(param: value) place: (...) rot: N
    }
    connections {
        Instance1.pin -> Instance2.pin
    }
}
```

### ports Block

Define external connection points of the module.

```aelm
ports {
    VCC: power inout      // Power port
    GND: passive inout    // Ground port
    IN: input in          // Input port
    OUT: output out       // Output port
}
```

direction: `in` | `out` | `inout`

### instances Block

Create instances of parts.

```aelm
instances {
    R1: Resistor                           // Default placement
    R2: Resistor(value: 4.7k)             // With parameter
    Q1: NPN place: (5, 8)                 // Absolute placement
    RE2: Resistor place: Q2.e(0, 4) rot: 90  // Pin-relative + rotation
}
```

### connections Block

Define connections between pins.

```aelm
connections {
    R1.a -> R2.b              // Basic connection
    Q1.c -> Q2.b              // Between different instances
    VCC -> RC1.a              // Connection to port
    Q1.e -> GND               // Connection to ground
}
```

### Module Instantiation (Hierarchical Design)

Modules can be instantiated just like parts. Their `ports` act as pins.

```aelm
// Sub-module definition
module Amplifier {
    ports {
        inp: passive in
        out: passive out
        vcc: power in
        gnd: power in
    }
    instances {
        Q1: NPN
        R_LOAD: Resistor(value: 10k)
    }
    connections {
        inp -> Q1.b
        vcc -> R_LOAD.a
        R_LOAD.b -> Q1.c
        Q1.c -> out
        Q1.e -> gnd
    }
}

// Instantiate in parent module
module MainBoard {
    instances {
        AMP1: Amplifier place: (5, 8)     // Use module like a part
        AMP2: Amplifier place: (15, 8)
        R_COUPLING: Resistor place: (10, 8)
    }
    connections {
        AMP1.out -> R_COUPLING.a          // Connect via ports
        R_COUPLING.b -> AMP2.inp
    }
}
```

**Rendering**: Module instances appear as rectangular blocks with port pins. Pin placement follows port direction (in → left, out → right, power → top/bottom).

**Constraints**:
- Maximum nesting depth: 3 levels
- Circular and self-references are forbidden
- Same-file module references are the primary use case (cross-file via `use` is a future extension)

---

## §5 Placement Hints

### place: (x, y) — Absolute Grid Coordinates

```aelm
Q1: NPN place: (5, 8)    // Place at grid coordinate (5, 8)
```

### place: Instance.pin(dx, dy) — Pin-Relative Placement

```aelm
RE2: Resistor place: Q2.e(0, 4)    // Place at Q2's e pin + offset (0, +4)
RC1: Resistor place: Q1.c(0, -4)   // Place at Q1's c pin + offset (0, -4)
```

### mirror: — Mirror (Flip)

```aelm
Q2: PNP place: (12, 5) mirror: horizontal   // Flip left-right
Q3: NPN place: (19, 5) mirror: vertical      // Flip up-down
```

| Value | Description |
|-------|-------------|
| `horizontal` | Horizontal flip (negate X coordinates) |
| `vertical` | Vertical flip (negate Y coordinates) |

**Transform order**: Mirror THEN Rotate (same as KiCad). `mirror: horizontal rot: 90` mirrors first, then rotates 90 degrees.

### rot: — Rotation

```aelm
RE2: Resistor place: Q2.e(0, 4) rot: 90   // 90-degree rotation (vertical)
```

| Value | Description |
|-------|-------------|
| `0` | No rotation (default, horizontal) |
| `90` | 90 degrees clockwise (vertical) |
| `180` | 180 degrees (horizontal, reversed) |
| `270` | 270 degrees clockwise |

### anchor: — Anchor Pin

```aelm
RE2: Resistor place: Q2.e(0, 4) anchor: a   // Place using pin a as anchor
```

### lock: — Layout Lock Level

Controls how aggressively the layout engine is allowed to move this instance.
Lock state lives in the `.aelm` source because it is structural, not visual.

```aelm
R1: Resistor place: (5, 3) lock: layout_locked
```

| Value | Behaviour |
|-------|-----------|
| `unlocked` (default) | Force-directed layout and grid snap apply normally |
| `group_locked` | Relative positions within a group are preserved |
| `layout_locked` | Absolute position is preserved; routing may still move |
| `route_locked` | Routing segments are locked in addition to layout |
| `full_locked` | Completely immutable |

Omit the hint entirely for `unlocked`. `apply-lock <file> <instance> unlocked`
removes the hint instead of writing a redundant annotation.

### expand: — Multi-unit expansion toggle (#139)

Controls whether a multi-unit part (one with a `units {}` block) expands into per-unit symbols or renders as a single package-level symbol.

```aelm
U1: LM358                   // Default: render as a single package symbol
U2: LM358 expand: true      // Split into U2A + U2B + U2_PWR
U3: LM358 expand: false     // Explicit package form (same as default)
```

| Value | Behaviour |
|-------|-----------|
| omitted (None) | Render one package-level symbol (default) |
| `true` | Split into per-unit symbols |
| `false` | Render one package-level symbol (same as None) |

### nc: — No-Connect annotation (#186)

Marks pins as intentionally unconnected; suppresses DRC-E001 (unconnected pin) and DRC-W001 (floating bidirectional) for the marked pins.

```aelm
ANT1: AntennaMonopole place: (0, 0) nc: all          // All pins intentionally unconnected
U1: LM358 nc: [A_IN+, A_IN-]                         // Only these two pins are NC
```

| Syntax | Meaning |
|--------|---------|
| `nc: all` | All pins on this instance are intentionally unconnected |
| `nc: [pin1, pin2, ...]` | Only the listed pins are intentionally unconnected |
| omitted | No suppression; unconnected pins produce DRC-E001 |

**Use cases**: Gallery / showcase files where parts are displayed without wiring; IC inputs intentionally left floating.

### Per-unit placement block (#137)

Append `{ Suffix: hints ... }` to a multi-unit instance to override `place:` / `rot:` / `mirror:` / `lock:` per unit. Units not listed inherit the parent instance's hints.

```aelm
U1: LM358 {
    A:    place: (0, 0)
    B:    place: (0, 10) rot: 180
    _PWR: place: (10, 5)
}
```

### Coordinate System

- **Origin**: Top-left
- **X-axis**: Positive to the right
- **Y-axis**: Positive downward
- **1 grid unit**: 2.54mm (0.1 inch)
- **Coordinate values**: Integers (grid units)

---

## §5.5 Custom Symbols (`symbol {}` block)

If a part needs a body shape not covered by the 35 built-ins, declare it inline with a `symbol {}` block alongside `part` and `module`. The block lives at the top level of a `.alib` or `.aelm` file. Seven draw primitives map 1-to-1 with the internal `DrawCommand` enum: `line`, `rect`, `circle`, `arc`, `polyline`, `filled_polygon`, `text`.

```aelm
symbol MyZigResistor {
    pins {
        p1: (-0.5, 0) direction: left  pin_length: 0.5
        p2: ( 0.5, 0) direction: right pin_length: 0.5
    }
    draw {
        line (-0.5, 0) -> (0.5, 0)
        polyline (-0.5, 0) (-0.3, 0.2) (-0.1, -0.2) (0.1, 0.2) (0.3, -0.2) (0.5, 0)
    }
    label_anchor: (0, -0.8) anchor: center
}

part CustomResistor {
    pins { p1: passive @1  p2: passive @2 }
    symbol: MyZigResistor
}
```

**Pin convention (same as built-ins)**: `direction` is outward (away from the body), `position` is where the pin meets the body, and `pin_length` extends the stub. The wire endpoint `position + direction × pin_length` must land on an integer grid (`E_SYM_003`).

**Validation codes**: `E_SYM_002` duplicate position, `E_SYM_003` off-grid stub tip, `E_SYM_004` duplicate pin name, `E_SYM_005` empty draw block, `E_SYM_006` empty pins block, `E_SYM_007` built-in name collision, `E_SYM_008` duplicate symbol name; `W_SYM_001` pin outside bbox, `W_SYM_002` only one draw command.

See the public wiki page [Custom Symbol Authoring](https://github.com/alphaelements/aelm/wiki/en-Symbol-Authoring) for the full reference.

---

## §6 Standard Library Catalog

### passive.alib

| Part | Pins | Parameters | Symbol |
|------|------|-----------|--------|
| Resistor | p1(passive), p2(passive) | value: Ohm = 10k | resistor |
| Capacitor | p1(passive), p2(passive) | value: Farad = 100n | capacitor |
| Inductor | p1(passive), p2(passive) | value: Henry = 10u | inductor |

### semiconductor.alib

| Part | Pins | Symbol |
|------|------|--------|
| NPN | b(input), c(output), e(passive) | transistor_npn |
| PNP | b(input), c(output), e(passive) | transistor_pnp |
| NMOS | g(input), d(output), s(passive) | transistor_nmos |
| PMOS | g(input), d(output), s(passive) | transistor_pmos |
| Diode | a(passive), k(passive) | diode |
| LED | a(passive), k(passive) | led |
| Zener | a(passive), k(passive) | zener |

**Note**: stdlib Resistor/Capacitor/Inductor pin names are `p1`/`p2`. If you define them locally with `a`/`b`, be careful about pin name mismatches.

### power.alib

| Part | Pin | Symbol | Description |
|------|-----|--------|-------------|
| VCC | pin(passive) | power_flag | Power supply flag (T-bar) |
| VDD | pin(passive) | power_flag | Power supply flag (T-bar) |
| POWER | pin(power) | power_arrow | Power supply arrow |
| GND | pin(passive) | gnd_flag | Signal ground (three lines) |
| CGND | pin(passive) | gnd_triangle | Chassis ground (▽) |
| EGND | pin(passive) | gnd_earth | Earth ground (hatch lines) |
| PGND | pin(passive) | gnd_protective | Protective earth (circle + earth) |

**Power symbol labels**: The instance name is displayed as the net name label. Example: `VCC1: VCC` → displays "VCC1".

---

## §7 Transistor Pin Layout

### NPN/PNP (R0 — Default)

```
       C (output, upper-right)
       |
  B ───┤  (input, left)
       |
       E (passive, lower-right)
```

- B (Base): Left side, input
- C (Collector): Upper-right, output
- E (Emitter): Lower-right, passive

### NMOS/PMOS (R0 — Default)

```
       D (output, upper-right)
       |
  G ───┤  (input, left)
       |
       S (passive, lower-right)
```

- G (Gate): Left side, input
- D (Drain): Upper-right, output
- S (Source): Lower-right, passive

**Important**: Transistors/MOSFETs are R0-fixed (DR-013, DR-014). Do not specify `rot:` for these components.

---

## §8 Drafting Rules Checklist

Check these drafting rules when writing circuits (see [BD-Layout §12](../Specification/BD-Layout#12-製図規則-bd-lay-013-iec-61082-準拠) for details):

- [ ] Signal flow is left-to-right (DR-001)
- [ ] Power at top, GND at bottom (DR-002)
- [ ] Transistors/MOSFETs/op-amps/ICs are R0-fixed (DR-013~016)
- [ ] Resistors/capacitors use R0 (horizontal) or R90 (vertical) only (DR-010, 011)
- [ ] VCC/GND ports are defined
- [ ] All pins are connected (no floating pins)

---

## §8.5 DRC Rule Reference

All DRC rules detected by `aelm check`. Errors cause build failure (exit 1), Warnings are informational (exit 0).

### Errors (Circuit Errors)

| Code | Category | Description |
|------|----------|-------------|
| DRC-E001 | Unconnected | Pin is not connected to any net |
| DRC-E002 | Power/GND | Power pin not connected to power net |
| DRC-E003 | Power/GND | Ground pin not connected to GND net |
| DRC-E004 | Type mismatch | Output-to-output short circuit (same net) |
| DRC-E005 | Unconnected | Input pin has no driver (undriven input) |
| DRC-E006 | Unconnected | OpenCollector pin without pull-up resistor |

### Warnings (Quality Warnings)

| Code | Category | Description |
|------|----------|-------------|
| DRC-W001 | Unconnected | Floating bidirectional pin |
| DRC-W002 | Power/GND | Power net has no power source (Power pin) |
| DRC-W003 | Type mismatch | OpenCollector + Output combination (pull-up recommended) |
| DRC-W004 | Unconnected | NoConnect pin has connections |
| DRC-W005 | Naming | Similar labels differing only in case (port/net names) |
| DRC-W006 | Parameters | Missing required parameter value |
| DRC-E007 | Naming | Duplicate instance name within a module |

### DrcConfig Control

Individual rule groups can be disabled in `.aproj`:

```json
{
  "drc": {
    "check_unconnected": true,
    "check_power": true,
    "check_type_mismatch": true,
    "check_naming": true,
    "check_params": true
  }
}
```

### DRC Suppression Comments (`// drc-ignore:`)

Suppress specific DRC warnings/errors for individual instances.

#### Scope

- **Per-instance**: Suppression applies only to the instance declaration **immediately following** the comment
- No module-wide or file-wide suppression
- Each instance requires its own `// drc-ignore:` comment

#### Syntax

Two placement options:

```aelm
// Option 1: On the preceding line
// drc-ignore: DRC-E001
R1: Resistor

// Option 2: Inline comment on the same line
R1: Resistor  // drc-ignore: DRC-E001
```

- Space required between `//` and `drc-ignore:`
- Code names are case-insensitive (`drc-e001` is valid)
- Multiple codes separated by commas

#### Examples

```aelm
// Intentionally unconnected test point
// drc-ignore: DRC-E001
TP1: TestPoint

// Intentionally connected NoConnect pin (debug use)
// drc-ignore: DRC-W004
U1: IC

// Suppress multiple DRC codes at once
// drc-ignore: DRC-E001, DRC-E005
R_UNUSED: Resistor
```

#### Available Codes

All codes listed in §8.5 DRC Rule Reference (DRC-E001~E007, DRC-W001~W006) can be specified.

#### Notes

- The suppression comment must be on the line **immediately before** the instance declaration (blank lines in between make it ineffective)
- Suppression for connections in the `connections` block is not supported (instance-level only)
- Suppressed errors are not shown in CLI output

See [BD-DRC Specification](../Specification/BD-DRC) for full details.

---

## §9 Typical Circuit Patterns

### 9.1 Constant-Current Source

**Overview**: Constant-current source using 2 NPN transistors + 4 resistors. Ic = VBE(Q1) / RE2 ≈ 0.6V / RE2

```aelm
// Constant-current source circuit
// Ic = VBE(Q1) / RE2 ≈ 0.6V / RE2

part NPN {
    pins {
        b: input @1
        c: output @2
        e: passive @3
    }
    symbol: transistor_npn
}

part Resistor {
    pins {
        a: passive @1
        b: passive @2
    }
    symbol: resistor
}

module ConstantCurrentSource {
    ports {
        VCC: power inout
        GND: passive inout
    }
    instances {
        Q1: NPN place: (5, 8)
        Q2: NPN place: (14, 8)
        RC1: Resistor place: Q1.c(0, -4) rot: 90
        RC2: Resistor place: Q2.c(0, -4) rot: 90
        Rb: Resistor place: Q1.b(-4, 0) rot: 90
        RE2: Resistor place: Q2.e(0, 4) rot: 90
    }
    connections {
        VCC -> RC1.a
        RC1.b -> Q1.c
        VCC -> RC2.a
        RC2.b -> Q2.c
        Q1.c -> Q2.b
        Q1.b -> Rb.a
        Rb.b -> Q2.e
        Q1.e -> GND
        Q2.e -> RE2.a
        RE2.b -> GND
    }
}
```

**How it works**:
1. RC1, RC2 are collector pull-up resistors (connection to VCC)
2. Q1's collector output drives Q2's base
3. Rb is Q1's base bias resistor (Q1.b → 0.6V node)
4. RE2 is the emitter resistor (0.6V node → GND)
5. VBE(Q1) = 0.6V determines the 0.6V node potential
6. Constant current Ic = 0.6V / RE2

### 9.2 (To Be Added)

- LED driver circuit
- Differential pair
- Feedback amplifier
- CMOS inverter

---

## §10 Common Mistakes

### 10.1 Missing VCC/GND Ports

```aelm
// BAD: No ports defined
module Bad {
    instances { R1: Resistor }
    connections { ... }
}

// GOOD: Ports defined
module Good {
    ports {
        VCC: power inout
        GND: passive inout
    }
    instances { R1: Resistor }
    connections { R1.a -> VCC; R1.b -> GND }
}
```

### 10.2 Pin Name Mismatch

stdlib Resistor uses `p1`/`p2`, while local definitions may use `a`/`b`. Be careful not to mix them.

```aelm
// When using stdlib Resistor via use: p1, p2
use "stdlib/parts/passive.alib"
// → R1.p1, R1.p2

// When defining Resistor locally: a, b
part Resistor { pins { a: passive @1; b: passive @2 } ... }
// → R1.a, R1.b
```

### 10.3 Specifying rot: on Transistors

```aelm
// BAD: NPN is R0-fixed
Q1: NPN place: (5, 5) rot: 90

// GOOD: No rotation
Q1: NPN place: (5, 5)
```

### 10.4 Missing Components

It's easy to omit resistors in circuit descriptions. Carefully check the reference image and include all components.
Example: Missing collector pull-up resistors, base bias resistors.

### 10.5 Missing Connections

Ensure all pins are connected to something. `aelm check` will warn about unconnected pins.
