---
title: Architecture
description: Understand how vgui mounts a GPUI root, preserves reactive state, and rerenders views.
---

vgui combines a compile-time view macro with GPUI elements and slot-cached reactive state. The important boundary is simple: `view!` describes elements, reactive primitives own state, and `VguiRoot` connects both to a GPUI window.

## The main pieces

| Piece | Responsibility |
| --- | --- |
| `vgui` | Exposes mounting, reactivity, control flow, widgets, styling, and the public macros. |
| `vgui-view` | Parses `view!` markup and `#[vgui_component]`, then emits Rust and GPUI builder expressions. |
| `VguiRoot` | Owns the root render closure and reactive scope as a GPUI entity. |
| `Scope` | Stores positional reactive slots, subscriptions, memos, effects, cleanups, and child scopes. |
| GPUI | Provides elements, entities, windows, events, and native rendering. |

Applications normally import the common API through `vgui::prelude::*`. The macros are re-exported by `vgui`, so application code does not need to invoke the proc-macro crate directly.

## Follow one render

The [Quick Start](../quick-start/) mounts an application with this callback:

```rust
|window, cx| vgui::mount(window, cx, app)
```

`mount` creates a `VguiRoot` and stores `app` as its render closure. Each GPUI render then performs these steps:

1. Reset the positional slot cursor for the root scope and its descendants.
2. Enter the reactive scope and invoke `app` from the beginning.
3. Evaluate signal reads, component calls, and the expressions inside `view!`.
4. Build a fresh GPUI element tree and leave the reactive scope.
5. Return the tree to GPUI for layout, painting, and event dispatch.

The markup is expanded at compile time. vgui does not parse HTML at runtime and does not create browser DOM nodes.

## What happens after a write

`WriteSignal::set` and `WriteSignal::update` update a GPUI entity when the value changes. Its observer then asks `VguiRoot` to process that dependency:

1. Memos and effects whose recorded dependency set contains the changed entity run.
2. Changed memo values notify their own dependents.
3. `cx.notify()` requests a render of the mounted root.
4. The root render closure runs again and constructs a new element tree.

This is **root rerendering, not a virtual-DOM diff**. Memos and effects avoid some recomputation, but vgui does not update an isolated text node or reconcile two virtual trees. A store write also requests a root render, even when no selected store slice changed.

See [Reactivity](../reactivity/) for the read, write, memo, effect, and store APIs.

## Why state survives a fresh render

Calling `create_signal`, `create_store`, `create_memo`, `create_effect`, or `on_cleanup` consumes the next slot in the current scope. On the first render, vgui creates and stores the primitive. On later renders, the same position and type retrieve the existing primitive; the initial value is not reapplied.

Consequently, reactive primitive creation must have a stable order and stable types across renders. Do not conditionally create primitives, reorder them, or place them in a loop whose length can change. Changing the slot kind or type at a position, or using a different number of slots after initialization, is an error.

`view!` also generates deterministic element IDs from call order for stateful elements. Stateful widget entities can be cached in slots, allowing GPUI state such as editing or focus state to survive reconstruction of the surrounding element tree.

## Use child scopes for conditional and item state

Some control-flow elements create separate child scopes with their own slot sequences:

- Each active `<Match>` branch in a `<Switch>` has a branch-local scope. Leaving the branch disposes that scope.
- Each `<Index>` position has an item-local scope. Shrinking the list disposes scopes beyond the new length.
- Disposal runs registered `on_cleanup` callbacks, clears subscriptions and slots, and disposes children before their parent.

An `<Index>` scope belongs to a **position**, not to a stable item key. Reordering items therefore does not move their local state with them. `<Show>` and `<For>` do not create child scopes; keep positional reactive primitives out of conditionally executed `<Show>` bodies and variable-length `<For>` rows.

[Control Flow](../control-flow/) shows when to choose each construct, and [Custom Components](../custom-components/) explains why component calls share the caller's scope unless one of these child scopes is active.

## Choose where state belongs

Use the lifetime you need as the placement rule:

- Put application-wide or shared view state in the root render path.
- Put branch-local drafts or subscriptions inside a `<Match>` branch.
- Put position-local row state inside an `<Index>` row.
- Keep stateless markup in ordinary functions and components.

For the syntax that produces the element tree, continue to [The `view!` Macro](../view-macro/). For native and WebAssembly boundaries, see [Native & Web](../native-and-web/).
