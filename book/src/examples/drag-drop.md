# Drag & Drop Example

## Live Demo

<iframe src="../wasm/drag-drop/" width="100%" height="560" style="border:1px solid #444; border-radius:4px;"></iframe>

## Source Code

```rust
{{#include ../../../examples/drag-drop/src/main.rs}}
```

## Key Concepts

### `drag={value}` — starting a drag

Attach `drag={value}` to any element to make it draggable. When the
user clicks and moves more than a few pixels, gpui starts a drag
operation carrying the typed value. The element automatically gets an
`.id()` so `on_drag` can register on the `StatefulInteractiveElement`.

```rust
<div drag={DragItem { id, color }} class="...">
    {format!("#{}", id)}
</div>
```

### Default drag preview

When `drag:preview` is omitted, vgui generates a default preview
constructor that clones the drag value and renders it. This works
when the drag value type implements `Clone + Render`:

```rust
#[derive(Clone, Copy)]
struct DragItem { id: usize, color: Hsla }

impl Render for DragItem { ... }
```

For types that don't impl `Clone + Render`, supply an explicit
preview constructor:

```rust
<div drag={my_value} drag:preview={move |item, pos, window, cx| {
    cx.new(|_| MyPreview { value: item.clone() })
}} class="...">
```

### `on:drop` — receiving drops

`on:drop` registers a drop listener. Rust infers the drag value type
`T` from the closure's first parameter type annotation. The listener
fires only when the dropped value's `TypeId` matches `T`.

```rust
<div on:drop={move |item: &DragItem, _window, cx| {
    set_dropped.update(cx, |_| Some(*item));
}} class="...">
```

### OS file drops with `ExternalPaths`

For files dragged from the OS file manager, the drop handler receives
`&ExternalPaths`. This works on native platforms; on WASM the handler
compiles but never fires.

```rust
<div on:drop={move |paths: &ExternalPaths, _window, cx| {
    set_files.update(cx, |_| paths.paths().iter().map(|p| p.display().to_string()).collect());
}} class="...">
```

### `on:drag_move` — tracking drag movement

`on:drag_move` fires while a drag of the matching type is moving over
the element. The handler receives `&DragMoveEvent<T>`, which provides
the mouse event, the element bounds, and access to the dragged item.

```rust
<div on:drag_move={move |e: &DragMoveEvent<TabDrag>, _w, _cx| {
    // e.bounds — element bounds
    // e.drag(_cx) — the dragged TabDrag value
}} class="...">
```

### `can_drop` — drop validation

`can_drop={predicate}` controls whether a drop is allowed. The
predicate receives `&dyn Any`, the window, and the app context, and
returns `bool`.

```rust
<div can_drop={move |value, _window, _cx| {
    value.is::<DragItem>()
}} class="...">
```

### `has_active_drag()` — imperative query

Check whether a drag operation is in progress from any reactive scope:

```rust
{format!("Drag active: {}", has_active_drag())}
```

For `stop_active_drag` and `set_active_drag_cursor_style`, call them
directly on the `App` context from within event handlers where
`&mut Window` is available:

```rust
on:click={move |_e, window, cx| {
    cx.stop_active_drag(window);
    cx.set_active_drag_cursor_style(gpui::CursorStyle::Default, window);
}}
```

### Tab reordering

The tab bar demonstrates reordering via drag. Each tab carries a
`TabDrag { ix }` value and accepts drops of the same type. On drop,
the source index is removed and inserted at the target index:

```rust
<div drag={TabDrag { ix }}
     on:drop={move |d: &TabDrag, _w, cx| {
         set_tab_order.update(cx, |order| {
             let from = d.ix;
             let to = ix;
             if from != to {
                 let item = order.remove(from);
                 order.insert(to, item);
             }
         });
     }}
     class="...">
    {label}
</div>
```

## Running

### Native

    cargo run -p vgui-drag-drop

### Web (WASM)

    cargo build --target wasm32-unknown-unknown -p vgui-drag-drop --release
    wasm-bindgen --target web --out-dir examples/drag-drop/dist \
        --no-typescript target/wasm32-unknown-unknown/release/drag-drop.wasm
    python3 scripts/serve_plain.py 8080 examples/drag-drop
