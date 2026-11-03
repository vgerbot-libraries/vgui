---
title: The view! Macro
description: Build GPUI element trees with vgui elements, attributes, interpolation, components, and control flow.
---

`view!` turns JSX-like Rust tokens into GPUI element builder expressions at compile time. It provides a declarative authoring syntax, not an HTML runtime or a browser DOM abstraction.

## Start with one root

A `view!` invocation accepts one root element, fragment, or interpolated expression. This excerpt from the [Quick Start](../quick-start/) combines text, an attribute, and an event:

```rust
view! {
    <div class="flex flex-col gap-3 p-4 text-white">
        <span>{format!("count = {}", count.get())}</span>
        <button
            class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded"
            on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
        >
            {"Increment"}
        </button>
    </div>
}
```

The macro emits ordinary Rust expressions. Values used inside the view, such as `count` and `set_count`, follow normal Rust capture and ownership rules.

## Insert text and Rust expressions

Use quoted literals for fixed text and braces for an expression:

```rust
view! {
    <div>
        "Plain text"
        <span>{format!("Hello, {}", name)}</span>
        {badge("New".to_string())}
    </div>
}
```

Interpolated expressions are converted with `vgui::into_child`. They can produce text, an element, a component, or another supported view child. A block such as `{|| { /* Rust code */ view! { <div>"result"</div> } }}` is an expression only when you invoke the closure; `{|| ...}` by itself is still a closure value.

Fragments group several nodes without requiring a named element:

```rust
view! {
    <>
        <span>"First"</span>
        <span>"Second"</span>
    </>
}
```

When fragments or control flow are direct children of an element, their children are added directly to that parent rather than through an extra control-flow wrapper.

## Choose built-ins or components

Tag spelling determines expansion:

- Lowercase tags such as `<div>`, `<button>`, and `<input>` select vgui built-ins. Most general containers become GPUI `div` builders with tag-specific defaults; specialized elements such as inputs construct widgets.
- Uppercase tags select components. Attributes and children become struct fields, while a tag with neither becomes a function call.
- `<Show>`, `<For>`, `<Index>`, `<Switch>`, and `<Match>` are recognized as control flow rather than ordinary components.

The HTML-like names do not imply complete browser HTML behavior. Use the attributes and element behavior implemented by vgui. See [Custom Components](../custom-components/) for component calls and [Control Flow](../control-flow/) for the special elements.

## Pass attributes

Every attribute requires `=`. Use a literal, a braced Rust expression, a bare identifier expression, or a boolean literal:

```rust
view! {
    <div
        id="summary"
        class="flex flex-col gap-2"
        style={css! { display: flex; gap: 8px; }}
        tabindex={0}
        role="region"
        aria:label="Summary"
    >
        {content}
    </div>
}
```

Common built-in attributes include:

| Form | Purpose |
| --- | --- |
| `class="..."` | Apply vgui-supported Tailwind-style utilities, including supported state and breakpoint variants. |
| `style={css! { ... }}` | Apply a Rust CSS style value. |
| `hover={...}`, `active={...}`, `focus={...}` | Apply styles for interaction states. |
| `id=...`, `tabindex=...` | Set element identity and focus behavior. |
| `role=...`, `aria:name=...` | Set supported accessibility metadata. |
| `ref={node_ref}` | Bind a `NodeRef` where supported. |

Class and CSS support are vgui subsets with GPUI behavior; they are not a browser stylesheet. Unsupported attributes or event names are compile errors rather than silently forwarded HTML.

## Handle events

Built-in events use `on:event={handler}`. Pass the closure shape required by the event, or use an adapter where vgui provides one:

```rust
on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
```

Here `click` adapts a closure receiving `&mut App`. A direct GPUI click listener instead receives the click event, window, and application context. Widget events can have different signatures: for example, text input callbacks receive the changed text and application context. Check the element's Rust API when adding a new event.

Event closures run outside the render expression that created them. Clone any signal or write handle that a `move` closure needs.

## Forward props with spread

Both `{..props}` and `{...props}` express one spread value. On a component, spread becomes Rust struct update syntax:

```rust
view! { <CardHeader {..props} title={"Override"} /> }
```

Explicit component fields are emitted before `..props`, so named fields override values supplied by the base struct. Only one spread is allowed. On supported built-ins, the value is applied through `vgui::Spread`; the props type must implement that trait for the concrete element type. Spread does not remove the normal type and attribute checks.

## Keep the syntax constraints visible

- Provide exactly one root and close every non-void element with its matching tag.
- `<input>`, `<br>`, `<hr>`, and `<wbr>` may omit the self-closing slash; other elements need `/>` or a closing tag.
- Use `{expr}` for dynamic children and attributes, not JSX strings or JavaScript values.
- Control-flow elements have their own required attributes and child shapes; see [Control Flow](../control-flow/).
- Reactive calls made while evaluating a view still follow positional scope rules; see [Reactivity](../reactivity/) and [Architecture](../architecture/).

The resulting value implements GPUI's element conversion traits and can be returned from a render function or passed as a child. For how root rendering uses that value, return to [Architecture](../architecture/).
