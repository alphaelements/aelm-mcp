> 📖 [日本語版はこちら](ja-Circuit-Patterns)

# Circuit Patterns

## 1. Overview

This guide presents common electronic circuit patterns as complete, runnable AELM DSL code. Each pattern includes part definitions, module wiring, and the key design equations so you can copy, adapt, and learn.

All examples are self-contained -- paste any block into a `.aelm` file and open it in the AELM editor to see the rendered schematic.

---

## 2. Voltage Divider

A voltage divider produces an output voltage that is a fraction of the input. Two resistors in series divide the supply voltage proportionally.

**Key equation:** `Vout = VCC * R2 / (R1 + R2)`

With equal resistors (R1 = R2 = 10k), the output is exactly half the supply voltage.

```aelm
part Resistor {
    params { value: Ohm = 10k }
    pins { p1: passive @1  p2: passive @2 }
    symbol: resistor
}

part VCC {
    pins { pin: power @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module VoltageDivider {
    instances {
        R1: Resistor(value: 10k) place: (5, 3)
        R2: Resistor(value: 10k) place: (5, 7) rot: 90
        PWR: VCC place: (5, 1)
        GND1: GND place: (5, 11)
    }
    connections {
        PWR.pin -> R1.p1
        R1.p2 -> R2.p1
        R2.p2 -> GND1.pin
    }
}
```

![en-Circuit-Patterns example 0](images/en-Circuit-Patterns_0.svg)

The output voltage is taken at the junction between R1.p2 and R2.p1. Changing the ratio of R1 to R2 changes the output voltage -- for example, R1 = 30k and R2 = 10k gives Vout = VCC / 4.

---

## 3. LED Driver with Current-Limiting Resistor

The simplest practical circuit: a resistor limits the current through an LED to prevent it from burning out.

**Key equation:** `R = (VCC - Vf) / If`

For a typical red LED (Vf = 2.0V, If = 20mA) on a 5V supply: R = (5 - 2) / 0.02 = 150 ohm.

```aelm
part Resistor {
    params { value: Ohm = 150 }
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part LED {
    pins { anode: passive @1  cathode: passive @2 }
    symbol: led
}

part VCC {
    pins { pin: power @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module LedDriver {
    instances {
        R1: Resistor(value: 150) place: (5, 3) rot: 90
        D1: LED place: (5, 7) rot: 90
        PWR: VCC place: (5, 1)
        GND1: GND place: (5, 11)
    }
    connections {
        PWR.pin -> R1.a
        R1.b -> D1.anode
        D1.cathode -> GND1.pin
    }
}
```

![en-Circuit-Patterns example 1](images/en-Circuit-Patterns_1.svg)

Current flows from VCC through R1 (which limits the current), through the LED (anode to cathode), and into ground. Always place the resistor before the LED -- the order does not affect the current but it is conventional to show it on the high side.

---

## 4. Common-Emitter Amplifier

The common-emitter configuration is the most widely used transistor amplifier topology. A base bias resistor sets the operating point, a collector load resistor converts current changes to voltage, and an emitter resistor provides thermal stability.

**Key equations:**
- `Ic ~ (VCC - Vbe) / R_BIAS * hFE` (approximate, depends on bias method)
- `Av ~ -R_LOAD / R_E` (voltage gain with emitter degeneration)

```aelm
part Resistor {
    params { value: Ohm = 10k }
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part NPN {
    pins {
        b: input @1
        c: output @2
        e: passive @3
    }
    symbol: transistor_npn
}

part VCC {
    pins { pin: passive @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module CommonEmitter {
    instances {
        Q1: NPN place: (8, 8)
        R_BIAS: Resistor(value: 100k) place: Q1.b(-3, 0)
        R_LOAD: Resistor(value: 4.7k) place: Q1.c(0, -3) rot: 90
        R_E: Resistor(value: 1k) place: Q1.e(0, 3) rot: 90
        PWR: VCC place: (8, 2)
        GND1: GND place: (8, 14)
    }
    connections {
        // Bias network
        PWR.pin -> R_BIAS.a
        R_BIAS.b -> Q1.b
        // Collector load
        PWR.pin -> R_LOAD.a
        R_LOAD.b -> Q1.c
        // Emitter degeneration
        Q1.e -> R_E.a
        R_E.b -> GND1.pin
    }
}
```

![en-Circuit-Patterns example 2](images/en-Circuit-Patterns_2.svg)

The input signal is applied to R_BIAS.a (AC-coupled in practice). The amplified, inverted output appears at the Q1.c node. R_E provides negative feedback that stabilizes the gain at the cost of reduced amplification.

---

## 5. Constant-Current Source (Current Mirror)

A current mirror uses two matched transistors to copy a reference current. The reference current is set by a resistor on one side, and the mirror transistor reproduces it on the other side regardless of load changes.

**Key equation:** `Ic = Vbe(Q1) / R_E ~ 0.6V / R_E`

```aelm
part NPN {
    pins {
        b: input @1
        c: output @2
        e: passive @3
    }
    symbol: transistor_npn
}

part Resistor {
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part VCC {
    pins { pin: passive @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module CurrentMirror {
    instances {
        Q1: NPN place: (5, 8)
        Q2: NPN place: (14, 8)
        RC1: Resistor place: Q1.c(0, -4) rot: 90
        RC2: Resistor place: Q2.c(0, -4) rot: 90
        RE: Resistor place: Q2.e(0, 4) rot: 180
        VCC1: VCC place: (6, 1)
        VCC2: VCC place: (15, 1)
        GND1: GND place: (6, 11)
        GND2: GND place: (15, 15)
    }
    connections {
        // VCC to collector resistors
        VCC1.pin -> RC1.a
        RC1.b -> Q1.c
        VCC2.pin -> RC2.a
        RC2.b -> Q2.c
        // Mirror connection: Q1 collector drives Q2 base
        Q1.c -> Q2.b
        // Base bias
        Q1.b -> RE.a
        RE.b -> Q2.e
        // Emitter to ground
        Q1.e -> GND1.pin
        Q2.e -> RE.a
        RE.b -> GND2.pin
    }
}
```

![en-Circuit-Patterns example 3](images/en-Circuit-Patterns_3.svg)

Q1 is diode-connected (collector tied to Q2's base). The current through Q1 sets the reference, and Q2 mirrors that current to its collector. The emitter resistor RE improves matching accuracy.

---

## 6. RC Low-Pass Filter

The simplest analog filter: a resistor and capacitor in series. High-frequency signals are attenuated while low-frequency signals pass through.

**Key equation:** `fc = 1 / (2 * pi * R * C)`

For R = 4.7k and C = 100nF: fc = 1 / (2 * 3.14159 * 4700 * 0.0000001) ~ 339 Hz.

```aelm
part Resistor {
    params { value: Ohm = 4.7k }
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part Capacitor {
    params { value: Farad = 100n }
    pins { a: passive @1  b: passive @2 }
    symbol: capacitor
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module RcLowPass {
    ports {
        inp: input in left.1
        outp: output out right.1
        gnd: passive in bottom.1
    }
    instances {
        R1: Resistor(value: 4.7k) place: (3, 5)
        C1: Capacitor(value: 100n) place: (8, 8) rot: 90
        GND1: GND place: (8, 12)
    }
    connections {
        inp -> R1.a
        R1.b -> C1.a
        R1.b -> outp
        C1.b -> GND1.pin
    }
}
```

![en-Circuit-Patterns example 4](images/en-Circuit-Patterns_4.svg)

The input signal enters through R1. At the junction of R1.b and C1.a, frequencies below the cutoff pass to the output while higher frequencies are shunted to ground through the capacitor. This module uses ports so it can be reused as a sub-circuit in larger designs.

---

## 7. Op-Amp Inverting Amplifier

An operational amplifier with input and feedback resistors forms a precision inverting amplifier. The gain is set entirely by the resistor ratio, independent of the op-amp's own gain.

**Key equation:** `Gain = -Rf / Rin`

With Rin = 10k and Rf = 100k, the voltage gain is -10 (inverted, 10x amplification).

```aelm
part Resistor {
    params { value: Ohm = 10k }
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part OpAmp {
    pins {
        INP: input @1
        INM: input @2
        OUT: output @3
        VCC: power @4
        VEE: power @5
    }
    symbol: op_amp
}

part VCC {
    pins { pin: power @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

module InvertingAmplifier {
    ports {
        vin: input in left.1
        vout: output out right.1
        vcc: power in top.1
        gnd: passive in bottom.1
    }
    instances {
        Rin: Resistor(value: 10k) place: (3, 8)
        Rf: Resistor(value: 100k) place: (8, 5)
        U1: OpAmp place: (8, 8)
        PWR: VCC place: (8, 2)
        GND1: GND place: (8, 14)
    }
    connections {
        // Input through Rin to inverting input
        vin -> Rin.a
        Rin.b -> U1.INM
        // Feedback from output back to inverting input
        U1.OUT -> Rf.b
        Rf.a -> U1.INM
        // Non-inverting input to ground (virtual ground)
        GND1.pin -> U1.INP
        // Power supply
        PWR.pin -> U1.VCC
        GND1.pin -> U1.VEE
        // Output
        U1.OUT -> vout
    }
}
```

![en-Circuit-Patterns example 5](images/en-Circuit-Patterns_5.svg)

The inverting input (INM) is a virtual ground -- the feedback loop forces it to the same voltage as the non-inverting input (INP), which is tied to ground. The input current Vin/Rin must equal the feedback current Vout/Rf, giving the gain formula.

---

## 8. Hierarchical Design -- Reusable Sub-Modules

AELM supports hierarchical design: define a module with ports, then instantiate it in a parent module like any other component. This lets you build complex systems from tested, reusable blocks.

```aelm
part Resistor {
    params { value: Ohm = 10k }
    pins { a: passive @1  b: passive @2 }
    symbol: resistor
}

part NPN {
    pins {
        b: input @1
        c: output @2
        e: passive @3
    }
    symbol: transistor_npn
}

part VCC {
    pins { pin: passive @1 }
    symbol: power_flag
}

part GND {
    pins { pin: passive @1 }
    symbol: gnd_flag
}

// Reusable amplifier stage with defined ports
module Amplifier {
    ports {
        inp: passive in left.1
        out: passive out right.1
        vcc: power in top.1
        gnd: power in bottom.1
    }
    instances {
        Q1: NPN place: (5, 5)
        R_BIAS: Resistor place: Q1.b(-2, 0)
        R_LOAD: Resistor place: Q1.c(0, -2) rot: 90
    }
    connections {
        inp -> R_BIAS.a
        R_BIAS.b -> Q1.b
        vcc -> R_LOAD.a
        R_LOAD.b -> Q1.c
        Q1.c -> out
        Q1.e -> gnd
    }
}

// Top-level design using two Amplifier instances
module MainBoard {
    instances {
        AMP1: Amplifier place: (5, 8)
        AMP2: Amplifier place: (15, 8)
        R_COUPLING: Resistor place: (10, 8)
        PWR: VCC place: (5, 2)
        GND1: GND place: (15, 14)
    }
    connections {
        // Power distribution
        PWR.pin -> AMP1.vcc
        PWR.pin -> AMP2.vcc
        // Signal path: AMP1 -> coupling resistor -> AMP2
        AMP1.out -> R_COUPLING.a
        R_COUPLING.b -> AMP2.inp
        // Ground
        AMP1.gnd -> GND1.pin
        AMP2.gnd -> GND1.pin
    }
}
```

![en-Circuit-Patterns example 6](images/en-Circuit-Patterns_6.svg)

The `Amplifier` module declares four ports (`inp`, `out`, `vcc`, `gnd`) that become pins when the module is instantiated. The `MainBoard` module creates two amplifier instances and wires them in cascade through a coupling resistor. Each `Amplifier` is a self-contained unit -- you can test, modify, or replace it independently.

**Design tips for hierarchical circuits:**

- Define `ports` with direction (`in`, `out`, `inout`) and side placement (`left.1`, `right.1`, `top.1`, `bottom.1`)
- Use meaningful port names that describe the signal, not the internal implementation
- Keep sub-modules focused on a single function for maximum reusability

---

## 9. Block Diagram -- System-Level Overview

A block diagram sketches an entire system at the subsystem level. Each node is a `block(shape: ..., label: ...)` instance, connections happen through bundle ports or `flow { }` edges. The `stereotype: block_diagram` hint switches the renderer to the block-lane defaults.

This pattern is the canonical starting point when a full circuit is still being decomposed -- see also the runnable sample at `examples/block_symbol_gallery.aelm`.

```aelm
bundle I2C {
    signals {
        sda: bidirectional
        scl: output
    }
    style { bundle_marker: slash }
}

module SensorHub {
    stereotype: block_diagram

    instances {
        MCU:    block(shape: rect,     label: "MCU") {
            ports {
                i2c_out: I2C out
            }
        }
        TEMP:   block(shape: sensor,   label: "Temp Sensor") {
            ports {
                i2c_in: I2C in
            }
        }
        CLOUD:  block(shape: cloud,    label: "Cloud API")
        RADIO:  block(shape: antenna,  label: "LTE") {
            ports {
                feed: passive @1
            }
        }
        STORE:  block(shape: database, label: "History DB")
    }

    connections {
        MCU.i2c_out == TEMP.i2c_in
    }
}
```

The `MCU` and `TEMP` blocks publish bundle ports, letting you keep the diagram abstract while still conveying that the two chips speak I2C. The `CLOUD`, `RADIO`, and `STORE` blocks remain pin-less placeholders -- they document intent without forcing every detail today.

---

## 10. Flow Chart -- Power-On Sequence

Flow charts model *behaviour*, not circuits. Each step is a `flow_node(shape: ..., label: ...)` node; edges live inside a dedicated `flow { }` block and accept labels such as `->{yes}` / `->{no}`.

This pattern matches `examples/flow-chart-power-on.aelm` in the repository.

```aelm
module PowerOnFlow {
    stereotype: flow_chart

    instances {
        START:    flow_node(shape: start,      label: "Power On")
        SELFTEST: flow_node(shape: process,    label: "Self-Test")
        OK:       flow_node(shape: decision,   label: "Pass?")
        LOAD:     flow_node(shape: subprocess, label: "Load Firmware")
        LOGERR:   flow_node(shape: document,   label: "Log Error")
        WAIT:     flow_node(shape: delay,      label: "Wait 5s")
        RUN:      flow_node(shape: io,         label: "Run User App")
        DONE:     flow_node(shape: end,        label: "Done")
    }

    flow {
        START -> SELFTEST -> OK
        OK ->{yes} LOAD -> RUN -> DONE
        OK ->{no}  LOGERR -> WAIT -> SELFTEST
    }
}
```

The `OK` decision branches with labelled arrows. `WAIT` loops back to `SELFTEST`, so the retry loop is explicit rather than hidden in pseudocode.

---

## 11. Mixed Hierarchy -- Circuit + Block + Flow

AELM lets you put real electrical parts, abstract blocks, and flow-chart nodes in the same module. The layout engine assigns each instance to its appropriate lane (electrical / block / flow). Source: `examples/mixed-hierarchy.aelm`.

```aelm
part Resistor {
    pins {
        a: passive @1
        b: passive @2
    }
    symbol: resistor
}

module MixedHierarchy {
    stereotype: block_diagram

    instances {
        // Classic circuit part
        R1: Resistor

        // Block-diagram abstract shapes
        MCU:  block(shape: rect,         label: "MCU")
        MEM:  block(shape: rounded_rect, label: "Memory")
        BUS:  block(shape: hexagon,      label: "Bus")

        // Flow-chart nodes
        BOOT: flow_node(shape: start,   label: "Boot")
        INIT: flow_node(shape: process, label: "Init Peripherals")
        IDLE: flow_node(shape: end,     label: "Idle Loop")
    }

    connections {
        R1.a -> R1.b          // electrical only
    }

    flow {
        BOOT -> INIT -> IDLE  // flow only
    }
}
```

Use this pattern when documenting firmware-level behaviour alongside the hardware that runs it, or when migrating a legacy block diagram into a real schematic one subsystem at a time.

---

## See Also

- [DSL Overview](DSL-Overview) -- Complete language reference
- [DSL Part Definition](DSL-Part-Definition) -- How to define custom parts
- [DSL Module Definition](DSL-Module-Definition) -- Module syntax and ports
- [DSL Connections](DSL-Connections) -- Connection syntax and net labels
- [DSL Block & Flow Diagrams](DSL-Block-Diagram) -- `block()` / `flow_node()` / `stereotype:`
- [DSL Port Bundles](DSL-Port-Bundle) -- `bundle { }` and the `==` operator
- [DSL Placement Hints](DSL-Placement-Hints) -- `place:` and `rot:` syntax
- [Stdlib Parts](Stdlib-Parts) -- Standard library components
- [Getting Started](Getting-Started) -- First steps with AELM
