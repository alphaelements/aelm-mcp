# DRC Rules

> :book: [日本語版はこちら](ja-DRC-Rules)

The Design Rule Check (DRC) system automatically validates circuit correctness in AELM. DRC runs on parse and on every save, providing immediate feedback through inline diagnostics in the VSCode editor. Rules are divided into **errors** (must fix) and **warnings** (advisory).

## Overview

DRC analyzes the netlist produced by the parser and checks for electrical violations, naming conflicts, and missing configurations. Each rule has a unique code (e.g., `DRC-E001`) that appears in the diagnostics panel, making it easy to look up the cause and resolution.

- **Error rules (E-codes)** block the circuit from being considered valid. They must be resolved before rendering or export.
- **Warning rules (W-codes)** flag potential issues that may be intentional. They do not block rendering but should be reviewed.

## Error Rules (E-codes)

Error rules indicate definite problems that must be fixed.

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| DRC-E001 | Warning | Unconnected pin | Pin not connected and not suppressed with `nc:` |
| DRC-E002 | Error | Supply pin not connected to any supply net | Supply pin (power or ground type) not connected to any power or ground net |
| DRC-E004 | Error | Output-to-output conflict | Two output pins connected to the same net |
| DRC-E005 | Error | Undriven input | Input pin not driven by any output |
| DRC-E006 | Error | Open collector without pull-up | Open collector pin has no pull-up resistor |
| DRC-E007 | Error | Duplicate instance name | Same instance name used twice in a module |

## Warning Rules (W-codes)

Warning rules flag conditions that may be intentional but deserve review.

| Code | Severity | Description | Trigger |
|------|----------|-------------|---------|
| DRC-W001 | Warning | Floating bidirectional pin | Bidirectional pin connected but not driven |
| DRC-W002 | Warning | Supply net has no supply source | Supply net (power or ground typed) declared but no supply-type pin connected |
| DRC-W003 | Warning | Open collector advisory | Open collector without explicit pull-up configuration |
| DRC-W004 | Warning | NoConnect pin has connections | Pin marked `no_connect` but has wires |
| DRC-W005 | Warning | Case-similar labels | Labels differ only in case (e.g., `vcc` vs `VCC`) |
| DRC-W006 | Warning | Missing parameter value | Required parameter has no value and no default |

## DRC Suppression

Specific DRC rules can be suppressed on a per-instance basis using a `drc-ignore` comment. Place the comment immediately before the instance or connection that triggers the diagnostic:

```
// Suppress specific rules on this instance:
// drc-ignore: DRC-E001, DRC-W005
```

**Important**: Suppression is per-instance, not global. Each suppression comment applies only to the next statement. This ensures that violations are intentionally acknowledged rather than silently hidden across the entire design.

## DRC Configuration

DRC behavior is configured in the `.aproj` project file. Each rule category can be enabled or disabled independently:

```
drc {
    check_unconnected: true    // DRC-E001
    check_power: true          // DRC-E002, W002
    check_type_mismatch: true  // DRC-E004, E005, E006
    check_naming: true         // DRC-E007, DRC-W005
    check_params: true         // DRC-W006
}
```

All checks are enabled by default. Disabling a category suppresses all rules within that category.

## Examples

### DRC-E001: Unconnected Pin

**Violation** -- Pin `p2` of `R1` is not connected and not suppressed:

```
// BAD: pin p2 is unconnected
module Bad {
    instances {
        R1: Resistor(value: 10k)
    }
    connections {
        // Only p1 is connected, p2 floats
    }
}
```

**Fix (option 1)** -- Connect the pin:

```
// GOOD: all pins connected
module Good {
    instances {
        R1: Resistor(value: 10k)
        R2: Resistor(value: 4.7k)
    }
    connections {
        R1.p2 -> R2.p1
    }
}
```

**Fix (option 2)** -- Suppress with the `nc:` hint when a pin is intentionally left unconnected:

```
// GOOD: nc: suppresses DRC-E001 for specific pins
module Good {
    instances {
        U1: MCU nc: [gpio5, gpio6]   // these two pins intentionally unused
        ANT1: Antenna nc: all        // all pins suppressed (e.g. pure mechanical part)
    }
    connections {
        // U1.gpio5 and U1.gpio6 do not need to be wired
    }
}
```

`nc: all` suppresses the warning for every pin on the instance. `nc: [pin1, pin2]` targets individual pins by name. The LSP provides pin-name completions inside the brackets.

### DRC-E004: Output-to-Output Conflict

**Violation** -- Two output pins are shorted together on the same net:

```
// BAD: two outputs on same net
module Bad {
    instances {
        U1: Buffer
        U2: Buffer
    }
    connections {
        U1.out -> U2.out   // Two outputs shorted!
    }
}
```

**Fix** -- Route outputs to separate nets, or use a multiplexer:

```
// GOOD: outputs on separate nets
module Good {
    instances {
        U1: Buffer
        U2: Buffer
        MUX1: Mux2to1
    }
    connections {
        U1.out -> MUX1.a
        U2.out -> MUX1.b
    }
}
```

### DRC-E005: Undriven Input

**Violation** -- An input pin has no driving output:

```
// BAD: input not driven
module Bad {
    instances {
        U1: Buffer
    }
    connections {
        // U1.in has no driver
        U1.out -> net_out
    }
}
```

**Fix** -- Connect the input to a driving source:

```
// GOOD: input driven by module port
module Good {
    ports {
        sig_in: input
    }
    instances {
        U1: Buffer
    }
    connections {
        sig_in -> U1.in
        U1.out -> net_out
    }
}
```

### DRC-E006: Open Collector Without Pull-Up

**Violation** -- An open collector output has no pull-up resistor on the net:

```
// BAD: open collector without pull-up
module Bad {
    instances {
        U1: Comparator  // output is open collector
    }
    connections {
        U1.out -> net_signal
    }
}
```

**Fix** -- Add a pull-up resistor to the net:

```
// GOOD: pull-up resistor on open collector output
module Good {
    instances {
        U1: Comparator
        R_PU: Resistor(value: 10k)
    }
    connections {
        U1.out -> R_PU.p1
        R_PU.p2 -> VCC
    }
}
```

## Electrical Type Compatibility

The following table shows which pin types can legally connect to each other. A check mark indicates a valid connection; an X indicates a DRC violation.

| | Input | Output | Bidirectional | Passive | Power | Ground | Open Collector | Open Emitter |
|---|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| **Input** | -- | OK | OK | OK | -- | -- | OK | OK |
| **Output** | OK | X (E004) | OK | OK | -- | -- | X (E004) | X (E004) |
| **Bidirectional** | OK | OK | OK | OK | -- | -- | OK | OK |
| **Passive** | OK | OK | OK | OK | OK | OK | OK | OK |
| **Power** | -- | -- | -- | OK | OK | OK | -- | -- |
| **Ground** | -- | -- | -- | OK | OK | OK | -- | -- |
| **Open Collector** | OK | X (E004) | OK | OK | -- | -- | OK | -- |
| **Open Emitter** | OK | X (E004) | OK | OK | -- | -- | -- | OK |

- `OK` = valid connection
- `X (code)` = DRC error with the indicated code
- `--` = not applicable or requires explicit net type

> **Power and Ground are both supply types.** A `power`-typed pin may connect to a `ground`-typed net and vice versa without triggering DRC-E002. The `power` vs `ground` distinction is retained only for rendering (glyph selection and bus-rail placement). See [Supply Pin Model](#supply-pin-model) below.

## Supply Pin Model

Pins declared with `power` or `ground` type in a `.alib` part definition are collectively called **supply pins**. At the DRC level, both types are treated identically:

- A supply pin is valid if it connects to **any** supply net, regardless of polarity (`power`-typed or `ground`-typed net both count).
- Example: an op-amp `vcc: power @4` pin connected to a `GND` labeled net does not trigger DRC-E002. Both `VCC` and `GND` are supply nets.
- The `power` vs `ground` distinction is retained in the IR solely for **rendering**: `power` pins draw with the VCC-flag glyph and place on the top bus rail; `ground` pins draw with the GND-flag glyph and place on the bottom rail.
- DRC-E002 fires only when a supply pin has **no connection** to any supply net at all.

This design allows flexible supply schemes (e.g., split-rail op-amps where V− connects to GND) without forcing artificial polarity constraints in the DRC.

---

## See Also

- [DSL Connections](DSL-Connections) -- Connection syntax reference
- [DSL Module Definition](DSL-Module-Definition) -- Module structure including ports and instances
- [DSL Part Definition](DSL-Part-Definition) -- Part definitions including pin types
