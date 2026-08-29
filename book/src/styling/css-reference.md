# CSS Property Reference

This page lists every CSS property supported by the `css!` macro, grouped by
category. Properties not listed here produce a compile-time error.

## Layout

| Property                  | Values                                                |
| ------------------------- | ----------------------------------------------------- |
| `display`                 | `flex` \| `block` \| `none` \| `grid`                 |
| `visibility`              | `hidden` \| `visible`                                 |
| `overflow`                | `hidden` \| `scroll` \| `visible`                     |
| `overflow-x`              | `hidden` \| `scroll` \| `visible`                     |
| `overflow-y`              | `hidden` \| `scroll` \| `visible`                     |
| `position`                | `relative` \| `absolute`                              |
| `flex-direction`          | `row` \| `column` \| `row-reverse` \| `column-reverse`|
| `flex-wrap`               | `nowrap` \| `wrap` \| `wrap-reverse`                  |
| `flex`                    | `N` (grow) \| `N N` (grow shrink) \| `N N basis`     |
| `flex-grow`               | number                                                |
| `flex-shrink`             | number                                                |
| `flex-basis`              | length                                                |
| `justify-content`         | `flex-start` \| `flex-end` \| `center` \| `space-between` \| `space-around` \| `space-evenly` |
| `align-items`             | `flex-start` \| `flex-end` \| `center` \| `baseline` \| `stretch` |
| `align-self`              | same as `align-items`                                 |
| `align-content`           | `flex-start` \| `flex-end` \| `center` \| `space-between` \| `space-around` \| `stretch` \| `space-evenly` |
| `gap`                     | length \| length length                               |
| `row-gap`                 | length                                                |
| `column-gap`              | length                                                |
| `grid-template-columns`   | number (column count)                                 |
| `grid-template-rows`      | number (row count)                                    |
| `grid-column`             | `span N` \| `A / B` \| `N`                            |
| `grid-column-start`       | number                                                |
| `grid-column-end`         | number                                                |
| `grid-row`                | `span N` \| `A / B` \| `N`                            |
| `grid-row-start`          | number                                                |
| `grid-row-end`            | number                                                |
| `scrollbar-width`         | `auto` \| `thin` \| `none`                            |

## Box Model

| Property          | Values                                                |
| ----------------- | ----------------------------------------------------- |
| `width`           | length                                                |
| `height`          | length                                                |
| `min-width`       | length                                                |
| `min-height`      | length                                                |
| `max-width`       | length                                                |
| `max-height`      | length                                                |
| `padding`         | length \| length length \| length length length length |
| `padding-top`     | length                                                |
| `padding-right`   | length                                                |
| `padding-bottom`  | length                                                |
| `padding-left`    | length                                                |
| `padding-inline`  | length (sets left + right)                            |
| `padding-block`   | length (sets top + bottom)                            |
| `margin`          | length \| length length \| length length length length |
| `margin-top`      | length                                                |
| `margin-right`    | length                                                |
| `margin-bottom`   | length                                                |
| `margin-left`     | length                                                |
| `margin-inline`   | length (sets left + right)                            |
| `margin-block`    | length (sets top + bottom)                            |
| `inset`           | length \| length length \| length length length length |
| `top`             | length                                                |
| `right`           | length                                                |
| `bottom`          | length                                                |
| `left`            | length                                                |
| `aspect-ratio`    | number                                                |

## Visual

| Property                  | Values                                                |
| ------------------------- | ----------------------------------------------------- |
| `background`              | color \| `linear-gradient(angle, from, to)`           |
| `background-color`        | color                                                 |
| `color`                   | color                                                 |
| `opacity`                 | number (0.0–1.0)                                      |
| `border`                  | `width style color` (e.g. `1px solid #ccc`)          |
| `border-color`            | color                                                 |
| `border-style`            | `solid` \| `dashed`                                   |
| `border-width`            | length                                                |
| `border-radius`           | length                                                |
| `border-top-left-radius`  | length                                                |
| `border-top-right-radius` | length                                                |
| `border-bottom-right-radius` | length                                             |
| `border-bottom-left-radius`| length                                               |
| `cursor`                  | `pointer` \| `default` \| `text` \| `crosshair` \| `not-allowed` \| `grab` \| `grabbing` |
| `box-shadow`              | `none` \| `sm` \| `md` \| `lg` \| `xl`               |

## Text

| Property                  | Values                                                |
| ------------------------- | ----------------------------------------------------- |
| `font-size`               | length (not `%`)                                      |
| `font-weight`             | `thin` \| `extra-light` \| `light` \| `normal` \| `medium` \| `semibold` \| `bold` \| `extrabold` \| `black` \| 100–900 |
| `font-style`              | `italic` \| `normal`                                  |
| `font-family`             | string literal or keyword                             |
| `text-align`              | `left` \| `center` \| `right`                         |
| `text-decoration`         | `underline` \| `line-through` \| `none`               |
| `text-decoration-color`   | color                                                 |
| `text-decoration-thickness`| length                                              |
| `text-decoration-style`   | `solid` \| `wavy`                                     |
| `text-overflow`           | `ellipsis` \| `clip`                                  |
| `text-background`         | color                                                 |
| `text-background-color`   | color                                                 |
| `white-space`             | `nowrap` \| `normal`                                  |
| `line-height`             | number (unitless → relative) or length                |
| `line-clamp`              | number                                                |

## Unsupported Properties

The following common CSS properties are **not** supported by `css!` because
`gpui`'s styling model does not provide equivalents:

- `transform` / `translate` / `rotate` / `scale`
- `transition` / `animation`
- `z-index` (use `tw!` `z-N` utilities instead)
- `box-sizing` (gpui uses content-box semantics)
- `grid-template-areas`
- `gap` with more than 2 values
- `outline` (use `border` instead)
- `list-style` / `list-style-type`
- `background-image` (use `linear-gradient` in `background` instead)
- `background-position` / `background-size` / `background-repeat`
- `float` / `clear`
- `@media` queries / responsive breakpoints
- CSS variables / custom properties
- `!important`

If you need a property not listed here, check whether a `tw!` utility covers
it, or use `gpui`'s styled element methods directly outside of `css!`.
