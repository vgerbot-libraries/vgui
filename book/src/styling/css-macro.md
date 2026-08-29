# CSS-in-Rust (`css!`)

The `css!` macro parses CSS-like declarations at compile time and emits a
`vgui::Css` value — a closure that mutates a `gpui::StyleRefinement`
field-by-field. There is no runtime CSS parser; every property and value is
validated at build time, so typos and unsupported properties produce compile
errors.

## Basic Usage

Pass `css!` to the `style` attribute of any element:

```rust
view! {
    <div style={css! {
        display: flex;
        flex-direction: column;
        gap: 12px;
        padding: 20px;
        background: rgb(30, 30, 30);
        color: #fff;
    }}>
        <span>{"Hello"}</span>
    </div>
}
```

Declarations are `property: value;` pairs, just like CSS. Semicolons separate
declarations; the trailing semicolon is optional.

An empty `css! {}` produces a no-op style:

```rust
let empty = css! {}; // Css::new(|_| {})
```

## Value Types

### Lengths

| Syntax    | Example      | Maps to                     |
| --------- | ------------ | --------------------------- |
| `Npx`     | `8px`        | `gpui::px(8.0)`             |
| `Nrem`    | `1.5rem`     | `gpui::rems(1.5)`           |
| `N%`      | `50%`        | `gpui::relative(0.5)`       |
| `auto`    | `auto`       | `gpui::Length::Auto`        |
| bare `N`  | `8`          | `gpui::px(8.0)` (treated as px) |

Multi-value shorthand is supported for `padding`, `margin`, `inset`, and `gap`:

```rust
css! {
    padding: 8px 16px;          /* top=8, right=16, bottom=8, left=16 */
    padding: 8px 16px 4px;      /* error — only 1, 2, or 4 values */
    padding: 8px 16px 4px 12px; /* top=8, right=16, bottom=4, left=12 */
    margin: 0 auto;             /* top/bottom=0, left/right=auto */
    gap: 12px;                  /* row-gap=12, column-gap=12 */
    gap: 8px 16px;              /* row-gap=8, column-gap=16 */
}
```

### Colors

| Syntax          | Example          | Maps to                     |
| --------------- | ---------------- | --------------------------- |
| Hex `#rgb`      | `#fff`           | `gpui::rgb(0xffffff)`       |
| Hex `#rrggbb`   | `#ff0000`        | `gpui::rgb(0xff0000)`       |
| Hex `#rrggbbaa` | `#0000ff80`      | `gpui::rgba(0x0000ff, 0x80)` |
| `rgb(r,g,b)`    | `rgb(30, 30, 30)`| `gpui::rgb(0x1e1e1e)`       |
| `rgba(r,g,b,a)` | `rgba(0,0,255,0.5)` | `gpui::rgba(0x0000ff, 0x80)` |
| Named           | `red`            | `gpui::red()`               |

Named colors: `black`, `white`, `red`, `green`, `blue`, `yellow`, `cyan`,
`magenta`, `orange`, `purple`, `gray` / `grey`.

### Gradients

`background` accepts `linear-gradient(angle, from, to)`:

```rust
css! {
    background: linear-gradient(90deg, #ff0000, #0000ff);
    background: linear-gradient(to right, #ff0000, #0000ff);
}
```

Supported angle forms: `Ndeg`, `to right`, `to left`, `to top`, `to bottom`.

### Keywords

Many properties accept keyword values that map to `gpui` enums:

```rust
css! {
    display: flex;           /* flex | block | none | grid */
    visibility: hidden;      /* hidden | visible */
    overflow: hidden;        /* hidden | scroll | visible */
    position: relative;      /* relative | absolute */
    flex-direction: column;  /* row | column | row-reverse | column-reverse */
    flex-wrap: wrap;         /* nowrap | wrap | wrap-reverse */
    justify-content: center; /* flex-start | flex-end | center | space-between | space-around | space-evenly */
    align-items: center;     /* flex-start | flex-end | center | baseline | stretch */
    text-align: center;      /* left | center | right */
    font-style: italic;      /* italic | normal */
    white-space: nowrap;     /* nowrap | normal */
    border-style: dashed;    /* solid | dashed */
    cursor: pointer;         /* pointer | default | text | crosshair | not-allowed | grab | grabbing */
}
```

### Numbers

Plain numeric literals are accepted where a number is expected:

```rust
css! {
    flex-grow: 1;
    flex-shrink: 0;
    opacity: 0.5;
    line-height: 1.5;       /* unitless → relative() */
    grid-template-columns: 3;
}
```

### Font weight

`font-weight` accepts both named and numeric forms:

```rust
css! {
    font-weight: bold;   /* thin | extra-light | light | normal | medium | semibold | bold | extrabold | black */
    font-weight: 700;    /* 100..900 */
}
```

## Pseudo-States

Pseudo-state styles do **not** go inside `css!`. Instead, they are applied as
separate attributes on the element itself:

```rust
view! {
    <button
        style={css! { padding: 8px 16px; background: #dc2626; border-radius: 4px; }}
        hover={css! { background: #b91c1c; }}
        active={css! { background: #991b1b; }}
        focus={css! { border: 2px solid #fbbf24; }}
        on:click={click(|_cx| {})}
    >
        {"Delete"}
    </button>
}
```

The `css!` macro rejects `&:hover { ... }` pseudo-selectors with a compile
error — they belong on the element as `hover`/`active`/`focus` attributes.

## Interpolation

Some properties accept a `{expr}` interpolation that splices a runtime Rust
expression into the style setter. The expression must produce a type that
converts into the appropriate `gpui` type:

```rust
let dynamic_color = gpui::rgb(0xff0000);
let dynamic_width = gpui::px(200.0);

view! {
    <div style={css! {
        background: {dynamic_color};     /* → gpui::Hsla */
        width: {dynamic_width};          /* → gpui::Length */
        opacity: {some_f32};             /* → f32 */
        flex-grow: {grow_val};           /* → f32 */
        gap: {gap_val};                  /* → gpui::DefiniteLength */
    }}>
        <span>{"Dynamic"}</span>
    </div>
}
```

Supported interpolation properties:

| Property                      | Expected type                |
| ----------------------------- | ---------------------------- |
| `width`, `height`             | `impl Into<gpui::Length>`    |
| `min-width`, `min-height`     | `impl Into<gpui::Length>`    |
| `max-width`, `max-height`     | `impl Into<gpui::Length>`    |
| `flex-grow`, `flex-shrink`    | `f32` (cast)                 |
| `flex-basis`                  | `impl Into<gpui::Length>`    |
| `gap`                         | `impl Into<gpui::DefiniteLength>` |
| `grid-template-columns/rows`  | `u16` (cast)                 |
| `aspect-ratio`                | `f32` (cast)                 |
| `opacity`                     | `f32` (cast)                 |
| `background` / `background-color` | `impl Into<gpui::Hsla>`  |
| `color`                       | `impl Into<gpui::Hsla>`      |
| `font-weight`                 | `gpui::FontWeight`           |
| `font-size`                   | `impl Into<gpui::DefiniteLength>` |
| `line-height`                 | `impl Into<gpui::DefiniteLength>` |
| `font-family`                 | `impl Into<gpui::SharedString>`  |
