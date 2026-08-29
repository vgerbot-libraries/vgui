# Control Flow

`vgui` provides two control-flow constructs in `view!`: `<Show>` for
conditional rendering and `<For>` for list rendering. These are special-cased
by the `view!` macro and expand to function calls in the `vgui` crate.

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

`<Show>` can have multiple children; they are wrapped in a `div`:

```rust
<Show when={is_loading.get()}>
    <span>{"Loading..."}</span>
    <progress value={0.5f64} max={1.0f64} />
</Show>
```

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

`for_each` creates a `gpui::div()` and calls `.child(closure(item, i))` for
each item in the iterator. `for_each_or` does the same, but renders the
fallback element when the iterator is empty.

> **Note:** `<For>` re-renders all items on every render — it does not
> key-track individual items for minimal diffing. The closure receives
> `(item, index)`, and each invocation produces a fresh element. Stateful
> widgets inside the closure (text inputs, sliders) are persisted via the
> reactive scope slot mechanism, which assigns stable ids based on call order.
