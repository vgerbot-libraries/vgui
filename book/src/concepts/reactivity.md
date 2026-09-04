# Reactivity

`vgui` uses a SolidJS-inspired reactivity model layered on top of `gpui`
entities. State lives in **signals** — lightweight observable cells that
automatically track which scopes read them. Derived state uses **memos**, and
side effects use **effects**.

## Signals

### Creating signals

`create_signal(initial)` returns a `(ReadSignal<T>, WriteSignal<T>)` pair:

```rust
let (count, set_count) = create_signal(0i32);
let (name, set_name) = create_signal("world".to_string());
```

The type parameter `T` must implement `Clone + PartialEq + 'static`. The
`PartialEq` bound is used to skip notifications when the new value equals the
old — `set_count.set(cx, 5)` when the count is already 5 is a no-op.

> **Rule:** `create_signal` must be called inside `app()` (or a function called
> from `app()` during render). It panics if no `VguiRoot` scope is active.
> Calls must appear in the same order on every render — like React hooks —
> because they are resolved by a per-render slot index.

### Reading signals

`ReadSignal::get()` returns a clone of the current value and registers the
current reactive scope as a dependency:

```rust
let value = count.get(); // registers dependency
```

`ReadSignal::get_with(cx)` reads directly from the gpui entity without
registering a dependency — useful when you need the latest value outside a
tracking context:

```rust
let id = next_id.get_with(cx);
```

`ReadSignal` is `Clone`, so you can pass copies into closures:

```rust
let count_clone = count.clone();
let doubled = create_memo(move || count_clone.get() * 2);
```

### Writing signals

`WriteSignal::set(cx, value)` replaces the value and notifies dependents if the
value changed:

```rust
set_count.set(cx, 10);
```

`WriteSignal::update(cx, f)` mutates the value in place and notifies dependents
if the result changed:

```rust
set_count.update(cx, |n| *n += 1);
set_todos.update(cx, |todos| todos.push(new_todo));
```

`update` returns the return value of the closure, which is useful for
extracting information while mutating:

```rust
let old_value = set_count.update(cx, |n| {
    let old = *n;
    *n = 0;
    old
});
```

`WriteSignal` is `Clone`.

## Stores

`create_store(initial)` creates a reactive store for aggregate state — a
single struct or enum holding multiple fields, rather than one signal per
field. It returns a `(Store<T>, SetStore<T>)` pair:

```rust
#[derive(Clone, Default)]
struct AppState {
    count: i32,
    name: String,
    items: Vec<String>,
}

let (state, set_state) = create_store(AppState::default());
```

The type parameter `T` must implement `Clone + 'static` — notably, **no
`PartialEq` is required**. Unlike signals, store writes always notify;
fine-grained filtering is delegated to `select` (below).

`create_store` follows the same slot-caching rules as `create_signal`: it
must be called inside `app()` in the same order on every render.

### Reading stores

`Store::get()` returns a clone of the entire state and registers the store
as a dependency:

```rust
let count = state.get().count;
```

`Store::with(f)` borrows the state through a closure without cloning:

```rust
let len = state.with(|s| s.items.len());
```

Both `get()` and `with()` track the entire store — any write triggers a
re-render. For fine-grained reactivity, use `select`.

### Fine-grained selectors

`Store::select(f)` derives a `ReadSignal<U>` from a slice of the store. The
closure acts as a **lens** — the Rust-idiomatic equivalent of SolidJS
path-level tracking:

```rust
let count = state.select(|s| s.count);
let name = state.select(|s| s.name.clone());
```

The selector recomputes whenever the store changes, but only notifies its
dependents when the selected value differs (requiring `U: Clone + PartialEq
+ 'static`). Updating `name` does **not** cause `count`'s dependents to
re-render — even though the store itself always notifies.

```rust
// Only re-renders when `count` changes.
set_state.update(cx, |s| s.name = "Alice".to_string()); // count unchanged
set_state.update(cx, |s| s.count = 42);                 // count changes
```

### Writing stores

`SetStore::set(cx, value)` replaces the entire state:

```rust
set_state.set(cx, AppState { count: 0, name: "reset".into(), items: vec![] });
```

`SetStore::update(cx, f)` mutates the state in place through a closure —
the idiomatic way to do partial updates:

```rust
set_state.update(cx, |s| {
    s.count += 1;
    s.items.push("new item".to_string());
});
```

Both methods always notify. Use `select` downstream to filter reactivity to
the slices that actually changed.

### Store vs. signal

| | `create_signal` | `create_store` |
|---|---|---|
| **Best for** | Single flat value | Aggregate state tree |
| **`PartialEq` on `T`** | Required | Not required |
| **Write notifies** | Only if value changed | Always |
| **Fine-grained** | N/A (single value) | Via `select` (lens closures) |
| **Slot-cached** | Yes | Yes |

## Memos

`create_memo(f)` creates a derived, cached value that recomputes only when its
dependencies change. It returns a `ReadSignal<T>`:

```rust
let doubled = create_memo({
    let count = count.clone();
    move || count.get() * 2
});

let remaining = create_memo({
    let todos = todos.clone();
    move || todos.get().iter().filter(|t| !t.done).count()
});
```

On the first render, the memo's closure runs once and its result is cached.
The signals it read (`count`, `todos`) are recorded as its dependencies. On
subsequent renders, the cached value is returned without re-running the
closure. When any dependency changes, `VguiRoot::notify_dep` re-runs the memo's
closure, updates the cache if the result changed, and propagates the
notification to the memo's own dependents.

Memos are ideal for:

- Computed values derived from one or more signals.
- Filtered/sorted lists.
- Expensive transformations you don't want to repeat every render.

## Effects

`create_effect(f)` runs a side effect immediately (during the first render) and
re-runs it whenever its dependencies change:

```rust
create_effect({
    let count = count.clone();
    move || {
        eprintln!("count changed to: {}", count.get());
    }
});
```

> **Note:** Effects run synchronously during render, not after paint. The
> first invocation happens at registration time. Re-entrancy from the first
> effect calling setters is user-visible and not deferred.

Effects are useful for:

- Logging / debugging.
- Persisting state to disk.
- Synchronizing external systems.

## Cleanup

`on_cleanup(f)` registers a callback that runs when the current scope is
disposed. Disposal happens when:

- A `<Switch>` branch becomes inactive (the user switches to another branch).
- An `<Index>` item is removed (the list shrinks below the item's position).
- A route change disposes the previous route's child scope.

```rust
let (mode, set_mode) = create_signal("edit");

// Inside a <Switch> branch:
on_cleanup(move || {
    eprintln!("edit scope disposed — saving draft");
});
```

`on_cleanup` uses the same slot-caching pattern as `create_effect`: on
re-renders the slot is recognised by position and the callback is not
re-registered. The callback runs once, when the scope is disposed.

> **Note:** `on_cleanup` is a no-op when no reactive scope is active (e.g.
> in standalone tests), so it can be safely called in test `view!`s without
> panicking.

Cleanups run depth-first: children are disposed before their parent. Within
a scope, cleanups run in registration order.

## Dependency Tracking

Dependency tracking is automatic and fine-grained. When `ReadSignal::get()` is
called, it pushes its `gpui::EntityId` onto a thread-local `TRACKING` list.
The tracking list is active during:

- `create_memo`'s initial computation.
- `create_effect`'s initial run.

After the closure completes, the collected entity IDs are stored as the
memo/effect's dependency set. When a signal changes, `VguiRoot::notify_dep`
iterates all memos and effects, and re-runs only those whose dependency set
contains the changed signal's entity ID.

```rust
let (a, set_a) = create_signal(1);
let (b, set_b) = create_signal(2);

let sum = create_memo({
    let a = a.clone();
    let b = b.clone();
    move || a.get() + b.get() // depends on both a and b
});

let only_a = create_memo({
    let a = a.clone();
    move || a.get() * 10 // depends only on a
});

// Changing b re-runs sum but not only_a
set_b.set(cx, 5);

// Changing a re-runs both sum and only_a
set_a.set(cx, 10);
```

## Auto IDs

The `view!` macro automatically assigns stable element IDs to elements that
need them — those with `on:click`, `hover`, `active`, `focus`, `class`, or
`tabindex` attributes but no explicit `id`.

`next_auto_id()` returns a `u64` from a per-render counter that resets to 0 on
every render. This means the same logical element receives the same id across
re-renders (preserving `gpui` stateful state like focus and interaction), while
distinct elements — such as siblings produced by a `<For>` closure invoked
multiple times — receive distinct ids.

When no reactive scope is active (e.g., in standalone tests), a fallback
counter starting at `u64::MAX / 2` is used to avoid collisions.

You rarely need to call `next_auto_id()` directly — the macro handles it
automatically. You only need it if you construct stateful `gpui` elements
outside of `view!`.
