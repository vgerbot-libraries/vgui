---
title: Actions and Form Controls
description: Build controlled controls with Button, TextField, Switch, and the Select builder API.
---

These examples are complete render functions for a vgui application. Mount them as described in the [Quick Start](../../docs/quick-start/). Each function applies the light theme; see the [component overview](../) for signal-driven theme switching.

## Button

`Button` accepts a variant, a size, an `on:click` callback, and children. This example owns its click count and updates it through a signal:

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn click_button() -> impl gpui::IntoElement {
    apply_theme(false);
    let (clicks, set_clicks) = create_signal(0_u32);

    view! {
        <Button
            variant={ButtonVariant::Default}
            size={ButtonSize::Default}
            on:click={move |cx| set_clicks.update(cx, |count| *count += 1)}
        >
            {format!("Clicked {} times", clicks.get())}
        </Button>
    }
}
```

- `ButtonVariant`: `Default`, `Destructive`, `Outline`, `Secondary`, `Ghost`, and `Link`.
- `ButtonSize`: `Default`, `Sm`, `Lg`, and `Icon`.
- The callback receives `&mut gpui::App`, not a click event. In a struct expression, the field is `on_click: Box<dyn Fn(&mut gpui::App) + 'static>`.
- The current API has no `disabled` prop. `IconButton` is a separate glyph-based convenience component fixed to the ghost variant and icon size.

## TextField and Switch

The public module is named `input`, but its macro component is `TextField`. Both controls below are controlled: callbacks write new values back to signals, and rendering reads those signals.

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn profile_form() -> impl gpui::IntoElement {
    apply_theme(false);
    let (name, set_name) = create_signal(String::new());
    let (notifications, set_notifications) = create_signal(false);

    view! {
        <div class="flex flex-col gap-3 p-4">
            {TextField {
                value: name.get(),
                placeholder: "Display name".to_string(),
                on_input: Box::new(move |value, cx| set_name.set(cx, value)),
                class: None,
            }}
            <div class="flex flex-row items-center gap-2">
                {Switch {
                    checked: notifications.get(),
                    on_change: Box::new(move |checked, cx| {
                        set_notifications.set(cx, checked)
                    }),
                }}
                <span>
                    {if notifications.get() { "Notifications on" } else { "Notifications off" }}
                </span>
            </div>
        </div>
    }
}
```

`TextField` requires `value`, `placeholder`, and `on_input`; `class` is an optional extra class for its outer container. Its input type is fixed to text, and `on_input` receives the new `String` and `&mut gpui::App`.

`Switch` requires `checked` and `on_change`. A click requests `!checked`; the caller remains responsible for storing the new value. Use the struct expression shown above because `<Switch>` is reserved for vgui's control-flow tag. The current switch API has no label or disabled prop.

## Select

Select uses a function plus a builder, not a PascalCase tag. `ShadcnSelectProps::new` takes items, the controlled `Option<V>` value, and an `Rc` change callback. Values must implement `Clone + PartialEq + 'static`.

```rust
use std::rc::Rc;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn language_select() -> gpui::AnyElement {
    apply_theme(false);
    let (language, set_language) = create_signal(Some("rust".to_string()));

    let props = ShadcnSelectProps::new(
        vec![
            SelectItem::new("Rust", "rust".to_string()),
            SelectItem::new("TypeScript", "typescript".to_string()),
            SelectItem::new("Python", "python".to_string()).disabled(true),
        ],
        language.get(),
        Rc::new(move |value, cx| set_language.set(cx, value)),
    )
    .placeholder("Choose a language")
    .searchable(true)
    .clearable(true)
    .accessibility_label("Programming language")
    .size(SelectSize::Md);

    select(props)
}
```

### Builder options

| API | Behavior |
| --- | --- |
| `SelectItem::new(title, value)` | Creates one option. Chain `.disabled(bool)` to prevent confirmation or `.display_title(...)` to customize the selected content in the trigger. |
| `SelectGroup::new(title, items)` | Creates a titled group. Pass groups directly, or use `SelectItems::grouped(groups)`; flat items can use `SelectItems::flat(items)`. |
| `.placeholder(...)`, `.title_prefix(...)`, `.icon(...)` | Configure the trigger's empty text, selected-title prefix, and custom icon. |
| `.searchable(bool)`, `.search_placeholder(...)` | Enable search and set its placeholder. The default matcher is a case-insensitive title substring match. |
| `.matcher(...)`, `.render_item(...)`, `.empty(...)` | Override matching, customize option-row content, and render content when filtering has no matches. Row events and sizing remain owned by Select. |
| `.clearable(bool)`, `.disabled(bool)` | Allow clearing a selected value or disable the whole control. Clearing publishes `None`. |
| `.size(SelectSize::...)` | Choose `Xs`, `Sm`, `Md`, or `Lg`; the default is `Md`. |
| `.menu_width(px(...))`, `.menu_max_h(px(...))` | Set popup width and maximum height. Width follows the trigger by default; maximum height defaults to 320 px. |
| `.accessibility_label(...)`, `.id(...)`, `.focus_ring(bool)`, `.appearance(bool)` | Set the trigger label and element ID, or toggle the focus ring and shadcn appearance. |
| `.on_dismiss(...)` | Run a callback when the popup closes. Confirmation calls `on_change` before closing. |

Selection is single and controlled. Confirming an enabled option publishes `Some(value)`; dismissing without confirmation does not change the committed value. Disabled options can receive the keyboard highlight but cannot be confirmed. There is no multi-select mode in this API.

Continue to [Data Display](../data-display/) for content-oriented components.
