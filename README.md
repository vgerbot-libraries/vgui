# vgui

A declarative, reactive GUI framework for Rust, built on top of
[`gpui`](https://github.com/zed-industries/zed) (the GPU-accelerated UI toolkit
behind the [Zed](https://zed.dev) editor).

`vgui` brings a familiar web-style authoring experience to native Rust
desktop apps and web (WASM) applications:

- **JSX-like views** via the `view!` macro — elements, attributes, children,
  fragments and component invocation, all in ergonomic markup.
- **CSS-in-Rust** via the `css!` macro — write real CSS declarations
  (`color: #fff; padding: 8px;`) that compile down to `gpui` style refinements.
- **Tailwind-style classes** via the `tw!` macro — `class="p-2 rounded
  hover:bg-[#000088]"` works out of the box, with `hover:`, `focus:` and
  `active:` variants.
- **Fine-grained reactivity** inspired by [SolidJS](https://www.solidjs.com/):
  `create_signal`, `create_memo` and `create_effect` track dependencies
  automatically and re-render only what changes.
- **Control-flow components** `<Show>` and `<For>` for conditional and list
  rendering with optional fallbacks.
- **ARIA semantic attributes** — `role` and `aria:name` on all elements for
  accessibility.
- **Context & Provider** — SolidJS-style dependency injection via `Context<T>`
  and `<Provider>`.
- **NodeRef** — imperative handles for focus, scroll, and bounds queries.
- **Focus management** — focus trap, focus restore, and roving tabindex for
  keyboard navigation.
- **Overlays** — `portal`, `dialog`, and `floating` for modals and floating
  elements on a separate layer.
- **Component variants** — the `variants!` macro declares base + dimension
  styles that compose into typed, `Copy` variant structs.
- **Animations & transitions** — `tw!` `animate-*` and `transition-*` utilities
  with easing and keyframe support.
- **Responsive breakpoints** — `sm:`, `md:`, `lg:`, `xl:` prefixes for
  viewport-conditional styling.
- **Dynamic class composition** — the `twc!` macro composes conditional
  Tailwind classes at runtime.
- **CSS variables & theming** — `theme!` macro, `var(--name)` in `css!`, and
  `set_theme()` for reactive light/dark switching.
- **SPA router** — signal-driven router with `:param` pattern matching and
  wildcard routes.
- **Dual-target** — every example compiles and runs natively (Linux) and on
  the web (WASM) with a single codebase.

## Status

`vgui` is early-stage, experimental software. The API is not yet stable and
breaking changes should be expected between releases. It is, however, fun to
build with — see the [examples](#examples).

## Table of contents

- [Workspace layout](#workspace-layout)
- [Prerequisites](#prerequisites)
- [System libraries](#system-libraries)
- [Building](#building)
- [Examples](#examples)
- [Usage](#usage)
- [Reactivity](#reactivity)
- [Styling](#styling)
- [Control flow](#control-flow)
- [Tables](#tables)
- [Input elements](#input-elements)
- [ARIA & Accessibility](#aria--accessibility)
- [Context & Provider](#context--provider)
- [Refs & NodeRef](#refs--noderef)
- [Focus Management](#focus-management)
- [Overlays (Portal/Dialog/Floating)](#overlays-portaldialogfloating)
- [Component Variants](#component-variants)
- [Animations & Transitions](#animations--transitions)
- [Responsive Breakpoints](#responsive-breakpoints)
- [Dynamic Classes](#dynamic-classes)
- [CSS Variables & Theming](#css-variables--theming)
- [Router](#router)
- [Web / WASM](#web--wasm)
- [Documentation](#documentation)

## Workspace layout

This repository is a Cargo workspace:

| Crate                  | Kind        | Description                                                    |
| ---------------------- | ----------- | ------------------------------------------------------------- |
| `vgui`                 | lib         | The main crate: reactivity, root mounting, styling traits, widgets. |
| `vgui-view`            | proc-macro  | The `view!` macro.                                            |
| `vgui-css`             | proc-macro  | The `css!` macro.                                             |
| `vgui-tailwind`        | proc-macro  | The `tw!` macro and the Tailwind class registry.              |
| `vgui-tailwind-core`   | lib         | Shared class-parse/tables for `tw!` and `tw_dynamic` (no gpui dep). |

Eleven example binaries live under [`examples/`](examples) — see
[Examples](#examples) below and the [book's Examples section](book/src/SUMMARY.md)
for the full list.

## Prerequisites

- A recent **nightly** Rust toolchain (pinned via `rust-toolchain.toml`).
- [`pkg-config`](https://www.freedesktop.org/wiki/Software/pkg-config/).
- A C/C++ build toolchain: `build-essential`, `cmake`.
- `libclang` for `bindgen` (used by `gpui` and its transitive deps).

## System libraries

`vgui` does not depend on system libraries directly, but `gpui` does — it
talks to the native window system (Wayland and X11 on Linux, Cocoa/Metal on
macOS, Win32/DirectX on Windows). The following development packages are
required to build `gpui` on a Debian/Ubuntu Linux host:

```bash
sudo apt-get install -y \
  build-essential \
  cmake \
  pkg-config \
  libclang-dev \
  libssl-dev \
  libzstd-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libglib2.0-dev \
  libgtk-3-dev \
  libasound2-dev \
  libdbus-1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libx11-dev \
  libxext-dev \
  libxrandr-dev \
  libxinerama-dev \
  libxcursor-dev \
  libxi-dev \
  libwayland-dev \
  libgl-dev \
  libegl-dev
```

### Other distributions

<details>
<summary>Fedora / RHEL</summary>

```bash
sudo dnf install -y \
  clang-devel openssl-devel libzstd-devel fontconfig-devel freetype-devel \
  glib2-devel gtk3-devel alsa-lib-devel dbus-devel \
  libxkbcommon-devel libxkbcommon-x11-devel \
  libX11-devel libXext-devel libXrandr-devel libXinerama-devel \
  libXcursor-devel libXi-devel wayland-devel \
  mesa-libGL-devel mesa-libEGL-devel \
  cmake pkg-config
```

</details>

<details>
<summary>Arch Linux</summary>

```bash
sudo pacman -S --needed \
  base-devel clang cmake pkgconf \
  openssl zstd fontconfig freetype2 glib2 gtk3 alsa-lib dbus \
  libxkbcommon libxkbcommon-x11 \
  libx11 libxext libxrandr libxinerama libxcursor libxi wayland \
  mesa
```

</details>

<details>
<summary>macOS</summary>

No extra system libraries are required beyond Xcode Command Line Tools:

```bash
xcode-select --install
```

`gpui` uses Metal/Cocoa natively on macOS.

</details>

<details>
<summary>Windows</summary>

Build with the MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
and the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
The Windows SDK provides the rest.

</details>

## Building

```bash
git clone https://github.com/vgerbot-libraries/vgui.git
cd vgui
cargo build
```

The first build compiles `gpui` and its graphics backends, so expect a longer
initial compile. Subsequent incremental builds are fast.

### Web (WASM)

To check the WASM target:

```bash
cargo +nightly check --target wasm32-unknown-unknown
```

See [Web / WASM](#web--wasm) below for building and serving examples in the
browser.

## Examples

Eleven end-to-end examples live under [`examples/`](examples):

| Example | Command | Description |
| ------- | ------- | ----------- |
| Counter | `cargo run -p vgui-counter` | Signals, `create_memo`, `<Show>`, `twc!` class composition. |
| Todo List | `cargo run -p vgui-todolist` | `<For>` with fallback, `css!` styling, filtering. |
| Inputs Demo | `cargo run -p vgui-inputs` | All `<input>` types with live echo. |
| Tags Demo | `cargo run -p vgui-tags-demo` | HTML tag coverage, tables, progress, details, dialog. |
| Theming | `cargo run -p vgui-theming` | CSS variables, `theme!` macro, light/dark switching. |
| Variants | `cargo run -p vgui-variants` | `variants!` macro, component variant system. |
| Focus Management | `cargo run -p vgui-focus-management` | Focus trap, restore, roving tabindex. |
| ref Demo | `cargo run -p vgui-ref-demo` | `NodeRef` imperative handles (focus, scroll, bounds). |
| Context | `cargo run -p vgui-context` | Context API, `<Provider>`, `use_context`. |
| Animation | `cargo run -p vgui-animation` | Animations, transitions, keyframes. |
| Select Test | `cargo run -p vgui-select-test` | Select with grouped/multiple options, datalist. |

## Usage

Add `vgui` (and `gpui`) to your `Cargo.toml`. Both are used as git
dependencies — neither crate is published to crates.io:

```toml
[dependencies]
vgui = { git = "https://github.com/vgerbot-libraries/vgui" }
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-platform = { git = "https://github.com/zed-industries/zed", package = "gpui_platform" }
```

`vgui` can also be used as a path dependency if you have a local checkout.

Then author a window with `view!`:

```rust
use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    view! {
        <div class="flex flex-col gap-3 p-4 w-[500px] h-[500px] justify-center items-center text-white">
            <span>{format!("count = {}", count.get())}</span>
            <button
                class="p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"
                on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
            >
                {"Increment"}
            </button>
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    });
}
```

The `vgui::prelude::*` import brings in `view!`, `css!`, `tw!`, `twc!`,
`variants!`, `theme!`, the reactive primitives, `click`, `mount`, context API,
`NodeRef`, input widget constructors, styling types, and overlay helpers.

## Reactivity

`vgui` uses a SolidJS-style reactivity model on top of `gpui` entities.

| Primitive            | Purpose                                                    |
| -------------------- | --------------------------------------------------------- |
| `create_signal(v)`   | Returns `(ReadSignal, WriteSignal)` for a piece of state. |
| `ReadSignal::get()`  | Reads the value and registers the current scope as a dep. |
| `ReadSignal::get_with(cx)` | Reads the value without tracking a dependency.       |
| `WriteSignal::update(cx, f)` | Mutates the value and notifies dependents.        |
| `create_memo(f)`     | A derived, cached value that recomputes when deps change. |
| `create_effect(f)`   | Runs a side effect whenever its deps change.              |

Signals are read inside `view!` interpolations (`{count.get()}`) and inside
`create_memo` / `create_effect` closures; `vgui` tracks which signals each
scope reads and re-runs only that scope when they change.

## Styling

Two complementary macros are available.

### `css!` — CSS-in-Rust

```rust
view! {
    <div style={css! {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 20px;
        background: rgb(30, 30, 30);
        color: #fff;
    }}>
        <span>{"Hello"}</span>
    </div>
}
```

Supported value forms include `px`, `rem`, `%`, `auto`, hex colors (`#fff`,
`#ff0000`, `#0000ff80`), `rgb(...)`/`rgba(...)`, and named colors
(`black`, `white`, `red`, ...). Pseudo-state refinements go on the element
itself via the `hover`, `active` and `focus` attributes:

```rust
view! {
    <button
        style={css! { padding: 8px 16px; background: #dc2626; border-radius: 4px; }}
        hover={css! { background: #b91c1c; }}
        on:click={click(|_cx| {})}
    >
        {"Delete"}
    </button>
}
```

### `tw!` / `class=` — Tailwind-style classes

```rust
view! {
    <div class="flex flex-col gap-3 p-4 bg-[#505050] w-[500px] h-[500px] justify-center items-center text-white">
        <button class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded">{"Click"}</button>
    </div>
}
```

The `class="..."` attribute is expanded through `tw!` and supports the
common spacing, sizing, color, flex, layout and typography utilities, plus
`hover:`, `focus:` and `active:` variants. Arbitrary values are written as
`bg-[#0000ff]`, `w-[500px]`, etc.

### Dynamic class composition (`twc!`)

The `twc!` macro composes conditional Tailwind classes at runtime — a base
string plus `Option<&str>` arguments that are included only when `Some`:

```rust
<button class={twc!(
    "p-2 rounded text-white",
    (delta > 0).then_some("bg-blue-500"),
    (delta < 0).then_some("bg-red-500")
)}>
```

See [Dynamic Classes](#dynamic-classes) below and the
[book](book/src/styling/tailwind-classes.md#dynamic-class-composition).

### Responsive breakpoints

`sm:`, `md:`, `lg:`, `xl:` prefixes apply styles only when the viewport width
meets the threshold. See the
[book](book/src/styling/tailwind-classes.md#responsive-breakpoints).

## Control flow

### `<Show>` — conditional rendering

```rust
view! {
    <Show when={count.get() > 0}>
        <span>{"positive"}</span>
    </Show>

    <Show when={count.get() & 1 == 1} fallback={view! { <span>{"even"}</span> }}>
        <span>{"odd"}</span>
    </Show>
}
```

### `<For>` — list rendering

```rust
view! {
    <For each={todos.get()} fallback={view! { <div>{"No todos."}</div> }}>
        {move |todo: Todo, _i: usize| todo_item(todo, set_todos.clone())}
    </For>
}
```

The child of `<For>` must be a closure `move |item, index| -> impl IntoElement`.

### Custom components

Any uppercase-tag element is treated as a component call. With no attributes
and a single child it expands to `Component(child)`; with attributes it
expands to a struct initializer `Component { field: value, ... }`. Event
attributes (`on:click`) map to `on_click`, etc.

```rust
fn greeting(name: &'static str) -> impl gpui::IntoElement {
    view! { <span>{format!("Hello, {name}!")}</span> }
}

view! {
    <div>
        <Greeting name={"world"} />
    </div>
```

## Tables

Table tags are supported via flex layout (gpui has no native table layout).
`<table>`, `<thead>`, `<tbody>`, and `<tfoot>` stack their children vertically
(`flex_col`); `<tr>` is a horizontal flex row, full width; `<td>` and `<th>`
are `flex_1` cells that share row width equally. `<th>` defaults to bold +
centered text. `colspan` is mapped to `flex_grow`, so a cell with
`colspan={2}` grows 2× relative to `colspan=1` cells. `rowspan`, `<colgroup>`,
and `<col>` are accepted (they compile) but have no visual effect — there is
no content-based column sizing in a flex layout. For specific column widths,
apply `class="w-[200px]"` or a `style` width on individual cells.

```rust
view! {
    <table class="w-full">
        <thead>
            <tr class="bg-[#333]">
                <th class="p-2 text-white">{"Name"}</th>
                <th class="p-2 text-white">{"Age"}</th>
                <th class="p-2 text-white">{"City"}</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td class="p-2">{"Alice"}</td>
                <td class="p-2">{"30"}</td>
                <td class="p-2">{"Beijing"}</td>
            </tr>
            <tr>
                <td class="p-2" colspan={2u32}>{"Bob (spanned 2 cols)"}</td>
                <td class="p-2">{"Shanghai"}</td>
            </tr>
        </tbody>
    </table>
}
```

## Input elements

`<input>` is a void element (self-closing — both `<input type="text">` and
`<input type="text" />` are accepted). The `type` attribute selects the
widget kind; if omitted, `type="text"` is assumed.

### Text-based types

`text`, `password`, `search`, `email`, `url`, `tel`, `number`, `date`,
`datetime-local`, `time`, `month`, `week`, `color` all render a text field
with full cursor, selection, keyboard editing, clipboard (Ctrl+A/C/V/X), and
IME (CJK composition) support. Date, time, and color types have picker
popups.

```rust
view! {
    <input
        type="text"
        placeholder="Name"
        on:input={move |v: &str, cx: &mut App| set_name.set(cx, v.to_string())}
    />
}
```

Supported attributes: `value`, `placeholder`, `disabled`, `readonly`, `min`,
`max`, `step` (for `number`), `on:input` (fires on every keystroke), `on:change`
(fires on Enter/blur), plus `style`, `class`, `hover`, `active`, `focus`, `id`.

### Checkbox & Radio

```rust
view! {
    <input type="checkbox" checked={done.get()} on:change={move |v: bool, cx: &mut App| set_done.set(cx, v)} />
    <input type="radio" checked={sel.get() == 0} on:change={move |_v: bool, cx: &mut App| set_sel.set(cx, 0)} />
}
```

### Range slider

```rust
view! {
    <input type="range" min={0.0f64} max={100.0f64} step={1.0f64} value={vol.get()}
        on:change={move |v: f64, cx: &mut App| set_vol.set(cx, v)} />
}
```

### File picker

```rust
view! {
    <input type="file" value="Browse..." multiple={true}
        on:change={move |paths: Vec<std::path::PathBuf>, _cx: &mut App| {
            eprintln!("selected: {:?}", paths);
        }} />
}
```

### Submit / Button / Reset

These render as clickable buttons (like `<button>`). The `value` attribute
becomes the button label. `on:click` wires the handler. Inside a `<form>`,
submit and reset buttons auto-invoke the form's `on:submit` / `on:reset`
handler.

### Hidden

`<input type="hidden">` renders nothing (`gpui::Empty`).

### Select

`<select>` supports `options` (a `Vec<(String, String)>` of value/label
pairs), `groups` (grouped options with `<optgroup>`-style labels), `multiple`
(multi-select mode), `value`, `disabled`, and `on:change`. See the
[book](book/src/elements/inputs.md) for details.

### Datalist

`<datalist>` provides autocomplete suggestions for text inputs. Register an
`id` and `options={Vec<String>}`, then reference it from an `<input>` via
`list=<id>`.

### Form

`<form>` wraps children in a form context. `on:submit` and `on:reset` take
`FnMut(&mut App)` closures. Child submit/reset buttons auto-invoke the form
handler. Pressing Enter in a single-line text input triggers `on:submit`.

## ARIA & Accessibility

All elements support `role` and `aria:name` attributes for ARIA roles,
labels, and states. See the
[book](book/src/concepts/view-macro.md#attribute-categories).

## Context & Provider

`Context<T>` is a zero-sized typed marker; `<Provider context={..}
value={..}>` pushes a value for descendants to consume via `use_context` or
`use_context_or`. See [Context & Provider](book/src/elements/context.md).

## Refs & NodeRef

`ref={node_ref}` binds a `NodeRef` handle to an element for imperative
operations: `focus()`, `scroll_to_bottom()`, `bounds()`. See
[Refs & NodeRef](book/src/elements/refs.md).

## Focus Management

vgui provides focus trap (for modals), focus restore (return focus when an
overlay closes), and roving tabindex (for radio groups with arrow-key
navigation). See the [Focus Management example](book/src/examples/focus-management.md).

## Overlays (Portal/Dialog/Floating)

`portal(content, priority)` renders content on a floating layer at a given
priority. `dialog(open, on_close, content)` wraps content in a modal dialog
with click-outside and Escape dismissal. `floating(position, content)`
renders a positioned floating element. See the
[book](book/src/api-reference.md#overlays).

## Component Variants

The `variants!` macro declares a base style plus dimensions (e.g., `variant`
for color, `size` for padding). It generates typed enum variants and a
`Copy` struct that implements `ApplyStyle`. See the
[Variants example](book/src/examples/variants.md).

## Animations & Transitions

`tw!` supports `animate-*` and `transition-*` utilities with easing functions
and keyframe definitions. See [Animations & Transitions](book/src/styling/animations.md).

## Responsive Breakpoints

Four prefixes — `sm:` (≥640px), `md:` (≥768px), `lg:` (≥1024px), `xl:`
(≥1280px) — apply styles only when the viewport width meets the threshold.
See the [book](book/src/styling/tailwind-classes.md#responsive-breakpoints).

## Dynamic Classes

The `twc!` macro composes conditional Tailwind classes at runtime. `TwClass`
provides a builder API (`add()`, `add_if()`). `tw_dynamic(classes: &str)`
interprets class strings at runtime. See the
[book](book/src/styling/tailwind-classes.md#dynamic-class-composition).

## CSS Variables & Theming

The `theme!` macro builds a `Theme` from `--name: value` declarations.
`var(--name)` in `css!` resolves against the thread-local theme store at
runtime. `set_theme()` installs a theme reactively — toggling a signal
re-runs render and re-resolves all `var()` references. See the
[Theming example](book/src/examples/theming.md).

## Router

`create_router(initial)` creates a signal-driven `Router`. `navigate(cx,
path)` updates the path; `match_pattern` supports `:param` segments and `*`
wildcards; `render(cx, routes, fallback)` dispatches the first matching
route. See [Router](book/src/concepts/router.md).

## Web / WASM

vgui is dual-target: every example compiles for both native and
`wasm32-unknown-unknown`. On WASM, use `gpui_platform::single_threaded_web()`
(not `application()`), and call `vgui::intercept_keyboard_events()` in every
`start()` function to prevent the browser from stealing keyboard focus.

Build WASM assets with:

```bash
scripts/build_wasm.sh <name>
```

See the [writing-examples rule](.agents/rules/writing-examples.md) for the
dual-target pattern.

## Documentation

The full documentation is an mdBook built with:

```bash
scripts/build_docs.sh
```

The output is written to `book/book/`. To build and serve:

```bash
scripts/build_docs.sh --serve
```

The site is then available at `http://127.0.0.1:8080`.

## License

Licensed under the [MIT License](LICENSE).
