//! Virtual list support — exposes gpui's `uniform_list` primitive for
//! rendering large fixed-height lists with only visible items materialised.
//!
//! The `view!` macro emits calls to `gpui::uniform_list` directly; this
//! module provides the [`use_scroll_handle`] hook for programmatic scroll
//! control and re-exports the gpui types users need.

pub use gpui::{ScrollStrategy, UniformListScrollHandle};

/// Get-or-create a persistent [`UniformListScrollHandle`] cached in the
/// current reactive scope slot.
///
/// On the first render a new handle is created and stored; subsequent
/// re-renders return the same handle so scroll position persists. The
/// handle can be passed to `<uniform_list scroll_handle={...}>` and used
/// between renders to call `.scroll_to_item(ix, ScrollStrategy::Top)`,
/// `.scroll_to_bottom()`, `.is_scrolled_to_end()`, etc.
///
/// Must be called inside a `VguiRoot` render scope (same precondition as
/// `create_signal`).
pub fn use_scroll_handle() -> UniformListScrollHandle {
    crate::reactive::get_or_create_slot(|_cx| UniformListScrollHandle::new())
}
