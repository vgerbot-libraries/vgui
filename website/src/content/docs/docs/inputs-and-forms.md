---
title: Inputs and Forms
description: Connect text, choice, range, select, textarea, and file widgets to signals while respecting vgui's form limitations.
---

vgui inputs are GPUI widgets configured through `view!` attributes. They do not create browser controls or participate in browser form submission. Store edited values in signals or other application state, and return those values through widget attributes on subsequent renders.

For general attributes and pointer/keyboard handlers, see [Built-in Elements and Events](../built-in-elements-and-events/). For focus and scroll handles, see [Refs and Context](../refs-and-context/).

## Input kinds and state flow

`input` is void and defaults to `type="text"`. The `type` value must be a string literal. Supported kinds are:

| Category | Types |
| --- | --- |
| Text-based | `text`, `password`, `search`, `email`, `url`, `tel`, `number`, `date`, `datetime-local`, `time`, `month`, `week`, `color` |
| Boolean | `checkbox`, `radio` |
| Numeric | `range` |
| File | `file` |
| Buttons | `submit`, `button`, `reset` |
| Hidden | `hidden` |

Text and range widgets keep GPUI entity state in reactive scope slots. Their `value` attributes synchronize props into that state; while a text input is focused, its in-progress edit is not overwritten by the incoming `value`. This is not a browser DOM binding, so application callbacks should still record changes explicitly.

## Text-based inputs

```rust
view! {
    <input
        type="text"
        value={name.get()}
        placeholder="Name"
        on:input={move |value: &str, cx: &mut gpui::App| {
            set_name.set(cx, value.to_string());
        }}
        on:change={move |value: &str, cx: &mut gpui::App| {
            set_committed_name.set(cx, value.to_string());
        }}
    />
}
```

`on:input` has the shape `FnMut(&str, &mut App)` and runs as the widget value changes. For single-line inputs, `on:change` runs on Enter and on focus loss. Text editing includes GPUI-backed selection, clipboard, and IME paths; password masks displayed characters. Date/time/month and color kinds include custom picker paths, while `week` remains a text kind rather than a browser week control.

Effective text-input attributes include:

| Attribute | Behavior |
| --- | --- |
| `value`, `placeholder` | Current string value and placeholder. Types supply format-oriented default placeholders when none is given. |
| `disabled`, `readonly` | Disable interaction or prevent edits. |
| `min`, `max` | Optional `f64` bounds used by number validation. |
| `step` | Accepted and stored as numeric metadata; current text validation does not enforce step matching. |
| `required`, `pattern`, `minlength` | HTML-like visual constraint checks. |
| `maxlength` | Accepted by the macro, but not currently synchronized into the persistent text-input state; do not rely on it for validation. |
| `list` | ID of a `datalist` suggestion registry. |
| `style`, `class`, `id`, `tabindex` | Widget styling, label lookup ID, and focus configuration. |

Direct `ref`, `role`, `aria:*`, spread, animation, and drag/drop attributes are not wired for text inputs. Wrap the input in a general element when a wrapper ref or ARIA metadata is needed.

### Validation and datalists

Validation changes the input border; it does not block callbacks or submit a form. The checks are deliberately smaller than HTML constraint validation:

- `required` rejects an empty string.
- `minlength` counts Rust characters, not UTF-8 bytes.
- `pattern` is an exact literal or one leading/trailing `*` wildcard (`foo*`, `*bar`), not a regular expression.
- `email` requires exactly one `@` with nonempty sides; `url` requires an `http://` or `https://` prefix.
- `number` parses as `f64` and applies `min`/`max` when present.

A `datalist` renders nothing. It registers `options={Vec<String>}` under a required `id`; a text input with the matching `list="id"` can present prefix-matched suggestions.

```rust
view! {
    <div>
        <datalist id="cities" options={vec!["Paris".to_string(), "London".to_string()]} />
        <input type="text" list="cities" value={city.get()} />
    </div>
}
```

## Checkbox and radio

Both widgets use `checked={bool}`, `disabled={bool}`, and `on:change={FnMut(bool, &mut App)}`. A checkbox reports the toggled value; a radio reports `true` when activated. The application remains responsible for updating the checked state.

```rust
view! {
    <input
        type="checkbox"
        checked={done.get()}
        on:change={move |checked: bool, cx: &mut gpui::App| set_done.set(cx, checked)}
    />
}
```

For a mutually exclusive choice, derive every radio's `checked` value from the same application state. There is no HTML `name`-based grouping behavior.

```rust
view! {
    <radiogroup>
        <input type="radio" checked={choice.get() == 0}
            on:change={move |_checked: bool, cx: &mut gpui::App| set_choice.set(cx, 0)} />
        <input type="radio" checked={choice.get() == 1}
            on:change={move |_checked: bool, cx: &mut gpui::App| set_choice.set(cx, 1)} />
    </radiogroup>
}
```

`radiogroup` takes no attributes. Its arrow keys move focus among registered radios with roving tab stops; moving focus does not itself select a radio or call `on:change`. Checkbox, radio, file, and button-style inputs support wrapper-level styling and `ref` where documented.

## Range

```rust
view! {
    <input
        type="range"
        min={0.0f64}
        max={100.0f64}
        step={1.0f64}
        value={volume.get()}
        on:change={move |value: f64, cx: &mut gpui::App| set_volume.set(cx, value)}
    />
}
```

Defaults are `min=0`, `max=100`, `step=1`, and `value=0`. `on:change` receives `f64` during pointer dragging and keyboard adjustment. Arrow keys use `step`, Shift multiplies it by ten, and Home/End select the bounds. `disabled`, `style`, `class`, `id`, and `tabindex` are supported. Direct `ref` is not bound; use a wrapper `div` as described in [Refs and Context](../refs-and-context/).

## Select

`select` takes data through attributes, not `option` children. Flat options are `Vec<(String, String)>` value-label pairs. A single optional closure child can customize both the selected content and option rows.

```rust
view! {
    <select
        options={vec![
            ("one".to_string(), "One".to_string()),
            ("two".to_string(), "Two".to_string()),
        ]}
        value={selection.get()}
        on:change={move |value: &str, cx: &mut gpui::App| {
            set_selection.set(cx, value.to_string());
        }}
    />
}
```

Use `groups={Vec<(String, Vec<(String, String)>)>}` for grouped rows. Nonempty groups drive popover rendering; flat options still participate in label lookup. With `multiple={true}`, `value` and the `on:change` argument are comma-separated selections and the popover remains open after a toggle. Consequently, option values containing commas cannot be represented reliably by this encoding.

A custom renderer has the form `Fn(&str, &str) -> impl IntoElement`:

```rust
view! {
    <select options={options} value={selection.get()}>
        {move |value: &str, label: &str| view! {
            <div class="flex gap-2">
                <span>{value.to_string()}</span>
                <span>{label.to_string()}</span>
            </div>
        }}
    </select>
}
```

`select` supports `style` and `class`; `name`, `id`, and `tabindex` are accepted but unused. Direct `ref`, ARIA attributes, spreads, and non-`change` events are rejected. Do not nest `option` or `optgroup` nodes expecting them to populate the widget.

## Textarea

`textarea` is also void and uses the same text callback types as single-line input. Enter inserts a newline; `on:change` runs on focus loss.

```rust
view! {
    <textarea
        rows={4u32}
        value={notes.get()}
        placeholder="Notes"
        on:input={move |value: &str, cx: &mut gpui::App| {
            set_notes.set(cx, value.to_string());
        }}
    />
}
```

It supports `value`, `placeholder`, `disabled`, `readonly`, `rows`, `on:input`, `on:change`, `style`, `class`, `id`, and `tabindex`. `name` is accepted but unused. `rows` controls minimum height; without it, the widget uses its default minimum. Direct `ref`, role/ARIA attributes, and unrelated events are rejected; use a wrapper when appropriate.

## File selection

```rust
view! {
    <input
        type="file"
        value="Choose files"
        multiple={true}
        on:change={move |paths: Vec<std::path::PathBuf>, cx: &mut gpui::App| {
            set_paths.set(cx, paths);
        }}
    />
}
```

For file inputs, `value` is the button label, not a selected filename. The callback receives selected `PathBuf` values after GPUI's file prompt. `multiple` controls multi-selection. There is no `accept` filter, file-content API, or directory-selection mode in this widget. Although common input parsing accepts `disabled`, the file widget does not currently wire it into its prompt action. `style`, `class`, state styles, `id`, `tabindex`, and wrapper `ref` are supported.

## Labels

Use `for` with a text-like widget's registered `id`, or wrap a widget so the label can discover its first label target:

```rust
view! {
    <div>
        <label for="username">{"Username"}</label>
        <input id="username" type="text" value={name.get()} />
    </div>
}
```

```rust
view! {
    <label class="flex items-center gap-2">
        <input type="checkbox" checked={done.get()} />
        <span>{"Done"}</span>
    </label>
}
```

An explicit label focuses its registered target. A wrapping label focuses the discovered target and, for widgets that register an action, invokes that action. This is vgui label-scope behavior, not arbitrary HTML label activation.

## Forms, buttons, reset, and hidden values

`form` is a styled GPUI container with construction-time form scope. It accepts `on:submit={FnMut(&mut App)}` and `on:reset={FnMut(&mut App)}` in addition to its supported styling, ref, and general event attributes. It has no HTML `action`, `method`, `FormData`, network submission, automatic control collection, or validation-blocking behavior.

Button-style inputs use `value` as their label and default to tab stop 0. An explicit `on:click` replaces the macro's automatic submit/reset binding:

```rust
view! {
    <form class="flex gap-2">
        <input type="text" value={name.get()}
            on:input={move |value: &str, cx: &mut gpui::App| set_name.set(cx, value.to_string())} />
        <input type="submit" value="Save"
            on:click={click(move |cx| set_saved.set(cx, true))} />
        <input type="reset" value="Clear"
            on:click={click(move |cx| set_name.set(cx, String::new()))} />
    </form>
}
```

The current implicit dispatch is limited: default submit/reset handlers look up the form stack when invoked, but that stack is popped after children are constructed. Enter in a single-line text input commits `on:change`, but should not be relied on for implicit submission either. Use explicit click handlers for save/reset behavior and perform validation, serialization, and state clearing in application code.

`input type="hidden"` emits `gpui::Empty`; its `value` is not collected anywhere. `output` is only a generic div alias. These widgets therefore do not provide HTML form semantics merely by being nested in `form`.
