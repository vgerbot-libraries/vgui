# vgui

A declarative, reactive GUI framework for Rust, built on top of
[`gpui`](https://crates.io/crates/gpui) (the GPU-accelerated UI toolkit behind
the [Zed](https://zed.dev) editor).

`vgui` brings a familiar web-style authoring experience to native Rust
desktop apps:

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
- [Input elements](#input-elements)

## Workspace layout

This repository is a Cargo workspace:

| Crate                  | Kind        | Description                                                    |
| ---------------------- | ----------- | ------------------------------------------------------------- |
| `vgui`                 | lib         | The main crate: reactivity, root mounting, styling traits.    |
| `vgui-view`            | proc-macro  | The `view!` macro.                                            |
| `vgui-css`             | proc-macro  | The `css!` macro.                                             |
| `vgui-tailwind`        | proc-macro  | The `tw!` macro and the Tailwind class registry.              |
| `examples/counter`     | bin         | A minimal counter demonstrating signals and `Show`.           |
| `examples/inputs`     | bin         | All `<input>` types: text, password, checkbox, radio, range, file, etc. |

## Prerequisites

- A recent stable **Rust** toolchain (edition 2021, ≥ 1.74 recommended).
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
git clone https://github.com/local/vgui.git
cd vgui
cargo build
```

The first build compiles `gpui` and its graphics backends, so expect a longer
initial compile. Subsequent incremental builds are fast.

## Examples

Two end-to-end examples live under [`examples/`](examples).

### Counter

A minimal app with two buttons, a derived `memo`, and conditional `<Show>`
blocks.

```bash
cargo run -p vgui-counter
```

### Todo list

A slightly larger app: add/toggle/delete todos, filter by status, and a
remaining-count memo. Demonstrates `<For>` with a fallback, `css!` styling,
and `hover`/`active` refinements.

```bash
cargo run -p vgui-todolist
```

### Inputs

A comprehensive demo of every `<input>` type: text with live echo, password,
checkbox, radio group, range slider, number, date, file picker, submit
button, and hidden.

```bash
cargo run -p vgui-inputs
```

## Usage

Add `vgui` (and `gpui`) to your `Cargo.toml`:

```toml
[dependencies]
vgui = "0.1"
gpui = "0.2"
```

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

The `vgui::prelude::*` import brings in `view!`, `css!`, `tw!`, the reactive
primitives, `click`, and `mount`.

## Reactivity

`vgui` uses a SolidJS-style reactivity model on top of `gpui` entities.

| Primitive            | Purpose                                                    |
| -------------------- | --------------------------------------------------------- |
| `create_signal(v)`   | Returns `(ReadSignal, WriteSignal)` for a piece of state. |
| `ReadSignal::get()`  | Reads the value and registers the current scope as a dep. |
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
}

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
IME (CJK composition) support.

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

> **Note:** `date`, `datetime-local`, `time`, `month`, `week`, and `color`
> are text-entry v1 — they use a format placeholder and light validation
> with no calendar/color popup. Popups are future work.

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
becomes the button label. `on:click` wires the handler. `submit` and `reset`
have no form semantics in vgui v1 — they behave as buttons.

### Hidden

`<input type="hidden">` renders nothing (`gpui::Empty`).

## License

Licensed under the [MIT License](LICENSE).
