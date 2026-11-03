---
title: Overlays and Feedback
description: Control dialogs and anchored popovers, and render alerts and determinate progress with current vgui-shadcn APIs.
---

This guide covers two overlays and two feedback components implemented in `vgui-shadcn`. The snippets are excerpts for a vgui reactive render closure, not complete applications. They assume the following imports and a preceding `apply_theme(false)` call in that closure.

```rust
use gpui::IntoElement;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;
```

## Component forms

The PascalCase values below are generated from functions marked with `#[vgui_component]`. The examples use struct expressions such as `Popover { ... }`; `view!` also supports their PascalCase tag form. The dialog wrapper is named `DialogComponent` because its source function is `dialog_component`, while its visual title and description helpers are `DialogTitle` and `DialogDescription`.

## Dialog

`DialogComponent` is controlled by `open`. Its `on_close` callback requests a state change; the owning view must set `open` to `false`. Children are arbitrary `AnyElement` values, so actions such as Cancel are composed explicitly.

```rust
let (open, set_open) = create_signal(false);
let trigger = {
    let set_open = set_open.clone();
    view! {
        <button on:click={click(move |cx| set_open.set(cx, true))}>
            {"Delete item"}
        </button>
    }
};
let dialog = DialogComponent {
    open: open.get(),
    on_close: Box::new({
        let set_open = set_open.clone();
        move |cx| set_open.set(cx, false)
    }),
    children: vec![
        DialogTitle {
            children: vec![view! { <span>{"Delete item?"}</span> }.into_any_element()],
        }
        .into_any_element(),
        DialogDescription {
            children: vec![
                view! { <span>{"This action cannot be undone."}</span> }.into_any_element(),
            ],
        }
        .into_any_element(),
        view! {
            <button on:click={click(move |cx| set_open.set(cx, false))}>
                {"Cancel"}
            </button>
        }
        .into_any_element(),
    ],
};

view! { <div>{trigger}{dialog}</div> }
```

The underlying vgui dialog supplies a full-screen backdrop that occludes background mouse events. Escape and mouse-down outside the content invoke `on_close`. Core focus handling moves focus into the dialog on open, cycles Tab and Shift+Tab within it, and restores prior focus after closing; native headless tests cover these core behaviors. `DialogTitle` and `DialogDescription` are styled spans and do not by themselves establish complete ARIA labelling.

## Popover

`Popover` renders its children on a floating layer below a `NodeRef` anchor. Bind the same ref to the trigger with `ref={anchor.clone()}`, and control visibility with application state.

```rust
let (open, set_open) = create_signal(false);
let anchor = NodeRef::new();
let trigger = {
    let set_open = set_open.clone();
    view! {
        <button
            ref={anchor.clone()}
            on:click={click(move |cx| {
                set_open.update(cx, |is_open| *is_open = !*is_open)
            })}
        >
            {"Project settings"}
        </button>
    }
};
let popover = Popover {
    open: open.get(),
    anchor,
    children: vec![
        view! { <div>{"Choose the settings to display."}</div> }.into_any_element(),
    ],
};

view! { <div>{trigger}{popover}</div> }
```

Placement uses the anchor's previous-frame bounds. The floating layer snaps inside the window with an 8 px margin when necessary; before an anchor is bound, its fallback position is the window origin. There is no placement option in this API. `Popover` also does not add outside-click dismissal, Escape handling, or focus trapping, so add and verify any interaction your application requires.

## Alert

`Alert` is a static container with `AlertVariant::Default` and `AlertVariant::Destructive` styles. Compose `AlertTitle`, `AlertDescription`, or other elements in `children`.

```rust
let alert = Alert {
    variant: AlertVariant::Destructive,
    children: vec![
        AlertTitle {
            children: vec![view! { <span>{"Save failed"}</span> }.into_any_element()],
        }
        .into_any_element(),
        AlertDescription {
            children: vec![
                view! { <span>{"Check available storage and try again."}</span> }
                    .into_any_element(),
            ],
        }
        .into_any_element(),
    ],
};

view! { <div>{alert}</div> }
```

The destructive variant changes border and foreground colors; it does not add an icon, close action, or automatic announcement. The source does not set an ARIA alert role or live-region behavior. Any dismissal action and semantics required by the application must be composed and verified separately.

## Progress

`Progress` is a determinate bar defined by `value` and `max`, both `f64`. Update the source value through application state when work advances.

```rust
let (value, set_value) = create_signal(35.0f64);
let progress = Progress {
    value: value.get(),
    max: 100.0,
};

view! {
    <div>
        {progress}
        <button on:click={click(move |cx| {
            set_value.update(cx, |value| *value = (*value + 10.0).min(100.0))
        })}>
            {"Advance"}
        </button>
    </div>
}
```

The filled fraction is `(value / max).clamp(0.0, 1.0)`, so values outside the range do not overflow the track. A `max` less than or equal to zero produces an empty bar. There is no indeterminate mode, built-in text label, or automatic progressbar value semantics in this implementation.

## Current limits

These components are vgui implementations with their own API surface, not a claim of complete upstream shadcn parity. Focus behavior, dismissal, announcements, and keyboard interaction differ by component; no accessibility certification or unverified browser interaction support is implied.

Return to the [component overview](../).
