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

Renders a modal dialog with a semi-transparent overlay. When `open` is false,
renders a hidden element.

```rust
view! {
    <dialog open={show_dialog.get()}>
        <div class="bg-white p-4 rounded">
            {"Dialog content"}
            <button on:click={click(move |cx| set_show.set(cx, false))}>
                {"Close"}
            </button>
        </div>
    </dialog>
}
```

### Attributes

| Attribute | Type | Default | Description                     |
| --------- | ---- | ------- | ------------------------------- |
| `open`    | bool | false   | Whether the dialog is visible.  |

When `open` is true, the dialog renders with a semi-transparent overlay
covering the window, and the children centered on top. When false, it renders
as a hidden element (no layout impact).
