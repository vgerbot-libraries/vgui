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
| `required`    | All text-based         | Fails validation when empty.       |
| `pattern`     | All text-based         | Literal match or `*` wildcard (`foo*`, `*bar`). Not JS RegExp. |
| `minlength`   | All text-based         | Minimum character count (usize).   |
| `maxlength`   | All text-based         | Maximum character count (usize).   |
| `list`        | All text-based         | Datalist id for autocomplete suggestions. |

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

> **Note:** `date`, `datetime-local`, `time`, `month`, and `week` render a
> calendar popup. `color` renders a preset color palette popup. `tel` is
> text-entry with format validation only.

### Validation

Text-based inputs check constraint rules on every render and turn the border
red (`hsla(0, 0.8, 0.5, 1)`) when the value is invalid. Validation does not
block `on:input` or `on:change` — it is visual feedback only.

Rules applied (in order):

1. `required` — value must not be empty.
2. `minlength` / `maxlength` — character count bounds.
3. `pattern` — exact literal match or a single `*` wildcard prefix/suffix
   (`foo*`, `*bar`, `foo`). This is not a JavaScript RegExp.
4. Type-specific checks:
   - `email` — exactly one `@` with non-empty local and domain parts.
   - `url` — must start with `http://` or `https://`.
   - `number` — must parse as `f64` and satisfy `min`/`max` if set.

### Color picker

`<input type="color">` displays a 24×full-height color swatch on the right
side of the input, showing the current `#RRGGBB` value (gray on parse
failure). Clicking the swatch opens an 8×3 preset palette of 24 colors.
Clicking a preset writes the uppercase `#RRGGBB` value, closes the popup,
and fires `on:input` + `on:change`. Free RGB sliders are not provided.

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

### Radio Groups (Roving Tabindex)

Wrap radios in a `<radiogroup>` to enable roving tabindex:

```rust
view! {
    <radiogroup>
        <input type="radio" checked={sel.get() == 0}
            on:change={move |_, cx| set_sel.set(cx, 0)} />
        <input type="radio" checked={sel.get() == 1}
            on:change={move |_, cx| set_sel.set(cx, 1)} />
        <input type="radio" checked={sel.get() == 2}
            on:change={move |_, cx| set_sel.set(cx, 2)} />
    </radiogroup>
}
```

Inside `<radiogroup>`:
- Only the checked radio is a tab stop (Tab moves to it, then Tab again
  leaves the group).
- Arrow keys (←/↑/→/↓) move focus between radios in the group.
- Clicking a radio focuses it and fires `on:change`.

`<radiogroup>` takes no attributes. Radios outside a `<radiogroup>` behave
as standalone focusable elements (the checked radio is a tab stop).

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

`<select>` renders a dropdown with a popover option list. The `options` attribute
is a `Vec<(String, String)>` of value-label pairs. The popover width always
matches the trigger width; clicking the trigger toggles the popover, clicking an
option fires `on:change` and closes it, and clicking outside or pressing Escape
closes it.

```rust
view! {
    <select
        options={vec![
            ("1".to_string(), "One".to_string()),
            ("2".to_string(), "Two".to_string()),
        ]}
        value={sel.get()}
        on:change={move |v: &str, cx: &mut App| set_sel.set(cx, v.to_string())}
    />
}
```

### Custom option content

To render rich content per option (icons, colored values, multi-line rows), pass
a single closure child. The closure receives `(value: &str, label: &str)` and
returns an element via `view! {}`. The same closure renders both the popover rows
and the trigger's display of the selected option.

```rust
view! {
    <select
        options={vec![
            ("1".to_string(), "One".to_string()),
            ("2".to_string(), "Two".to_string()),
        ]}
        value={sel.get()}
        on:change={move |v: &str, cx: &mut App| set_sel.set(cx, v.to_string())}
    >
        {move |value: &str, label: &str| view! {
            <div class="flex items-center gap-2">
                <span class="text-[#0f0]">{value.to_string()}</span>
                <span>{label.to_string()}</span>
            </div>
        }}
    </select>
}
```

When no child closure is given, each option renders as plain label text.

### Attributes

| Attribute    | Type                          | Description                    |
| ------------ | ----------------------------- | ------------------------------ |
| `options`    | `Vec<(String, String)>`       | Value-label pairs (flat list). |
| `groups`     | `Vec<(String, Vec<(String, String)>)>` | Grouped options; group name + value-label pairs. When non-empty, the popover renders by group. |
| `value`      | `String`                      | Currently selected value. In multiple mode, comma-separated values (`a,b`). |
| `multiple`   | bool                          | Enables multi-select. Default `false`. |
| `disabled`   | bool                          | Disables the select.           |
| `on:change`  | `FnMut(&str, &mut App)`       | Fires on selection change. In multiple mode, passes the comma-separated string. |
| `style`/`class`/`id` | —                      | Standard styling.              |
### Multiple selection

When `multiple={true}`, clicking an option toggles that value in the
comma-separated `value` string. The popover stays open after each toggle
(click outside or press Escape to close). The trigger text shows the selected
labels joined by `", "` (empty if none).

```rust
view! {
    <select
        multiple={true}
        options={vec![
            ("a".to_string(), "Alpha".to_string()),
            ("b".to_string(), "Beta".to_string()),
        ]}
        value={sel.get()}
        on:change={move |v: &str, cx: &mut App| set_sel.set(cx, v.to_string())}
    />
}
```

### Grouped options

Use `groups` instead of `options` to render options under non-clickable bold
group headers. Each group is `(group_name, Vec<(value, label)>)`. When
`groups` is non-empty, the popover renders by group; flat `options` are
ignored for rendering but still used for label lookups.

```rust
view! {
    <select
        groups={vec![
            ("Fruits".to_string(), vec![
                ("apple".to_string(), "Apple".to_string()),
                ("banana".to_string(), "Banana".to_string()),
            ]),
            ("Vegetables".to_string(), vec![
                ("carrot".to_string(), "Carrot".to_string()),
            ]),
        ]}
        value={sel.get()}
        on:change={move |v: &str, cx: &mut App| set_sel.set(cx, v.to_string())}
    />
}
```

`<select>` does not read `<option>` or `<optgroup>` child nodes; use the
`options` or `groups` prop instead.

### Child closure signature

The optional child is a closure `Fn(&str, &str) -> impl IntoElement`. The first
argument is the option's value, the second is its label. The closure must be
`Fn` (not `FnMut`) because it is invoked from multiple render sites.


## Datalist

`<datalist>` provides autocomplete suggestions for text inputs. It renders
nothing but registers a list of options under an `id`. A text input with a
matching `list=<id>` attribute shows prefix-matched suggestions (up to 8)
below the input when focused and the value is non-empty. Clicking a suggestion
writes it to the input and fires `on:input` + `on:change`.

```rust
view! {
    <datalist id="cities" options={vec!["Paris".to_string(), "London".to_string()]} />
    <input type="text" list="cities" on:input={move |v: &str, cx: &mut App| {}} />
}
```

`<datalist>` requires an `id` attribute and accepts `options={Vec<String>}`.
It does not read `<option>` child nodes; use the `options` prop.

## Output

`<output>` is a pure `<div>` alias — a generic container with no special
behavior. Use it to display computed results.
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
| `rows`        | `u32`                         | Minimum visible line count.     |
| `on:input`    | `FnMut(&str, &mut App)`       | Fires on every keystroke.       |
| `on:change`   | `FnMut(&str, &mut App)`       | Fires on blur.                  |
| `style`/`class`/`id`/`tabindex` | —               | Standard styling.               |

`name` is accepted but unused. When `rows` is set, the textarea's minimum
height is `rows` times the line height plus 8px of vertical padding; without
`rows` the minimum height is 80px. The textarea is a multi-line text input
with the same cursor/selection/clipboard/IME support as text inputs.

## Submit/Button/Reset

These render as clickable buttons (like `<button>`). The `value` attribute
becomes the button label. `on:click` wires the handler. Without an explicit
`tabindex` they default to `tabindex={0}` and are in Tab order.

```rust
view! {
    <input type="submit" value="Submit" on:click={click(move |_cx| {})} />
    <input type="button" value="Click" on:click={click(move |_cx| {})} />
    <input type="reset" value="Reset" on:click={click(move |_cx| {})} />
}
```

> **Note:** Inside a `<form>`, `submit` and `reset` buttons automatically
> invoke the form's `on:submit` / `on:reset` handler. An explicit `on:click`
> overrides this — the auto-bind is skipped to avoid double-firing.

## Form

`<form>` wraps its children in a form context. `on:submit` and `on:reset`
accept `FnMut(&mut App)` closures. Child `<input type="submit">` and
`<input type="reset">` buttons auto-invoke the enclosing form's handler;
pressing Enter in a single-line text input also triggers `on:submit`.

```rust
view! {
    <form on:submit={move |cx| { /* ... */ }} on:reset={move |cx| { /* ... */ }}>
        <input type="text" required={true} />
        <input type="submit" value="Go" />
        <input type="reset" value="Clear" />
    </form>
}
```

When no `<form>` ancestor is present, submit/reset buttons are no-ops.
`<form>` supports standard styling attributes (`style`, `class`, `hover`,
`active`, `focus`, `id`, `tabindex`, `ref`) and event handlers.


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
focusable child input. This works with all input types: text-based inputs
(`text`, `password`, `email`, …), `textarea`, `range`, `checkbox`, `radio`,
`file`, and `select`.

`<label>` supports all standard styling attributes (`style`, `class`, `hover`,
`active`, `focus`, `id`, `tabindex`) and event handlers.
