# Built-in HTML Elements

The `view!` macro maps lowercase HTML tags to `gpui` element builders. Most
elements expand to `gpui::div()` with appropriate default styling. This page
lists every supported tag and its behavior.

## Container Elements

All of these expand to `gpui::div()` with no additional default styling:

| Tag                                                | Notes                        |
| `<div>`, `<span>`, `<p>`, `<output>`                   | Generic containers.          |
| `<header>`, `<footer>`, `<nav>`, `<main>`         | Semantic sectioning.         |
| `<section>`, `<article>`, `<aside>`, `<address>`  | Semantic sectioning.         |
| `<form>`                                            | Form context: `on:submit` / `on:reset` + child submit/reset buttons. |
| `<fieldset>`, `<legend>`                            | Form containers (pure div aliases). |
| `<figure>`, `<figcaption>`                         | Figure containers.           |
| `<pre>`, `<blockquote>`, `<q>`                     | Text containers.             |

## Text Elements

| Tag                                    | Default styling                          |
| -------------------------------------- | ---------------------------------------- |
| `<h1>`                                 | font-size 2.0rem, font-weight 600        |
| `<h2>`                                 | font-size 1.5rem, font-weight 600        |
| `<h3>`                                 | font-size 1.25rem, font-weight 600       |
| `<h4>`                                 | font-size 1.0rem, font-weight 600        |
| `<h5>`                                 | font-size 0.875rem, font-weight 600      |
| `<h6>`                                 | font-size 0.85rem, font-weight 500       |
| `<strong>`, `<b>`                      | font-weight: bold                        |
| `<em>`, `<i>`                          | font-style: italic                       |
| `<u>`                                  | text-decoration: underline               |
| `<s>`, `<del>`, `<strike>`             | text-decoration: line-through            |
| `<mark>`                               | background: yellow                       |
| `<small>`                              | font-size: 0.875rem                      |
| `<code>`, `<kbd>`, `<samp>`, `<var>`   | font-family: monospace                   |
| `<cite>`, `<abbr>`, `<dfn>`, `<bdi>`, `<bdo>`, `<time>` | Plain div (no defaults)    |

## List Elements

| Tag        | Default styling           |
| ---------- | ------------------------- |
| `<ul>`     | flex column               |
| `<ol>`     | flex column               |
| `<li>`     | plain div                 |
| `<dl>`     | flex column               |
| `<dt>`     | font-weight: bold         |
| `<dd>`     | padding-left: 16px        |

gpui has no `list-style`; draw numbering or bullets with text prefixes yourself.

## Link Elements

| Tag     | Default styling                                    |
| ------- | -------------------------------------------------- |
| `<a>`   | cursor: pointer, text-color: blue (hsla 220, 100%, 50%) |

`<a>` supports the `href` attribute (accepted but not navigable in vgui v1 —
use `on:click` for navigation actions).

## Button Elements

`<button>` expands to `gpui::div().cursor_pointer()` and defaults to
`tabindex={0}` so it is in Tab order. An explicit `tabindex` overrides that
default. Keyboard activation of `on:click` is handled by gpui.

## Media Elements

| Tag      | Attributes  | Behavior                                   |
| -------- | ----------- | ------------------------------------------ |
| `<img>`  | `src` (required), `object_fit`, `alt` | `gpui::img(src)`. `object_fit` accepts `fill`, `contain`, `cover`, `scale-down`, `none`. `alt` is accepted and unused for a11y. |
| `<svg>`  | `src` (required) | `gpui::svg().path(src)`                 |

### Void elements

| Tag     | Behavior                                              |
| ------- | ----------------------------------------------------- |
| `<br>`  | A 1-line-height empty div.                            |
| `<hr>`  | Full-width 1px gray divider.                          |
| `<wbr>` | Renders nothing (`gpui::Empty`).                      |

### No-op elements

`<colgroup>` and `<col>` are accepted (they compile) but render nothing
(`gpui::Empty`). Column widths are controlled per-cell via `class` or `style`.

`<datalist>` renders nothing (`gpui::Empty`) but registers its `id` and
`options` in a thread-local map so text inputs with `list=<id>` can show
autocomplete suggestions. It requires an `id` attribute and accepts
`options={Vec<String>}`. See [Input Elements](./inputs.md#datalist).

`<option>` and `<optgroup>` are accepted as standalone tags (they compile as
pure `<div>` aliases) but are not read by `<select>`. Use the `options` or
`groups` prop on `<select>` instead.

## Tables

Tables use a flex-based layout — `gpui` has no native table layout:

| Tag        | Layout behavior                                          |
| ---------- | -------------------------------------------------------- |
| `<table>`  | flex column                                              |
| `<thead>`  | flex column                                              |
| `<tbody>`  | flex column                                              |
| `<tfoot>`  | flex column                                              |
| `<caption>`| plain div                                                |
| `<tr>`     | flex row, full width                                     |
| `<td>`     | `flex_1` (shares row width equally)                     |
| `<th>`     | `flex_1`, bold, centered text                           |

`colspan` on `<td>`/`<th>` is mapped to `flex_grow`, so a cell with
`colspan={2}` grows 2× relative to `colspan=1` cells. `rowspan` is accepted
but has no visual effect.

```rust
view! {
    <table class="w-full">
        <thead>
            <tr class="bg-[#333]">
                <th class="p-2 text-white">{"Name"}</th>
                <th class="p-2 text-white">{"Age"}</th>
            </tr>
        </thead>
        <tbody>
            <tr>
                <td class="p-2">{"Alice"}</td>
                <td class="p-2">{"30"}</td>
            </tr>
            <tr>
                <td class="p-2" colspan={2u32}>{"Bob (spanned)"}</td>
            </tr>
        </tbody>
    </table>
}
```

For specific column widths, apply `class="w-[200px]"` or a `style` width on
individual cells.

## Common Attributes

All built-in elements support these attributes:

| Attribute    | Syntax              | Description                                    |
| ------------ | ------------------- | ---------------------------------------------- |
| `style`      | `style={css!{...}}` | CSS-in-Rust styles.                            |
| `class`      | `class="..."`       | Tailwind utility classes (expanded via `tw!`). |
| `hover`      | `hover={css!{...}}` | Styles applied on mouse hover.                 |
| `active`     | `active={css!{...}}`| Styles applied on mouse-down.                  |
| `focus`      | `focus={css!{...}}` | Styles applied on keyboard focus.              |
| `id`         | `id="my-id"`        | Sets the gpui element id.                      |
| `tabindex`   | `tabindex={0}`      | `≥0` sets tab order; `<0` is focusable only.   |
| `on:event`   | `on:click={handler}`| Event handler (see [view! macro](../concepts/view-macro.md)). |
| `ref`         | `ref={node_ref}`    | Binds a `NodeRef` handle for imperative ops (focus, scroll, bounds). See [Refs](./refs.md). |

Elements that have `on:click`, `hover`, `active`, `focus`, `class`,
`tabindex`, or `ref` but no explicit `id` automatically receive a stable
auto-generated id (see [Auto IDs](../concepts/reactivity.md#auto-ids)).
