# Keyboard Shortcuts Example

## Live Demo

<iframe src="../wasm/keyboard-shortcuts/" width="100%" height="520" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
{{#include ../../../examples/keyboard-shortcuts/src/main.rs}}
```

## Key Concepts

### `use_shortcuts` — declarative shortcut engine

vgui integrates the `shortcuts` crate to provide configurable keyboard
shortcuts with human-readable combination strings (`"Ctrl+K, Ctrl+P"`),
a macro registry, a stateful sequence matcher, a context stack with
fallbacks, and an interceptor middleware chain.

```rust
let sc = use_shortcuts();
sc.keymap(KeymapOptions { commands, contexts });
sc.on("save", move |_sev, _w, cx| { /* ... */ });
```

### Shortcut combinations

Shortcuts are expressed as strings using `+` for simultaneous keys and
`,` for sequential steps:

| Combination | Meaning |
|-------------|---------|
| `"Ctrl+S"` | Ctrl and S pressed together |
| `"Ctrl+Shift+S"` | Ctrl, Shift, and S together (exact modifier match) |
| `"Ctrl+K,Ctrl+P"` | Ctrl+K, then Ctrl+P in sequence |
| `"G,G"` | G pressed twice in sequence |

### Context stack with fallbacks

Commands are grouped into named contexts. A context can declare
`fallbacks` — other contexts whose commands are inherited. The
`switch_context` method pushes a context onto the stack; a `Drop` guard
pops it automatically.

```rust
// "palette" context inherits all commands from "global"
contexts.insert("palette", ContextOptions {
    commands: vec!["next", "prev", "select", "close"],
    fallbacks: Some(vec!["global".to_string()]),
    ..
});
```

### Partial-match tracking

When a multi-step shortcut (e.g. `Ctrl+K, Ctrl+P`) is mid-sequence, the
engine reports the partially-matched command names via
`on_partial_change`. The example displays a yellow indicator showing
which command is awaiting its next key.

### `prevent_default` → `stop_propagation`

The TS library's `preventDefault` (DOM default-action cancel) maps to
`KeyboardEvent::stop_propagation()` in vgui. When a command matches and
`prevent_default` is `true` (the default), the dispatcher calls
`e.stop_propagation()`, preventing later hand-written `use_key_down`
handlers from also processing the key.

## Running

### Native

    cargo run -p vgui-keyboard-shortcuts

### Web (WASM)

    cargo build --target wasm32-unknown-unknown -p vgui-keyboard-shortcuts --release
    wasm-bindgen --target web --out-dir examples/keyboard-shortcuts/dist \
        --no-typescript target/wasm32-unknown-unknown/release/keyboard-shortcuts.wasm
    python3 scripts/serve_plain.py 8080 examples/keyboard-shortcuts
