extern crate self as vgui;

pub use vgui_css::css;
pub use vgui_tailwind::tw;
pub use vgui_view::view;

mod child;
mod control;
pub mod prelude;
mod reactive;
mod root;
mod style;

pub use crate::child::{click, into_child, IntoViewChild};
pub use crate::control::{for_each, for_each_or, show, show_when};
pub use crate::reactive::{create_effect, create_memo, create_signal, next_auto_id, ReadSignal, WriteSignal};
pub use crate::root::{mount, VguiRoot};
pub use crate::style::{ApplyStyle, Css, TwStyle};

// Internal test helpers — not part of the public API. Exposed so integration
// tests can simulate a render scope without spinning up a full gpui App.
pub use crate::reactive::{__test_enter_render_scope, __test_exit_render_scope};

#[cfg(test)]
mod style_tests;
