---
title: Control Flow
description: Render conditions, lists, and branches with Show, For, Index, Switch, and Match in vgui.
---

The `view!` macro gives control-flow elements dedicated expansion rules. Choose them by both the UI shape and the reactive scope you need; their state lifetimes are intentionally different.

## Render conditionally with Show

Use `<Show>` for one boolean condition. Its children render when `when` is true, and the optional `fallback` renders otherwise:

```rust
view! {
    <Show
        when={count.get() > 0}
        fallback={view! { <span>"No items"</span> }}
    >
        <span>{format!("{} items", count.get())}</span>
    </Show>
}
```

Without `fallback`, a false condition produces `gpui::Empty`. `when` must evaluate to `bool`, and `<Show>` accepts only `when` and `fallback`.

`<Show>` does **not** create a child reactive scope. Its body is skipped while the condition is false, so creating signals, stores, memos, effects, or cleanups only inside that body would change the parent's positional slot sequence. Keep that state outside the condition, or use a scoped `<Match>` branch when the state belongs to a branch.

## Iterate simple lists with For

Use `<For>` for straightforward rendering of an `IntoIterator`. Its only child must be an interpolated closure receiving `(item, index)` and returning an element:

```rust
view! {
    <For
        each={todos.get()}
        fallback={view! { <div>"No todos here."</div> }}
    >
        {move |todo: Todo, index: usize| view! {
            <div>{format!("{}: {}", index + 1, todo.title)}</div>
        }}
    </For>
}
```

The closure runs for every item on every root render. If the iterator is empty, the fallback renders; without a fallback, nothing renders. `<For>` accepts only `each` and `fallback`.

`<For>` has no keyed reconciliation and no per-item child scopes. All closure invocations share the parent's positional slot sequence. Keep rows stateless and pass in state or callbacks from outside; do not create reactive primitives in a variable-length `<For>` row, including indirectly through a stateful component. Choose `<Index>` when each position needs its own reactive state.

## Give each position a scope with Index

`<Index>` has the same `each`, optional `fallback`, and `(item, index)` closure shape, but it enters a persistent child scope for every position:

```rust
view! {
    <Index
        each={items.get()}
        fallback={view! { <div>"No items."</div> }}
    >
        {move |item: String, index: usize| view! {
            <div>{format!("Item {}: {}", index, item)}</div>
        }}
    </Index>
}
```

Signals, memos, effects, stores, and cleanup callbacks created in the row belong to that position's scope. They survive root renders while the position remains present. When the list shrinks, scopes beyond the new length are disposed and their `on_cleanup` callbacks run.

This is **keyed by position**, not keyed by item identity. Inserting, removing, or reordering items can associate existing position-local state with different item values. Use `<Index>` when position is the intended identity; do not treat it as a keyed-list diffing mechanism.

## Select the first branch with Switch and Match

Use `<Switch>` when exactly one of several conditions should render. `<Match>` conditions are evaluated in source order and the first true branch wins:

```rust
view! {
    <Switch fallback={view! { <div>"Unknown status"</div> }}>
        <Match when={status.get() == "loading"}>
            <div>"Loading..."</div>
        </Match>
        <Match when={status.get() == "error"}>
            <div>"Error"</div>
        </Match>
        <Match when={status.get() == "ready"}>
            <div>"Ready"</div>
        </Match>
    </Switch>
}
```

If no condition matches, the fallback renders; without one, the result is `gpui::Empty`. `<Switch>` accepts only `fallback`, its children must be `<Match>` elements, and each `<Match>` requires `when` as its only attribute.

Every `<Match>` branch has an isolated child scope and slot sequence. State created in the active branch persists across root renders while that branch remains active. Changing branches disposes the inactive branch scope, runs its cleanups, and creates its state afresh if that branch is entered again later.

## Compare scope and layout behavior

| Construct | Reactive scope | Empty or inactive result | Primary use |
| --- | --- | --- | --- |
| `<Show>` | Shares the parent scope | Fallback or `gpui::Empty` | One condition with state managed outside the condition |
| `<For>` | Shares the parent scope for every row | Fallback or nothing | Simple, stateless iteration |
| `<Index>` | One persistent child scope per position | Fallback or `gpui::Empty` | Position-local item state and cleanup |
| `<Switch>` / `<Match>` | One child scope per branch | Fallback or `gpui::Empty` | First-match branching with branch-local state |

When any of these elements is a direct child of another element, `view!` adds the selected or iterated children directly to that parent, preserving its flex or grid layout without a control-flow wrapper. As standalone expressions, the non-empty `For` and `Index` helper functions return a `div` containing their rows.

Reactive creation and disposal rules are covered in [Reactivity](../reactivity/). For the underlying parsing and direct-child expansion, see [The `view!` Macro](../view-macro/); for passing row state into components, see [Custom Components](../custom-components/).
