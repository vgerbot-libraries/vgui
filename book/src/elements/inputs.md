# Input Elements

## Overview

`<input>` is a void element — it cannot have children and is configured
entirely through attributes. Both `<input type="text">` and
`<input type="text" />` (self-closing) are accepted.

The `type` attribute selects the widget kind. If omitted, `type="text"` is
assumed. Supported types:

| Category    | Types                                                               |
| ----------- | ------------------------------------------------------------------- |
| Text-based  | `text`, `password`, `search`, `email`, `url`, `tel`, `number`, `date`, `datetime-local`, `time`, `month`, `week`, `color` |
| Boolean     | `checkbox`, `radio`                                                 |
| Numeric     | `range`                                                             |
| File        | `file`                                                              |
| Button      | `submit`, `button`, `reset`                                         |
| Hidden      | `hidden`                                                            |

## Text-based Types

All text-based types render a full-featured text field with cursor movement,
selection, keyboard editing, clipboard (Ctrl+A/C/V/X), and IME (CJK
composition) support.

```rust
view! {
    <input
        type="text"
        placeholder="Name"
        on:input={move |v: &str, cx: &mut App| set_name.set(cx, v.to_string())}
    />
}
```

### Supported attributes

| Attribute     | Applies to             | Description                          |
| ------------- | ---------------------- | ------------------------------------ |
| `value`       | All text-based         | Initial/current value (string).      |
| `placeholder` | All text-based         | Placeholder text.                    |
| `disabled`    | All text-based         | Disables input.                      |
| `readonly`    | All text-based         | Read-only mode.                      |
| `min`         | `number`               | Minimum numeric value (f64).         |
| `max`         | `number`               | Maximum numeric value (f64).         |
| `step`        | `number`               | Step increment (f64).                |
| `on:input`    | All text-based         | Fires on every keystroke. `FnMut(&str, &mut App)`. |
| `on:change`   | All text-based         | Fires on Enter/blur. `FnMut(&str, &mut App)`. |
| `style`       | All text-based         | CSS-in-Rust styles.                  |
| `class`       | All text-based         | Tailwind classes.                    |
| `id`          | All text-based         | Element id (for `<label for=>`).     |
| `tabindex`    | All text-based         | Tab order.                           |

### Type-specific placeholder defaults

When no `placeholder` is specified, each type provides a format-appropriate
default:

| Type             | Default placeholder    |
| ---------------- | ---------------------- |
| `text`           | (empty)                |
| `password`       | (empty)                |
| `search`         | `Search…`              |
| `email`          | `email@example.com`    |
| `url`            | `https://example.com`  |
| `tel`            | `+1 (555) 000-0000`    |
| `number`         | `0`                    |
| `date`           | `YYYY-MM-DD`           |
| `datetime-local` | `YYYY-MM-DDTHH:MM`     |
| `time`           | `HH:MM`                |
| `month`          | `YYYY-MM`              |
| `week`           | `YYYY-Www`             |
| `color`          | `#RRGGBB`              |

> **Note:** `date`, `datetime-local`, `time`, `month`, `week`, and `color` are
> text-entry v1 — they use a format placeholder and light validation with no
> calendar/color popup. Popups are future work.

## Checkbox & Radio

```rust
view! {
    <input type="checkbox"
        checked={done.get()}
        on:change={move |v: bool, cx: &mut App| set_done.set(cx, v)}
    />
}
```

```rust
view! {
    <input type="radio"
        checked={sel.get() == 0}
        on:change={move |_v: bool, cx: &mut App| set_sel.set(cx, 0)}
    />
}
```

### Attributes

| Attribute    | Type   | Description                          |
| ------------ | ------ | ------------------------------------ |
| `checked`    | `bool` | Checked state.                       |
| `disabled`   | `bool` | Disables input.                      |
| `on:change`  | `FnMut(bool, &mut App)` | Fires on click. New checked state. |
| `style`/`class`/`hover`/`active`/`focus`/`id`/`tabindex` | — | Standard styling attributes. |

The checkbox renders as an 18×18px rounded box with a checkmark (✓) when
checked. The radio renders as a similar box (radio-specific styling is planned).

## Range Slider

```rust
view! {
    <input type="range"
        min={0.0f64}
        max={100.0f64}
        step={1.0f64}
        value={vol.get()}
        on:change={move |v: f64, cx: &mut App| set_vol.set(cx, v)}
    />
}
```

### Attributes

| Attribute    | Type  | Default | Description                |
| ------------ | ----- | ------- | -------------------------- |
| `min`        | f64   | 0.0     | Minimum value.             |
| `max`        | f64   | 100.0   | Maximum value.             |
| `step`       | f64   | 1.0     | Step increment.            |
| `value`      | f64   | 0.0     | Current value.             |
| `disabled`   | bool  | false   | Disables the slider.       |
| `on:change`  | `FnMut(f64, &mut App)` | — | Fires on drag. New value. |
| `style`/`class`/`id`/`tabindex` | — | — | Standard styling. |

The range slider is a persistent `gpui` entity — its drag state survives
re-renders via the reactive scope slot mechanism.

## File Picker

```rust
view! {
    <input type="file"
        value="Browse..."
        multiple={true}
        on:change={move |paths: Vec<std::path::PathBuf>, _cx: &mut App| {
            eprintln!("selected: {:?}", paths);
        }}
    />
}
```

### Attributes

| Attribute    | Type   | Description                              |
| ------------ | ------ | ---------------------------------------- |
| `value`      | string | Button label text.                       |
| `multiple`   | bool   | Allow multiple file selection.           |
| `on:change`  | `FnMut(Vec<PathBuf>, &mut App)` | Fires on file selection. |
| `style`/`class`/`hover`/`active`/`focus`/`id`/`tabindex` | — | Standard styling. |

`accept` and `name` are accepted but unused in v1.

## Select

`<select>` uses an `options` attribute (a `Vec<(String, String)>` of
value-label pairs) rather than child `<option>` elements:

```rust
view! {
    <select
        options={vec![
            ("1".to_string(), "One".to_string()),
            ("2".to_string(), "Two".to_string()),
        ]}
        value={"1".to_string()}
        on:change={move |v: &str, cx: &mut App| set_sel.set(cx, v.to_string())}
    />
}
```

### Attributes

| Attribute    | Type                          | Description                    |
| ------------ | ----------------------------- | ------------------------------ |
| `options`    | `Vec<(String, String)>`       | Value-label pairs.             |
| `value`      | `String`                      | Currently selected value.      |
| `disabled`   | bool                          | Disables the select.           |
| `on:change`  | `FnMut(&str, &mut App)`       | Fires on selection change.     |
| `style`/`class`/`id` | —                      | Standard styling.              |

> **Note:** The select dropdown cycles through options on click in v1 — a
> full popup dropdown is future work.

## Textarea

`<textarea>` is a void element (no children) configured through attributes:

```rust
view! {
    <textarea
        placeholder="Enter text"
        value={text.get()}
        on:input={move |v: &str, cx: &mut App| set_text.set(cx, v.to_string())}
    />
}
```

### Attributes

| Attribute     | Type                          | Description                     |
| ------------- | ----------------------------- | ------------------------------- |
| `value`       | `String`                      | Current content.                |
| `placeholder` | string                        | Placeholder text.               |
| `disabled`    | bool                          | Disables editing.               |
| `readonly`    | bool                          | Read-only mode.                 |
| `on:input`    | `FnMut(&str, &mut App)`       | Fires on every keystroke.       |
| `on:change`   | `FnMut(&str, &mut App)`       | Fires on blur.                  |
| `style`/`class`/`id`/`tabindex` | —               | Standard styling.               |

`rows` and `name` are accepted but unused in v1. The textarea is a multi-line
text input with the same cursor/selection/clipboard/IME support as text
inputs.

## Submit/Button/Reset

These render as clickable buttons (like `<button>`). The `value` attribute
becomes the button label. `on:click` wires the handler.

```rust
view! {
    <input type="submit" value="Submit" on:click={click(move |_cx| {})} />
    <input type="button" value="Click" on:click={click(move |_cx| {})} />
    <input type="reset" value="Reset" on:click={click(move |_cx| {})} />
}
```

> **Note:** `submit` and `reset` have no form semantics in vgui v1 — they
> behave as buttons.

## Hidden

`<input type="hidden">` renders nothing (`gpui::Empty`):

```rust
view! {
    <input type="hidden" value="invisible" />
}
```

## Label

`<label>` associates text with an input for click-to-focus behavior. Two forms
are supported:

### Explicit `for` attribute

```rust
view! {
    <label for="username" class="text-sm">{"Username"}</label>
    <input type="text" id="username" placeholder="Enter username" />
}
```

Clicking the label focuses the input with `id="username"`.

### Wrapping label

```rust
view! {
    <label class="flex flex-col gap-1">
        <span class="text-sm">{"Wrapped input"}</span>
        <input type="text" placeholder="Click label to focus" />
    </label>
}
```

When `<label>` wraps an input, clicking anywhere on the label focuses the first
focusable child input.

`<label>` supports all standard styling attributes (`style`, `class`, `hover`,
`active`, `focus`, `id`, `tabindex`) and event handlers.
