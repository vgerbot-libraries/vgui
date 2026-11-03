---
title: Reactivity
description: Manage vgui state with signals, memos, effects, stores, and scoped cleanup.
---

vgui stores reactive values in GPUI entities and caches their handles in positional scope slots. Signals hold independent values, memos derive cached values, effects synchronize side effects, and stores group aggregate state.

## Create and update a signal

```rust
let (count, set_count) = create_signal(0i32);
let doubled = create_memo({
    let count = count.clone();
    move || count.get() * 2
});

view! {
    <div>
        <span>{format!("count = {}", count.get())}</span>
        <span>{format!("doubled = {}", doubled.get())}</span>
        <button on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}>
            {"Increment"}
        </button>
    </div>
}
```

`create_signal(initial)` returns a `ReadSignal<T>` and a `WriteSignal<T>`. Both handles are cheap to clone into derived computations and event closures. `T` must implement `Clone + PartialEq + 'static`.

Use `set_count.set(cx, value)` to replace a value or `set_count.update(cx, |value| ...)` to mutate it. Signal writes compare old and new values: an equal value does not notify. `update` also returns the closure's result.

## Choose get or get_with

`ReadSignal::get()` clones the value from its reactive cache and records the signal when dependency tracking is active:

```rust
let value = count.get();
```

`ReadSignal::get_with(cx)` instead reads the current value directly from the GPUI entity, refreshes the cache, and does not record a dependency:

```rust
let value = count.get_with(cx);
```

Use `get()` in views, memos, and effects. Use `get_with(cx)` when code outside a render scope, such as a later event or task, specifically needs an entity-backed read without establishing a reactive dependency. `Store` provides the same distinction.

## Derive cached values with memos

`create_memo` runs its closure when the memo is first created, records signal and store reads made during that initial computation, and caches the result. Later root renders reuse the memo slot without recomputing it. A recorded dependency change runs the computation again; an equal new result is not published to the memo's own dependents.

```rust
let remaining = create_memo({
    let todos = todos.clone();
    move || todos.get().iter().filter(|todo| !todo.done).count()
});
```

The memo result is another `ReadSignal<T>`, with `T: Clone + PartialEq + 'static`. Use memos for derived values whose computation you want to cache, not for side effects.

## Run effects and register cleanup

`create_effect` records dependencies during its initial run and executes its closure immediately, synchronously during rendering:

```rust
create_effect({
    let count = count.clone();
    move || {
        eprintln!("count changed to: {}", count.get());
    }
});
```

The effect reruns when one of its recorded dependencies changes; it does not rerun merely because the root rendered. Effects are not deferred until after paint. Avoid writes that recursively retrigger the same effect without a termination condition.

`on_cleanup` is separate from effect reruns. It registers a positional cleanup callback that runs when the enclosing scope is disposed, not before every effect execution:

```rust
on_cleanup(move || {
    eprintln!("scope disposed");
});
```

Use cleanup for resources owned by a `<Match>` branch or an `<Index>` position. Disposal runs child cleanups before parent cleanups and clears the scope's slots and subscriptions. See [Control Flow](../control-flow/) for those lifetimes.

## Group aggregate state in a store

Use `create_store` when related fields belong to one state value:

```rust
#[derive(Clone, Default)]
struct AppState {
    count: i32,
    name: String,
    flag: bool,
}

let (store, set_store) = create_store(AppState::default());
let count = store.select(|s| s.count);
let name = store.select(|s| s.name.clone());
```

A store requires `T: Clone + 'static`, but not `PartialEq`. Its read and write APIs are:

| API | Behavior |
| --- | --- |
| `store.get()` | Clone the whole state and track the entire store when tracking is active. |
| `store.with(|state| ...)` | Borrow the whole state without cloning it; also tracks the entire store. |
| `store.get_with(cx)` | Read directly from the GPUI entity and refresh the cache without tracking. |
| `store.select(|state| slice)` | Create a memo-backed `ReadSignal` for a selected value. |
| `set_store.set(cx, state)` | Replace the whole state and always notify. |
| `set_store.update(cx, |state| ...)` | Mutate the state and always notify, even for a no-op mutation. |

A selector recomputes after store writes but publishes only when its selected `Clone + PartialEq` value changes. This filters downstream memo and effect notifications; it does **not** prevent the root rerender requested by every store write. Reading with `get` or `with` depends on the whole store rather than one field.

For a single independent value, a signal is simpler. For aggregate state with useful derived slices, use a store and selectors. The current store integration tests exercise both whole-state reads and unchanged-slice filtering.

## Preserve positional identity

All these creation APIs participate in the current scope's slot sequence. Create them unconditionally in a stable order and with stable types across root renders. A component does not automatically create a new scope, so primitives created in its body also occupy the caller's slots.

Do not place reactive creation in a conditionally rendered `<Show>` body or in a variable-length `<For>` closure. Use a `<Match>` branch for branch-local state or `<Index>` for position-local item state, and remember that `<Index>` positions are not item keys.

[Architecture](../architecture/) explains the notification and root-rerender cycle. For a small complete state example, revisit [Your First Reactive View](../first-reactive-view/).
