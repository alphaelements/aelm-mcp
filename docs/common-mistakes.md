# Common Mistakes

> :book: [日本語版はこちら](ja-Common-Mistakes)

This guide covers the most frequent mistakes when writing AELM circuits and how to fix them. Each section includes the relevant DRC code, a bad example, and the corrected version.

## 1. Missing Power and Ground Connections

**Problem**: DRC-E002 / DRC-E003 -- Power and ground pins are not connected to power/ground nets. This is the most common mistake for beginners: placing active components without wiring their supply pins.

```
// BAD: Missing power connections
module Bad {
    instances {
        U1: OpAmp
    }
    connections {
        // No VCC/GND connected to U1.vcc and U1.vee!
    }
}

// GOOD: Power connected
module Good {
    instances {
        U1: OpAmp
        PWR: VCC
        GND1: GND
    }
    connections {
        PWR.pin -> U1.vcc
        U1.vee -> GND1.pin
    }
}
```

**Tip**: Always add VCC and GND instances first, then wire them to every power/ground pin before connecting signals.

## 2. Pin Name Mismatch

**Problem**: Using the wrong pin names causes parse errors or unconnected-pin DRC violations. The stdlib `Resistor` uses `p1` / `p2`, but a custom part definition might use `a` / `b`.

```
// BAD: Wrong pin name for stdlib Resistor
connections {
    U1.out -> R1.a       // Resistor has no pin named 'a'
}

// GOOD: Correct stdlib pin names
connections {
    U1.out -> R1.p1      // Resistor uses p1 and p2
}
```

**Fix**: Check the part definition in stdlib or your `.alib` file. In the VSCode editor, hover over an instance name to see its pin list via LSP.

## 3. Placement Hints Inside Parentheses

**Problem**: `rot:` and `place:` are instance-level hints, NOT parameters inside `()`. Putting them inside parentheses causes a parse error.

```
// BAD: rot inside params
R1: Resistor(value: 10k, rot: 90)

// GOOD: rot is a hint outside ()
R1: Resistor(value: 10k) rot: 90
```

**Rule**: Everything inside `()` is a parameter passed to the part. Everything after `()` is a layout hint for the instance.

## 4. Forgetting Pin Electrical Types

**Problem**: Declaring a pin with the wrong electrical type leads to DRC false positives or missed violations. For example, using `passive` for a power pin means DRC will not check power connectivity.

```
// BAD: Power pin declared as passive
part BadPower {
    pins {
        vcc: passive       // DRC won't enforce power net connection
    }
}

// GOOD: Correct electrical types
part GoodPower {
    pins {
        vcc: power         // DRC-E002 will fire if not on power net
        gnd: ground        // DRC-E003 will fire if not on ground net
        sig_in: input      // DRC-E005 will fire if undriven
        sig_out: output    // DRC-E004 will fire if shorted
    }
}
```

**Fix**: Use the correct type: `power` for VCC pins, `ground` for GND pins, `input` / `output` for signal pins, `passive` only for truly passive elements (resistor leads, etc.).

## 5. Floating Connections (Unconnected Pins)

**Problem**: DRC-E001 -- Pins that are not connected and not marked `no_connect`. This is especially common with multi-pin ICs where not all pins are used in the design.

```
// BAD: Unused pins left floating
module Bad {
    instances {
        U1: QuadNand   // 14-pin IC, only 3 gates used
    }
    connections {
        // Gate D pins (U1.d_in1, U1.d_in2, U1.d_out) not connected
    }
}

// GOOD: Unused pins explicitly marked
module Good {
    instances {
        U1: QuadNand
    }
    connections {
        // ... active gate connections ...
        // drc-ignore: DRC-E001
        // Mark unused gate pins as no_connect
    }
}
```

**Fix**: Either connect unused pins to appropriate nets or use a `drc-ignore` comment to acknowledge the floating pins intentionally.

## 6. Output-to-Output Short

**Problem**: DRC-E004 -- Two output pins connected to the same net. This creates an electrical conflict that can damage real hardware.

```
// BAD: Two outputs shorted
module Bad {
    instances {
        U1: Buffer
        U2: Buffer
    }
    connections {
        U1.out -> U2.out   // Two outputs on same net!
    }
}

// GOOD: Use tristate or separate nets
module Good {
    instances {
        U1: TriBuffer
        U2: TriBuffer
        MUX1: Mux2to1
    }
    connections {
        U1.out -> MUX1.a
        U2.out -> MUX1.b
    }
}
```

**Fix**: Use tristate or open_collector output types if multiple drivers are needed, or route outputs through a multiplexer.

## 7. Incorrect Rotation Direction

**Problem**: Component rotated the wrong way because the rotation direction is misunderstood.

```
// Rotation is counter-clockwise (CCW) from default orientation
R1: Resistor(value: 10k) rot: 90    // 90 degrees CCW
R2: Resistor(value: 10k) rot: 270   // 270 degrees CCW (= 90 CW)
```

**Key point**: `rot: 90` means 90 degrees counter-clockwise. If you want a clockwise rotation, use `rot: 270` (which is 90 degrees CW). Use the `R` key in the editor to preview rotation interactively before committing to a value.

## 8. Module Port Missing Side/Slot

**Problem**: Module ports declared without `side.slot` placement information lead to unpredictable pin positions when the module is used as an instance in another module.

```
// BAD: No placement info
ports {
    inp: passive in
    outp: passive out
}

// GOOD: Explicit placement
ports {
    inp: passive in left.1
    outp: passive out right.1
}
```

**Fix**: Always specify `side.slot` for module ports. Available sides are `left`, `right`, `top`, `bottom`. Slots are numbered starting from 1.

## 9. Case Sensitivity Issues

**Problem**: DRC-W005 -- Labels that differ only in letter case. AELM is case-sensitive, so `vcc` and `VCC` are different identifiers. This can cause subtle connection bugs.

```
// BAD: Inconsistent casing
module Bad {
    instances {
        vcc: VCC           // Instance named 'vcc' (lowercase)
    }
    connections {
        VCC.pin -> U1.vcc  // Refers to 'VCC' (uppercase) -- not the same!
    }
}

// GOOD: Consistent naming convention
module Good {
    instances {
        VCC1: VCC          // UPPER_SNAKE for instances
        GND1: GND
    }
    connections {
        VCC1.pin -> U1.vcc
        U1.vee -> GND1.pin
    }
}
```

**Convention**: Use PascalCase for part/module type names, UPPER_SNAKE_CASE for instance names, and snake_case or UPPER for pin names.

## See Also

- [DRC Rules](DRC-Rules) -- Full DRC rule reference and electrical type compatibility
- [DSL Overview](DSL-Overview) -- AELM DSL language overview
- [DSL Part Definition](DSL-Part-Definition) -- How to define parts and pin types
- [DSL Module Definition](DSL-Module-Definition) -- Module structure including ports and instances
- [DSL Placement Hints](DSL-Placement-Hints) -- Rotation, placement, and layout hints
- [DSL Connections](DSL-Connections) -- Connection syntax reference
- [Stdlib Parts](Stdlib-Parts) -- Standard library part list and pin names
