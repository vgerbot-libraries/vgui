---
title: Styling Overview
description: Choose between vgui utility classes, CSS declarations, dynamic class composition, themes, and typed component variants.
---

vgui styles GPUI elements from Rust. Utility classes and CSS-like declarations become mutations of `gpui::StyleRefinement`; vgui does not create a browser stylesheet, DOM nodes, or a CSS cascade. Tailwind-style spelling is an authoring convenience, not a promise of complete Tailwind or browser CSS compatibility.

The styling macros and types used on these pages are available from `vgui::prelude::*`.

## Choose a styling tool

| Tool | Use it when | Evaluation |
| --- | --- | --- |
| `class="..."` | You have a literal utility string in `view!` and want interaction, responsive, animation, or transition handling. | `view!` passes the literal through `tw!` at compile time. |
| `style={css! { ... }}` | You prefer CSS declarations, need supported Rust interpolation, or use `var()` theme values. | Declarations and values are checked by the `css!` macro at compile time. |
| `tw!("...")` | You need to construct a `TwStyle` explicitly rather than let `class` do it. | A literal uses the compile-time path; a non-literal expression delegates to `IntoTwStyle`. |
| `class={twc!(...)}` | You are composing conditional class fragments. | `twc!` builds a `TwClass`; `class` converts it to `TwStyle` at runtime. |
| `class={classes}` or `tw_dynamic(classes)` | The complete class string is only known at runtime. | The runtime interpreter parses the supported utilities when the style is built. `tw_dynamic` is a function, not a macro. |
| `variants!` | A component has typed dimensions such as color and size, with a shared base style. | The macro generates enums, a variants struct, builders, and `ApplyStyle`. |

For normal elements, prefer `class` over applying `tw!` through `style`. The `class` integration is what wires hover/focus/active closures, breakpoints, and animation configuration into the element; `TwStyle`'s general `ApplyStyle` implementation applies only its base closure.

## Start with static utilities

```rust
use vgui::prelude::*;

fn card() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-3 p-4 w-[320px] rounded-lg bg-white shadow-md">
            <span class="text-lg font-semibold text-gray-900">{"Account"}</span>
            <button class="px-4 py-2 rounded bg-blue-600 hover:bg-blue-700 text-white">
                {"Open"}
            </button>
        </div>
    }
}
```

No Tailwind installation or generated CSS file is required. The supported class names are translated directly to GPUI style fields. Unknown utility names are silently ignored, so a misspelling can result in missing styling rather than a compiler error.

See [Tailwind-Style Classes & Variants](../tailwind-and-variants/) for utility families, runtime composition, breakpoints, and `variants!`.

## Use CSS-like declarations when they are clearer

```rust
let accent = gpui::rgb(0x2563eb);

let element = view! {
    <button
        style={css! {
            padding: 8px 16px;
            border-radius: 6px;
            background: {accent};
            color: #ffffff;
        }}
        hover={css! { opacity: 0.85; }}
        focus={css! { border-width: 2px; border-color: #111827; }}
    >
        {"Save"}
    </button>
};
```

`css!` supports a focused set of GPUI-backed properties and values, including layout, spacing, sizing, color, typography, borders, and selected visual effects. Supported `{expr}` interpolation accepts Rust values for the target property. Unlike unknown utility classes, unknown CSS properties and unsupported values produce compile errors.

Pseudo-selectors do not go inside `css!`. Use the element's `hover`, `focus`, and `active` attributes as above, or the corresponding utility prefixes in `class`.

Theme variables use the same declaration syntax:

```rust
let themed = css! {
    --accent: #2563eb;
    color: var(--accent);
    background: var(--surface, #ffffff);
};
```

A local custom property supplies a default for that block; an installed `Theme` can override it. [Theming & Animation](../theming-and-animation/) explains resolution, scoped themes, reactive switching, and the current animation limits.

## Compose runtime classes

```rust
let selected = true;
let classes = twc!(
    "px-4 py-2 rounded",
    selected.then_some("bg-blue-600 text-white"),
    (!selected).then_some("bg-gray-200 text-gray-900")
);

let button = view! {
    <button class={classes}>{"Filter"}</button>
};
```

`twc!` accepts class strings, optional class sources, and nested `TwClass` values. Classes are applied sequentially per variant; when two classes write the same GPUI field, the later one wins. A plain `String` or `&str` expression can also be passed to `class`, while `tw_dynamic("flex p-4")` exposes the runtime interpreter directly.

## Keep the GPUI boundary in mind

- Styling support is a subset chosen for GPUI, not a browser compatibility layer.
- Classes and declarations do not form the global cascade of rules found in an external browser stylesheet.
- Static and dynamic utilities share the supported class parser, but dynamic strings still cannot use utilities that vgui does not implement.
- Animation and transition names map to GPUI's imperative animation API and have narrower behavior than CSS.

Continue with [Tailwind-Style Classes & Variants](../tailwind-and-variants/) or [Theming & Animation](../theming-and-animation/). For exact Rust signatures, use the [vgui API documentation](../../api/vgui/).
