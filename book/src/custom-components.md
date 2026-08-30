# Custom Components

Any uppercase-tag element in `view!` is treated as a component call. The
`view!` macro generates different code depending on whether the component has
attributes and how many children it has.

## Component Invocation

A component is any function or struct that produces an `impl gpui::IntoElement`.
The tag name must start with an uppercase letter — lowercase tags are built-in
HTML elements.

```rust
fn greeting(name: &'static str) -> impl gpui::IntoElement {
    view! { <span>{format!("Hello, {name}!")}</span> }
}

view! {
    <div>
        <Greeting name={"world"} />
    </div>
}
```

## Without Attributes

### No children

```rust
view! {
    <Greeting />
}
```

Expands to a function call with no arguments:

```rust
Greeting()
```

### Single child

```rust
view! {
    <Greeting>{"world"}</Greeting>
}
```

Expands to:

```rust
Greeting(::vgui::into_child("world"))
```

### Multiple children

```rust
view! {
    <Greeting>
        <span>{"Hello"}</span>
        <span>{"World"}</span>
    </Greeting>
}
```

Expands to a `Vec` of children:

```rust
Greeting(::std::vec![child1, child2])
```

## With Attributes

When attributes are present, the macro generates a **struct initializer**. Each
attribute maps to a struct field:

```rust
view! {
    <Greeting name={"world"} age={42} />
}
```

Expands to:

```rust
Greeting { name: "world", age: 42 }
```

With children:

```rust
view! {
    <Card title={"My Card"} class="p-4">
        <span>{"Content"}</span>
    </Card>
}
```

Expands to:

```rust
Card {
    title: "My Card",
    class: ::vgui::tw!("p-4"),
    children: ::std::vec![child1],
}
```

### Attribute-to-field mapping

| Attribute syntax   | Field name     | Notes                                    |
| ------------------ | -------------- | ---------------------------------------- |
| `name={value}`     | `name`         | Direct field assignment.                 |
| `on:click={h}`     | `on_click`     | Event attributes map to `on_{event}`.    |
| `on:mouse_down={h}`| `on_mouse_down`|                                          |
| `style={css!{...}}`| `style`        |                                          |
| `class="..."`      | `class`        |                                          |
| `hover={css!{...}}`| `hover`        |                                          |
| `active={css!{...}}`| `active`      |                                          |
| `focus={css!{...}}`| `focus`        |                                          |
| `id="my-id"`       | `id`           |                                          |
| `src={path}`       | `src`          |                                          |
| `type="text"`      | `r#type`       | Raw identifier (Rust keyword escaping).  |
| `tabindex={0}`     | `tabindex`     |                                          |
| `for="id"`         | `r#for`        | Raw identifier.                          |
| `ref={node_ref}`    | `r#ref`        | Opt-in: component must have a `r#ref: NodeRef` field. |

## Event Mapping

Event attributes (`on:event`) are mapped to struct fields named `on_{event}`.
This means your component struct must have a field named `on_click` to receive
an `on:click` handler:

```rust
struct Button {
    on_click: impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App),
    children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Button { /* ... */ }

view! {
    <Button on:click={click(move |_cx| {})}>{"Click"}</Button>
}
```

Expands to:

```rust
Button {
    on_click: click(move |_cx| {}),
    children: ::std::vec![::vgui::into_child("Click")],
}
```

## Children

Children are always passed as the `children` field, which is a
`Vec<gpui::AnyElement>`. Each child is wrapped via `::vgui::into_child()`.

If your component has no children, simply omit the `children` field from the
struct initializer — but your struct must not require it.

### Pattern: component with props and children

```rust
fn card(title: String, children: Vec<gpui::AnyElement>) -> impl gpui::IntoElement {
    view! {
        <div class="rounded-lg p-4 bg-white shadow">
            <h3 class="font-bold text-lg">{title}</h3>
            <div class="mt-2">
                {children}
            </div>
        </div>
    }
}

// But with attributes, you need a struct:
struct Card {
    title: String,
    children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Card {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        view! {
            <div class="rounded-lg p-4 bg-white shadow">
                <h3 class="font-bold text-lg">{self.title}</h3>
                <div class="mt-2">{self.children}</div>
            </div>
        }.into_element()
    }
}

view! {
    <Card title={"My Card".to_string()}>
        <span>{"Content here"}</span>
    </Card>
}
```

> **Tip:** For simple components without attributes, prefer plain functions
> that take arguments and return `impl IntoElement`. Use struct initializers
> only when you need attribute-style syntax.

## Spread & Rest Props

The `{..expr}` spread syntax forwards a props value onto a component or
built-in element. This is vgui's equivalent of SolidJS's `{...props}` and
enables the rest-props forwarding pattern.

### Forwarding pattern

A wrapper component can forward its inner props sub-struct directly:

```rust
struct Outer {
    label: String,
    inner: Inner,
}

struct Inner {
    text: String,
    color: gpui::Hsla,
}

impl gpui::IntoElement for Outer {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        view! {
            <div>
                <span>{self.label}</span>
                <Inner {..self.inner} />
            </div>
        }
        .into_element()
    }
}
```

`<Inner {..self.inner} />` expands to `Inner { ..self.inner }` — Rust's struct
update syntax. The entire `inner` field is moved into the `Inner` initializer.

### Split-props pattern

Keep a "rest" sub-struct for forwarded props, separate from fields the wrapper
consumes itself:

```rust
struct Card {
    title: String,          // consumed by Card
    rest: CardRest,         // forwarded to inner div
}

struct CardRest {
    class: String,
    style: vgui::Css,
}

impl gpui::IntoElement for Card {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        view! {
            <div {..self.rest}>
                <h3>{self.title}</h3>
            </div>
        }
        .into_element()
    }
}
```

Here `<div {..self.rest} />` calls `Spread<gpui::Div>` (or
`Spread<gpui::Stateful<gpui::Div>>` depending on other attributes). Implement
`Spread` for `CardRest` to apply `class` and `style` onto the div.

### Override pattern

Explicit fields always win over the spread, regardless of order:

```rust
view! { <Greeting {..props} name={"override"} /> }
// → Greeting { name: "override", ..props }
```

This is Rust's struct update rule: named fields take precedence over `..base`.
Use it to override individual fields from a spread props value.

### Limitations

- **One spread per element.** Rust struct update syntax permits a single
  `..base`, so only one `{..expr}` is allowed. To merge multiple props sources,
  construct a single merged struct in Rust.
- **Built-in spread requires a `Spread<E>` impl.** See
  [Spread Attributes](../concepts/view-macro.md#spread-attributes) in the
  `view!` macro reference for the trait definition and element-type rules.
