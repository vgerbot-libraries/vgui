# vgui Feature Comparison with HTML+CSS

> This page compares vgui features against HTML+CSS item by item, helping developers quickly build a comprehensive understanding of vgui's capability boundaries.
> Support level markers: ✅ Full support · 🔶 Partial support / differences · ❌ Not supported

## 1. Markup Language & Elements

### 1.1 Built-in HTML Element Mapping

vgui's `view!` macro accepts lowercase tags matching HTML names, mapping them to `gpui` elements under the hood. Container elements have no default styling; text elements carry corresponding default font styles.

| HTML Elements | vgui Mapping | Notes |
|---------------|--------------|-------|
| `div` `span` `p` `header` `footer` `nav` `main` `section` `article` `aside` `address` `form` `fieldset` `legend` `figure` `figcaption` `pre` `blockquote` `q` | `gpui::div()` | No default styling, pure containers |
| `h1`–`h6` `strong` `b` `em` `i` `u` `s` `del` `strike` `mark` `small` `code` `kbd` `samp` `var` `cite` `abbr` `dfn` `bdi` `bdo` `time` | `gpui::div()` | With corresponding default font styles |
| `ul` `ol` `li` `dl` `dt` `dd` | flex column / bold / padding-left | No `list-style`; must manually draw numbers/bullets |
| `a` | `div().cursor_pointer()` + blue text | `href` attribute accepted but not navigable; use `on:click` for navigation |
| `button` | `div().cursor_pointer()` | Default `tabindex=0` |
| `img` | `gpui::img(src)` | Supports `object_fit` (`fill`/`contain`/`cover`/`scale-down`/`none`); `alt` accepted but not used for a11y |
| `svg` | `gpui::svg().path(src)` | — |
| `br` `hr` `wbr` | empty line / separator / empty | void elements |
| `table` `thead` `tbody` `tfoot` `tr` `td` `th` `caption` | flex layout simulation | `colspan`→`flex_grow`; `rowspan` has no visual effect |
| `colgroup` `col` `datalist` `option` `optgroup` | compiles but renders empty or special-purpose | no-op elements |
| `canvas` | `vgui::canvas_element(paint)` | `<canvas paint={\|ctx\| ...}>` with `Context2D` 2D drawing API; supports `style`/`class`, no children/events |

### 1.2 Custom Components

| HTML/CSS | vgui | Support |
|----------|------|---------|
| No native component system (relies on frameworks) | Uppercase tags call functions/structs, attribute→field mapping | ✅ |
| Attribute spreading | `{..props}` → Rust struct update syntax / `Spread<E>` trait | ✅ |

Attribute name mapping conventions: `on:click`→`on_click`, `type`→`r#type`, `ref`→`r#ref`.

### 1.3 Control Flow

| HTML/CSS | vgui | Support |
|----------|------|---------|
| No built-in conditional/list rendering (relies on framework `v-if`/`v-for`, React conditional rendering, etc.) | `<Show when={}>` / `<For each={}>`, supports fallback | ✅ |

## 2. Styling System

### 2.1 CSS Property Support (`css!` Macro)

#### Layout

| HTML/CSS Property | `css!` Support | Differences |
|-------------------|----------------|-------------|
| `display` (`flex`/`block`/`none`/`grid`) | ✅ | — |
| `visibility` | ✅ | — |
| `overflow` (+`-x`/`-y`) | ✅ | — |
| `position` (`relative`/`absolute`) | 🔶 | No `fixed`/`sticky` |
| `flex-direction` `flex-wrap` `flex` `flex-grow` `flex-shrink` `flex-basis` | ✅ | — |
| `justify-content` `align-items` `align-self` `align-content` | ✅ | — |
| `gap` (+`row-gap` `column-gap`) | ✅ | `gap` does not support more than 2 values |
| `grid-template-columns` `grid-template-rows` | ✅ | No `fr` unit; use numeric column counts |
| `grid-column` `grid-row` (+`-start`/`-end`) | ✅ | — |
| `grid-template-areas` | ✅ | String literals define named areas; auto-infers column/row counts |
| `grid-area` | ✅ | `"name"` resolves against `grid-template-areas`, or `N` / `row / col` / four values |
| `scrollbar-width` | ✅ | — |

#### Box Model

| HTML/CSS Property | `css!` Support | Differences |
|-------------------|----------------|-------------|
| `width` `height` `min-*` `max-*` | ✅ | — |
| `padding` (+`-inline`/`-block`) | ✅ | — |
| `margin` (+`-inline`/`-block`) | ✅ | — |
| `inset` `top` `right` `bottom` `left` | ✅ | — |
| `aspect-ratio` | ✅ | — |

#### Visual

| HTML/CSS Property | `css!` Support | Differences |
|-------------------|----------------|-------------|
| `background` (color + `linear-gradient`) | ✅ | — |
| `background-color` `color` `opacity` | ✅ | — |
| `border` (+per-side/`-color`/`-style`/`-width`) | ✅ | — |
| `border-radius` (+per-corner) | ✅ | — |
| `cursor` `box-shadow` | ✅ | — |

#### Text

| HTML/CSS Property | `css!` Support | Differences |
|-------------------|----------------|-------------|
| `font-size` | ✅ | Does not support `%` |
| `font-weight` (named + numeric 100–900) | ✅ | — |
| `font-style` `font-family` | ✅ | — |
| `text-align` | ✅ | `left`/`center`/`right` |
| `text-decoration` (+`-color`/`-thickness`/`-style`) | ✅ | — |
| `text-overflow` `text-background` `white-space` `line-height` `line-clamp` | ✅ | — |

#### CSS Variables & Interpolation

| Capability | Support | Differences |
|------------|---------|-------------|
| `--name: value` definition | ✅ | Inside `css!` |
| `var(--name, fallback)` reference | ✅ | — |
| `theme!` macro + `set_theme()` | ✅ | Global thread-local theme |
| `{expr}` runtime expression interpolation | ✅ | — |

### 2.2 Unsupported CSS Properties

| CSS Property | Reason | Alternative |
|--------------|--------|-------------|
| `transform` / `translate` / `rotate` / `scale` | gpui has no div transforms | ❌ None |
| `transition` / `animation` (inside `css!`) | No equivalent in gpui styling model | Use `tw!` `transition-*`/`animate-*` classes |
| `z-index` (inside `css!`) | No equivalent in gpui styling model | Use `tw!` `z-N` utilities |
| `box-sizing` | gpui uses content-box semantics | ❌ None |
| `outline` | No equivalent | Use `border` |
| `list-style` / `list-style-type` | No list-style rendering | Manually draw numbers/bullets |
| `background-image` | No equivalent | Use `linear-gradient` |
| `background-position` / `background-size` / `background-repeat` | No equivalent | ❌ None |
| `float` / `clear` | No equivalent | ❌ None |
| `@media` queries (inside `css!`) | `css!` does not support media queries | Use `tw!` responsive prefixes `sm:`/`md:`/`lg:`/`xl:` |
| `!important` | No equivalent | ❌ None |
| `position: fixed` / `sticky` | gpui has no fixed/sticky positioning | ❌ None |

### 2.3 Tailwind Utilities (`tw!` Macro)

| Tailwind Category | Support | Differences |
|-------------------|---------|-------------|
| Display: `flex`/`block`/`hidden`/`grid`/`inline-flex` | ✅ | `inline-flex` same as `flex` |
| Flex: direction/wrap/grow/shrink | ✅ | — |
| Justify / Align | ✅ | — |
| Position: `relative`/`absolute`/`static` | ✅ | No `fixed`/`sticky` |
| Overflow | ✅ | — |
| Spacing: `p`/`m`/`gap` full range | ✅ | spacing scale 0–96 |
| Sizing: `w`/`h`/`min`/`max` | ✅ | — |
| Colors: 22 palettes × 11 shades + `black`/`white`/`transparent` | ✅ | — |
| Typography: weight/size/family/line-height/decoration/overflow/style/white-space | ✅ | — |
| Borders: width/style/color/radius | ✅ | — |
| Shadows: `sm`–`2xl`/`none` | ✅ | — |
| Cursor / Opacity (0–100) / Inset | ✅ | — |
| Grid: `grid-cols-N`/`grid-rows-N`/`col-N`/`row-N` | ✅ | — |
| Aspect ratio / Line clamp (1–10) | ✅ | — |
| Z-index: `z-0`–`z-50`/`z-auto` | ✅ | — |
| Arbitrary values: `[Npx]`/`[Nrem]`/`[N%]`/`[#hex]`/`[rgb()]`/`[rgba()]` | ✅ | — |
| Opacity modifier: `/NN` | ✅ | — |

### 2.4 Pseudo-States / Variants

| CSS Pseudo-Class | vgui Support | Differences |
|-----------------|--------------|-------------|
| `:hover` | `hover={css!{}}` attribute / `hover:` prefix | ✅ |
| `:focus` | `focus={css!{}}` attribute / `focus:` prefix | ✅ |
| `:active` | `active={css!{}}` attribute / `active:` prefix | ✅ |
| `:focus-within` `:focus-visible` `:visited` `:link` `:target` `:nth-child` etc. | ❌ | No equivalent |
| Responsive breakpoints `@media (min-width: …)` | `sm:`/`md:`/`lg:`/`xl:` prefixes | ✅ Applied at runtime based on viewport width |

Responsive breakpoint thresholds (matching Tailwind defaults):

| Prefix | Min Width |
|--------|-----------|
| `sm:` | ≥ 640px |
| `md:` | ≥ 768px |
| `lg:` | ≥ 1024px |
| `xl:` | ≥ 1280px |

### 2.5 Animations & Transitions

| CSS animation/transition | vgui | Support |
|--------------------------|------|---------|
| `@keyframes` | Built-in `animate-pulse`/`bounce`/`ping` | ✅ |
| `animate-spin` | gpui has no rotation transform | ❌ |
| Custom animations | `animate={\|el\| el.with_animation(...)}` closure | ✅ |
| `transition` | `transition`/`transition-opacity`/`transition-colors`/`transition-all` | ✅ |
| `duration-*` / `ease-*` | — | ✅ |
| `delay-*` | Parsed and stored but no effect | 🔶 |

### 2.6 Component Variants System

| HTML/CSS | vgui | Support |
|----------|------|---------|
| No native variant system (relies on libraries like CVA) | `variants!` macro generates enum + composite struct + `ApplyStyle` impl | ✅ |

## 3. Layout Model

### 3.1 Flexbox

| HTML/CSS | vgui | Support |
|----------|------|---------|
| CSS flexbox full-featured | Based on gpui flexbox | ✅ |

`flex-direction`/`wrap`/`grow`/`shrink`/`basis`/`justify`/`align`/`gap` all fully supported.

### 3.2 Grid

| HTML/CSS Grid | vgui | Support |
|---------------|------|---------|
| `grid-template-columns`/`rows` (column/row counts) | ✅ | — |
| `grid-column`/`row` (+`-start`/`-end`, `span N`) | ✅ | — |
| `grid-template-areas` + `grid-area` | ✅ | thread-local stack parses named areas |
| `fr` unit / `minmax()` / `repeat()` / `auto-fit` / `auto-fill` | ❌ | Use numeric column counts |
| `grid-auto-flow` / `grid-auto-columns` / `grid-auto-rows` | ❌ | No equivalent |

### 3.3 Positioning

| HTML/CSS | vgui | Support |
|----------|------|---------|
| `position: relative` / `absolute` | ✅ | — |
| `position: fixed` / `sticky` | ❌ | No equivalent |
| `top`/`right`/`bottom`/`left`/`inset` | ✅ | — |

### 3.4 Table Layout

| HTML/CSS | vgui | Support |
|----------|------|---------|
| `table` semantic layout | flex simulation | 🔶 |
| `colspan` | `flex_grow` | ✅ |
| `rowspan` | No visual effect | ❌ |
| Automatic column width from content | Must manually specify `w-[Npx]` | 🔶 |

## 4. Responsive Design

| HTML/CSS | vgui | Support |
|----------|------|---------|
| `@media` queries | `tw!` responsive prefixes `sm:`/`md:`/`lg:`/`xl:` | ✅ Runtime viewport width check, not compile-time |
| Programmatic breakpoint query | `Breakpoint::from_width(width)` + `reactive::get_viewport_width()` | 🔶 No dedicated `use_breakpoint()` hook; must combine manually |
| CSS Container Queries | ❌ | No equivalent |

> `css!` macro does not support `@media`; responsive capabilities are limited to `tw!` utilities. The `breakpoint.rs` module doc-comment mentions a `use_breakpoint()` hook, but that function is not implemented or exported. The actual approach is to combine `Breakpoint::from_width()` with `reactive::get_viewport_width()` to obtain the current breakpoint.

## 5. Input Controls

| HTML input type | vgui Support | Differences |
|-----------------|--------------|-------------|
| `text`/`password`/`search`/`email`/`url`/`tel` | Full-featured text input (cursor/selection/clipboard/IME) | ✅ |
| `number` | Text input + numeric validation + `min`/`max`/`step` | ✅ |
| `date`/`datetime-local`/`time`/`month`/`week` | Calendar popup | ✅ |
| `color` | Preset color palette popup | 🔶 No free RGB sliders |
| `checkbox` | 18×18 rounded box + ✓ | ✅ |
| `radio` | roving tabindex (arrow key navigation within radiogroup) | ✅ |
| `range` | Draggable slider | ✅ |
| `file` | File picker | 🔶 `multiple` supported, `accept` unused |
| `submit`/`button`/`reset` | Clickable button | ✅ Auto-binds `on:submit`/`on:reset` inside form |
| `hidden` | Empty render | ✅ |
| `textarea` | Multi-line text input | ✅ `rows` attribute |
| `select` | Dropdown popup | ✅ options/groups/multiple/custom option content rendering |
| `datalist` | Autocomplete suggestions | ✅ `list=id` association |
| `label` | `for=` explicit association / wrapping association | ✅ |
| `form` | `on:submit`/`on:reset` + Enter to submit | ✅ |
| `input type="image"` / standalone `<button>` element / `<output>` | — | 🔶 `<output>` is only a div alias; no image input |

## 6. Event System

| HTML Event | vgui Event | Support |
|------------|------------|---------|
| `click` | `on:click` | ✅ Also `click()` helper function for simplified signature |
| `keydown` / `keyup` | `on:keydown` / `on:keyup` | ✅ |
| `pointerdown` / `pointerup` / `pointermove` | `on:pointerdown`/`up`/`move` | ✅ |
| `dblclick` | `on:dblclick` | ✅ |
| `contextmenu` | `on:contextmenu` | ✅ |
| `scroll` / `wheel` | `on:scroll` / `on:wheel` | ✅ |
| `resize` | `on:resize` | ✅ |
| `modifiers_changed` | `on:modifiers_changed` | ✅ |
| `mouse_down_out` / `mouse_up_out` | `on:mouse_down_out` / `on:mouse_up_out` | ✅ gpui-specific |
| `any_mouse_down` | `on:any_mouse_down` | ✅ gpui-specific |
| `input` / `change` | `on:input` / `on:change` | ✅ Signature varies by input type |
| `submit` / `reset` | `on:submit` / `on:reset` | ✅ Form context |
| `close` | `on:close` | ✅ Dialog-specific |
| `focus`/`blur`/`mouseenter`/`mouseleave`/`load`/Drag events/touch events | ❌ | gpui pointer events cover some |

## 7. Accessibility (ARIA)

| HTML ARIA | vgui | Support |
|-----------|------|---------|
| `role` attribute | `role="button"` etc., maps to `accesskit::Role` | ✅ |
| `aria:label` | `aria:label="..."` | ✅ |
| `aria:description` `aria:keyshortcuts` `aria:selected` `aria:expanded` | — | ✅ |
| `aria:toggled` `aria:valuenow`/`aria:numeric_value` `aria:value` `aria:placeholder` | — | ✅ |
| `aria:numeric_value_step` | — | ✅ |
| `role`/`aria:*` on `input`/`select`/`textarea`/`label`/`form` | Must use wrapping element | 🔶 |
| HTML native a11y (`alt` attribute) | `img` `alt` accepted but not used for a11y | 🔶 |
| accesskit integration | gpui exposes accessibility tree via accesskit | ✅ |

## 8. State Management & Reactivity

| Web Framework Pattern | vgui | Support |
|-----------------------|------|---------|
| React `useState` | `create_signal` (SolidJS-style fine-grained) | ✅ |
| React `useMemo` | `create_memo` | ✅ |
| React `useEffect` | `create_effect` (synchronous execution, not async) | ✅ |
| Dependency tracking | Automatic fine-grained tracking | ✅ |
| React Context API | `Context<T>` + `<Provider>` + `use_context` | ✅ |
| Re-runs entire `app()` closure on each render | Not SolidJS compile-time fine-grained, but slot model keeps state persistent | 🔶 |
| `useReducer` / `useRef` | Use signal + `NodeRef` as replacement | 🔶 No direct equivalent |

## 9. Overlays & Popups

| HTML Element | vgui | Support |
|--------------|------|---------|
| `<dialog>` (HTML5) | `<dialog>` | ✅ Portal rendering, focus trap, focus restore, click-outside, Escape to close |
| Custom modal/portal | `<portal priority={N}>` | ✅ |
| tooltip/popover positioning | `<floating position={...}>` | ✅ Automatic overflow avoidance |
| `<details>`/`<summary>` | `<details open={}>` + `<summary>` | ✅ `open` is a controlled prop |
| `<progress>` | `<progress value={} max={}>` | ✅ |
| `<meter>` | `<meter value={} min={} max={} low={} high={} optimum={}>` | ✅ |

## 10. Canvas 2D Drawing

| HTML Canvas API | vgui `Context2D` | Support |
|-----------------|------------------|---------|
| `fillRect` / `strokeRect` | `fill_rect` / `stroke_rect` | ✅ |
| `clearRect` | `clear_rect` | 🔶 No-op (immediate mode; repaints from blank each frame) |
| `beginPath` / `moveTo` / `lineTo` / `closePath` | `begin_path` / `move_to` / `line_to` / `close_path` | ✅ |
| `quadraticCurveTo` / `bezierCurveTo` | `quadratic_curve_to` / `bezier_curve_to` | ✅ |
| `arc` | `arc` | ✅ Flattened to line segments (`max(2, ceil(sweep/(π/16)))`) |
| `fill` / `stroke` | `fill` / `stroke` | ✅ |
| `fillText` | `fill_text` | ✅ `y` is baseline; parses CSS font shorthand |
| `strokeText` | `stroke_text` | ❌ No-op (gpui has no text outline API) |
| `measureText` | `measure_text` → `TextMetrics { width }` | ✅ |
| `save` / `restore` | `save` / `restore` | ✅ |
| `translate` / `rotate` / `scale` | `translate` / `rotate` / `scale` | ✅ |
| `setTransform` / `resetTransform` | `set_transform` / `reset_transform` | ✅ |
| `fillStyle` / `strokeStyle` | `fill_style` / `stroke_style` (`Hsla`) | 🔶 Solid colors only; no gradients/patterns |
| `lineWidth` | `line_width` | ✅ |
| `font` | `font` (CSS shorthand, e.g. `"16px sans-serif"`) | ✅ |
| `textAlign` | `text_align` (`CanvasTextAlign`) | ✅ `Start`/`Left`/`Center`/`Right`/`End` |
| `globalAlpha` | `global_alpha` | ✅ Clamped 0–1 |
| `lineCap` / `lineJoin` | — | ❌ lyon types not re-exported from gpui |
| `createLinearGradient` / `createRadialGradient` / `createPattern` | — | ❌ No gradient/pattern support |
| `drawImage` | — | ❌ Requires `RenderImage` (not exposed) |
| `clip` | — | ❌ gpui content mask is axis-aligned bounds only |
| `putImageData` / `getImageData` / `createImageData` | — | ❌ No pixel-level access |
| `shadowBlur` / `shadowColor` / `shadowOffset*` | — | ❌ No shadow on paths (use `box-shadow` on wrapping element) |
| Canvas events (`mousemove`, `click` on canvas) | — | 🔶 Wrap `<canvas>` in `<div on:click={...}>` |

Runtime CSS color parser `color()` supports `#rgb`/`#rgba`/`#rrggbb`/`#rrggbbaa`, `rgb()`, `rgba()`, `hsl()`, `hsla()`, 140+ named colors, and `"transparent"`.

See [Canvas Example](./examples/canvas.md) for a live demo.

## 11. Routing

| HTML/CSS | vgui | Support |
|----------|------|---------|
| No native SPA routing (relies on frameworks) | `router` module | ✅ |

API overview:

- `create_router(initial)` → `Router` (signal-driven)
- `match_pattern("/users/:id", path)` → `RouteMatch` with params
- `:param` placeholder + `*` wildcard
- `build_path(pattern, params)` reverse building
- `router.navigate(cx, path)` navigation
- `router.render(cx, &[(pattern, callback)])` declarative route matching

See [Router](./concepts/router.md) for details.

## 12. Refs & Imperative Operations

| HTML DOM ref / React ref | vgui `NodeRef` | Support |
|--------------------------|----------------|---------|
| `focus` / `is_focused` / `contains_focused` | — | ✅ |
| `bounds` / `scroll_offset` / `scroll_to` / `scroll_to_top` / `scroll_to_bottom` / `set_scroll_offset` | — | ✅ |
| `child_bounds` / `child_count` | — | ✅ |
| Full DOM handle | Based on gpui `FocusHandle` + `ScrollHandle`; data from previous frame | 🔶 |
| Direct ref on `select`/`textarea`/text input/`range` | Use wrapping div | 🔶 |

## 13. Theming System

| HTML/CSS | vgui | Support |
|----------|------|---------|
| CSS Custom Properties (`--var`) | `--name: value` + `var(--name)` inside `css!` | ✅ |
| CSS theme switching | `theme!` macro + `set_theme()` global thread-local | ✅ |
| Automatic reactive theme changes | Must read signal inside render closure + `set_theme` to trigger re-render | 🔶 |
| CSS media query theme switching | Combine breakpoint + signal manually | 🔶 |

## 14. Cross-Platform

| HTML/CSS | vgui | Support |
|----------|------|---------|
| Cross-platform (browser) | Native + WASM dual-target | ✅ |

- Linux: Wayland/X11
- macOS: Cocoa/Metal
- Windows: Win32/DirectX
- Web: `wasm32-unknown-unknown` + wasm-bindgen
- GPU-accelerated rendering (gpui)

## 15. Unsupported Features Summary

| Feature | Alternative |
|---------|-------------|
| `transform` / `translate` / `rotate` / `scale` | ❌ None |
| `transition` / `animation` (inside `css!`) | `tw!` `transition-*` / `animate-*` |
| `z-index` (inside `css!`) | `tw!` `z-N` |
| `box-sizing` | ❌ None (content-box semantics) |
| `outline` | `border` |
| `list-style` / `list-style-type` | Manually draw numbers/bullets |
| `background-image` | `linear-gradient` |
| `background-position` / `background-size` / `background-repeat` | ❌ None |
| `float` / `clear` | ❌ None |
| `@media` queries (inside `css!`) | `tw!` responsive prefixes `sm:`/`md:`/`lg:`/`xl:` |
| `!important` | ❌ None |
| `position: fixed` / `sticky` | ❌ None |
| `animate-spin` | ❌ None (no rotation transform) |
| `:focus-within` / `:focus-visible` / `:visited` / `:link` / `:target` / `:nth-child` etc. | ❌ None |
| CSS Container Queries | ❌ None |
| `focus` / `blur` / `mouseenter` / `mouseleave` / `load` / Drag / touch events | gpui pointer events cover some |
| Grid `fr` / `minmax()` / `repeat()` / `auto-fit` / `auto-fill` / `grid-auto-*` | Numeric column counts |
| `rowspan` visual effect | ❌ None |
| `input type="image"` | ❌ None |
| Canvas `strokeText` / `drawImage` / `clip` / gradients / patterns / pixel ops | ❌ See [Canvas 2D Drawing](#10-canvas-2d-drawing) for details |
| Canvas `lineCap` / `lineJoin` | ❌ lyon types not re-exported |

## 16. Documentation vs Source Consistency Notes

This section summarizes the alignment between book documentation and actual source code, for developer reference.

**Already-documented features** (this page aggregates and compares them; they are not new discoveries):

- Responsive breakpoints `sm:`/`md:`/`lg:`/`xl:` — see [Tailwind Classes](./styling/tailwind-classes.md)
- ARIA `role` / `aria:*` attributes — see [The `view!` Macro](./concepts/view-macro.md), [Built-in HTML Elements](./elements/builtin-elements.md)
- SPA Router — see [Router](./concepts/router.md)
- `grid-template-areas` / `grid-area` — see [CSS Property Reference](./styling/css-reference.md)

**Source vs documentation discrepancies**:

- **`use_breakpoint()` hook**: The `crates/vgui/src/breakpoint.rs` module doc-comment (line 3) mentions a `use_breakpoint()` hook, but that function is neither implemented nor exported. The actual approach to obtain the current breakpoint is to combine `Breakpoint::from_width()` with `reactive::get_viewport_width()`. The book only documents the `tw!` responsive prefixes and does not cover this programmatic API.
- **`css!` does not support `@media`**: [CSS Property Reference](./styling/css-reference.md) lists `@media` as an unsupported `css!` property and points to `tw!` responsive prefixes. This description is accurate — the `css!` macro itself does not support media queries; responsive capabilities are provided by `tw!`.
