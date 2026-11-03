---
title: Refs and Context
description: Bind GPUI focus and scroll handles with NodeRef, and pass synchronous construction-time values with Context and Provider.
---

`NodeRef` and `Context` solve different problems. A ref gives imperative access to GPUI focus, geometry, and scroll state. Context makes a cloned Rust value available while a provider's descendants are being constructed. Neither is a DOM reference, a general event-lifetime store, or a replacement for reactive state.

## NodeRef is a GPUI handle

```rust
let node_ref = NodeRef::new();

let element = view! {
    <div ref={node_ref.clone()}>{"Content"}</div>
};
```

`NodeRef::new()` and `NodeRef::default()` create an unbound shell. Clones share the same binding, so one clone can be attached in markup and another retained by an event handler. Binding installs a GPUI `FocusHandle` and `ScrollHandle`; it does not expose an HTML element or arbitrary DOM operations.

`is_bound()` is safe before attachment and returns whether a binding exists. Other ref methods panic while unbound. Check the flag before an early event or conditional render can use the ref.

## Focus and scrolling

```rust
let list_ref = NodeRef::new();
let list_for_click = list_ref.clone();

view! {
    <div>
        <div ref={list_ref.clone()} class="h-48 overflow-y-scroll">
            {"Items"}
        </div>
        <button on:click={move |_event, window, cx| {
            if list_for_click.is_bound() {
                list_for_click.focus(window, cx);
                list_for_click.scroll_to_bottom();
            }
        }}>
            {"Scroll to bottom"}
        </button>
    </div>
}
```

`focus` requests GPUI focus. Focus queries require GPUI window/app context as shown by their signatures, while scrolling and geometry use the stored handles.

| Method | Purpose |
| --- | --- |
| `is_bound()` | Test whether the ref has been attached. |
| `focus(window, cx)` | Focus the attached element. |
| `is_focused(window)` | Test whether its focus handle is focused. |
| `contains_focused(window, cx)` | Test whether it contains the focused element. |
| `bounds()` | Return the attached element's recorded bounds. |
| `scroll_offset()` | Read the current scroll offset. |
| `scroll_to(ix)` | Scroll a GPUI list handle to an item index. |
| `scroll_to_top(ix)` | Scroll so the indexed item is at the top. |
| `scroll_to_bottom()` | Scroll to the bottom. |
| `set_scroll_offset(point)` | Set a pixel scroll offset. |
| `child_bounds(ix)` | Read recorded bounds for an indexed child. |
| `child_count()` | Read the recorded child count. |
| `focus_handle()` | Access the underlying GPUI focus handle. |
| `scroll_handle()` | Access the underlying GPUI scroll handle. |

Bounds and child measurements come from the previous painted frame, not a fresh synchronous layout pass. Treat initial or just-changed geometry accordingly. Scroll operations also depend on the attached GPUI element and its scroll/list behavior; a `NodeRef` does not make every container a virtualized list.

## Ref support on built-ins

General elements that accept `ref`, plus `form`, `label`, and button/checkbox/radio/file input wrappers, can bind a `NodeRef`. The entity-backed text and range implementations do not bind a direct `ref`; `select` and `textarea` reject that attribute. Use a supported wrapper element when you need a handle around those widgets.

```rust
let input_wrapper = NodeRef::new();

view! {
    <div ref={input_wrapper.clone()}>
        <input type="text" value={name.get()} />
    </div>
}
```

A wrapper ref addresses the wrapper, not the text editor's internal GPUI entity. Use input callbacks and application state for values as described in [Inputs and Forms](../inputs-and-forms/).

## Context is keyed by Rust type

```rust
#[derive(Clone, Default)]
struct Palette(&'static str);

static PALETTE: Context<Palette> = Context::new();
```

`Context<T>` is a zero-sized, const-constructable key. Context lookup uses `TypeId`, so there is one logical context per `T`, not per static variable. Two `Context<String>` statics still address the same type key. Use newtypes such as `Palette` when separate values would otherwise have the same Rust type.

Values must be cloneable because lookup returns an owned clone. Context carries no implicit signal tracking or update subscription.

## Provider and use_context

`Provider` accepts exactly `context` and `value`. It is layout-transparent: it adds no div, styling, ref, or event target. While its children are constructed, the value is pushed onto a thread-local provider stack; construction then pops it.

```rust
#[derive(Clone, Default)]
struct Palette(&'static str);

static PALETTE: Context<Palette> = Context::new();

fn label() -> impl gpui::IntoElement {
    let palette = use_context_or(&PALETTE, Palette::default);
    view! { <span>{palette.0}</span> }
}

fn app() -> impl gpui::IntoElement {
    view! {
        <Provider context={PALETTE} value={Palette("dark")}>
            <div>{label()}</div>
        </Provider>
    }
}
```

Call `use_context(&PALETTE)` to obtain `Option<Palette>`, cloning the nearest matching value. `use_context_or` supplies a fallback when no value is present. The consumer function must run during descendant construction, as `label()` does above; saving work for a later click, async task, or unrelated render will not retain the provider stack.

A standalone provider supports zero or one direct child. Wrap multiple siblings in one explicit container, as in the example, to keep the construction boundary clear.

## Nested providers and shadowing

Providers of the same `T` shadow outer values for their descendants:

```rust
view! {
    <Provider context={PALETTE} value={Palette("outer")}>
        <div>
            <span>{use_context(&PALETTE).unwrap().0}</span>
            <Provider context={PALETTE} value={Palette("inner")}>
                <span>{use_context(&PALETTE).unwrap().0}</span>
            </Provider>
        </div>
    </Provider>
}
```

The outer lookup clones `"outer"`, and the inner lookup clones `"inner"`. Once each provider finishes constructing its children, its entry is removed; lookups outside that boundary do not see it.

## Manual provide_context scopes

Imperative Rust construction can use the same mechanism without the `Provider` element:

```rust
let subtree = {
    let _guard = provide_context(&PALETTE, Palette("high-contrast"));
    label()
};
```

`provide_context` returns a `ProviderGuard`. Keep it alive while constructing all intended descendants, then drop it to pop the value. The explicit scope above makes that lifetime visible. Do not move the guard into an event handler or async lifetime as a way to make context persistent.

## Context and reactive state

Context answers “which value is nearest during this synchronous construction?” It does not answer “how does this value update over time?” Replacing a provider value does not by itself notify consumers, and a cloned context value is not automatically subscribed to its source.

Use context for construction-time configuration, service handles, or signal handles that descendants need. Use [signals, memos, and effects](../reactivity/) for values that must publish changes. Keep provider use on the constructing thread and within the provider/guard lifetime; do not treat it as a global, async, or cross-thread store.

For complete method signatures, see the [vgui API documentation](../../api/vgui/).
