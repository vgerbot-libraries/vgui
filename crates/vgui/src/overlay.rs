use gpui::{anchored, deferred, Display, InteractiveElement, IntoElement, ParentElement, Pixels, Point, Styled};
use std::sync::Arc;

/// Render `content` on a floating layer drawn after all non-deferred ancestors.
///
/// `priority` controls stacking relative to other deferred elements: higher
/// values paint on top. This is the base "portal" primitive — `dialog` and
/// `floating` wrap it with higher-level behavior.
pub fn portal(content: impl gpui::IntoElement, priority: usize) -> gpui::AnyElement {
    deferred(content).priority(priority).into_any_element()
}

/// Render a modal dialog overlay.
///
/// When `open` is false, returns a hidden element with no layout impact.
/// When `open` is true, paints a full-screen semi-transparent backdrop on a
/// deferred layer (priority 100) above all non-deferred content. The backdrop
/// occludes mouse events so they never reach elements behind it. `on_close` is
/// invoked when the user clicks outside the content (on the backdrop) or presses
/// Escape while focus is within the dialog.
pub fn dialog(
    open: bool,
    on_close: impl Fn(&mut gpui::App) + 'static,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    if !open {
        let mut el = gpui::div();
        el.style().display = Some(Display::None);
        return el.into_any_element();
    }
    let on_close = Arc::new(on_close);
    deferred(
        anchored()
            .position(Point::default())
            .child(
                gpui::div()
                    .id("vgui-dialog-backdrop")
                    .size_full()
                    .bg(gpui::hsla(0., 0., 0., 0.5))
                    .flex()
                    .items_center()
                    .justify_center()
                    .occlude()
                    .on_key_down({
                        let on_close = on_close.clone();
                        move |event, _window, cx| {
                            if event.keystroke.key == "escape" {
                                on_close(cx);
                            }
                        }
                    })
                    .child(
                        gpui::div()
                            .id("vgui-dialog-content")
                            .on_mouse_down_out({
                                let on_close = on_close.clone();
                                move |_, _, cx| on_close(cx)
                            })
                            .child(content),
                    ),
            ),
    )
    .priority(100)
    .into_any_element()
}

/// Render a positioned floating element at a window-coordinate point.
///
/// Uses `anchored()` for overflow-aware placement: if the content would extend
/// past the window edge, it snaps inside with an 8px margin. Paints on a
/// deferred layer at priority 50 (below `dialog`'s 100). No built-in dismissal
/// — add `on:mouse_down_out` to the content for click-outside behavior.
pub fn floating(
    position: Point<Pixels>,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    deferred(
        anchored()
            .position(position)
            .snap_to_window_with_margin(gpui::px(8.))
            .child(content),
    )
    .priority(50)
    .into_any_element()
}
