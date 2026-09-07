# Virtual List (`<uniform_list>`)

`<uniform_list>` exposes gpui's `uniform_list` primitive for rendering large
fixed-height lists efficiently. Only the visible items are materialised —
10,000+ item lists scroll smoothly because off-screen rows are never laid out
or painted.

## Syntax

```rust
view! {
    <uniform_list
        count={item_count}
        render={|range, window, cx| {
            range.map(|i| view! { <div>{format!("Item {i}")}</div> }).collect()
        }}
    />
}
```

### Attributes

| Attribute | Required | Description |
|-----------|----------|-------------|
| `count` | Yes | `usize` — total number of items. |
| `render` | Yes | Closure `Fn(Range<usize>, &mut Window, &mut App) -> Vec<R>` where `R: IntoElement`. |
| `scroll_handle` | No | `UniformListScrollHandle` for programmatic scroll control. |
| `style` | No | CSS-in-Rust style object (`css! { ... }`). |
| `class` | No | Tailwind class string. |

`<uniform_list>` cannot have children. Unsupported attributes produce a
compile-time error.

## Height Constraint

The element **or an ancestor** must have a definite or max height for
virtualization to engage. Without it, gpui's `ListSizingBehavior::Infer`
makes the list as tall as all items (no virtualization).

Use `h-full` on the `<uniform_list>` and ensure the parent has a bounded
height, or use an explicit height like `h-[500px]`:

```rust
view! {
    <div class="flex-1 min-h-0">
        <uniform_list
            count={count}
            render={|range, _w, _cx| { /* ... */ }}
            class="h-full overflow-y-scroll"
        />
    </div>
}
```

## Programmatic Scroll

Use `use_scroll_handle()` to obtain a persistent `UniformListScrollHandle`,
then pass it to `scroll_handle={...}`:

```rust
let scroll_handle = use_scroll_handle();

view! {
    <button on:click={click({
        let sh = scroll_handle.clone();
        move |cx| {
            sh.scroll_to_item(5000, ScrollStrategy::Top);
            cx.refresh_windows();
        }
    })}>
        {"Scroll to 5000"}
    </button>
    <uniform_list
        count={count}
        render={|range, _w, _cx| { /* ... */ }}
        scroll_handle={scroll_handle.clone()}
    />
}
```

Available methods on `UniformListScrollHandle`:

- `scroll_to_item(ix, ScrollStrategy::Top|Center|Bottom|Nearest)`
- `scroll_to_bottom()`
- `is_scrolled_to_end() -> Option<bool>`

`ScrollStrategy` variants: `Top`, `Center`, `Bottom`, `Nearest`.

## Render Closure Constraints

- **`Fn` (not `FnMut`)**: gpui requires the closure to be `Fn`. Use
  `Rc<RefCell<T>>` for interior mutability inside the closure.

- **Signal reads work**: `ReadSignal::get()` reads from a cached value and
  does not require an active reactive scope. The closure runs during gpui's
  prepaint phase (after render), so cached signal values are available.

- **No signal creation**: `create_signal` inside the closure does **not**
  work — no reactive scope is active during prepaint. Create signals in the
  `view!` body (the render scope) and capture them by clone.

## Dynamic Item Counts

The `count` attribute is re-evaluated every render. When a signal drives the
count, updating the signal causes a re-render with the new count:

```rust
let (count, set_count) = create_signal(10_000usize);

view! {
    <uniform_list
        count={count.get()}
        render={|range, _w, _cx| { /* ... */ }}
    />
}
```
