extern crate self as vgui;

pub use vgui_css::css;
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
mod style;

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

// Internal test helpers — not part of the public API. Exposed so integration
// tests can simulate a render scope without spinning up a full gpui App.
pub use crate::reactive::{__test_enter_render_scope, __test_exit_render_scope};

#[cfg(test)]
mod style_tests;
