---
title: Navigation and Layout
description: Compose breadcrumbs, pagination, tabs, and drag-resizable split panels with the current vgui-shadcn Rust APIs.
---

This guide covers four implemented `vgui-shadcn` components. The snippets are excerpts for a vgui reactive render closure, not complete applications. They assume the following imports and a preceding `apply_theme(false)` call in that closure.

```rust
use gpui::IntoElement;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;
```

## Component forms

Functions marked with `#[vgui_component]` generate PascalCase components, such as `Breadcrumb { ... }` and `Pagination { ... }`. Ordinary functions retain their snake_case names: the resizable split uses `h_resizable(...)` or `v_resizable(...)`. These examples use Rust struct expressions for generated components; `view!` also supports their PascalCase tag form. Neither form is a drop-in replacement for upstream shadcn components.

## Breadcrumb

`Breadcrumb` accepts `Vec<BreadcrumbItem>`. Each item is a `(String, Option<Box<dyn Fn(&mut gpui::App)>>)` pair. The final item is always rendered as non-interactive current-page text; earlier items become buttons only when they have callbacks.

```rust
let home: BreadcrumbItem = (
    "Home".into(),
    Some(Box::new(|_cx| {})),
);
let breadcrumb = Breadcrumb {
    items: vec![
        home,
        ("Components".into(), None),
        ("Navigation".into(), None),
    ],
};

view! { <div>{breadcrumb}</div> }
```

The empty callback shows the required shape; replace it with the owning view's navigation action. An earlier item with `None` remains plain text, so an absent callback does not imply a disabled link. Separators are inserted automatically, and the row wraps when space is limited.

## Pagination

`Pagination` is controlled: pass the current `page`, a `page_count`, and an `on_change` callback. Store the page in application state and update it in the callback.

```rust
let (page, set_page) = create_signal(5usize);
let pagination = Pagination {
    page: page.get(),
    page_count: 20,
    on_change: Box::new(move |next, cx| set_page.set(cx, next)),
};

view! { <div>{pagination}</div> }
```

At most seven pages are shown without collapsing. Larger ranges retain the first and last pages, show a window around the current page, and insert non-interactive ellipses. For example, page 5 of 20 produces `1 … 4 5 6 … 20`. Previous and next callbacks are suppressed at the corresponding boundary. Internally, `page` and `page_count` are each clamped to at least 1; an out-of-range high page is not otherwise normalized to `page_count`.

## Tabs

The tabs family provides layout and clickable triggers, not an automatic selected-tab state machine. `Tabs` contains `TabsList` and any panel content you supply. Each `TabsTrigger` receives its own `active` flag and callback.

```rust
let (active, set_active) = create_signal("account");
let account = TabsTrigger {
    active: active.get() == "account",
    on_click: Box::new({
        let set_active = set_active.clone();
        move |cx| set_active.set(cx, "account")
    }),
    children: vec![view! { <span>{"Account"}</span> }.into_any_element()],
};
let password = TabsTrigger {
    active: active.get() == "password",
    on_click: Box::new(move |cx| set_active.set(cx, "password")),
    children: vec![view! { <span>{"Password"}</span> }.into_any_element()],
};
let panel = if active.get() == "account" {
    view! { <span>{"Account settings"}</span> }.into_any_element()
} else {
    view! { <span>{"Password settings"}</span> }.into_any_element()
};
let tabs = Tabs {
    children: vec![
        TabsList {
            children: vec![account.into_any_element(), password.into_any_element()],
        }
        .into_any_element(),
        panel,
    ],
};

view! { <div>{tabs}</div> }
```

The conditional panel is application code. There is no `TabsContent` component, value-based trigger registration, or built-in arrow-key navigation in this implementation; do not assume those behaviors from upstream tabs.

## Resizable split panels

`h_resizable(left, right, initial_left, total_width)` creates a horizontal split with a draggable divider. `v_resizable(top, bottom, initial_top, total_height)` provides the vertical equivalent. Both take `AnyElement` children and pixel dimensions.

```rust
let split = h_resizable(
    view! { <div>{"Navigator"}</div> }.into_any_element(),
    view! { <div>{"Main content"}</div> }.into_any_element(),
    240.0,
    800.0,
);

view! {
    <div style={css! { height: 320px; }}>
        {split}
    </div>
}
```

The first pane starts at 240 px and the second fills the remainder. Dragging the 6 px divider updates an internal signal; the implementation clamps the drag using a 48 px minimum-size constant and hides overflow in both panes. The total width or height is an explicit fixed pixel value, not a measurement of a responsive parent. A native headless interaction test exercises horizontal divider dragging; that does not establish browser or every-platform pointer behavior.

## Current limits

These APIs implement a focused subset of shadcn-style behavior. They do not claim complete upstream feature parity, accessibility certification, or unverified browser interaction support. In particular, breadcrumb navigation, tab selection, and page data remain the application's responsibility.

Return to the [component overview](../).
