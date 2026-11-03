---
title: Custom Components
description: Create typed, reusable vgui components with props, children, callbacks, and prop spreading.
---

Use ordinary Rust functions for small rendering helpers and `#[vgui_component]` when you want uppercase tags with named props. The attribute macro generates a typed props struct and its GPUI `IntoElement` implementation.

## Define a component with named props

This follows the same pattern as vgui's current component implementations:

```rust
#[vgui_component]
pub fn card_header(
    title: String,
    description: String,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-1">
            <span class="font-bold">{title}</span>
            <span>{description}</span>
        </div>
    }
}
```

Call it with the PascalCase form of the function name:

```rust
view! {
    <CardHeader title="Tasks" description="Three remaining" />
}
```

For `card_header`, the macro generates a public `CardHeader` struct with fields for the function parameters and an `IntoElement` implementation that calls the original render body. The generated struct is what makes the uppercase attribute syntax work.

Parameters must use simple identifier patterns, methods with `self` are not supported, and the function must have a non-unit return type such as `impl gpui::IntoElement`. Parameters of type `String` become generic fields accepting `Into<String>`, which is why the example accepts string literals. `Option<String>` receives corresponding `Into<String>` support for its contained value.

## Accept children

Declare a `children: Vec<gpui::AnyElement>` parameter when the component should contain arbitrary view children. The current `Card` pattern uses `for_each` to append them:

```rust
#[vgui_component]
pub fn card(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-2 rounded-lg p-4">
            {for_each(children, |child, _| child)}
        </div>
    }
}
```

The macro converts each supplied child into `gpui::AnyElement` and fills the `children` field:

```rust
view! {
    <Card>
        <span>"Panel title"</span>
        <span>"Panel body"</span>
    </Card>
}
```

Children are supplied only when present in the tag; the macro does not synthesize an empty vector for a required `children` field. Other fields are likewise required unless supplied by a spread base. To represent an absent value, use an optional prop and pass it explicitly, or supply it through a spread base.

## Receive event callbacks

An `on:event` attribute maps to an `on_event` field and is boxed by component expansion. Declare the matching boxed callback type:

```rust
#[vgui_component]
pub fn action_button(
    label: String,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    view! {
        <button on:click={click(move |cx| on_click(cx))}>
            {label}
        </button>
    }
}
```

The caller passes a closure without manually constructing the `Box`:

```rust
view! {
    <ActionButton
        label="Save"
        on:click={move |cx: &mut gpui::App| {
            set_saved.set(cx, true);
        }}
    />
}
```

The inner built-in `<button>` still uses its normal event API, here through the `click` adapter. Component callback types are your own contract: clone captured write handles into `move` closures and choose argument types that match the information the component exposes.

Attribute names otherwise map directly to fields. Rust keywords use raw field identifiers where needed: `type=` maps to `r#type`, `for=` to `r#for`, and `ref=` to `r#ref`.

## Forward and override props

Component spread uses Rust struct update syntax. Given a compatible `props` value, this supplies the remaining fields and overrides `title`:

```rust
view! {
    <CardHeader {..props} title={"Updated title"} />
}
```

Only one `{..props}` or `{...props}` spread is allowed per component. Explicit named fields take precedence over the base value. This is struct composition, not automatic merging of unrelated props objects, and it does not generate defaults for missing fields.

Spread on a built-in element is different: it calls the `vgui::Spread` trait for the concrete element type. See [The `view!` Macro](../view-macro/) for that distinction.

## Understand expansion and scope

For any uppercase tag, current expansion depends on its contents:

| Tag form | Expansion shape |
| --- | --- |
| `<Greeting />` | `Greeting()` |
| `<Greeting name={"world"} />` | `Greeting { name: "world" }` |
| `<Greeting>"world"</Greeting>` | `Greeting { children: vec![...] }` |

Use `#[vgui_component]` for the struct-literal forms. For a simple helper that does not need named props, an ordinary Rust function called through interpolation is often clearer:

```rust
fn badge(label: String) -> impl gpui::IntoElement {
    view! { <span class="rounded px-2">{label}</span> }
}

view! { <div>{badge("New".to_string())}</div> }
```

Components do not create a new reactive scope merely by being called. Signals, stores, memos, effects, and cleanups created in a component body consume the caller's positional slots. Keep their creation unconditional and ordered; use `<Match>` for branch-local state or `<Index>` for position-local state, and avoid reactive creation in variable-length `<For>` rows.

See [Reactivity](../reactivity/) for slot rules and [Control Flow](../control-flow/) for scoped lifetimes. Component rendering still participates in the root rerender described in [Architecture](../architecture/); it is not a separate virtual-DOM subtree.
