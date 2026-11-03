---
title: Theming & Animation
description: Resolve css! variables with Theme, switch themes through signal-driven rendering, and use vgui's GPUI-backed animations and hover transitions.
---

vgui themes supply typed values to `var()` references in `css!`. Animation and transition utilities use GPUI's imperative animation API. Neither feature is browser CSS: theme variables do not form a DOM cascade, and motion supports only the properties and behaviors described here.

## CSS variables and defaults

A custom property inside `css!` defines a compile-time fallback for references in the same block. `var(--name, fallback)` can provide a fallback at the use site instead:

```rust
let style = css! {
    --accent: #2563eb;
    --gap: 12px;
    color: var(--accent);
    gap: var(--gap);
    background: var(--surface, #ffffff);
};
```

When the style is applied, vgui looks up each variable in the current thread-local `Theme`. A theme value wins over a local or inline fallback. If neither the theme nor a fallback supplies the variable, style application panics. A value whose type does not match the target property also panics with a descriptive error.

Variables support typed colors, lengths, numbers, and keywords where the receiving `css!` property supports `var()`. For example, a keyword can drive `flex-direction`, a length can drive `gap`, and a number can drive `opacity`. `tw!` and runtime utility strings are not the theme-variable path; use `css!` for `var()`.

## Install and scope themes

Build a theme with `theme!` and install it with `set_theme`:

```rust
let light = theme! {
    --accent: #2563eb;
    --surface: #ffffff;
    --foreground: #111827;
};

set_theme(light);

let element = view! {
    <span style={css! {
        padding: 8px;
        color: var(--foreground);
        background: var(--surface);
    }}>
        {"Themed text"}
    </span>
};
```

`theme!` accepts only custom-property declarations. For values computed at runtime, use the typed `Theme` builders; names do not include the leading `--`:

```rust
let mut custom = Theme::new();
custom.set_color("accent", gpui::rgb(0x2563eb));
custom.set_number("muted-opacity", 0.5);
set_theme(custom);
```

`set_theme` replaces the current thread's theme. It is not attached to a GPUI subtree and does not provide browser-style inheritance to descendants.

Use `with_theme` for a dynamic scope around element construction:

```rust
let contrast = theme! {
    --accent: #ffff00;
};

let element = with_theme(&contrast, || {
    view! {
        <span style={css! { color: var(--accent); }}>
            {"Scoped theme"}
        </span>
    }
});
```

`with_theme` installs a clone for the closure, then restores the previous theme after the closure returns. It scopes variable resolution while styles are constructed; it is not a persistent theme-provider component.

## Switch themes through a signal

**`set_theme` is not auto-reactive.** It does not notify GPUI or request a render. The source-supported pattern is to read a signal inside the render closure and call `set_theme` there, before constructing styled elements:

```rust
fn themed_app() -> impl gpui::IntoElement {
    let (dark, set_dark) = create_signal(false);

    set_theme(if dark.get() {
        theme! {
            --surface: #111827;
            --foreground: #f9fafb;
        }
    } else {
        theme! {
            --surface: #ffffff;
            --foreground: #111827;
        }
    });

    view! {
        <button
            style={css! {
                padding: 8px 16px;
                border-radius: 6px;
                color: var(--foreground);
                background: var(--surface);
            }}
            on:click={click(move |cx| {
                set_dark.update(cx, |value| *value = !*value);
            })}
        >
            {"Toggle theme"}
        </button>
    }
}
```

The `dark.get()` read registers the render dependency. Updating `dark` requests another render; that render installs the selected theme and rebuilds the elements with the new variable values. Calling `set_theme` alone from an event handler would change the thread-local theme without causing this rerender.

## Built-in animations

The `animate-*` utilities create repeating GPUI animations:

| Class | Current behavior | Default cycle |
| --- | --- | --- |
| `animate-pulse` | Opacity moves from 1 to 0.5 and back. | 2 seconds |
| `animate-bounce` | Top margin moves from 0 to -10 px and back. | 1 second |
| `animate-ping` | Opacity reaches 0 over the first 75% of the cycle, then stays 0 until the next cycle. It does not scale the element. | 1 second |
| `animate-spin` | No visual effect; GPUI `div` elements do not provide the required rotation support. | 1 second |

```rust
let element = view! {
    <div class="p-4 rounded bg-blue-500 text-white animate-pulse duration-1000 ease-out">
        {"Loading"}
    </div>
};
```

`duration-N` sets milliseconds. The supported easing classes are `ease-linear`, `ease-in`, `ease-out`, and `ease-in-out`; built-in animations default to `ease-in-out`.

## Custom GPUI animations

For behavior beyond the built-ins, use the `animate` attribute and GPUI's `AnimationExt::with_animation` directly:

```rust
use std::time::Duration;
use gpui::{Animation, AnimationExt};

let element = view! {
    <div
        class="p-4 bg-blue-500 text-white"
        animate={|el| el.with_animation(
            "fade",
            Animation::new(Duration::from_secs(1)).repeat(),
            |el, delta| el.opacity(1.0 - delta),
        )}
    >
        {"Custom animation"}
    </div>
};
```

The closure receives the element and returns its `AnimationElement`. This is a GPUI animation, not a CSS keyframe declaration. An explicit `animate` attribute takes precedence over animation or transition configuration from `class` on that element.

## Hover transitions

A `transition-*` utility animates between base and `hover:` styles when the pointer enters or leaves an element:

| Class | Interpolated properties |
| --- | --- |
| `transition-opacity` | Opacity |
| `transition-colors` | Text color, border color, and solid background color |
| `transition`, `transition-all` | Opacity plus the supported colors above |

```rust
let element = view! {
    <button class="px-4 py-2 rounded bg-indigo-500 text-white hover:bg-blue-600 transition-colors duration-300 ease-in-out">
        {"Hover me"}
    </button>
};
```

`view!` creates the hover signal and snapshots the base and base-plus-hover styles. Transitions default to 150 ms and `ease-in-out` unless overridden. Non-selected hover properties change immediately.

This mechanism is limited to hover transitions. It does not animate focus/active/responsive changes or arbitrary signal-driven style changes. `transition-shadow` and `transition-transform` do not provide transitions, and layout properties such as margin and padding are not interpolated by `transition-all`.

## Timing and composition limits

- `delay-N` is parsed and stored, but currently has no effect; GPUI does not provide the native delay used by this configuration.
- `duration-N` and `ease-*` configure base animation/transition classes, not variant-prefixed utilities.
- If one element has both a class animation and a class transition, the animation takes priority.
- `animate-spin`, shadow transitions, and transform transitions should not be used as indicators of visible motion in the current implementation.

For class support and state prefixes, see [Tailwind-Style Classes & Variants](../tailwind-and-variants/). For help choosing `css!`, `class`, or dynamic composition, return to the [Styling Overview](../overview/).
