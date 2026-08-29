# Introduction

## What is vgui?

`vgui` is a declarative, reactive GUI framework for Rust, built on top of
[`gpui`](https://crates.io/crates/gpui) — the GPU-accelerated UI toolkit behind
the [Zed](https://zed.dev) editor. It brings a familiar web-style authoring
experience to native desktop applications:

- **JSX-like views** via the `view!` macro — elements, attributes, children,
  fragments, and component invocation, all in ergonomic markup.
- **CSS-in-Rust** via the `css!` macro — write real CSS declarations
  (`color: #fff; padding: 8px;`) that compile down to `gpui` style refinements.
- **Tailwind-style classes** via the `tw!` macro — `class="p-2 rounded
  hover:bg-[#000088]"` works out of the box, with `hover:`, `focus:`, and
  `active:` variants.
- **Fine-grained reactivity** inspired by [SolidJS](https://www.solidjs.com/):
  `create_signal`, `create_memo`, and `create_effect` track dependencies
  automatically and re-render only what changes.
- **Control-flow components** `<Show>` and `<For>` for conditional and list
  rendering with optional fallbacks.
- **Built-in input widgets** — text fields with full cursor/selection/clipboard/
  IME support, checkboxes, radio buttons, range sliders, file pickers, select
  dropdowns, and text areas.

Under the hood, `vgui` maps every HTML element to a `gpui` flexbox `div` (or a
specialized widget for inputs), compiles CSS/Tailwind declarations into
`gpui::StyleRefinement` mutations at build time, and drives reactivity through a
per-render scope-slot model that keeps stateful widgets alive across re-renders.

## Features

### Declarative markup

The `view!` macro lets you write UI trees that look almost identical to JSX:

```rust
view! {
    <div class="flex flex-col gap-3 p-4 justify-center items-center">
        <span>{"Hello, world!"}</span>
        <button on:click={click(move |_cx| {})}>{"Click me"}</button>
    </div>
}
```

Lowercase tags (`<div>`, `<span>`, `<button>`, `<input>`, …) map to built-in
HTML elements. Uppercase tags (`<Greeting>`, `<Card>`) invoke your own
component functions or struct constructors.

### Compile-time styling

Both `css!` and `tw!` are proc-macros — there is no runtime CSS parser. Every
declaration is validated and lowered to typed `gpui` style mutations at compile
time, so typos and unsupported properties produce build errors, not silent
rendering bugs.

```rust
// CSS-in-Rust
<div style={css! {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: rgb(30, 30, 30);
    color: #fff;
}}>

// Tailwind classes
<div class="flex flex-col gap-3 p-4 bg-[#505050] text-white rounded-lg">
```

### Fine-grained reactivity

State is managed through signals — lightweight, observable cells that
automatically track which scopes read them:

```rust
let (count, set_count) = create_signal(0i32);
let doubled = create_memo({ let count = count.clone(); move || count.get() * 2 });

view! {
    <span>{format!("count = {}", count.get())}</span>
    <span>{format!("doubled = {}", doubled.get())}</span>
    <button on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}>
        {"Increment"}
    </button>
}
```

When `set_count` fires, only the scopes that read `count` (the two `<span>`s
and the `doubled` memo) are re-evaluated — not the entire tree.

### Control flow

`<Show>` and `<For>` are first-class constructs in `view!`:

```rust
<Show when={count.get() > 0} fallback={view! { <span>{"zero or negative"}</span> }}>
    <span>{"positive"}</span>
</Show>

<For each={todos.get()} fallback={view! { <div>{"No todos."}</div> }}>
    {move |todo: Todo, _i: usize| todo_item(todo)}
</For>
```

### Rich input widgets

`<input>` supports 19 `type` variants — text, password, search, email, url,
tel, number, date, datetime-local, time, month, week, color, checkbox, radio,
range, file, submit/button/reset, and hidden — each with appropriate event
handler signatures. Text-based inputs have full cursor movement, selection,
clipboard (Ctrl+A/C/V/X), and IME (CJK composition) support.

## Status

`vgui` is early-stage, experimental software. The API is not yet stable and
breaking changes should be expected between releases. It is, however, fun to
build with — see the [Examples](./examples/counter.md) section for end-to-end
applications.

## License

Licensed under the [MIT License](https://github.com/vgerbot-libraries/vgui/blob/main/LICENSE).
