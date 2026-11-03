---
title: Store and Lists
description: Manage aggregate state with Store selectors and render collections with For or position-scoped Index.
---

A `Store` groups related fields behind one read handle and one write handle. This page uses focused excerpts from the current store integration tests, then shows the current `For` and `Index` list syntax. These excerpts belong inside a mounted reactive view; they are not complete applications.

## Model aggregate state

The store test uses one cloneable state struct:

```rust
#[derive(Clone, PartialEq, Default)]
struct AppState {
    count: i32,
    name: String,
    flag: bool,
}
```

Create the store and select the slices used by the view:

```rust
let (store, set_store) = create_store(AppState::default());
let count_sig = store.select(|s| s.count);
let name_sig = store.select(|s| s.name.clone());

view! { <div>{format!("{} {}", count_sig.get(), name_sig.get())}</div> }
```

`create_store` returns a `Store<AppState>` and a `SetStore<AppState>`. Each `select` call returns a read signal for the selected value. Copyable fields such as `count` can be returned directly; clone owned data such as `name` when the selector must return an owned `String`.

## Selectors filter unchanged values

A store write notifies even when the update closure does not change anything. This is deliberately different from a signal's equal-value notification skipping. On each write, selectors recompute and compare their selected slices:

- after `s.count = 42`, `count_sig` changes while `name_sig` remains empty;
- after `s.name = "hello".to_string()`, the count remains `42` and the name changes;
- after changing only `s.flag`, neither selected slice changes.

Selectors therefore filter propagation of unchanged selected values. They do **not** prevent the store notification, the selector recomputation, or every root render. Do not describe them as keyed DOM updates or as a guarantee that unrelated rows never render.

For direct reads, the current test also exercises both `get` and `with`:

```rust
let via_get = store.get().count;
let via_with = store.with(|s| s.name.clone());
```

`get` obtains the state value, while `with` borrows it through a closure so you can choose what to read or clone.

## Mutate fields or replace the state

Use the write handle and GPUI application context for partial mutation:

```rust
set_store.update(cx, |s| s.count = 42);
set_store.update(cx, |s| s.name = "hello".to_string());
set_store.update(cx, |s| {
    s.count = 99;
    s.name = "world".to_string();
});
set_store.update(cx, |s| s.flag = true);
```

Use `set` when replacing the complete value is clearer:

```rust
set_store.set(
    cx,
    AppState {
        count: 100,
        name: "replaced".to_string(),
        flag: true,
    },
);
```

Even `set_store.update(cx, |_s| {})` notifies. Use `select` when downstream consumers should observe only changes to a particular slice.

## Render values with `For`

`For` accepts an `each` iterator, an optional `fallback`, and one child closure receiving `(item, index)`. This focused excerpt renders owned strings and an empty-state message:

```rust
let items = vec!["alpha".to_string(), "beta".to_string()];

view! {
    <div>
        <For each={items} fallback={view! { <span>{"No items"}</span> }}>
            {move |item, index| view! {
                <span>{format!("{}: {}", index, item)}</span>
            }}
        </For>
    </div>
}
```

`For` enumerates the iterator during rendering and builds the row elements. It is appropriate for rows derived directly from item values. It does not perform keyed reconciliation and does not provide `Index`'s per-position reactive child scopes. A changed collection is rendered again when its parent view renders.

## Retain position-based scopes with `Index`

`Index` has the same `each`, optional `fallback`, and `(item, index)` closure shape:

```rust
view! {
    <Index each={vec![1u32, 2, 3]} fallback={view! { <div>{"empty"}</div> }}>
        {|n, i| view! { <div>{format!("{}: {}", i, n)}</div> }}
    </Index>
}
```

Inside a reactive scope, each position gets a persistent child scope. Signals, memos, and effects created in the row closure survive re-renders while the item remains at that position. When the list shrinks, excess position scopes are disposed and their cleanup callbacks run.

This identity is **position-based, not item-ID-based**. Inserting or reordering items can associate existing position scopes with different item values. Choose `Index` when per-position row state is useful; choose `For` for straightforward value rendering. Both controls require a closure as their only child, and both support an empty-list fallback.

## Test coverage and next steps

The native store integration tests drive a real GPUI window through count, name, combined, unrelated-field, whole-state, and no-op updates, then assert the recorded values. The list compile coverage includes non-empty and empty `Index` cases; a compile-fail UI test rejects a non-closure `Index` child. These checks are not pointer interaction tests or browser rendering tests.

Compare store notifications with signal updates in [Your First Reactive View](../../first-reactive-view/), or return to the complete [Counter](../counter/) for a minimal mounted application. Platform verification boundaries are covered in [Native & Web](../../native-and-web/).
