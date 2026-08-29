# API Reference

The API reference is generated automatically by rustdoc. The links below point
to each crate's generated documentation:

## [vgui](https://docs.rs/vgui)

The main crate. Key exports:

### Macros

| Macro    | Source crate    | Description                          |
| -------- | --------------- | ------------------------------------ |
| `view!`  | `vgui-view`     | JSX-like view markup.                |
| `css!`   | `vgui-css`      | CSS-in-Rust style declarations.      |
| `tw!`    | `vgui-tailwind` | Tailwind utility class compilation.  |

### Reactivity

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `create_signal<T>(initial) -> (ReadSignal<T>, WriteSignal<T>)` | Creates a reactive signal. |
| `create_memo<T>(f) -> ReadSignal<T>`   | Creates a derived, cached value.         |
| `create_effect(f)`                     | Creates a side effect that re-runs on dep change. |
| `ReadSignal<T>::get() -> T`            | Reads value, registers dependency.       |
| `ReadSignal<T>::get_with(cx) -> T`     | Reads value without tracking.            |
| `WriteSignal<T>::set(cx, value)`       | Sets value, notifies if changed.         |
| `WriteSignal<T>::update(cx, f) -> R`   | Mutates value in place, notifies if changed. |
| `next_auto_id() -> u64`                | Stable per-render element id (used by `view!`). |

### Mounting

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `mount(cx, render_fn) -> Entity<VguiRoot>` | Creates the root entity and reactive scope. |
| `VguiRoot`                             | The gpui entity owning the reactive scope. |

### Control flow

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `show(when, then, fallback) -> AnyElement` | Conditional render with fallback.    |
| `show_when(when, then) -> AnyElement`  | Conditional render (no fallback).        |
| `for_each(items, child_fn) -> AnyElement` | List rendering.                       |
| `for_each_or(items, fallback, child_fn) -> AnyElement` | List rendering with fallback. |
| `progress(value, max) -> Div`          | Progress bar.                            |
| `details(open, summary, content) -> AnyElement` | Collapsible container.           |
| `dialog(open, content) -> AnyElement`  | Modal dialog.                            |

### Input widgets

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `text_input(TextInputProps) -> Entity<TextInput>` | Text-based input.              |
| `text_area(TextAreaProps) -> Entity<TextInput>` | Multi-line text input.          |
| `checkbox(CheckboxProps) -> Stateful<Div>` | Checkbox widget.                     |
| `radio(RadioProps) -> Stateful<Div>`   | Radio button widget.                     |
| `range_input(RangeProps) -> Entity<RangeInput>` | Range slider.                  |
| `file_input(FileProps) -> Stateful<Div>` | File picker button.                    |
| `select(SelectProps) -> Stateful<Div>` | Select dropdown.                         |

### Props structs

| Struct           | Fields                                                        |
| ---------------- | ------------------------------------------------------------- |
| `TextInputProps` | `kind`, `multiline`, `value`, `placeholder`, `disabled`, `readonly`, `min`, `max`, `step`, `on_input`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `TextAreaProps`  | `value`, `placeholder`, `disabled`, `readonly`, `on_input`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `CheckboxProps`  | `checked`, `disabled`, `on_change`                            |
| `RadioProps`     | `checked`, `disabled`, `on_change`                            |
| `RangeProps`     | `value`, `min`, `max`, `step`, `disabled`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `FileProps`      | `multiple`, `on_change`                                       |
| `SelectProps`    | `options: Vec<(String, String)>`, `value`, `disabled`, `on_change` |

### Enums

| Enum         | Variants                                                              |
| ------------ | --------------------------------------------------------------------- |
| `TextKind`   | `Text`, `Password`, `Search`, `Email`, `Url`, `Tel`, `Number`, `Date`, `DateTime`, `Time`, `Month`, `Week`, `Color` |

### Style types

| Item                | Description                                                  |
| ------------------- | ------------------------------------------------------------ |
| `Css`               | Output of `css!`. Wraps a `FnOnce(&mut StyleRefinement)`.    |
| `TwStyle`           | Output of `tw!`. Holds `base`, `hover`, `focus`, `active` closures. |
| `ApplyStyle<E>`     | Trait for applying a style to an element.                    |

### Helpers

| Item                | Description                                                  |
| ------------------- | ------------------------------------------------------------ |
| `click(f)`          | Wraps `Fn(&mut App)` into a gpui click handler.             |
| `into_child(value)` | Converts any `IntoElement` (or `IntoViewChild`) to `AnyElement`. |
| `input_cb(f)`       | Wraps `FnMut(&str, &mut App)` for `on:input`.               |
| `str_change_cb(f)`  | Wraps `FnMut(&str, &mut App)` for `on:change` (text).       |
| `bool_change_cb(f)` | Wraps `FnMut(bool, &mut App)` for `on:change` (checkbox/radio). |
| `f64_change_cb(f)`  | Wraps `FnMut(f64, &mut App)` for `on:change` (range).       |
| `files_cb(f)`       | Wraps `FnMut(Vec<PathBuf>, &mut App)` for `on:change` (file). |
| `str_select_change_cb(f)` | Wraps `FnMut(&str, &mut App)` for `on:change` (select). |

### Prelude

`use vgui::prelude::*` brings into scope:

`view`, `css`, `tw`, `create_signal`, `create_memo`, `create_effect`,
`ReadSignal`, `WriteSignal`, `click`, `mount`, `text_input`, `checkbox`,
`radio`, `range_input`, `file_input`, `input_cb`, `str_change_cb`,
`bool_change_cb`, `f64_change_cb`, `files_cb`, `CheckboxProps`, `FileProps`,
`RadioProps`, `RangeProps`, `TextInputProps`, `TextKind`, and all of
`gpui::prelude::*`.

## [vgui-view](https://docs.rs/vgui-view)

The `view!` proc-macro crate. No runtime API — the macro expands at compile
time.

## [vgui-css](https://docs.rs/vgui-css)

The `css!` proc-macro crate. No runtime API — the macro expands at compile
time.

## [vgui-tailwind](https://docs.rs/vgui-tailwind)

The `tw!` proc-macro crate. No runtime API — the macro expands at compile
time.

## Building Locally

To generate rustdoc for all crates locally:

```bash
cargo doc --no-deps --workspace --open
```

To build and serve this mdBook:

```bash
cargo install mdbook
cd book
mdbook serve --open
```

The book will be available at `http://localhost:3000`.
