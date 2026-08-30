# Refs & NodeRef

vgui provides a SolidJS-style `ref` system that lets you obtain a handle to a
rendered element for imperative operations — focus management, scroll control,
and layout measurement.

## Why refs?

gpui uses an immediate-mode element model: the entire element tree is rebuilt
every frame and dropped before the next. There is no persistent DOM tree, so
SolidJS-style `ref` (which returns a persistent `HTMLElement`) has no direct
equivalent.

However, gpui provides persistent **handles** that survive across frames:
`FocusHandle` (focus state) and `ScrollHandle` (scroll offset + element bounds
+ child bounds). `NodeRef` wraps both.

## Creating a NodeRef

```rust
use vgui::prelude::*;

let my_ref = NodeRef::new();
```

`NodeRef::new()` creates an **empty shell**. The handle is unbound until it's
used as a `ref=` attribute in `view!`. Calling any method before the first
render panics with a clear message — mirroring SolidJS where `ref` is
`undefined` until mount.

## Binding with `ref=`

```rust
let my_ref = NodeRef::new();

view! {
    <div ref={my_ref.clone()}>
        {"content"}
    </div>
}
```

The `ref=` attribute:

1. **Forces an auto-id** if the element has no explicit `id` (required for
   `track_focus` / `track_scroll`, which need a `StatefulInteractiveElement`).
2. Calls `__bind_ref` to cache a `FocusHandle` + `ScrollHandle` in the reactive
   scope slot, populating the `NodeRef` on first render and reusing them on
   subsequent renders.
3. Applies `track_focus(&handle)` and `track_scroll(&handle)` to the element so
   gpui keeps the handles in sync across frames.

### With explicit id

```rust
<div id="my-list" ref={my_ref.clone()}>
```

The explicit `id` is preserved — `ref=` does not overwrite it.

## Methods

Once bound, `NodeRef` exposes these imperative methods:

| Method | Signature | Description |
|--------|-----------|-------------|
| `focus` | `(&self, &mut Window, &mut App)` | Move keyboard focus to the element. |
| `is_focused` | `(&self, &Window) -> bool` | Whether the element is currently focused. |
| `contains_focused` | `(&self, &Window, &App) -> bool` | Whether the element contains the focused element. |
| `bounds` | `(&self) -> Bounds<Pixels>` | Painted bounds from the previous frame. |
| `scroll_offset` | `(&self) -> Point<Pixels>` | Current scroll offset. |
| `scroll_to` | `(&self, ix: usize)` | Scroll so child `ix` is visible (minimal scroll). |
| `scroll_to_top` | `(&self, ix: usize)` | Scroll so child `ix` is the first visible element. |
| `scroll_to_bottom` | `(&self)` | Scroll to the bottom of the content. |
| `set_scroll_offset` | `(&self, Point<Pixels>)` | Set the scroll offset explicitly. |
| `child_bounds` | `(&self, ix: usize) -> Option<Bounds<Pixels>>` | Painted bounds of child `ix`. |
| `child_count` | `(&self) -> usize` | Number of tracked children. |
| `focus_handle` | `(&self) -> FocusHandle` | Clone the underlying focus handle. |
| `scroll_handle` | `(&self) -> ScrollHandle` | Clone the underlying scroll handle. |

All methods read state from the **previous frame** — sufficient for layout
calculations and imperative actions.

## Usage in event handlers

`NodeRef` is `Clone` (internally `Rc<RefCell<...>>`). Clone it before passing
into closures so the `ref=` attribute can consume its own clone:

```rust
let scroll_ref = NodeRef::new();
let btn_ref = scroll_ref.clone();

view! {
    <div ref={scroll_ref.clone()} class="overflow-y-scroll h-64">
        {/* ... scrollable items ... */}
    </div>
    <button on:click={click(move |_cx| {
        btn_ref.scroll_to_bottom();
    })}>
        {"Scroll to bottom"}
    </button>
}
```

## Component support

Components (uppercase tags) support `ref=` on an opt-in basis. The component's
props struct must have a `r#ref: NodeRef` field:

```rust
struct MyList {
    r#ref: NodeRef,
    items: Vec<String>,
}

view! {
    <MyList r#ref={my_ref.clone()} items={data} />
}
```

Components without a `r#ref` field will produce a compile error if the user
passes `ref=`.

## Limitations

- **`<select>` and `<textarea>`**: `ref=` is not supported directly. Use a
  wrapping `<div ref={...}>` around these elements instead.
- **`<Show>` and `<For>`**: `ref=` is rejected — these are control-flow
  wrappers with no meaningful target element.
- **`<input type="text">` and `<input type="range">`**: These return
  `Entity`-backed views, not divs. `ref=` is not supported on them directly.
  For checkbox/radio/file/submit input types, `ref=` binds to the wrapper div.
