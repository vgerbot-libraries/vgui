---
title: Built-in Elements and Events
description: Use vgui's HTML-like elements, GPUI attributes, accessibility metadata, media, canvas, and normalized events.
---

The `view!` macro recognizes a fixed set of lowercase tags. It emits GPUI elements, not browser DOM nodes. HTML-like spelling does not imply HTML layout, navigation, form, or event behavior; the mappings and limits below are part of the API.

For input-specific tags, see [Inputs and Forms](../inputs-and-forms/). For `ref`, see [Refs and Context](../refs-and-context/).

## Element categories

| Category | Tags and current mapping |
| --- | --- |
| Generic containers | `div`, `span`, `p`, `output`, `option`, `optgroup`, `header`, `footer`, `nav`, `main`, `section`, `article`, `aside`, `address`, `fieldset`, `legend`, `figure`, `figcaption`, `pre`, `blockquote`, and `q` become `gpui::div()`. |
| Headings | `h1` through `h6` become divs with default rem sizes and font weights. `h1` is 2rem/600; `h2` 1.5rem/600; `h3` 1.25rem/600; `h4` 1rem/600; `h5` 0.875rem/600; `h6` 0.85rem/500. |
| Text formatting | `strong`/`b` are bold; `em`/`i` italic; `u` underlined; `s`/`del`/`strike` struck through; `mark` highlighted; `small` 0.875rem; `code`/`kbd`/`samp`/`var` monospace. `cite`, `abbr`, `dfn`, `bdi`, `bdo`, and `time` are plain divs. |
| Lists | `ul`, `ol`, and `dl` are flex columns; `li` is a plain div; `dt` is bold; `dd` has 16px left padding. Bullets, numbering, and HTML list semantics are not generated. |
| Links and buttons | `a` adds a pointer cursor and blue text; `button` adds a pointer cursor, centered text, and default `tabindex={0}`. |
| Tables | `table`, `thead`, `tbody`, and `tfoot` are flex columns; `caption` is a plain div; `tr` is a full-width flex row; `td` and `th` use `flex_1`, with `th` bold and centered. |
| Void and no-op tags | `br` emits a one-line-high div; `hr` a full-width 1px divider; `wbr`, `colgroup`, and `col` emit `gpui::Empty`. |

Most semantic tags are styling aliases only. In particular, `output`, `option`, and `optgroup` have no form behavior of their own.

### HTML differences to account for

- `href` on `a` is accepted but unused; attach an application action such as `on:click` instead of expecting navigation.
- Tables use flex layout. `colspan` on `td`/`th` becomes a relative `flex_grow`; `rowspan` is accepted but has no layout effect. Set explicit cell widths when equal flex sizing is not appropriate.
- `br`, `hr`, and `wbr` have the mappings above rather than browser line-break and word-break semantics.
- Unknown tags and unknown attributes are compile errors. Each attribute needs `=value`; boolean attributes are written explicitly, such as `disabled={true}`.

## GPUI-oriented attributes

General built-in elements support the following attributes where the emitted GPUI element has the corresponding capability:

| Attribute | Purpose |
| --- | --- |
| `style={css! { ... }}` | Apply vgui CSS-in-Rust styling. |
| `class="..."` | Apply Tailwind-style utilities, including supported state and responsive variants. |
| `hover={...}`, `active={...}`, `focus={...}` | Apply state styles. |
| `id="..."` | Set the GPUI element ID. Features that require stateful interaction can generate an ID when one is absent. |
| `tabindex={0}` | Nonnegative values set a tab index; negative values use GPUI's focusable-only path. |
| `ref={node_ref.clone()}` | Bind persistent focus and scroll handles where supported. |
| `animate={...}` | Apply a supported animation expression after children are built. |
| `{..props}` | Apply one GPUI spread to a general element. |
| `drag={...}`, `drag:preview={...}`, `can_drop={...}` | Configure GPUI drag and drop behavior. |

Specialized elements deliberately accept smaller attribute sets. Inputs, `select`, `textarea`, `form`, `canvas`, `dialog`, `portal`, `floating`, and `uniform_list` do not inherit every row above.

## ARIA and roles

General built-ins accept `role="..."` and `aria:name=value`. The role string is resolved to a GPUI/AccessKit role; an unknown role becomes `GenericContainer`. Semantic HTML tag names do not automatically add the equivalent ARIA role.

Supported ARIA mappings are:

- `aria:label`, `aria:description`, `aria:keyshortcuts`
- `aria:selected`, `aria:expanded`
- `aria:toggled="true"`, `"mixed"`, or another string for false
- `aria:valuenow` or `aria:numeric_value`, and `aria:numeric_value_step`
- `aria:value`, `aria:placeholder`

```rust
view! {
    <div role="slider" aria:label="Volume" aria:valuenow={50f64}>
        {"Volume"}
    </div>
}
```

Other `aria:*` names are currently ignored rather than forwarded. `input`, `select`, `textarea`, `label`, and `form` reject direct `role`/`aria:*` attributes; put accessibility metadata on a suitable general wrapper when needed. An `img` `alt` attribute is accepted but is not used for accessibility.

## Images, meters, and disclosure

```rust
view! {
    <img
        src={"logo.png"}
        alt={"Logo"}
        object_fit="contain"
        on:load={move |_cx: &mut gpui::App| {}}
        on:error={move |_cx: &mut gpui::App| {}}
    />
}
```

`img` requires `src`. `object_fit` accepts the string values `fill`, `contain`, `cover`, `scale-down`, and `none`. `on:load` and `on:error` are image-specific `Fn(&mut App)` callbacks without an event payload; they observe GPUI asset-load state transitions. The current callback wrapper cannot observe an `ImageSource::Image` source, so those callbacks are unavailable for that source variant. `svg` also requires `src` and maps it to `gpui::svg().path(src)`.

`progress` accepts `value` and `max` (defaults 0 and 1). `meter` accepts `value`, `min`, `max`, `low`, `high`, and `optimum`; the last three are optional numeric values. `details` accepts `open={bool}` and renders its children through vgui's disclosure helper; `summary` only adds pointer-cursor styling.

## Canvas drawing

`canvas` requires `paint={|ctx| ...}` and accepts only `paint`, `style`, and `class`. It cannot have children or event attributes. The closure receives a `Context2D` during GPUI painting; coordinates are canvas-local and drawing is immediate-mode.

```rust
use vgui::prelude::*;

view! {
    <canvas
        class="w-full h-48"
        paint={move |ctx: &mut Context2D| {
            ctx.set_fill_style(color("#2563eb"));
            ctx.fill_rect(12.0, 12.0, 96.0, 48.0);
        }}
    />
}
```

`Context2D` is a GPUI-backed drawing API, not a browser `HTMLCanvasElement` or a retained canvas context. Consult the API documentation for its supported paths, transforms, text, and drawing operations.

## Other specialized built-ins

| Tag | Required and optional attributes |
| --- | --- |
| `dialog` | `open={bool}` and `on:close={Fn(&mut App)}`; no other attributes. |
| `portal` | Optional `priority={usize}`; higher values paint above lower ones. |
| `floating` | Required `position={Point<Pixels>}` for window-coordinate placement. |
| `uniform_list` | Required `count={usize}` and `render={|range, window, cx| ...}`; optional `scroll_handle`, `style`, and `class`; no children or events. |
| `datalist` | Required `id` and optional `options={Vec<String>}`; registers suggestions without rendering a node. |
| `radiogroup` | No attributes; coordinates descendant radio focus. |

The input-related tags are covered in [Inputs and Forms](../inputs-and-forms/).

## Normalized pointer, keyboard, wheel, and window events

Normalized handlers use vgui event structs and receive `(&event, &mut Window, &mut App)`:

| Attribute | Payload | Semantics and limits |
| --- | --- | --- |
| `on:keydown`, `on:keyup` | `&KeyboardEvent` | GPUI key events with `key`, best-effort `code`, modifier flags, `key_char`, and `repeat`. `code` can be empty because GPUI has no physical key code; `repeat` is always false on keyup. |
| `on:pointerdown`, `on:pointerup`, `on:pointermove` | `&PointerEvent` | Mouse-backed events with window-space pixel coordinates, button/mask values, modifiers, and click count. Pointer up is registered through GPUI's capture-any-mouse-up path. |
| `on:wheel` | `&WheelEvent` | Pixel deltas plus modifier flags. Line deltas are converted using 16px per line. |
| `on:dblclick` | `&PointerEvent` | Mouse click data when the GPUI click count is at least two. |
| `on:contextmenu` | `&PointerEvent` | Mouse data for secondary-button clicks. |
| `on:resize` | `&ResizeEvent` | Registers a handler for the window viewport's new width and height. It is window-level, not an element `ResizeObserver`, even though the attribute is written on an element. |

```rust
view! {
    <div
        tabindex={0}
        on:keydown={move |event: &KeyboardEvent, _window, cx| {
            if event.key == "Escape" {
                set_open.set(cx, false);
                event.stop_propagation();
            }
        }}
        on:pointermove={move |event: &PointerEvent, _window, _cx| {
            set_position.set(_cx, (event.client_x, event.client_y));
        }}
    >
        {"Interactive area"}
    </div>
}
```

Element keyboard listeners require the relevant GPUI focus path. `KeyboardEvent::stop_propagation()` is translated to GPUI propagation stopping after the closure returns and also affects vgui's global keyboard listeners. There is no general DOM `Event`, `prevent_default`, `current_target`, or pointer `stop_propagation` API.

Pointer events are currently mouse-only: `pointer_type` is `Mouse`, `pointer_id` is 1, and `is_primary` is true. There are no normalized touch/pen streams, pointer-cancel events, or pointer-capture methods. On pointer down, `button` uses 0/1/2 for left/middle/right; on move it is -1. `buttons` is 0 on pointer up because GPUI does not provide the remaining pressed-button state.

## GPUI and state-specific event attributes

These attributes remain available alongside the normalized layer:

| Attribute | Handler shape or behavior |
| --- | --- |
| `on:click` | GPUI `ClickEvent` listener, or use `click(move |cx| ...)` when only `&mut App` is needed. |
| `on:scroll` | Raw GPUI scroll-wheel listener; use `on:wheel` for `WheelEvent`. |
| `on:mouseenter`, `on:mouseleave` | `Fn(&mut Window, &mut App)` driven by GPUI hover transitions. |
| `on:hover` | `Fn(bool, &mut Window, &mut App)` for both enter and leave. |
| `on:modifiers_changed` | Raw GPUI modifiers listener. |
| `on:mouse_down_out`, `on:mouse_up_out`, `on:any_mouse_down` | Raw GPUI mouse listeners; `mouse_up_out` is left-button-specific. |
| `on:drop`, `on:drag_move` | GPUI drag/drop listeners, used with the drag attributes where applicable. |

`on:input`, `on:change`, `on:submit`, `on:reset`, `on:close`, `on:load`, and `on:error` belong to specific widgets rather than to every element. Unsupported event names are compile errors.
