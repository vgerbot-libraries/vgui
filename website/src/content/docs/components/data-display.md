---
title: Data Display
description: Present structured content with Card, Badge, DescriptionList, and the DataTable function API.
---

These examples are complete render functions for a vgui application. Mount them as described in the [Quick Start](../../docs/quick-start/). Each function applies the light theme; see the [component overview](../) for theme setup.

## Card and Badge

`Card` accepts arbitrary children. `CardHeader` supplies a title and description, while `Badge` labels content with one of four variants:

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn project_card() -> impl gpui::IntoElement {
    apply_theme(false);

    Card {
        children: vec![
            CardHeader {
                title: "vgui-shadcn".to_string(),
                description: "Styled components for vgui".to_string(),
            }
            .into_any_element(),
            Badge {
                variant: BadgeVariant::Secondary,
                children: vec!["Rust".to_string().into_any_element()],
            }
            .into_any_element(),
        ],
    }
}
```

`Card` provides the themed container, padding, border, and vertical gap; it does not define separate body or footer components. `CardHeader` always renders both its `title` and `description` strings. `BadgeVariant` supports `Default`, `Secondary`, `Outline`, and `Destructive`, and badge children are `Vec<gpui::AnyElement>`.

## DescriptionList

`DescriptionList` renders an optional title and `(term, detail)` rows. Each detail is an `AnyElement`, so it can contain text or another component:

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn package_details() -> impl gpui::IntoElement {
    apply_theme(false);

    DescriptionList {
        title: Some("Package".to_string()),
        items: vec![
            (
                "Name".to_string(),
                "vgui-shadcn".to_string().into_any_element(),
            ),
            (
                "Kind".to_string(),
                Badge {
                    variant: BadgeVariant::Outline,
                    children: vec!["Component library".to_string().into_any_element()],
                }
                .into_any_element(),
            ),
        ],
    }
}
```

Set `title` to `None` to omit it. Terms have a fixed 160 px column; every row has a bottom border, including the last. This component is for structured read-only details, not two-way form binding.

## DataTable

`table` is a function API. `DataTable::new` accepts fixed-width columns and rows of string cells, and the function returns `(gpui::AnyElement, Store<TableState>)`:

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn team_table() -> impl gpui::IntoElement {
    apply_theme(false);

    let (element, state) = table(DataTable::new(
        vec![
            TableColumn::new("Name", 120.0).sortable(),
            TableColumn::new("Role", 160.0),
        ],
        vec![
            vec!["Alice".into(), "Engineer".into()],
            vec!["Bob".into(), "Designer".into()],
            vec!["Carol".into(), "Engineer".into()],
        ],
    ));

    view! {
        <div class="flex flex-col gap-2 p-4">
            <span>{format!("{} row(s) selected", state.get().selected.len())}</span>
            {element}
        </div>
    }
}
```

- `TableColumn::new(title, width)` creates a non-sortable column; chain `.sortable()` to enable header-click sorting.
- Clicking a sortable header cycles ascending, descending, then unsorted. Sorting compares cell strings lexicographically, not as numbers or dates.
- Clicking a body row toggles its selection. Multiple rows can be selected, and `TableState.selected` stores indices in the original input order even after sorting.
- Read `sort_col`, `sort_asc`, and `selected` through the returned store. Column widths are fixed pixel values, and cells are strings rather than arbitrary child elements.

This is the crate's self-contained table implementation, not a TanStack Table or Tabulator integration. Return to the [component catalog](../) for the other display modules.
