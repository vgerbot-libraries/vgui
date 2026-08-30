# Animation Example

## Live Demo

<iframe src="../wasm/animation/" width="100%" height="680" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

This example demonstrates vgui's animation and transition support:

- `animate-pulse` / `animate-bounce` / `animate-ping` keyframe animations.
- `transition-opacity` and `transition-colors` with `duration-*` / `ease-*` timing.
- Custom `animate={...}` attribute for user-defined keyframe animations.

## Source Code

```rust
{{#include ../../../examples/animation/src/main.rs}}
```

## Key Concepts

### Built-in animations

`animate-pulse`, `animate-bounce`, and `animate-ping` are Tailwind-compatible
classes parsed at compile time by `tw!`. Each maps to a gpui `Animation` with a
repeating loop and a custom animator closure.

### Transitions

`transition-opacity` / `transition-colors` animate between the base style and the
`hover:` style when the pointer enters or leaves the element. The `view!` macro
creates a hover signal, registers `on_hover`, and calls `apply_transition` to
interpolate the relevant properties.

### Custom `animate={...}`

The `animate` attribute accepts a closure `|el| -> AnimationElement<E>`. This
gives full access to gpui's `with_animation` API for custom keyframes, durations,
easings, and repeat behavior.

### Running

**Native:**

```bash
cargo run -p vgui-animation
```

**Web (WASM):**

```bash
# Build the WASM binary
cargo +nightly build --target wasm32-unknown-unknown -p vgui-animation --release

# Generate JS bindings
wasm-bindgen --target web --out-dir examples/animation/dist \
    --no-typescript target/wasm32-unknown-unknown/release/animation.wasm

# Serve and open in a browser
python3 scripts/serve_plain.py 8080 examples/animation
```
