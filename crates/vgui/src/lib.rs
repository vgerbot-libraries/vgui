//! vgui — a declarative, reactive GUI framework for Rust, built on gpui.
//!
//! Provides the `view!` macro for JSX-like views, `css!`/`tw!` for styling,
//! SolidJS-style reactivity (`create_signal`, `create_memo`, `create_effect`),
//! and built-in control-flow and input components.
extern crate self as vgui;

pub use vgui_css::css;
pub use vgui_css::theme;
pub use vgui_tailwind::tw;
pub use vgui_view::view;

mod child;
mod control;
mod input_text;
mod input_widgets;
mod label;
pub mod prelude;
mod reactive;
mod root;
mod tw_dynamic;
mod style;
pub mod theme;

pub use crate::child::{click, into_child, IntoViewChild};
pub use crate::control::{details, dialog, for_each, for_each_or, progress, show, show_when};
pub use crate::input_text::{
    input_cb, str_change_cb, text_area, text_input, TextAreaProps, TextKind, TextInputProps,
};
pub use crate::input_widgets::{
    bool_change_cb, checkbox, CheckboxProps, f64_change_cb, file_input, files_cb, radio,
    range_input, select, FileProps, RadioProps, RangeProps, SelectProps,
    str_select_change_cb,
};
pub use crate::label::{focus_label_target, __label_scope_enter, label_scope_exit};
pub use crate::reactive::{create_effect, create_memo, create_signal, next_auto_id, ReadSignal, WriteSignal};
pub use crate::root::{mount, VguiRoot};
pub use crate::style::{ApplyStyle, Css, TwStyle};
pub use crate::style::{IntoTwStyle, TwClass, TwClassSource};
pub use crate::tw_dynamic::tw_dynamic;
pub use crate::theme::{set_theme, with_theme, CssValue, Theme};
// Hidden helpers called by macro-emitted `var()` code.
pub use crate::theme::{
    __apply_text_decoration, __apply_text_decoration_style, __resolve_align_content,
    __resolve_align_items, __resolve_border_style, __resolve_box_shadow, __resolve_cursor,
    __resolve_display, __resolve_flex_direction, __resolve_flex_wrap, __resolve_font_style,
    __resolve_justify, __resolve_overflow, __resolve_position, __resolve_scrollbar_width,
    __resolve_text_align, __resolve_text_overflow, __resolve_visibility, __resolve_white_space,
    __var_absolute, __var_color, __var_definite, __var_font_family, __var_font_weight,
    __var_keyword, __var_length, __var_line_height, __var_number, __weight_from_name,
    __weight_from_number,
};

// Internal test helpers — not part of the public API. Exposed so integration
// tests can simulate a render scope without spinning up a full gpui App.
pub use crate::reactive::{__test_enter_render_scope, __test_exit_render_scope};

#[cfg(test)]
mod style_tests;
