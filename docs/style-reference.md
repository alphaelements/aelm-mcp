# AI .astyle Stylesheet Reference

**Last updated**: 2026-03-25
**Japanese version**: [AI-Style-Reference](AI-Style-Reference)

---

## §1 Overview

`.astyle` files are CSS-like stylesheets that customize the visual appearance of circuit schematics.
They are independent of the circuit structure (`.aelm`) and control colors, stroke widths, opacity, and backgrounds.

### Loading Priority

1. CLI `--style` explicit path
2. Auto-discovered `.astyle` file with the same name as the `.aelm` file (e.g., `circuit.aelm` → `circuit.astyle`)
3. `stdlib/themes/default.astyle` (fallback)

The VSCode extension also auto-applies a co-located `.astyle` file.

---

## §2 Basic Syntax

```astyle
// Line comment
/* Block comment */

// Pseudo-element selector (lowercase keyword)
background { color: #fffacd; }

wire {
    color: #ff0000;
    width: 0.5mm;
}

// Universal selector
* {
    stroke_color: #000000;
    stroke_width: 0.25mm;
}

// Type selector (PascalCase required — matches part name)
Resistor {
    stroke_color: #0000ff;
    fill_color: #cce0ff;
}

// Id selector (matches instance name)
#R1 {
    stroke_color: #ff0000;
    opacity: 0.6;
}
```

### Rules

- Properties use `name: value;` syntax (semicolon required)
- Colors: `#RRGGBB` or `#RRGGBBAA` (8-digit for alpha)
- Lengths: `<number>mm` (e.g., `0.25mm`)
- Multiple properties per rule block `{ }`

---

## §3 Selectors

### Types and Specificity

| Priority | Selector | Syntax | Matches | Example |
|:--------:|----------|--------|---------|---------|
| 100 | Id | `#name` | Instance name | `#R1 { }` |
| 10 | Type | `TypeName` | Part name (PascalCase required) | `Resistor { }` |
| 10 | Pseudo | `keyword` | Schematic structural elements | `wire { }` |
| 1 | Universal | `*` | All components | `* { }` |

Same-priority rules: later definition wins.

### Type Selector Notes

- **Must start with an uppercase letter (PascalCase)**. Lowercase names are interpreted as pseudo-elements
- Matches the `part` definition name in `.aelm` (`Resistor`, `NPN`, etc.)
- Does NOT match symbol template names (`resistor`, `transistor_npn`)

---

## §4 Pseudo-Elements

| Pseudo-Element | Target | Available Properties |
|----------------|--------|---------------------|
| `background` | Canvas background | `color` |
| `grid` | Grid lines | `color`, `sub_color`, `visible` |
| `wire` | Wires | `color`, `width` |
| `bus` | Bus wires | `color`, `width` |
| `text` | Text labels | `font_color`, `font_size` |
| `pin` | Pin stubs | `stroke_color`, `stroke_width` |
| `net_label` | Net labels | `font_color`, `font_size` |
| `error` | DRC error overlay | `color`, `font_size` |

---

## §5 Property Reference

### Component Properties (`*`, Type, Id selectors)

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `stroke_color` | `#RRGGBB` | `#000000` | Stroke color |
| `stroke_width` | `<n>mm` | `0.25mm` | Stroke width |
| `fill_color` | `#RRGGBBAA` | `#00000000` | Fill color (transparent by default) |
| `opacity` | `0.0-1.0` | `1.0` | Opacity (applied to alpha channel) |

### Text Properties (`text` pseudo-element or component)

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `font_size` | `<n>mm` | `1.27mm` | Font size |
| `font_color` | `#RRGGBB` | `#000000` | Text color |

Label color cascade: component style > `text` pseudo-element > default black.

### Wire Properties

| Pseudo-Element | `color` Default | `width` Default |
|----------------|-----------------|-----------------|
| `wire` | `#006400` (dark green) | `0.25mm` |
| `bus` | `#00008B` (dark blue) | `0.5mm` |

### Grid Properties

| Property | Default | Description |
|----------|---------|-------------|
| `color` | `#DCDCDC` | Main grid line color |
| `sub_color` | `#F0F0F0` | Subdivision grid color |
| `visible` | `true` | Set `false` to hide grid |

---

## §6 Value Types

| Type | Syntax | Examples |
|------|--------|----------|
| Color | `#RRGGBB` / `#RRGGBBAA` | `#ff0000`, `#00000080` |
| Length | `<number>mm` | `0.25mm`, `1.5mm` |
| Boolean | `true` / `false` | `true` |
| Number | integer or float | `90`, `0.5` |
| Tuple | `(x, y)` | `(5.08, 7.62)` |
| String | `"text"` | `"sans-serif"` |

---

## §7 Cascade (Resolution Order)

Styles are resolved in this order (later overrides earlier):

```
Default values (hardcoded)
    ↓ overridden by
Universal selector (*) properties
    ↓ overridden by
Type selector (Resistor) properties
    ↓ overridden by
Id selector (#R1) properties
```

Example: With `Resistor { stroke_color: #0000ff; }` and `#R1 { stroke_color: #ff0000; }`:
- R1 renders red (Id selector wins)
- R2 renders blue (Type selector applies)

---

## §8 Complete Example

```astyle
// Dark theme style

background { color: #1e1e1e; }

grid {
    color: #333333;
    sub_color: #2a2a2a;
    visible: true;
}

* {
    stroke_color: #cccccc;
    stroke_width: 0.25mm;
}

text {
    font_size: 1.27mm;
    font_color: #e0e0e0;
}

wire {
    color: #00cc66;
    width: 0.3mm;
}

bus {
    color: #6699ff;
    width: 0.5mm;
}

pin {
    stroke_color: #999999;
    stroke_width: 0.15mm;
}

error {
    color: #ff4444;
    font_size: 1.0mm;
}

// Blue resistors, green capacitors
Resistor {
    stroke_color: #4488ff;
    fill_color: #1a2a4a;
}

Capacitor {
    stroke_color: #44cc44;
}

NPN {
    stroke_color: #cc88ff;
    stroke_width: 0.35mm;
}

// Highlight a specific instance
#R1 {
    stroke_color: #ff8800;
    stroke_width: 0.5mm;
    opacity: 0.9;
}
```

---

## §9 Default Theme

Contents of `stdlib/themes/default.astyle`:

```astyle
background { color: #ffffff; }
grid { color: #dcdcdc; sub_color: #f0f0f0; visible: true; }
* { stroke_color: #000000; stroke_width: 0.25mm; fill_color: #00000000; opacity: 1.0; }
text { font_size: 1.27mm; font_color: #000000; }
wire { color: #006400; width: 0.25mm; }
bus { color: #00008b; width: 0.5mm; }
pin { stroke_color: #000000; stroke_width: 0.15mm; }
error { color: #ff4444; font_size: 1.0mm; }
```

---

## §10 CLI Usage

```bash
# Export PNG with custom style
aelm export-png circuit.aelm --style dark-theme.astyle -o output.png --grid

# Auto-discover co-located .astyle (uses circuit.astyle if it exists)
aelm export-svg circuit.aelm -o output.svg

# Default theme only (uses stdlib/themes/default.astyle if no .astyle found)
aelm export-png circuit.aelm -o output.png
```
