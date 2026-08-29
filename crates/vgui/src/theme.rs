//! Runtime theme store for CSS custom properties (`var()`).
//!
//! `--name: value` inside `css!` provides a compile-time default (inlined as
//! the fallback); `var(--name)` / `var(--name, fallback)` emits a runtime
//! lookup against the thread-local current theme. Cross-block sharing and
//! dark/light theming work by swapping the theme via [`set_theme`] or
//! [`with_theme`]. Scope is global (no DOM-subtree cascade).

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::{AbsoluteLength, DefiniteLength, FontWeight, Hsla, Length, SharedString, StyleRefinement};

/// A typed CSS value stored in a [`Theme`].
///
/// Each variant corresponds to a value category the `css!` macro can produce.
/// The `as_*` methods coerce to the concrete gpui type expected by a given
/// property, panicking with a descriptive message on a type mismatch.
#[derive(Clone, Debug)]
pub enum CssValue {
    /// A color (`#rrggbb`, `rgb(...)`, named colors, …).
    Color(Hsla),
    /// A length that may be `auto` (`px`, `rem`, `%`, `auto`).
    Length(Length),
    /// A definite length (`px`, `rem`, `%` — never `auto`).
    DefiniteLength(DefiniteLength),
    /// An absolute length (`px`, `rem` — never `%` or `auto`).
    AbsoluteLength(AbsoluteLength),
    /// A bare number (`0.5`, `2`, …).
    Number(f32),
    /// A keyword identifier (`column`, `bold`, `none`, …).
    Keyword(SharedString),
}

impl CssValue {
    fn found(&self) -> &'static str {
        match self {
            CssValue::Color(_) => "color",
            CssValue::Length(_) => "length",
            CssValue::DefiniteLength(_) => "definite length",
            CssValue::AbsoluteLength(_) => "absolute length",
            CssValue::Number(_) => "number",
            CssValue::Keyword(_) => "keyword",
        }
    }

    /// Coerce to a color, panicking on mismatch.
    pub fn as_color(&self, name: &str) -> Hsla {
        match self {
            CssValue::Color(c) => *c,
            _ => panic!("css var '{name}' holds a {}, expected color", self.found()),
        }
    }

    /// Coerce to a length (definite/absolute widen via `Into`), panicking on mismatch.
    pub fn as_length(&self, name: &str) -> Length {
        match self {
            CssValue::Length(l) => *l,
            CssValue::DefiniteLength(d) => (*d).into(),
            CssValue::AbsoluteLength(a) => (*a).into(),
            _ => panic!("css var '{name}' holds a {}, expected length", self.found()),
        }
    }

    /// Coerce to a definite length (absolute widens; `Length`/`Auto` panics).
    pub fn as_definite(&self, name: &str) -> DefiniteLength {
        match self {
            CssValue::DefiniteLength(d) => *d,
            CssValue::AbsoluteLength(a) => (*a).into(),
            _ => panic!("css var '{name}' holds a {}, expected definite length", self.found()),
        }
    }

    /// Coerce to an absolute length, panicking on mismatch.
    pub fn as_absolute(&self, name: &str) -> AbsoluteLength {
        match self {
            CssValue::AbsoluteLength(a) => *a,
            _ => panic!("css var '{name}' holds a {}, expected absolute length", self.found()),
        }
    }

    /// Coerce to a number, panicking on mismatch.
    pub fn as_number(&self, name: &str) -> f32 {
        match self {
            CssValue::Number(n) => *n,
            _ => panic!("css var '{name}' holds a {}, expected number", self.found()),
        }
    }

    /// Coerce to a keyword, panicking on mismatch.
    pub fn as_keyword(&self, name: &str) -> SharedString {
        match self {
            CssValue::Keyword(k) => k.clone(),
            _ => panic!("css var '{name}' holds a {}, expected keyword", self.found()),
        }
    }
}

/// A collection of CSS custom-property values applied at runtime.
///
/// Build with the `set_*` convenience methods or the [`theme!`](crate::theme)
/// macro, then install via [`set_theme`] or scope via [`with_theme`].
#[derive(Clone, Default)]
pub struct Theme {
    /// Variable name (without `--`) → typed value.
    pub vars: HashMap<String, CssValue>,
}

impl Theme {
    /// Create an empty theme.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a variable to a typed value. Returns `&mut Self` for chaining.
    pub fn set(&mut self, name: impl Into<String>, value: CssValue) -> &mut Self {
        self.vars.insert(name.into(), value);
        self
    }

    /// Set a color variable.
    pub fn set_color(&mut self, name: impl Into<String>, value: Hsla) -> &mut Self {
        self.set(name, CssValue::Color(value))
    }

    /// Set a length variable.
    pub fn set_length(&mut self, name: impl Into<String>, value: Length) -> &mut Self {
        self.set(name, CssValue::Length(value))
    }

    /// Set a definite-length variable.
    pub fn set_definite(&mut self, name: impl Into<String>, value: DefiniteLength) -> &mut Self {
        self.set(name, CssValue::DefiniteLength(value))
    }

    /// Set an absolute-length variable.
    pub fn set_absolute(&mut self, name: impl Into<String>, value: AbsoluteLength) -> &mut Self {
        self.set(name, CssValue::AbsoluteLength(value))
    }

    /// Set a number variable.
    pub fn set_number(&mut self, name: impl Into<String>, value: f32) -> &mut Self {
        self.set(name, CssValue::Number(value))
    }

    /// Set a keyword variable.
    pub fn set_keyword(&mut self, name: impl Into<String>, value: impl Into<SharedString>) -> &mut Self {
        self.set(name, CssValue::Keyword(value.into()))
    }
}

thread_local! {
    static CURRENT: RefCell<Theme> = RefCell::new(Theme::default());
}

/// Replace the current thread-local theme with `theme`.
///
/// Not auto-reactive: gpui is not notified. To re-render on a theme swap, read
/// a signal inside the render closure and call `set_theme` there.
pub fn set_theme(theme: Theme) {
    CURRENT.with(|c| *c.borrow_mut() = theme);
}

/// Run `f` with `theme` as the current theme, restoring the previous theme
/// afterwards. Enables scoped overrides (a future `<ThemeProvider>`).
pub fn with_theme<R>(theme: &Theme, f: impl FnOnce() -> R) -> R {
    let prev = CURRENT.with(|c| std::mem::replace(&mut *c.borrow_mut(), theme.clone()));
    let r = f();
    CURRENT.with(|c| *c.borrow_mut() = prev);
    r
}

// ---------------------------------------------------------------------------
// Hidden accessors — called by macro-emitted code. Not part of the public API.
// ---------------------------------------------------------------------------

macro_rules! var_accessor {
    ($fn:ident, $ret:ty, $method:ident) => {
        #[doc(hidden)]
        pub fn $fn(name: &str, default: Option<$ret>) -> $ret {
            CURRENT.with(|c| {
                if let Some(v) = c.borrow().vars.get(name) {
                    v.$method(name)
                } else {
                    default.unwrap_or_else(|| panic!("css var '{name}' is not set"))
                }
            })
        }
    };
}

var_accessor!(__var_color, Hsla, as_color);
var_accessor!(__var_length, Length, as_length);
var_accessor!(__var_definite, DefiniteLength, as_definite);
var_accessor!(__var_absolute, AbsoluteLength, as_absolute);
var_accessor!(__var_number, f32, as_number);
var_accessor!(__var_keyword, SharedString, as_keyword);
var_accessor!(__var_font_family, SharedString, as_keyword);

/// Resolve a `var()` for `font-weight`. Theme `Number` → numeric weight;
/// `Keyword` → named weight; else panic.
#[doc(hidden)]
pub fn __var_font_weight(name: &str, default: Option<FontWeight>) -> FontWeight {
    CURRENT.with(|c| {
        if let Some(v) = c.borrow().vars.get(name) {
            match v {
                CssValue::Number(n) => __weight_from_number(*n as i64),
                CssValue::Keyword(s) => __weight_from_name(s.as_str()),
                _ => panic!(
                    "css var '{name}' holds a {}, expected number or keyword for font-weight",
                    v.found()
                ),
            }
        } else {
            default.unwrap_or_else(|| panic!("css var '{name}' is not set"))
        }
    })
}

/// Resolve a `var()` for `line-height`. Theme `Number(n)` → `relative(n)`;
/// length variants → definite length; else panic.
#[doc(hidden)]
pub fn __var_line_height(name: &str, default: Option<DefiniteLength>) -> DefiniteLength {
    CURRENT.with(|c| {
        if let Some(v) = c.borrow().vars.get(name) {
            match v {
                CssValue::Number(n) => gpui::relative(*n),
                CssValue::DefiniteLength(d) => *d,
                CssValue::AbsoluteLength(a) => (*a).into(),
                _ => panic!(
                    "css var '{name}' holds a {}, expected number or length for line-height",
                    v.found()
                ),
            }
        } else {
            default.unwrap_or_else(|| panic!("css var '{name}' is not set"))
        }
    })
}

// ---------------------------------------------------------------------------
// Runtime keyword resolvers — transcribed from the compile-time match tables
// in `vgui-css/src/keywords.rs`, `layout.rs`, `visual.rs`, `text.rs`. Kept
// separate so literal keywords retain compile-time validation.
// ---------------------------------------------------------------------------

#[doc(hidden)]
pub fn __resolve_display(s: &str) -> gpui::Display {
    match s {
        "flex" => gpui::Display::Flex,
        "block" => gpui::Display::Block,
        "none" => gpui::Display::None,
        "grid" => gpui::Display::Grid,
        _ => panic!("unsupported CSS value for 'display': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_visibility(s: &str) -> gpui::Visibility {
    match s {
        "hidden" => gpui::Visibility::Hidden,
        "visible" => gpui::Visibility::Visible,
        _ => panic!("unsupported CSS value for 'visibility': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_overflow(s: &str) -> gpui::Overflow {
    match s {
        "hidden" => gpui::Overflow::Hidden,
        "scroll" => gpui::Overflow::Scroll,
        "visible" => gpui::Overflow::Visible,
        _ => panic!("unsupported CSS value for 'overflow': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_position(s: &str) -> gpui::Position {
    match s {
        "relative" => gpui::Position::Relative,
        "absolute" => gpui::Position::Absolute,
        _ => panic!("unsupported CSS value for 'position': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_flex_direction(s: &str) -> gpui::FlexDirection {
    match s {
        "row" => gpui::FlexDirection::Row,
        "column" => gpui::FlexDirection::Column,
        "row-reverse" => gpui::FlexDirection::RowReverse,
        "column-reverse" => gpui::FlexDirection::ColumnReverse,
        _ => panic!("unsupported CSS value for 'flex-direction': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_flex_wrap(s: &str) -> gpui::FlexWrap {
    match s {
        "nowrap" => gpui::FlexWrap::NoWrap,
        "wrap" => gpui::FlexWrap::Wrap,
        "wrap-reverse" => gpui::FlexWrap::WrapReverse,
        _ => panic!("unsupported CSS value for 'flex-wrap': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_justify(s: &str) -> gpui::JustifyContent {
    match s {
        "flex-start" | "start" => gpui::JustifyContent::FlexStart,
        "flex-end" | "end" => gpui::JustifyContent::FlexEnd,
        "center" => gpui::JustifyContent::Center,
        "space-between" => gpui::JustifyContent::SpaceBetween,
        "space-around" => gpui::JustifyContent::SpaceAround,
        "space-evenly" => gpui::JustifyContent::SpaceEvenly,
        _ => panic!("unsupported CSS value for 'justify-content': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_align_items(s: &str) -> gpui::AlignItems {
    match s {
        "flex-start" | "start" => gpui::AlignItems::FlexStart,
        "flex-end" | "end" => gpui::AlignItems::FlexEnd,
        "center" => gpui::AlignItems::Center,
        "baseline" => gpui::AlignItems::Baseline,
        "stretch" => gpui::AlignItems::Stretch,
        _ => panic!("unsupported CSS value for 'align-items': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_align_content(s: &str) -> gpui::AlignContent {
    match s {
        "flex-start" | "start" => gpui::AlignContent::FlexStart,
        "flex-end" | "end" => gpui::AlignContent::FlexEnd,
        "center" => gpui::AlignContent::Center,
        "space-between" => gpui::AlignContent::SpaceBetween,
        "space-around" => gpui::AlignContent::SpaceAround,
        "stretch" => gpui::AlignContent::Stretch,
        "space-evenly" => gpui::AlignContent::SpaceEvenly,
        _ => panic!("unsupported CSS value for 'align-content': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_cursor(s: &str) -> gpui::CursorStyle {
    match s {
        "pointer" => gpui::CursorStyle::PointingHand,
        "default" => gpui::CursorStyle::Arrow,
        "text" => gpui::CursorStyle::IBeam,
        "crosshair" => gpui::CursorStyle::Crosshair,
        "not-allowed" => gpui::CursorStyle::OperationNotAllowed,
        "grab" => gpui::CursorStyle::OpenHand,
        "grabbing" => gpui::CursorStyle::ClosedHand,
        _ => panic!("unsupported CSS value for 'cursor': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_border_style(s: &str) -> gpui::BorderStyle {
    match s {
        "solid" => gpui::BorderStyle::Solid,
        "dashed" => gpui::BorderStyle::Dashed,
        _ => panic!("unsupported CSS value for 'border-style': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_font_style(s: &str) -> gpui::FontStyle {
    match s {
        "italic" => gpui::FontStyle::Italic,
        "normal" => gpui::FontStyle::Normal,
        _ => panic!("unsupported CSS value for 'font-style': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_text_align(s: &str) -> gpui::TextAlign {
    match s {
        "left" => gpui::TextAlign::Left,
        "center" => gpui::TextAlign::Center,
        "right" => gpui::TextAlign::Right,
        _ => panic!("unsupported CSS value for 'text-align': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_white_space(s: &str) -> gpui::WhiteSpace {
    match s {
        "nowrap" => gpui::WhiteSpace::Nowrap,
        "normal" => gpui::WhiteSpace::Normal,
        _ => panic!("unsupported CSS value for 'white-space': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_text_overflow(s: &str) -> gpui::TextOverflow {
    match s {
        "ellipsis" => gpui::TextOverflow::Truncate(SharedString::new_static("…")),
        "clip" => gpui::TextOverflow::Truncate(SharedString::new_static("")),
        _ => panic!("unsupported text-overflow: {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_box_shadow(s: &str) -> Vec<gpui::BoxShadow> {
    match s {
        "none" => Vec::new(),
        "sm" => vec![
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(1.)),
                blur_radius: gpui::px(3.),
                spread_radius: gpui::px(0.),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(1.)),
                blur_radius: gpui::px(2.),
                spread_radius: gpui::px(-1.),
                inset: false,
            },
        ],
        "md" => vec![
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(4.)),
                blur_radius: gpui::px(6.),
                spread_radius: gpui::px(-1.),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(2.)),
                blur_radius: gpui::px(4.),
                spread_radius: gpui::px(-2.),
                inset: false,
            },
        ],
        "lg" => vec![
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(10.)),
                blur_radius: gpui::px(15.),
                spread_radius: gpui::px(-3.),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(4.)),
                blur_radius: gpui::px(6.),
                spread_radius: gpui::px(-4.),
                inset: false,
            },
        ],
        "xl" => vec![
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(20.)),
                blur_radius: gpui::px(25.),
                spread_radius: gpui::px(-5.),
                inset: false,
            },
            gpui::BoxShadow {
                color: gpui::hsla(0., 0., 0., 0.1),
                offset: gpui::point(gpui::px(0.), gpui::px(8.)),
                blur_radius: gpui::px(10.),
                spread_radius: gpui::px(-6.),
                inset: false,
            },
        ],
        _ => panic!("unsupported CSS value for 'box-shadow': {s}"),
    }
}

#[doc(hidden)]
pub fn __resolve_scrollbar_width(s: &str) -> Option<AbsoluteLength> {
    match s {
        "thin" => Some(AbsoluteLength::Pixels(gpui::px(4.))),
        "none" => Some(AbsoluteLength::Pixels(gpui::px(0.))),
        _ => Some(AbsoluteLength::Pixels(gpui::px(12.))),
    }
}

#[doc(hidden)]
pub fn __weight_from_number(n: i64) -> FontWeight {
    match n {
        100 => FontWeight::THIN,
        200 => FontWeight::EXTRA_LIGHT,
        300 => FontWeight::LIGHT,
        400 => FontWeight::NORMAL,
        500 => FontWeight::MEDIUM,
        600 => FontWeight::SEMIBOLD,
        700 => FontWeight::BOLD,
        800 => FontWeight::EXTRA_BOLD,
        900 => FontWeight::BLACK,
        _ => panic!("unsupported font-weight number: {n}"),
    }
}

#[doc(hidden)]
pub fn __weight_from_name(s: &str) -> FontWeight {
    match s {
        "thin" => FontWeight::THIN,
        "extra-light" => FontWeight::EXTRA_LIGHT,
        "light" => FontWeight::LIGHT,
        "normal" => FontWeight::NORMAL,
        "medium" => FontWeight::MEDIUM,
        "semibold" => FontWeight::SEMIBOLD,
        "bold" => FontWeight::BOLD,
        "extrabold" | "extra-bold" => FontWeight::EXTRA_BOLD,
        "black" => FontWeight::BLACK,
        _ => panic!("unsupported font-weight keyword: {s}"),
    }
}

#[doc(hidden)]
pub fn __apply_text_decoration(s: &mut StyleRefinement, kw: &str) {
    match kw {
        "underline" => {
            s.text.underline = Some(gpui::UnderlineStyle {
                thickness: gpui::px(1.),
                ..Default::default()
            });
        }
        "line-through" => {
            s.text.strikethrough = Some(gpui::StrikethroughStyle {
                thickness: gpui::px(1.),
                ..Default::default()
            });
        }
        "none" => {
            s.text.underline = None;
            s.text.strikethrough = None;
        }
        _ => panic!("unsupported CSS value for 'text-decoration': {kw}"),
    }
}

#[doc(hidden)]
pub fn __apply_text_decoration_style(s: &mut StyleRefinement, kw: &str) {
    let wavy = match kw {
        "solid" => false,
        "wavy" => true,
        _ => panic!("unsupported text-decoration-style: {kw}"),
    };
    s.text.underline.get_or_insert_with(Default::default).wavy = wavy;
}
