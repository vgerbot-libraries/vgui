# Other Components

Beyond standard HTML elements and input widgets, `vgui` provides several
specialized components that are accessible as lowercase tags in `view!`.

## `<progress>` — Progress Bar

Renders a horizontal progress bar. The fill width is determined by `value` /
`max`:

```rust
view! {
    <progress value={0.5f64} max={1.0f64} />
}
```

### Attributes

| Attribute | Type | Default | Description              |
| --------- | ---- | ------- | ------------------------ |
| `value`   | f64  | 0.0     | Current progress value.  |
| `max`     | f64  | 1.0     | Maximum value.           |

The bar is rendered as a `gpui::Div` with a filled portion proportional to
`value / max`. Standard styling attributes (`class`, `style`) can be applied to
customize appearance.

## `<meter>` — Meter Gauge

`<meter>` is an alias for `<progress>` — it accepts the same `value` and `max`
attributes and renders identically:

```rust
view! {
    <meter value={0.7f64} max={1.0f64} />
}
```

## `<details>` — Collapsible Container

Renders a collapsible container with a summary header and hidden content. The
`open` attribute controls content visibility; the summary is always visible.

```rust
view! {
    <details open={open.get()}>
        <summary on:click={click(move |cx| set_open.update(cx, |v| *v = !*v))}>
            {"Click to toggle"}
        </summary>
        <div>{"Hidden content"}</div>
    </details>
}
```

### Attributes

| Attribute | Type | Default | Description                     |
| --------- | ---- | ------- | ------------------------------- |
| `open`    | bool | false   | Whether content is visible.     |

### Children

`<details>` children are split: the first child (typically `<summary>`) is the
always-visible header, and the remaining children form the collapsible content.
`<summary>` is rendered as a `gpui::div()` with `cursor_pointer()`.

> **Note:** In vgui v1, `open` is a prop, not internal state — you must manage
> it with a signal and toggle it in the `<summary>` click handler, as shown
> above.

## `<dialog>` — Modal Dialog

Renders a modal dialog that floats above all non-deferred content via a portal
(deferred paint layer) at z-index priority 100. When `open` is false, renders a
hidden element with no layout impact.

```rust
view! {
    <button on:click={click(move |cx| set_show.set(cx, true))}>
        {"Open Dialog"}
    </button>
    <dialog open={show_dialog.get()} on:close={move |cx| set_show.set(cx, false)}>
        <div class="bg-white p-4 rounded text-black">
            <p>{"Dialog content — click outside or press Escape to close."}</p>
            <button on:click={click(move |cx| set_show.set(cx, false))}>
                {"Close"}
            </button>
        </div>
    </dialog>
}
```

### Attributes

| Attribute   | Type              | Default | Description                          |
| ----------- | ----------------- | ------- | ------------------------------------ |
| `open`      | bool              | false   | Whether the dialog is visible.       |
| `on:close`  | `Fn(&mut App)`    | no-op   | Called when the dialog is dismissed. |

### Dismissal

The dialog can be dismissed three ways:

- **Click-outside:** Clicking the backdrop (outside the content) fires
  `on:close`. The backdrop occludes mouse events, so elements behind the dialog
  never receive them.
- **Escape key:** Pressing Escape fires `on:close` — but only when focus is
  within the dialog. This is a gpui constraint: key events dispatch only along
  the focus path (root → focused node). Clicking into the dialog moves focus
  there, so Escape works in the natural case. If focus remains on background
  content, Escape will not fire.
- **Explicit close:** Call your `on:close` handler from a button inside the
  dialog (as shown above).

The `on:close` closure takes `Fn(&mut App)` directly — no `click()` wrapper
needed, unlike `on:click`. This is because `on:close` is a vgui abstraction,
not a gpui event.

### Portal rendering

The dialog paints on a deferred layer (priority 100), so it floats above all
non-deferred content regardless of sibling paint order. It stays centered even
while the page scrolls — the portal layer is independent of scroll content.

## `<portal>` — Portal Floating Layer

Renders content on a floating layer drawn after all non-deferred ancestors.
This is the base portal primitive — `dialog` and `floating` wrap it with
higher-level behavior.

```rust
view! {
    <portal priority={200}>
        <div class="bg-white p-2 rounded">
            {"Rendered on top of everything"}
        </div>
    </portal>
}
```

### Attributes

| Attribute  | Type | Default | Description                                              |
| ---------- | ---- | ------- | -------------------------------------------------------- |
| `priority` | usize | 0       | Stacking order; higher values paint on top of lower ones. |

Use `<portal>` when you need custom stacking control — for example, rendering a
dialog above another dialog (priority 100) by wrapping content at priority 200.

## `<floating>` — Positioned Floating Element

Renders content at a window-coordinate point with automatic overflow avoidance.
If the content would extend past the window edge, it snaps inside with an 8px
margin. Paints on a deferred layer at priority 50 (below `dialog`'s 100).

```rust
view! {
    <floating position={gpui::point(gpui::px(100.), gpui::px(200.))}>
        <div class="bg-white p-2 rounded"
             on:mouse_down_out={move |_, _, cx| /* dismiss */}>
            {"Floating tooltip or popover"}
        </div>
    </floating>
}
```

### Attributes

| Attribute  | Type             | Default | Description                              |
| ---------- | ---------------- | ------- | ---------------------------------------- |
| `position` | `Point<Pixels>`  | —       | Window-coordinate position (required).   |

`<floating>` has no built-in dismissal. Add `on:mouse_down_out` to the content
for click-outside behavior, as shown above.
