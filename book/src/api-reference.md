# API Reference

The API reference is generated automatically by rustdoc. The links below point
to each crate's generated documentation:

## [vgui](https://docs.rs/vgui)

The main crate. Key exports:

### Macros

| Macro       | Source crate         | Description                                      |
| ----------- | -------------------- | ------------------------------------------------ |
| `view!`     | `vgui-view`          | JSX-like view markup.                            |
| `css!`      | `vgui-css`           | CSS-in-Rust style declarations.                  |
| `tw!`       | `vgui-tailwind`      | Tailwind utility class compilation.              |
| `variants!` | `vgui`               | Component variant system (base + dimension styles). |
| `theme!`    | `vgui`               | CSS variable theme builder.                      |
| `twc!`      | `vgui`               | Dynamic Tailwind class composition at runtime.   |

### Reactivity

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `create_signal&lt;T&gt;(initial) -> (ReadSignal&lt;T&gt;, WriteSignal&lt;T&gt;)` | Creates a reactive signal. |
| `create_memo&lt;T&gt;(f) -> ReadSignal&lt;T&gt;`   | Creates a derived, cached value.         |
| `create_effect(f)`                     | Creates a side effect that re-runs on dep change. |
| `ReadSignal&lt;T&gt;::get() -> T`            | Reads value, registers dependency.       |
| `ReadSignal&lt;T&gt;::get_with(cx) -> T`     | Reads value without tracking.            |
| `WriteSignal&lt;T&gt;::set(cx, value)`       | Sets value, notifies if changed.         |
| `WriteSignal&lt;T&gt;::update(cx, f) -> R`   | Mutates value in place, notifies if changed. Returns the closure's result. |
| `next_auto_id() -> u64`                | Stable per-render element id (used by `view!`). |

### Mounting

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `mount(cx, render_fn) -> Entity<VguiRoot>` | Creates the root entity and reactive scope. |
| `VguiRoot`                             | The gpui entity owning the reactive scope. |

### Router

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `create_router(initial: &str) -> Router` | Creates a router backed by a path signal. |
| `Router::navigate(cx, path)`           | Updates the path signal, triggering re-render. |
| `Router::path() -> String`             | Reactive read of current path.           |
| `Router::path_with(cx) -> String`      | Non-tracking read of current path.       |
| `Router::path_signal() -> ReadSignal<String>` | The underlying read signal.       |
| `Router::match_route(pattern) -> Option<RouteMatch>` | Match a single pattern against current path. |
| `Router::render(cx, routes, fallback) -> E` | Render first matching route from `&[(&str, F)]`. |
| `RouteMatch`                           | `{ pattern, path, params: HashMap<String, String> }`. |
| `match_pattern(pattern, path) -> Option<RouteMatch>` | Standalone pattern match with `:param` and `*` wildcard. |
| `build_path(pattern, params) -> String` | Substitute `:param` placeholders with values. |

See [Router](./concepts/router.md) for a full guide.

### Context

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `Context&lt;T&gt;`                           | Zero-sized typed marker for a context key. |
| `use_context(&Context&lt;T&gt;) -> Option&lt;T&gt;` | Read nearest ancestor provider value.    |
| `use_context_or(&Context&lt;T&gt;, \|\| T) -> T` | Read nearest provider, or fallback default. |
| `provide_context(&Context&lt;T&gt;, value) -> ProviderGuard` | RAII manual provider (pops on drop). |
| `ProviderGuard`                        | Guard returned by `provide_context`.     |

See [Context & Provider](./elements/context.md) for a full guide.

### Control flow

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `show(when, then, fallback) -> AnyElement` | Conditional render with fallback.    |
| `show_when(when, then) -> AnyElement`  | Conditional render (no fallback).        |
| `for_each(items, child_fn) -> AnyElement` | List rendering.                       |
| `for_each_or(items, fallback, child_fn) -> AnyElement` | List rendering with fallback. |
| `progress(value, max) -> Div`          | Progress bar.                            |
| `meter(value, min, max, low, high, optimum) -> Div` | Meter gauge.                |
| `details(open, summary, content) -> AnyElement` | Collapsible container.           |

### Overlays

| Item                                   | Description                              |
| -------------------------------------- | ---------------------------------------- |
| `portal(content, priority) -> AnyElement` | Portal floating layer at a given priority. |
| `dialog(open, on_close, content) -> AnyElement` | Modal dialog with portal, click-outside, escape. |
| `floating(position, content) -> AnyElement` | Positioned floating element.       |

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
| `select_with_options(SelectProps, R) -> Stateful<Div>` | Select with per-option content renderer closure. |
| `datalist`                             | Autocomplete suggestions for text inputs (via `<datalist>` element). |

### Props structs

| Struct           | Fields                                                        |
| ---------------- | ------------------------------------------------------------- |
| `TextInputProps` | `kind`, `multiline`, `value`, `placeholder`, `disabled`, `readonly`, `min`, `max`, `step`, `on_input`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `TextAreaProps`  | `value`, `placeholder`, `disabled`, `readonly`, `on_input`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `CheckboxProps`  | `checked`, `disabled`, `on_change`                            |
| `RadioProps`     | `checked`, `disabled`, `on_change`                            |
| `RangeProps`     | `value`, `min`, `max`, `step`, `disabled`, `on_change`, `style`, `class`, `id`, `tabindex` |
| `FileProps`      | `multiple`, `on_change`                                       |
| `SelectProps`    | `options: Vec<(String, String)>`, `groups: Vec<SelectGroup>`, `value`, `multiple`, `disabled`, `on_change` |

### Enums

| Enum         | Variants                                                              |
| ------------ | --------------------------------------------------------------------- |
| `TextKind`   | `Text`, `Password`, `Search`, `Email`, `Url`, `Tel`, `Number`, `Date`, `DateTime`, `Time`, `Month`, `Week`, `Color` |
| `Breakpoint` | `Sm`, `Md`, `Lg`, `Xl`                                                |

### Refs

| Item                | Description                                                  |
| ------------------- | ------------------------------------------------------------ |
| `NodeRef::new()`    | Creates a new `NodeRef` handle.                              |
| `NodeRef::focus()`  | Focuses the bound element.                                   |
| `NodeRef::scroll_to_bottom()` | Scrolls the bound element to the bottom.            |
| `NodeRef::bounds()` | Returns the bounds of the bound element.                     |

See [Refs & NodeRef](./elements/refs.md) for the full API.

### Style types

| Item                | Description                                                  |
| ------------------- | ------------------------------------------------------------ |
| `Css`               | Output of `css!`. Wraps a `FnOnce(&mut StyleRefinement)`.    |
| `TwStyle`           | Output of `tw!`/`twc!`. Holds `base`, `hover`, `focus`, `active` closures. |
| `ApplyStyle&lt;E&gt;`     | Trait for applying a style to an element.                    |
| `TwClass`           | Builder for composing Tailwind classes (`new()`, `add()`, `add_if()`). |
| `TwClassSource`     | Trait implemented for `&str`, `String`, `Option&lt;T&gt;`, `TwClass`. |
| `IntoTwStyle`       | Trait converting class sources into `TwStyle`.               |
| `tw_dynamic(classes: &str)` | Runtime Tailwind class interpreter (counterpart to `tw!`). |
| `TwAnimation`       | Animation definition type.                                   |
| `TwTransition`      | Transition definition type.                                  |
| `Easing`            | Easing function enum for animations/transitions.             |
| `Theme`             | CSS variable theme store (built by `theme!`).                |
| `CssValue`          | Enum of CSS value kinds (color, length, number, keyword).    |
| `set_theme(theme)`  | Installs a theme into the thread-local store.                |
| `with_theme(theme, f)` | Runs a closure with a temporary theme.                     |
| `Spread&lt;E&gt;`         | Trait for spreading props onto a built-in element.           |

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
| `intercept_keyboard_events()` | Installs window-level keyboard event listeners (WASM-only; no-op on native). Required in every WASM `start()`. |

### Prelude

`use vgui::prelude::*` brings into scope:

`view`, `css`, `tw`, `twc`, `tw_dynamic`, `variants`, `theme`, `set_theme`,
`with_theme`, `create_signal`, `create_memo`, `create_effect`, `create_router`,
`ReadSignal`, `WriteSignal`, `RouteMatch`, `Router`, `Breakpoint`, `click`,
`mount`, `Context`, `use_context`, `use_context_or`, `provide_context`,
`NodeRef`, `KeyboardEvent`, `PointerEvent`, `PointerType`, `ResizeEvent`,
`WheelEvent`, `checkbox`, `radio`, `range_input`, `file_input`, `input_cb`,
`bool_change_cb`, `f64_change_cb`, `files_cb`, `CheckboxProps`,
`FileProps`, `RadioProps`, `RangeProps`, `TextInputProps`, `TextKind`, `Theme`,
`TwClass`, `TwClassSource`, `IntoTwStyle`, `portal`, `floating`, and all of
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

## [vgui-tailwind-core](https://docs.rs/vgui-tailwind-core)

Shared library crate (not a proc-macro) providing the parse logic and class
tables used by both `tw!` (compile-time) and `tw_dynamic` (runtime). Has no
`gpui` dependency.

## Building Locally

The entire documentation site — WASM demos, this mdBook, and the rustdoc API
reference — is built with a single command:

```bash
scripts/build_docs.sh
```

The output is written to `book/book/`. To build and immediately serve it:

```bash
scripts/build_docs.sh --serve
```

The site is then available at `http://127.0.0.1:8080`.

Prerequisites: `mdbook`, `wasm-bindgen`, and the nightly Rust toolchain (the
repo pins nightly via `rust-toolchain.toml`).
