---
name: vgui-authoring
description: Write vgui UI — view!, css!, tw!, signals, stores, control flow, components, events. Use when creating or editing examples, app views, custom components, styling, overlays, router, or WASM/native dual-target UI code.
---

# vgui Authoring

vgui is a SolidJS-inspired reactive GUI on **gpui**. It is **not** the DOM, React, Leptos, Dioxus, or CSS-in-browser. Unknown Tailwind classes are silently skipped. Unknown `css!` properties are compile errors.

Read current APIs from `book/src/` and `crates/vgui/src/prelude.rs`. Do not invent HTML/CSS/JS APIs.

Also follow `.agents/rules/writing-examples.md`, `cross-platform.md`, and `final-state-only.md`.

## Default app shape

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    view! {
        <div class="w-full h-full flex items-center justify-center">
            <button
                class="p-2 rounded bg-[#0000ff] hover:bg-[#000088] text-white"
                on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
            >
                {format!("{}", count.get())}
            </button>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);
    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() { run(); }

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
```

Hard rules:

- Import `vgui::prelude::*`. Do not reach for React/Solid runtime APIs.
- Outermost node must be `w-full h-full` (or `width: 100%; height: 100%` in `css!`). Never fixed px on the root.
- WASM: `single_threaded_web()`, `run_embedded` + `mem::forget`, `web_init()`, `intercept_keyboard_events()`. Never `application()` on WASM.
- Native: `application()` + `run()`.
- `on:click` for simple handlers: wrap with `click(move |cx| ...)`.
- Text children that are not string literals go in `{...}`: `{format!(...)}`, `{label}`.

## Reactivity (hooks-order)

`create_signal` / `create_memo` / `create_effect` / `create_store` / `on_cleanup` must be called **inside `app()` (or from it during render)** in the **same order every render**. They are slot-indexed.

| API | Use |
| --- | --- |
| `create_signal(t)` | Flat value. `T: Clone + PartialEq + 'static`. Writes notify only if changed. |
| `create_store(t)` | Aggregate state. `T: Clone + 'static` (no `PartialEq`). Writes always notify. Slice with `store.select(|s| ...)`. |
| `create_memo(f)` | Cached derived `ReadSignal`. Clone captured signals into the closure. |
| `create_effect(f)` | Side effects. |
| `on_cleanup(f)` | Runs when the owning child scope is disposed (`<Switch>` / `<Index>` / routes). |

```rust
let (count, set_count) = create_signal(0i32);
let doubled = create_memo({
    let count = count.clone();
    move || count.get() * 2
});
set_count.update(cx, |n| *n += 1); // needs &mut App
let _ = count.get();               // tracks
let _ = count.get_with(cx);        // no track
```

Do **not** put `create_*` behind conditionals or loops. Use `<Show>` / `<Switch>` / `<Index>` for conditional *UI* and per-branch state.

`ReadSignal` / `WriteSignal` are `Clone`. Clone before moving into closures or child functions.

## `view!`

- Lowercase tags → built-in HTML mapped to `gpui::div()` (or widgets). See `book/src/elements/builtin-elements.md`.
- Uppercase tags → components. No attrs → `Foo()` / `Foo(child)` / `Foo(vec![...])`. With attrs → struct literal `Foo { field, children, on_click, ... }`.
- Fragments: `<>...</>`.
- Interpolation: `{expr}` via `into_child`.
- Special tags: `<Show>`, `<For>`, `<Switch>`/`<Match>`, `<Index>`, `<Provider>`.

```rust
<Show when={count.get() > 0} fallback={view! { <span>{"zero"}</span> }}>
    <span>{"positive"}</span>
</Show>

<For each={todos.get()} fallback={view! { <div>{"empty"}</div> }}>
    {move |todo: Todo, _i: usize| todo_item(todo)}
</For>
```

`<For>` child must be **one** `{closure}` with `|item, index|`. It re-renders all items; no keyed DOM diff.

## Styling

Prefer `class="..."` (`tw!`) for utilities. Use `css!` for properties `tw!` cannot express. Use `twc!` when classes depend on runtime conditions.

```rust
<div class="flex gap-3 p-4 text-white bg-[#505050]">
<button class={twc!("p-2 rounded", cond.then_some("bg-red-500"))}>
<div style={css! { display: flex; padding: 8px 16px; color: #fff; }}>
<div hover={css! { background: #000088; }}>
```

Supported `tw!` prefixes: `hover:`, `focus:`, `active:`, `sm:`/`md:`/`lg:`/`xl:`. Arbitrary values: `bg-[#0000ff]`, `w-[500px]`. Opacity: `bg-blue-500/50`.

Do **not**:

- Write `&:hover { }` inside `css!` — use `hover={css! { ... }}`.
- Assume full Tailwind v3/v4 / browser CSS. Unsupported `css!` props fail compilation; unknown `tw!` classes are dropped.
- Use `className`, `sx`, `styled-components`, or CSS files.

Component variants: `variants! { Button { base => css!{...}, size { sm => css!{...} } } }` then apply the generated `*Variants` struct.

Theming: `theme!`, `var(--name)` in `css!`, `set_theme()` / `with_theme()`.

## Components

Uppercase tag + attributes → struct init. `on:click` maps to field `on_click`. `type`/`for`/`ref` map to `r#type` / `r#for` / `r#ref`.

```rust
fn IconButton(label: &'static str) -> impl gpui::IntoElement {
    view! { <button class="p-2">{label}</button> }
}

struct Card { title: String, children: Vec<gpui::AnyElement> }
impl gpui::IntoElement for Card { /* apply title + children */ }
```

Keep components returning `impl gpui::IntoElement`. Do not introduce a virtual DOM or `HtmlElement`.

## Events, refs, overlays, router

- Pointer/keyboard: `on:click`, `on:pointer_down`, `on:key_down`, … See `crates/vgui-view/src/lib.rs` `AttrKind`.
- `NodeRef::new()` then `ref={my_ref.clone()}`. Methods panic until first bind.
- Overlays: `portal(content, priority)`, `dialog(...)`, `floating(...)` — not HTML `<dialog>` browser behavior.
- Router: `create_router("/")`, `router.navigate(cx, path)`, `match_pattern("/users/:id", path)`. In-app only; `<a href>` is not a browser navigation.

## Verification

After UI or example changes:

```bash
cargo +nightly check -p vgui-<name>
cargo +nightly check --target wasm32-unknown-unknown -p vgui-<name>
```

New examples: register in root `Cargo.toml`, `scripts/build_docs.sh` `EXAMPLES`, and `book/src/SUMMARY.md`. Prefer extending an existing example.

Do not claim done after a native-only check.
