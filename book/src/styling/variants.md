# Component Variants (`variants!`)

The `variants!` macro provides a declarative way to define component variant
dimensions — for example, a `Button` with `primary`/`secondary`/`danger` colors
and `sm`/`md`/`lg` sizes. Instead of writing manual `if/else` chains that select
between `css!{...}` blocks, you declare the dimensions and their options once;
the macro generates the enums, a combined `Copy` struct, and an `ApplyStyle`
impl that applies the base style plus each selected dimension style
sequentially.

## Syntax

```rust
variants! {
    Button {
        base => css! {
            border-radius: 4px;
            cursor: pointer;
        },

        variant {
            primary => css! { background: #2563ff; color: #fff; },
            secondary => css! { background: #6c757d; color: #fff; },
            danger => css! { background: #dc2626; color: #fff; },
        },

        size {
            sm => css! { padding: 4px 8px; font-size: 12px; },
            md => css! { padding: 8px 16px; font-size: 14px; },
            lg => css! { padding: 12px 24px; font-size: 16px; },
        },
    }
}
```

Each entry inside the body is either:

- `base => <expr>` — a style expression (typically `css!{...}`) applied to
  every instance. Optional; at most one.
- `<dimension> { <option> => <expr>, ... }` — a dimension with named options,
  each mapping to a style expression. One or more dimensions.

The macro requires at least a `base` or one dimension. Each dimension must have
at least one option.

## Generated Types

For the `Button` definition above, the macro generates:

| Type | Description |
| --- | --- |
| `enum ButtonVariant { Primary, Secondary, Danger }` | One enum per dimension; name = `{Component}{Dimension}` in PascalCase. |
| `enum ButtonSize { Sm, Md, Lg }` | Option names are PascalCased (`sm` → `Sm`). |
| `struct ButtonVariants { pub variant: ButtonVariant, pub size: ButtonSize }` | Combined `Copy` struct; name = `{Component}Variants`. |
| `impl Default for ButtonVariants` | Selects the **first** option in each dimension. |
| `impl ButtonVariants { fn variant(..), fn size(..) }` | Builder methods (one per dimension, consuming `self`). |
| `impl ApplyStyle<E> for ButtonVariants` | Applies `base` then each dimension's selected style in order. |

All generated enums and the struct derive `Clone, Copy, PartialEq, Eq, Debug`.

## Usage with `view!`

The generated `ButtonVariants` struct implements `ApplyStyle`, so it drops
directly into `style={...}` with no changes to the `view!` macro:

```rust
view! {
    <button style={ButtonVariants::default().variant(ButtonVariant::Danger).size(ButtonSize::Lg)}>
        {"Delete"}
    </button>
}
```

`Default` selects the first option per dimension, so `ButtonVariants::default()`
gives you `Primary` + `Sm`. Chain builder methods to override:

```rust
ButtonVariants::default()
    .variant(ButtonVariant::Secondary)
    .size(ButtonSize::Md)
```

## Usage in Component Structs

Store the enum values as fields on your component struct, then compose a
`ButtonVariants` in the `IntoElement` impl:

```rust
pub struct Button {
    pub variant: ButtonVariant,
    pub size: ButtonSize,
    pub on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Button {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let variants = ButtonVariants::default()
            .variant(self.variant)
            .size(self.size);
        let on_click = self.on_click;
        let children = self.children;
        view! {
            <button style={variants} on:click={click(move |cx| on_click(cx))}>
                {vgui::for_each(children, |c, _| c)}
            </button>
        }
        .into_any_element()
    }
}
```

## Edge Cases

- **No `base`**: omit the `base => ...` line; the `ApplyStyle` impl applies only
  the dimension styles.
- **No dimensions** (only `base`): generates a unit struct `ButtonVariants;`
  whose `ApplyStyle` applies only the base style.
- **Single dimension**: works normally; the struct has one field.
- **Keyword names**: if a dimension or option name is a Rust keyword (e.g.
  `type`, `move`), the macro emits raw identifiers (`r#type` for the field/
  method, `r#Type` for the enum variant).

## `tw!` Limitation

`tw!("...")` expressions work as variant style values, but only the **base**
class is applied (via `TwStyle`'s `ApplyStyle` impl). Hover/focus/active states
from `tw!` are not applied by the generated `ApplyStyle` impl. For interactive
states, use `css!` for variant styles and the `class`/`hover`/`focus`/`active`
attributes on the element itself.

## Full Example

See `examples/variants/src/main.rs` for a complete dual-target example that
renders a grid of buttons across all variant × size combinations.

### Running

**Native:**

    cargo run -p vgui-variants

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-variants --release
    wasm-bindgen --target web --out-dir examples/variants/dist \
        --no-typescript target/wasm32-unknown-unknown/release/variants.wasm
    python3 scripts/serve_plain.py 8080 examples/variants
