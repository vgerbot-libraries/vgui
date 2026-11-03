---
title: Components
description: Install vgui-shadcn, apply its theme, and explore the current component modules.
---

`vgui-shadcn` provides shadcn-style components for vgui applications. Its crate root exposes `components`, `hoc`, `icons`, `mobile`, `prelude`, and `theme`; the component catalog below reflects the public modules in `components/mod.rs`.

## Installation and imports

Complete the [Installation](../docs/installation/) prerequisites and keep the GPUI dependencies from the [Quick Start](../docs/quick-start/). Add `vgui-shadcn` alongside `vgui`:

```toml
[dependencies]
vgui = { git = "https://github.com/vgerbot-libraries/vgui" }
vgui-shadcn = { git = "https://github.com/vgerbot-libraries/vgui" }
```

Import both preludes in a module that renders components:

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;
```

The vgui-shadcn prelude re-exports the crate's component, HOC, and mobile exports, the `icons` module, and `apply_theme`, `light_theme`, and `dark_theme`. Public component modules also remain available under `vgui_shadcn::components`. One current exception to the glob re-exports is `dialog`; import its items from `vgui_shadcn::components::dialog` directly.

## Apply the theme

Call `apply_theme` from a render closure. Pass `false` for the light token set or `true` for the dark token set:

```rust
fn themed_view() -> impl gpui::IntoElement {
    apply_theme(false);

    view! {
        <div class="p-4">{"Themed content"}</div>
    }
}
```

When switching themes at runtime, read the signal in that same render closure so the change re-renders it: `apply_theme(dark.get())`. The theme supplies colors and `--radius`; component styles do not require a browser stylesheet.

## Two API styles

- **PascalCase macro components** come from `#[vgui_component]` functions. For example, `button` becomes `Button`, `text_field` becomes `TextField`, and `description_list` becomes `DescriptionList`. Use them as tags in `view!` or as struct expressions with `.into_any_element()` when a child vector requires it.
- **Function and builder APIs** are ordinary Rust functions and structs. For example, `select(ShadcnSelectProps::new(...))` renders a select, while `table(DataTable::new(...))` returns both an element and a reactive state store. Their option structs use chained methods such as `.sortable()` and `.searchable(true)`.

Read each component's signature rather than assuming that every module supports tags, children, or the same callback types.

## Component catalog

These are module names, not claims of API or behavioral equivalence with an upstream library. Follow the family links for focused examples.

### [Actions and form controls](actions-and-form-controls/)

`attachment`, `button`, `button_group`, `calendar`, `code_editor`, `combobox`, `command_palette`, `date_picker`, `dropdown_button`, `form_field`, `input`, `number_input`, `otp_input`, `questionnaire`, `rating`, `select`, `setting`, `slider`, `switch`, `time_field`, `toggle`, `touch_selection`

### [Data display](data-display/)

`avatar`, `badge`, `bubble`, `card`, `carousel`, `chart`, `description_list`, `diff`, `icon`, `inspector`, `kbd`, `markdown`, `plot`, `table`, `tag`, `tree`, `virtual_list`

### [Navigation and layout](navigation-and-layout/)

`breadcrumb`, `collapsible`, `dock`, `group_box`, `menu`, `menu_bar`, `nav_stack`, `pagination`, `resizable`, `separator`, `sidebar`, `tabs`, `toolbar`, `window_chrome`

### [Overlays and feedback](overlays-and-feedback/)

`alert`, `dialog`, `drawer`, `hover_card`, `notification`, `popover`, `progress`, `shimmer`, `skeleton`, `spinner`, `tooltip`

## Current limitations

The crate is shadcn-style, not a complete upstream shadcn port. Accessibility behavior is implemented component by component; the catalog is not a suite-wide accessibility conformance claim, and these pages do not claim browser support.

`GAPS.md` records rendering and framework gaps. Transform-dependent magnification, marquees, rotating spinners, and watermarks are not available; fixed/sticky positioning, background images, and HTML audio also have no direct equivalent. JavaScript-backed Recharts, TanStack Table, Tabulator, and the HOC data table are not ported. Individual components may use simpler layouts or behavior where the source documents an alternative.
