# The `view!` Macro

## Syntax Overview

The `view!` macro parses JSX-like syntax and expands it into `gpui` element
builder expressions at compile time. It is a hand-rolled token-tree parser —
no external parser crate — so the syntax is close to JSX but with Rust-specific
extensions.

A `view!` invocation takes a single root node (element, fragment, or
interpolation):

```rust
view! {
    <div class="p-4">
        <span>{"Hello"}</span>
    </div>
}
```

The macro expands to `{ let el = EXPR; el }`, where `EXPR` is a chain of
`gpui` builder calls.

### Node types

A `view!` body can contain four kinds of nodes:

| Node          | Syntax                    | Expands to                               |
| ------------- | ------------------------- | ---------------------------------------- |
| Element       | `<div>...</div>`          | `gpui::div().child(...)...`              |
| Fragment      | `<>...</>`                | `gpui::div().child(...)` (anonymous div) |
| Interpolation | `{expr}`                  | `::vgui::into_child(expr)`               |
| Text          | `"literal"` or `{"expr"}` | `::vgui::into_child("literal")`          |

## Elements

### Built-in HTML elements (lowercase tags)

Any lowercase tag maps to a built-in HTML element. See
[Built-in HTML Elements](../elements/builtin-elements.md) for the full list.
Most elements expand to `gpui::div()` with appropriate default styling:

```rust
view! {
    <div>      // → gpui::div()
    <span>     // → gpui::div()
    <button>   // → gpui::div().cursor_pointer()
    <h1>       // → gpui::div().text_size(rems(2.0)).font_weight(600)
    <strong>   // → gpui::div().font_weight(FontWeight::BOLD)
    <a>        // → gpui::div().cursor_pointer().text_color(blue)
}
```

### Custom components (uppercase tags)

Any tag starting with an uppercase letter is treated as a component call. See
[Custom Components](../custom-components.md) for details:

```rust
view! {
    <Greeting name={"world"} />
    // → Greeting { name: "world" }
}
```

### Control-flow components

`<Show>`, `<For>`, `<Switch>`, and `<Index>` are special-cased by the macro
and expand to `vgui::show` / `vgui::show_when` / `vgui::for_each` /
`vgui::for_each_or` / `vgui::index_list` / `vgui::index_list_or` function
calls, plus hidden scope-management helpers for `<Switch>`. See
[Control Flow](../elements/control-flow.md).

`<Provider>` is special-cased to push a context value before evaluating its
children and pop it after, via `vgui::__provider_scope_enter` /
`vgui::__provider_scope_exit`. See [Context & Provider](../elements/context.md).

### Self-closing and void elements

Both `<input type="text">` and `<input type="text" />` are accepted. Void
elements like `<br>` and `<hr>` never have children. `<input>`, `<textarea>`,
and `<select>` reject child nodes — they are configured entirely through
attributes.

## Attributes

Attributes appear inside the opening tag as `name=value` pairs. Values can be:

| Form             | Example              | Notes                                    |
| ---------------- | -------------------- | ---------------------------------------- |
| String literal   | `class="flex p-4"`   | Parsed at compile time where relevant.   |
| Expression       | `on:click={expr}`    | Any Rust expression in braces.           |
| Boolean literal  | `disabled={true}`    | `true` / `false` literals.               |
| Integer literal  | `tabindex={0}`       | Numeric literal or `{expr}`.             |

### Attribute categories

| Attribute     | Syntax              | Applies to              | Effect                                    |
| ------------- | ------------------- | ----------------------- | ----------------------------------------- |
| `style`       | `style={css!{...}}` | All elements            | Applies CSS-in-Rust styles.               |
| `class`       | `class="..."`       | All elements            | Expands via `tw!` to Tailwind utilities.  |
| `hover`       | `hover={css!{...}}` | All elements            | Pseudo-state style on hover.              |
| `active`      | `active={css!{...}}`| All elements            | Pseudo-state style on mouse-down.         |
| `focus`       | `focus={css!{...}}` | All elements            | Pseudo-state style on focus.              |
| `id`          | `id="my-id"`        | All elements            | Sets the gpui element id.                 |
| `tabindex`    | `tabindex={0}`      | All elements            | `≥0` sets tab order; `<0` is focusable.   |
| `on:event`    | `on:click={handler}`| All elements            | Attaches an event handler.                |
| `type`        | `type="text"`       | `<input>` only          | Selects the input widget kind.            |
| `src`         | `src={path}`        | `<img>`, `<svg>` only   | Image/SVG path.                           |
| `for`         | `for="id"`          | `<label>` only          | Associates label with input by id.        |
| `ref`         | `ref={node_ref}`    | All elements            | Binds a `NodeRef` handle for imperative ops (focus, scroll, bounds). |
| `role`        | `role="button"`     | All elements            | Sets ARIA role via `__resolve_aria_role`. |
| `aria:name`   | `aria:label="..."`  | All elements            | Sets ARIA attribute (`aria:label`, `aria:description`, `aria:keyshortcuts`, `aria:selected`, `aria:expanded`, `aria:toggled`, `aria:valuenow`/`aria:numeric_value`, `aria:value`, `aria:placeholder`, `aria:numeric_value_step`). |

### Event handlers

Events use the `on:event={handler}` syntax. Supported events:

| Event                 | Handler signature                                          |
| --------------------- | --------------------------------------------------------- |
| `on:click`            | `Fn(&ClickEvent, &mut Window, &mut App)`                  |
| `on:keydown`          | `Fn(&KeyboardEvent, &mut Window, &mut App)`               |
| `on:keyup`            | `Fn(&KeyboardEvent, &mut Window, &mut App)`               |
| `on:pointerdown`      | `Fn(&PointerEvent, &mut Window, &mut App)`                |
| `on:pointerup`        | `Fn(&PointerEvent, &mut Window, &mut App)`                |
| `on:pointermove`      | `Fn(&PointerEvent, &mut Window, &mut App)`                |
| `on:resize`           | `Fn(&ResizeEvent, &mut Window, &mut App)`                 |
| `on:scroll`           | `Fn(&ScrollWheelEvent, &mut Window, &mut App)`            |
| `on:wheel`            | `Fn(&WheelEvent, &mut Window, &mut App)`                  |
| `on:dblclick`         | `Fn(&PointerEvent, &mut Window, &mut App)`                |
| `on:contextmenu`      | `Fn(&PointerEvent, &mut Window, &mut App)`                |
| `on:modifiers_changed`| `Fn(&ModifiersChangedEvent, &mut Window, &mut App)`       |
| `on:mouse_down_out`   | `Fn(&MouseDownEvent, &mut Window, &mut App)`              |
| `on:mouse_up_out`     | `Fn(&MouseUpEvent, &mut Window, &mut App)`                |
| `on:any_mouse_down`   | `Fn(&MouseDownEvent, &mut Window, &mut App)`              |

For `on:click`, the `click` helper wraps a simpler closure:

```rust
on:click={click(move |cx: &mut App| { /* ... */ })}
```

For `<input>`, two additional events are available:

| Event        | Handler signature (text-based)       | Handler signature (checkbox/radio) | Handler signature (range) | Handler signature (file) |
| ------------ | ------------------------------------ | ---------------------------------- | ------------------------- | ------------------------ |
| `on:input`   | `FnMut(&str, &mut App)`              | —                                  | —                         | —                        |
| `on:change`  | `FnMut(&str, &mut App)`              | `FnMut(bool, &mut App)`            | `FnMut(f64, &mut App)`    | `FnMut(Vec<PathBuf>, &mut App)` |


## Spread Attributes

The `{..expr}` (or `{...expr}`) syntax spreads a props value onto an element
or component — vgui's equivalent of SolidJS's `{...props}`. This enables the
rest-props forwarding pattern.

### On components (struct update syntax)

For custom components, spread expands to Rust's struct update syntax:

```rust
view! { <Greeting {..props} /> }
// → Greeting { ..props }

view! { <Greeting {..props} name={"override"} /> }
// → Greeting { name: "override", ..props }
```

Explicit fields always override the spread, regardless of source order — this
is Rust's struct update rule (named fields win over `..base`). Only one spread
is allowed per component (Rust struct update syntax permits a single `..base`).

### On built-in elements (Spread trait)

For built-in HTML elements (`<div>`, `<button>`, …), spread calls the
`Spread<E>` trait after the element is built and children are attached:

```rust
view! { <div {..extras}>{"x"}</div> }
// → let mut el = gpui::div();
//   el = el.child("x");
//   el = ::vgui::Spread::spread(extras, el);
```

Implement `Spread<E>` for your props type, where `E` is the concrete gpui
element type after other attributes are applied:

- Bare `<div {..p} />` → `E = gpui::Div`
- With `class` / `on:click` / `ref` / `tabindex` / `id` / `active` / `focus` →
  `E = gpui::Stateful<gpui::Div>`

```rust
struct DivExtras { bg: gpui::Hsla }

impl ::vgui::Spread<gpui::Div> for DivExtras {
    fn spread(self, el: gpui::Div) -> gpui::Div {
        el.bg(self.bg)
    }
}
```

Explicit attributes are applied before `spread`, so they take precedence.

### Limitations

- **One spread per element** — both components and built-ins accept at most one
  `{..expr}`. To merge multiple props sources, construct a single merged struct
  in Rust.
- **Not supported on `<input>`, `<select>`, `<textarea>`, `<label>`** — these
  specialized elements reject spread attributes with a compile error.

## Children

Children appear between the opening and closing tags. Each child is one of the
four node types (element, fragment, interpolation, text):

```rust
view! {
    <div>
        <span>{"First child"}</span>
        {some_function()}
        "Plain text"
        <>
            <p>{"Fragment child A"}</p>
            <p>{"Fragment child B"}</p>
        </>
    </div>
}
```

Multiple children are chained via `.child()` calls:

```rust
// Expands to:
let mut el = gpui::div();
el = el.child(child1);
el = el.child(child2);
el
```

An element with no children simply omits the `.child()` chain.

## Interpolation

`{expr}` interpolates any Rust expression that implements `gpui::IntoElement`
(or `IntoViewChild`). The expression is wrapped in `::vgui::into_child(expr)`:

```rust
view! {
    <div>
        {format!("count = {}", count.get())}
        {increment_button(set_count.clone())}
    </div>
}
```

Common interpolatable values:

- `String` / `&str` — rendered as text.
- `format!(...)` — rendered as text.
- Any `impl IntoElement` — rendered as a child element.
- Component function calls returning `impl IntoElement`.

## Refs

The `ref={node_ref}` attribute binds a [`NodeRef`](../elements/refs.md) handle
to an element, enabling imperative operations like `focus()`, `scroll_to()`,
and `bounds()`. This is vgui's equivalent of SolidJS's `ref`.

```rust
let my_ref = NodeRef::new();
view! {
    <div ref={my_ref.clone()}>
        {"content"}
    </div>
}
// Later, in an event handler:
// my_ref.focus(window, cx);
// my_ref.scroll_to_bottom();
```

See [Refs & NodeRef](../elements/refs.md) for the full API.
