# Canvas Animation Example

## Live Demo

<iframe src="../wasm/canvas-animation/" width="100%" height="640" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
{{#include ../../examples/canvas-animation/src/main.rs}}
```

## Key Concepts

### Combining `use_interval` with `<canvas>`

The animation loop is driven by `use_interval`, which ticks a reactive timer
and updates a `Vec<Particle>` signal on every tick. The `<canvas>` element's
`paint` closure reads the particle snapshot during render, so each signal
update triggers a re-render and a fresh canvas paint.

### Reactive speed control

The interval delay is derived from a `speed` signal via `create_memo`:

```rust
let delay = create_memo(move || {
    if running.get() { 50 / speed.get().max(1) } else { 0 }
});
use_interval(move |cx| set_particles.update(cx, |ps| { /* ... */ }), delay);
```

Changing the speed or pausing re-runs the effect inside `use_interval`,
cancelling the old timer and starting a new one. Setting delay to `0` pauses
the animation entirely.

### Particle physics

Each tick updates particle positions and bounces them off the canvas
boundaries. The `paint` closure also draws constellation lines between
nearby particles, with opacity proportional to distance.

## Running

### Native

    cargo run -p vgui-canvas-animation

### Web (WASM)

    cargo build --target wasm32-unknown-unknown -p vgui-canvas-animation --release
    wasm-bindgen --target web --out-dir examples/canvas-animation/dist \
        --no-typescript target/wasm32-unknown-unknown/release/canvas-animation.wasm
    python3 scripts/serve_plain.py 8080 examples/canvas-animation
