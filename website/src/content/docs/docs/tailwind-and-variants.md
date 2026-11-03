---
title: Tailwind-Style Classes & Variants
description: Use vgui's supported static and dynamic utilities, interaction and responsive prefixes, and typed component variants.
---

vgui accepts a focused set of Tailwind-style class names and applies them to GPUI style fields. The names are familiar, but there is no Tailwind build step and no browser CSS output. Use the examples here as representative supported forms rather than assuming every Tailwind utility exists.

## Static classes in views

A literal `class` attribute is the usual path:

```rust
let element = view! {
    <div class="flex items-center justify-between gap-4 p-4 w-full bg-slate-100 rounded-md">
        <span class="text-sm font-medium text-slate-900">{"Downloads"}</span>
        <span class="text-xs text-slate-500">{"12"}</span>
    </div>
};
```

`view!` invokes `tw!` for the attribute and connects all generated state, breakpoint, animation, and transition configuration. Calling `tw!("flex p-4")` directly returns the same `TwStyle` type, but its standalone `ApplyStyle` implementation applies only base styles; keep full styling on `class` unless you are deliberately working with `TwStyle` yourself.

Unknown or unsupported class names are skipped silently in both compile-time and runtime paths. Check the rendered style when a class appears to have no effect.

## Supported utility families

The compile-time macro and runtime interpreter share class parsing and cover the same core families:

| Family | Supported forms include |
| --- | --- |
| Layout | `flex`, `block`, `grid`, `hidden`; flex direction/wrap/grow/shrink; `justify-*`, `items-*`, `self-*`, `content-*`; relative/absolute positioning and inset edges |
| Spacing and sizing | `p-*`, `m-*`, directional and axis spacing, `gap-*`; width/height, min/max sizes, `w-full`, `h-auto`, `m-auto`; `aspect-square`, `aspect-video`, and numeric ratios |
| Typography | Palette and arbitrary text colors; `text-xs` through `text-9xl`; font weights and families; alignment, line height, italic, decoration, whitespace, truncation, and line clamp |
| Visual styling | `bg-*`, `text-*`, and `border-*` colors; opacity; border widths/styles and per-side widths; corner radii; shadows; cursor styles |
| Grid and overflow | `grid-cols-N`, `grid-rows-N`; overflow and visibility utilities |

Spacing suffixes use vgui's Tailwind-like scale, such as `p-2` for 8 px and `p-4` for 16 px. Color palettes include the standard named families with shades 50 through 950. These are GPUI mappings: similarly named browser utilities can have different behavior.

### Arbitrary values and color opacity

Supported length utilities accept bracketed `px`, `rem`, percentage, or bare-number values. Supported color utilities accept hex, `rgb(...)`, and `rgba(...)` values:

```rust
let element = view! {
    <div class="w-[500px] h-[50%] p-[12px] bg-[#505050] text-[rgb(255,255,255)] rounded-[8px]">
        <span class="bg-blue-500/50 text-white">{"Arbitrary values"}</span>
    </div>
};
```

A `/NN` suffix sets color opacity for supported palette, black, and white color utilities. Arbitrary values still depend on the target utility: brackets do not make arbitrary CSS properties available.

## Dynamic class composition

Use `twc!` when conditions select class fragments:

```rust
let selected = true;
let disabled = false;

let classes = twc!(
    "flex items-center px-4 py-2 rounded",
    selected.then_some("bg-blue-600 text-white"),
    (!selected).then_some("bg-gray-200 text-gray-900"),
    disabled.then_some("opacity-50")
);

let element = view! {
    <button class={classes}>{"Filter"}</button>
};
```

`twc!` returns `TwClass`, not `TwStyle`. Its sources can be `&str`, `String`, `Option<T>`, or another `TwClass`. The equivalent builder API is `TwClass::new().add(...).add_if(condition, ...).build()`.

Within each variant, classes are applied in order and later writes win at the GPUI field level. For example, `twc!("p-4", "p-2")` produces 8 px padding. This is sequential style refinement, not a generated Tailwind stylesheet's cascade ordering.

For a class string assembled elsewhere, pass the string directly or call the runtime function:

```rust
let runtime_classes = String::from("flex items-center gap-2 p-3 hover:bg-slate-200");
let style = tw_dynamic(runtime_classes.as_str());
let element = view! {
    <div class={style}>{"Runtime classes"}</div>
};
```

`class={some_string}` and `tw!(some_string)` also use `IntoTwStyle` for non-literal input. `tw_dynamic(&str)` is useful when you need the `TwStyle` value explicitly; there is no `tw_dynamic!` macro.

## Interaction states

The supported state prefixes are `hover:`, `focus:`, and `active:`:

```rust
let element = view! {
    <button class="px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 focus:border-2 focus:border-blue-300 active:bg-blue-800">
        {"Save"}
    </button>
};
```

`view!` maps these closures to GPUI's corresponding element state styling. For `css!`, use separate `hover={...}`, `focus={...}`, and `active={...}` attributes instead.

Use one variant prefix per utility. Do not rely on stacked combinations such as `md:hover:bg-blue-700`: the current parser stores a single variant for each class, not a combination of state and breakpoint.

## Responsive breakpoints

The responsive prefixes are mobile-first minimum widths:

| Prefix | Minimum viewport width |
| --- | --- |
| `sm:` | 640 px |
| `md:` | 768 px |
| `lg:` | 1024 px |
| `xl:` | 1280 px |

```rust
let element = view! {
    <div class="flex flex-col gap-2 p-4 md:flex-row md:gap-6 lg:p-8">
        <span>{"Navigation"}</span>
        <span>{"Content"}</span>
    </div>
};
```

During rendering, `view!` checks the current viewport width and applies every breakpoint at or below the active level, in `sm`, `md`, `lg`, `xl` order after base styles. Thus `md:flex-row` remains active at larger widths unless a later applicable breakpoint writes the same field. Outside a render viewport, breakpoint resolution falls back to base.

These are viewport-width checks in GPUI, not browser media queries. The supported prefixes are the four listed above; do not infer arbitrary or max-width breakpoints from Tailwind.

## Typed component variants

Use `variants!` for reusable component style dimensions instead of maintaining parallel conditional class selections:

```rust
variants! {
    Button {
        base => css! {
            border-radius: 6px;
            cursor: pointer;
            font-weight: 500;
        },
        variant {
            primary => css! { background: #2563eb; color: #ffffff; },
            danger => css! { background: #dc2626; color: #ffffff; },
        },
        size {
            sm => css! { padding: 4px 8px; font-size: 12px; },
            md => css! { padding: 8px 16px; font-size: 14px; },
        },
    }
}

let element = view! {
    <button
        style={ButtonVariants::default()
            .variant(ButtonVariant::Danger)
            .size(ButtonSize::Md)}
        hover={css! { opacity: 0.85; }}
    >
        {"Delete"}
    </button>
};
```

For this declaration, the macro generates `ButtonVariant`, `ButtonSize`, and `ButtonVariants`. `ButtonVariants::default()` selects the first option in each dimension (`Primary` and `Sm` here), and builder methods replace those selections. Applying the struct adds the optional base style first, then the selected style for each dimension in declaration order.

`css!` is the clearest choice for variant values and works with theme variables. A `tw!` expression can supply only base utility fields through this generic `ApplyStyle` path; its state, responsive, animation, and transition configuration is not wired by `variants!`. Keep interaction styles on the element as shown above.

For theming and motion, continue to [Theming & Animation](../theming-and-animation/). See the [Styling Overview](../overview/) for a comparison of all styling entry points.
