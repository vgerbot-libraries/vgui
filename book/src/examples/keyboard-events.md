# Keyboard Events Example

## Live Demo

<iframe src="../wasm/keyboard-events/" width="100%" height="480" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
{{#include ../../../examples/keyboard-events/src/main.rs}}
```

## Key Concepts

### Three layers of keyboard handling

vgui provides three complementary ways to respond to keyboard input.
This example demonstrates all three in a single command-palette UI:

### `use_key_down` / `use_key_up` — global hooks

Register a handler that fires on every key press (or release) regardless
of which element has focus. Internally, the handler is stored in the
render `Scope` and dispatched by `VguiRoot` from the root element's
`on_key_down` / `on_key_up` listener.

```rust
use_key_down(move |e: &KeyboardEvent, _w, cx| {
    if e.ctrl_key && e.key == "k" {
        set_palette_open.update(cx, |v| *v = !*v);
    }
});
```

In the example, the global handler manages:

- **Ctrl+K** — toggle the command palette
- **Escape** — close any open overlay
- **?** — toggle the help panel
- **Arrow keys / j / k** — navigate palette entries
- **Enter** — execute the selected command

`use_key_up` tracks modifier-key releases (Shift, Control, Alt, Meta) to
show the down/up cycle in the inspector.

### `on:keydown` — element-level handler

Individual elements can listen for keyboard events when they have focus.
The inspector div uses `tabindex="0"` to become focusable and
`on:keydown` to display the raw `KeyboardEvent` fields (key, code,
modifiers, repeat, key_char).

### `stop_propagation` — controlling event dispatch

`KeyboardEvent::stop_propagation()` sets a flag that:

1. **Halts global dispatch** — `VguiRoot` checks the flag after each
   global handler and breaks out of the dispatch loop.
2. **Stops gpui event bubbling** — the `__dom_key_down` / `__dom_key_up`
   wrappers call `cx.stop_propagation()` after the user closure returns
   if the flag is set.

In the example, the inspector div calls `e.stop_propagation()` when
**Tab** is pressed while it has focus. This prevents the global
`use_key_down` handler from also processing the Tab key, keeping focus
trapped within the inspector.

```rust
on:keydown={move |e: &KeyboardEvent, _w, _cx| {
    if e.key == "Tab" {
        e.stop_propagation();
    }
}}
```

### `KeyboardEvent` fields

The `KeyboardEvent` struct exposes Web-standard fields:

| Field | Type | Description |
|-------|------|-------------|
| `key` | `String` | The key value (e.g. `"a"`, `"Enter"`, `"Shift"`) |
| `code` | `String` | The physical key code (e.g. `"KeyA"`, `"Enter"`) |
| `ctrl_key` | `bool` | Control modifier held |
| `shift_key` | `bool` | Shift modifier held |
| `alt_key` | `bool` | Alt modifier held |
| `meta_key` | `bool` | Meta (Super/Command) modifier held |
| `repeat` | `bool` | Key is auto-repeating |
| `key_char` | `Option<String>` | Produced character, if any |

## Running

### Native

    cargo run -p vgui-keyboard-events

### Web (WASM)

    cargo build --target wasm32-unknown-unknown -p vgui-keyboard-events --release
    wasm-bindgen --target web --out-dir examples/keyboard-events/dist \
        --no-typescript target/wasm32-unknown-unknown/release/keyboard-events.wasm
    python3 scripts/serve_plain.py 8080 examples/keyboard-events
