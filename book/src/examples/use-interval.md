# useInterval Example

## Live Demo

<iframe src="../wasm/use-interval/" width="100%" height="480" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
{{#include ../../examples/use-interval/src/main.rs}}
```

## Key Concepts

### `set_interval` — low-level repeating timer

Analogous to JavaScript's `setInterval`. Spawns a gpui foreground task that
calls the callback every `duration`. The callback receives `&mut gpui::AsyncApp`
so it can update signals and entities across await points. The returned
`IntervalHandle` owns the underlying task; dropping it cancels the interval
immediately (equivalent to `clearInterval`).

### `use_interval` — reactive hook

Combines `create_effect` + `on_cleanup` to provide a declarative interval that
responds to reactive state:

- Takes a `ReadSignal<u64>` delay in milliseconds. When the value is `0` the
  interval is paused.
- When the delay changes, the effect re-runs: the previous interval is
  cancelled and a new one starts with the updated period.
- When the enclosing scope is disposed (e.g. a `<Switch>` branch goes
  inactive), `on_cleanup` cancels the interval automatically.

### Pattern: pausing via memo

A common pattern is to derive the effective delay from multiple signals —
e.g. a `running` boolean and a user-selected delay — using `create_memo`:

```rust
let effective_delay = create_memo(move || {
    if running.get() { delay.get() } else { 0 }
});
use_interval(move |cx| set_count.update(cx, |n| *n += 1), effective_delay);
```

Setting `running` to false feeds `0` into `use_interval`, which pauses the
interval without dropping the hook.

## Running

### Native

    cargo run -p vgui-use-interval

### Web (WASM)

    cargo build --target wasm32-unknown-unknown -p vgui-use-interval --release
    wasm-bindgen --target web --out-dir examples/use-interval/dist \
        --no-typescript target/wasm32-unknown-unknown/release/use-interval.wasm
    python3 scripts/serve_plain.py 8080 examples/use-interval
