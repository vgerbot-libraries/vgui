//! Window management primitives.
//!
//! Exposes gpui's window-control APIs through vgui's reactive scope so user
//! code can set titles, minimize, toggle fullscreen, close, drag, open
//! additional windows, configure app menus, and intercept close — all from
//! inside `app()` closures without direct access to the `&mut Window` param.
//!
//! The [`with_window`] helper bridges reactive scope to the gpui window by
//! reading the `AnyWindowHandle` stored on [`crate::root::VguiRoot`] at
//! `mount` time and updating it imperatively.

pub use gpui::{
    AnyWindowHandle, Menu, MenuItem, ResizeEdge, TitlebarOptions,
    WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowHandle,
    WindowKind, WindowOptions,
};
use gpui::AppContext;

use crate::reactive::with_root_cx;

/// Open a new gpui window with a vgui root, convenience wrapper over
/// [`gpui::App::open_window`] + [`crate::root::mount`].
///
/// Returns a typed [`WindowHandle<VguiRoot>`] so the caller can update the
/// secondary window imperatively if needed. Must be called from inside a
/// reactive scope (the same precondition as `create_signal`) because it
/// relies on `with_root_cx` to obtain the `&mut App`.
pub fn open_window<R: gpui::IntoElement + 'static>(
    cx: &mut gpui::App,
    options: gpui::WindowOptions,
    render: impl FnMut() -> R + 'static,
) -> anyhow::Result<gpui::WindowHandle<crate::root::VguiRoot>> {
    cx.open_window(options, |window, cx| crate::root::mount(window, cx, render))
}

/// Run a closure with imperative access to the current window and app.
///
/// Resolves the window handle stored on `VguiRoot` at `mount` time and calls
/// `f` inside `cx.update_window`. Returns `None` when no reactive scope is
pub fn with_window<R>(f: impl FnOnce(&mut gpui::Window, &mut gpui::App) -> R) -> Option<R> {
    with_root_cx(|cx| {
        let handle = crate::reactive::get_window_handle();
        handle.and_then(|h| cx.update_window(h, |_, window, cx| f(window, cx)).ok())
    })
}

/// Set the native application menu bar.
///
/// Wraps [`gpui::App::set_menus`]. Must be called from inside a reactive
/// scope.
pub fn set_app_menus(menus: impl IntoIterator<Item = gpui::Menu>) {
    with_root_cx(|cx| {
        cx.set_menus(menus);
    });
}

/// Register an app-level callback fired when any window closes.
///
/// Wraps [`gpui::App::on_window_closed`]. Must be called from inside a
/// reactive scope. The returned [`gpui::Subscription`] keeps the handler
/// alive while held.
pub fn on_window_closed(
    handler: impl FnMut(&mut gpui::App, gpui::WindowId) + 'static,
) -> gpui::Subscription {
    with_root_cx(|cx| cx.on_window_closed(handler))
}

/// Register a close-interception handler for the current window.
///
/// The handler is called when the user attempts to close the window; returning
/// `false` prevents the close. Registration is guarded by a reactive scope
/// slot so the handler is only installed once per `VguiRoot` (per window) —
/// repeated calls on re-render are no-ops.
///
/// Must be called from inside a reactive scope.
pub fn use_window_should_close(
    handler: impl Fn(&mut gpui::Window, &mut gpui::App) -> bool + 'static,
) {
    let registered: std::rc::Rc<std::cell::Cell<bool>> =
        crate::reactive::get_or_create_slot(|_| std::rc::Rc::new(std::cell::Cell::new(false)));
    if registered.get() {
        return;
    }
    registered.set(true);
    let _ = with_window(|window, cx| {
        window.on_window_should_close(cx, handler);
    });
}
