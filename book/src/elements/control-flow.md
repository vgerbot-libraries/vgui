# Control Flow

`vgui` provides four control-flow constructs in `view!`: `<Show>` for
conditional rendering, `<For>` for list rendering, `<Switch>`/`<Match>` for
multi-branch conditional branching, and `<Index>` for keyed-by-position list
rendering with per-item state. These are special-cased by the `view!` macro
and expand to function calls in the `vgui` crate.

## `<Show>` — Conditional Rendering

`<Show>` conditionally renders its children based on a `when` boolean
expression. An optional `fallback` renders when the condition is false.

### Without fallback

When `when` is false, renders nothing (`gpui::Empty`):

```rust
view! {
    <Show when={count.get() > 0}>
        <span>{"positive"}</span>
    </Show>
}
```

Expands to:

```rust
vgui::show_when(count.get() > 0, { <span>{"positive"}</span> })
```

### With fallback

When `when` is false, renders the `fallback`:

```rust
view! {
    <Show when={count.get() & 1 == 1} fallback={view! { <span>{"even"}</span> }}>
        <span>{"odd"}</span>
    </Show>
}
```

Expands to:

```rust
vgui::show(count.get() & 1 == 1, { <span>{"odd"}</span> }, { <span>{"even"}</span> })
```

### Required attributes

| Attribute   | Type   | Required | Description              |
| ----------- | ------ | -------- | ------------------------ |
| `when`      | `bool` | Yes      | Condition expression.    |
| `fallback`  | element| No       | Element when `when` is false. |

No other attributes are accepted on `<Show>` — the macro produces a compile
error for unsupported attributes.

### Multiple children

`<Show>` can have multiple children; they are added directly to the parent
element:

```rust
<div class="flex flex-col">
    <Show when={is_loading.get()}>
        <span>{"Loading..."}</span>
        <progress value={0.5f64} max={1.0f64} />
    </Show>
</div>
```

Both `<span>` and `<progress>` become direct flex children of the `<div>`.

## `<For>` — List Rendering

`<For>` iterates over a collection and renders a closure for each item. An
optional `fallback` renders when the collection is empty.

### Basic usage

```rust
view! {
    <For each={todos.get()}>
        {move |todo: Todo, _i: usize| todo_item(todo)}
    </For>
}
```

Expands to:

```rust
vgui::for_each(todos.get(), move |todo: Todo, _i: usize| todo_item(todo))
```

### With fallback

```rust
view! {
    <For each={visible_todos.get()} fallback={view! {
        <div>{"No todos here."}</div>
    }}>
        {move |todo: Todo, _i: usize| todo_item(todo, set_todos.clone())}
    </For>
}
```

Expands to:

```rust
vgui::for_each_or(visible_todos.get(), { <div>{"No todos here."}</div> }, move |todo, _i| todo_item(todo, set_todos.clone()))
```

### Required attributes and children

| Attribute   | Type             | Required | Description                    |
| ----------- | ---------------- | -------- | ------------------------------ |
| `each`      | `impl IntoIterator` | Yes  | The collection to iterate.     |
| `fallback`  | element          | No       | Element when collection is empty. |

The child of `<For>` must be exactly one interpolation `{...}` containing a
closure. The closure signature is `move |item: T, index: usize| -> impl IntoElement`.

```rust
{move |item: ItemType, index: usize| view! { <div>{format!("Item {}", item.name)}</div> }}
```

### How it works

The macro emits a `for` loop that calls `parent.child(closure(item, i))` for
each item. When the iterator is empty and no `fallback` is provided, nothing
is added. When a `fallback` is provided and the iterator is empty, the
fallback element is added as a single child.

> **Note:** `<For>` re-renders all items on every render — it does not
> key-track individual items for minimal diffing. The closure receives
> `(item, index)`, and each invocation produces a fresh element. Stateful
> widgets inside the closure (text inputs, sliders) are persisted via the
> reactive scope slot mechanism, which assigns stable ids based on call order.

## `<Switch>`/`<Match>` — Multi-Branch Conditional

`<Switch>` renders the first `<Match>` child whose `when` condition is true.
An optional `fallback` renders when no branch matches. Each branch gets its
own persistent child scope, so signals/memos/effects created inside a branch
survive across re-renders as long as that branch remains active. When the
active branch changes, the previously active branch's scope is disposed and
its `on_cleanup` callbacks run.

### Basic usage

```rust
view! {
    <Switch fallback={view! { <div>{"no match"}</div> }}>
        <Match when={status.get() == "loading"}>
            <div>{"Loading..."}</div>
        </Match>
        <Match when={status.get() == "error"}>
            <div>{"Error!"}</div>
        </Match>
        <Match when={status.get() == "ready"}>
            <div>{"Ready"}</div>
        </Match>
    </Switch>
}
```

The `when` expressions are evaluated in order; the first `true` branch wins
(short-circuit). If none match, the `fallback` renders. When `fallback` is
omitted, `gpui::Empty` is rendered.

### Without fallback

```rust
view! {
    <Switch>
        <Match when={count.get() == 0}>
            <div>{"zero"}</div>
        </Match>
        <Match when={count.get() > 0}>
            <div>{"positive"}</div>
        </Match>
    </Switch>
}
```

### Per-branch scope isolation

Each `<Match>` branch runs inside its own child scope (keyed by
`switch:{id}:{branch_index}`). This means:

- `create_signal` / `create_memo` / `create_effect` calls inside a branch
  are slot-cached per-branch, not per-render.
- State persists across re-renders as long as the same branch stays active.
- When the active branch changes, the old branch's scope is disposed:
  `on_cleanup` callbacks run, signals/memos/effects are dropped.

```rust
view! {
    <Switch>
        <Match when={mode.get() == "edit"}>
            // This signal persists as long as the "edit" branch is active.
            // When the user switches to another branch, the scope is
            // disposed and on_cleanup runs.
            {|| {
                let (draft, set_draft) = create_signal(String::new());
                on_cleanup(move || eprintln!("edit scope disposed"));
                view! {
                    <input type="text" value={draft.get()} on:input={input_cb(move |v, _| set_draft.set(v.clone()))} />
                }
            }}
        </Match>
        <Match when={mode.get() == "view"}>
            <div>{"Read-only view"}</div>
        </Match>
    </Switch>
}
```

### Required attributes and children

| Element     | Attribute  | Type   | Required | Description              |
| ----------- | ---------- | ------ | -------- | ------------------------ |
| `<Switch>`  | `fallback` | element| No       | Element when no `<Match>` matches. |
| `<Match>`   | `when`     | `bool` | Yes      | Condition expression.    |

`<Switch>` children must be `<Match>` elements only — any other child node
produces a compile error. `<Match>` accepts only the `when` attribute; other
attributes produce a compile error.

### How it works

The macro expands to:

```rust
{
    let __switch_id = vgui::next_auto_id();
    let __active: Option<usize> =
        if status.get() == "loading" { Some(0) }
        else if status.get() == "error" { Some(1) }
        else if status.get() == "ready" { Some(2) }
        else { None };
    vgui::__switch_dispose_inactive(__switch_id, __active, 3);
    match __active {
        Some(0) => {
            vgui::__switch_enter_branch(__switch_id, 0);
            parent = parent.child(<div>{"Loading..."}</div>);
            vgui::__switch_exit_branch();
        }
        // ... other arms
        _ => { /* fallback or nothing */ }
    }
}
```

`__switch_dispose_inactive` removes and disposes child scopes for all
non-active branches before entering the active one. `__switch_enter_branch` /
`__switch_exit_branch` wrap the branch body in a child scope.

## `<Index>` — Keyed-by-Position List

`<Index>` iterates over a collection and renders a closure for each item.
Unlike `<For>`, each item gets its own persistent child scope (keyed by
`index:{list_id}:{position}`), so state created inside the closure (signals,
memos, effects) survives re-renders as long as the item remains at the same
position. When the list shrinks, excess scopes are disposed and their
`on_cleanup` callbacks run.

### Basic usage

```rust
view! {
    <Index each={items.get()}>
        {move |item: String, i: usize| view! {
            <div>{format!("Item {}: {}", i, item)}</div>
        }}
    </Index>
}
```

Expands to:

```rust
vgui::index_list(items.get(), move |item: String, i: usize| view! {
    <div>{format!("Item {}: {}", i, item)}</div>
})
```

### With fallback

```rust
view! {
    <Index each={visible.get()} fallback={view! {
        <div>{"No items."}</div>
    }}>
        {move |item: Item, i: usize| item_row(item, i)}
    </Index>
}
```

### Required attributes and children

| Attribute   | Type             | Required | Description                    |
| ----------- | ---------------- | -------- | ------------------------------ |
| `each`      | `impl IntoIterator` | Yes  | The collection to iterate.     |
| `fallback`  | element          | No       | Element when collection is empty. |

The child of `<Index>` must be exactly one interpolation `{...}` containing a
closure. The closure signature is `move |item: T, index: usize| -> impl IntoElement`.

### `<For>` vs `<Index>`

| Feature              | `<For>`                        | `<Index>`                              |
| -------------------- | ------------------------------ | -------------------------------------- |
| Per-item scope       | No (shared parent scope)       | Yes (child scope per position)         |
| State persistence    | Via slot order in parent scope | Via dedicated child scope per position |
| Disposal on shrink   | No                             | Yes (`on_cleanup` runs)                |
| Use case             | Simple lists, stateless items  | Lists with per-item state (inputs, toggles) |

`<For>` is lighter-weight — items share the parent's reactive scope and rely
on slot-call-order for state identity. `<Index>` is heavier but gives each
item its own isolated scope, making it suitable for lists where each item
has independent stateful widgets (text inputs, checkboxes) that should be
cleaned up when the item is removed.

### How it works

The macro assigns a unique `list_id` via `next_auto_id()`, then for each item
enters a child scope keyed by `index:{list_id}:{position}` before calling the
closure and exits it after. After all items are rendered,
`__index_dispose_excess(list_id, n)` removes and disposes child scopes for
positions `>= n` (items that no longer exist).
