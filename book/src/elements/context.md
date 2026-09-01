# Context & Provider

vgui provides a SolidJS-style context API for dependency injection. Values are
provided by ancestor elements and consumed by descendants via a typed key,
without prop drilling.

## `Context<T>` — typed marker

`Context<T>` is a zero-sized, `const`-constructable typed marker. It is keyed
by `TypeId` of `T`, so one context exists per type. Store it in a plain
`static`:

```rust
use vgui::prelude::Context;

static THEME: Context<Theme> = Context::new();
```

`Context<T>` uses `PhantomData<fn() -> T>`, making it `Copy`/`Clone` and
`Send + Sync` regardless of `T` — no bounds are required on `T` to live in a
`static`. For multiple contexts of the same logical type, use newtype wrappers.

## `<Provider>` — declarative provider

The `<Provider>` builtin pushes a value onto a thread-local stack before
evaluating its children and pops it after. Descendants constructed between
enter and exit observe the pushed value.

```rust
use vgui::prelude::*;
use vgui::view;

static THEME: Context<Theme> = Context::new();

fn app() -> impl gpui::IntoElement {
    let theme = Theme { /* ... */ };
    view! {
        <Provider context={THEME} value={theme}>
            <Child />
        </Provider>
    }
}
```

Both `context` and `value` attributes are required; other attributes are
rejected. The `context` attribute takes a `Context<T>` and `value` takes the
corresponding `T` (which must be `Clone + 'static`).

## Consuming context

| Function | Signature | Description |
|----------|-----------|-------------|
| `use_context` | `(&Context&lt;T&gt;) -> Option&lt;T&gt;` | Returns the nearest ancestor provider's value, or `None` if no provider is active. |
| `use_context_or` | `(&Context&lt;T&gt;, \|\| T) -> T` | Like `use_context`, falling back to the supplied default when no provider is active. |

Both walk the thread-local provider stack top-down and return the nearest
matching entry.

```rust
fn child() -> impl gpui::IntoElement {
    let theme = use_context_or(&THEME, || Theme::default());
    view! {
        <div style={theme.background}>
            {"Themed content"}
        </div>
    }
}
```

## `provide_context` — manual RAII provider

For use outside `view!` (tests, setup code), `provide_context` pushes a value
and returns a `ProviderGuard` that pops the stack on drop:

```rust
static THEME: Context<Theme> = Context::new();

{
    let _guard = provide_context(&THEME, Theme::dark());
    // use_context(&THEME) returns Some(Theme::dark()) here
}
// guard dropped — stack popped
```

## Nested override

An inner `<Provider>` with the same `Context<T>` shadows the outer provider
within its subtree. Descendants between the inner enter/exit see the inner
value; after the inner exit, the outer value is visible again.

## Stack scope

The provider stack is per-render, not per-module. vgui renders synchronously,
depth-first, in a single flat per-render scope — there is no nested owner
tree. The `view!` macro emits `__provider_scope_enter` before evaluating
children and `__provider_scope_exit` after, so the stack correctly reflects
the element tree's nesting at consumption time.

See the [Context example](../examples/context.md) for a complete working
demonstration.
