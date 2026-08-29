# Tailwind Classes (`tw!`)

The `tw!` macro compiles Tailwind-style utility class strings into
`gpui::StyleRefinement` mutations at build time. It is invoked automatically
when you use the `class="..."` attribute on any element in `view!`, or you can
call it directly.

## Basic Usage

In `view!`, the `class` attribute is expanded through `tw!`:

```rust
view! {
    <div class="flex flex-col gap-3 p-4 bg-[#505050] w-[500px] h-[500px] justify-center items-center text-white">
        <button class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded">{"Click"}</button>
    </div>
}
```

You can also call `tw!` directly, though this is rarely needed:

```rust
let style = tw!("flex p-4 bg-white");
```

The macro produces a `vgui::TwStyle` struct with four closures: `base`,
`hover`, `focus`, and `active`. The `view!` macro wires these to the
appropriate `gpui` pseudo-state handlers automatically.

Unknown classes are silently skipped — the macro does not error on
unrecognized utilities.

## Variants

Three variant prefixes are supported, each applying styles only in the
corresponding interaction state:

| Prefix    | State       | Example                     |
| --------- | ----------- | --------------------------- |
| `hover:`  | Mouse hover | `hover:bg-[#000088]`        |
| `focus:`  | Keyboard focus | `focus:border-blue-500`  |
| `active:` | Mouse down  | `active:bg-[#000066]`       |

```rust
view! {
    <button class="bg-blue-600 hover:bg-blue-700 active:bg-blue-800 focus:ring-2 text-white px-4 py-2 rounded">
        {"Save"}
    </button>
}
```

Variants can be stacked with any utility class: `hover:text-white`,
`focus:outline-none`, `active:scale-95` (if supported).

## Arbitrary Values

Arbitrary values use the `[...]` bracket syntax:

```rust
class="bg-[#0000ff] w-[500px] h-[300px] text-[#ff0000] rounded-[8px] p-[12px]"
```

Supported arbitrary value types:

| Category | Syntax              | Example              |
| -------- | ------------------- | -------------------- |
| Colors   | `[#hex]`            | `bg-[#ff0000]`       |
| Colors   | `[rgb(r,g,b)]`      | `bg-[rgb(255,0,0)]`  |
| Colors   | `[rgba(r,g,b,a)]`   | `bg-[rgba(0,0,255,0.5)]` |
| Lengths  | `[Npx]`             | `w-[500px]`          |
| Lengths  | `[Nrem]`            | `w-[20rem]`          |
| Lengths  | `[N%]`              | `w-[50%]`            |
| Lengths  | `[N]` (bare)        | `w-[200]` (treated as px) |

### Opacity modifier

Color utilities accept an `/NN` opacity suffix:

```rust
class="bg-blue-500/50 text-black/75"
```

The `/NN` value (0–100) sets the alpha channel of the color.

## Supported Utilities

### Display

| Class           | Effect                              |
| --------------- | ----------------------------------- |
| `flex`          | `display: flex`                     |
| `block`         | `display: block`                    |
| `hidden`        | `display: none`                     |
| `grid`          | `display: grid`                     |
| `inline-flex`   | `display: flex`                     |

### Flex direction

`flex-row`, `flex-col`, `flex-row-reverse`, `flex-col-reverse`

### Flex wrap

`flex-wrap`, `flex-nowrap`, `flex-wrap-reverse`

### Flex grow / shrink

| Class          | Effect                              |
| -------------- | ----------------------------------- |
| `flex-1`       | grow=1, shrink=1, basis=0           |
| `flex-auto`    | grow=1, shrink=1, basis=auto        |
| `flex-none`    | grow=0, shrink=0, basis=auto        |
| `flex-grow`    | grow=1                              |
| `flex-grow-0`  | grow=0                              |
| `flex-shrink`  | shrink=1                            |
| `flex-shrink-0`| shrink=0                            |
| `flex-grow-N`  | grow=N (arbitrary number)           |
| `flex-shrink-N`| shrink=N                            |

### Justify content

`justify-start`, `justify-end`, `justify-center`, `justify-between`,
`justify-around`, `justify-evenly`

### Align items

`items-start`, `items-end`, `items-center`, `items-baseline`, `items-stretch`

### Align self

`self-start`, `self-end`, `self-center`, `self-stretch`, `self-baseline`

### Align content

`content-center`, `content-start`, `content-end`, `content-between`,
`content-around`, `content-stretch`, `content-evenly`

### Position

`relative`, `absolute`, `static`

### Overflow

`overflow-hidden`, `overflow-scroll`, `overflow-auto`, `overflow-visible`,
`overflow-x-*`, `overflow-y-*`

### Visibility

`visible`, `invisible`

### Spacing

The spacing scale maps class suffixes to pixel values:

| Suffix | px   | Suffix | px   | Suffix | px   |
| ------ | ---- | ------ | ---- | ------ | ---- |
| `0`    | 0    | `4`    | 16   | `12`   | 48   |
| `px`   | 1    | `5`    | 20   | `14`   | 56   |
| `0.5`  | 2    | `6`    | 24   | `16`   | 64   |
| `1`    | 4    | `7`    | 28   | `20`   | 80   |
| `1.5`  | 6    | `8`    | 32   | `24`   | 96   |
| `2`    | 8    | `9`    | 36   | `32`   | 128  |
| `2.5`  | 10   | `10`   | 40   | `48`   | 192  |
| `3`    | 12   | `11`   | 44   | `96`   | 384  |

Padding utilities: `p-N` (all), `px-N` (inline), `py-N` (block), `pt-N`,
`pr-N`, `pb-N`, `pl-N`, `ps-N`, `pe-N`.

Margin utilities: `m-N`, `mx-N`, `my-N`, `mt-N`, `mr-N`, `mb-N`, `ml-N`,
`ms-N`, `me-N`. Also `m-auto`, `mx-auto`, `my-auto`, `mt-auto`, `mr-auto`,
`mb-auto`, `ml-auto`.

### Gap

`gap-N`, `gap-x-N`, `gap-y-N`

### Sizing

| Class        | Effect                |
| ------------ | --------------------- |
| `w-full`     | width: 100%           |
| `w-auto`     | width: auto           |
| `w-fit`      | width: auto           |
| `w-screen`   | width: 100%           |
| `h-full`     | height: 100%          |
| `h-auto`     | height: auto          |
| `h-fit`      | height: auto          |
| `h-screen`   | height: 100%          |
| `min-w-full` | min-width: 100%       |
| `min-w-auto` | min-width: auto       |
| `min-h-full` | min-height: 100%      |
| `min-h-auto` | min-height: auto      |
| `max-w-full` | max-width: 100%       |
| `max-w-none` | max-width: auto       |
| `max-h-full` | max-height: 100%      |
| `max-h-none` | max-height: auto      |

Arbitrary: `w-[500px]`, `h-[300px]`, `min-w-[200px]`, `max-h-[400px]`.

### Colors

22 named color palettes, each with 11 shades (50–950):

`slate`, `gray`, `zinc`, `neutral`, `stone`, `red`, `orange`, `amber`,
`yellow`, `lime`, `green`, `emerald`, `teal`, `cyan`, `sky`, `blue`,
`indigo`, `violet`, `purple`, `fuchsia`, `pink`, `rose`

```rust
class="bg-blue-500 hover:bg-blue-600 text-gray-100 border-gray-300"
```

Special colors: `bg-black`, `bg-white`, `bg-transparent`, `text-black`,
`text-white`, `text-transparent`, `border-black`, `border-white`,
`border-transparent`.

### Typography

Font weight: `font-thin`, `font-light`, `font-normal`, `font-medium`,
`font-semibold`, `font-bold`, `font-extrabold`, `font-black`.

Font size: `text-xs` (12px), `text-sm` (14px), `text-base` (16px),
`text-lg` (18px), `text-xl` (20px), `text-2xl` (24px), `text-3xl` (30px),
`text-4xl` (36px), `text-5xl` (48px), `text-6xl` (60px), `text-7xl` (72px),
`text-8xl` (96px), `text-9xl` (128px).

Text align: `text-left`, `text-center`, `text-right`.

Font family: `font-mono`, `font-sans`, `font-serif`.

Line height: `leading-none`, `leading-tight` (1.25), `leading-normal` (1.5),
`leading-loose` (2.0).

Text decoration: `underline`, `line-through`, `no-underline`,
`decoration-solid`, `decoration-wavy`, `decoration-none`, `decoration-2`
(thickness).

Text overflow: `truncate` (overflow hidden + nowrap + ellipsis),
`text-ellipsis`, `text-clip`.

Font style: `italic`, `not-italic`.

White space: `whitespace-normal`, `whitespace-nowrap`.

### Borders

Border width: `border` (1px all), `border-t`, `border-r`, `border-b`,
`border-l`, `border-t-N`, `border-r-N`, `border-b-N`, `border-l-N`.

Border style: `border-solid`, `border-dashed`.

Border color: `border-{color}-{shade}`, `border-[#hex]`.

### Border radius

`rounded` (4px), `rounded-sm` (2px), `rounded-md` (6px), `rounded-lg` (8px),
`rounded-xl` (12px), `rounded-2xl` (16px), `rounded-3xl` (24px),
`rounded-full` (9999px), `rounded-none` (0px).

Per-corner: `rounded-tl`, `rounded-tr`, `rounded-bl`, `rounded-br`,
`rounded-t`, `rounded-r`, `rounded-b`, `rounded-l` (with optional size suffix).

### Shadows

`shadow-sm`, `shadow`, `shadow-md`, `shadow-lg`, `shadow-xl`, `shadow-2xl`,
`shadow-none`.

### Cursor

`cursor-pointer`, `cursor-default`, `cursor-text`, `cursor-not-allowed`,
`cursor-grab`, `cursor-grabbing`, `cursor-crosshair`.

### Opacity

`opacity-0` through `opacity-100` (in increments of 5).

### Inset

`inset-0`, `inset-auto`, `top-N`, `right-N`, `bottom-N`, `left-N`.

### Grid

`grid-cols-N`, `grid-rows-N`, `col-N`, `row-N`.

### Aspect ratio

`aspect-square` (1.0), `aspect-video` (16/9), `aspect-N/M` (arbitrary ratio).

### Line clamp

`line-clamp-N` (1–10).

### Z-index

`z-0`, `z-10`, `z-20`, `z-30`, `z-40`, `z-50`, `z-auto`.
