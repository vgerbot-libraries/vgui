use gpui::{
    anchored, deferred, Display, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent,
    ParentElement, Pixels, Point, StatefulInteractiveElement, Styled, Window,
};
use std::sync::Arc;

/// Per-dialog-instance focus state, cached in a reactive scope slot via
/// `get_or_create_slot`. Stored as `Arc` so deferred callbacks can capture
/// clones and read/mutate the shared state.
struct DialogFocusState {
    content_handle: FocusHandle,
    saved_focus: std::cell::RefCell<Option<FocusHandle>>,
    was_open: std::cell::RefCell<bool>,
}

/// Render `content` on a floating layer drawn after all non-deferred ancestors.
///
/// `priority` controls stacking relative to other deferred elements: higher
/// values paint on top. This is the base "portal" primitive — `dialog` and
/// `floating` wrap it with higher-level behavior.
pub fn portal(content: impl gpui::IntoElement, priority: usize) -> gpui::AnyElement {
    deferred(content).priority(priority).into_any_element()
}

/// Render a modal dialog overlay with focus trap and focus restore.
///
/// When `open` is false, returns a hidden element with no layout impact.
/// When `open` is true, paints a full-screen semi-transparent backdrop on a
/// deferred layer (priority 100) above all non-deferred content. The backdrop
/// occludes mouse events so they never reach elements behind it.
///
/// **Focus management:**
/// - On open, the previously focused element is saved and focus moves into the
///   dialog content.
/// - Tab/Shift+Tab cycle within the dialog content (focus trap); focus cannot
///   escape to background elements.
/// - On close (Escape, click-outside, or `on_close`), focus is restored to the
///   saved element.
pub fn dialog(
    open: bool,
    on_close: impl Fn(&mut gpui::App) + 'static,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    let state = crate::reactive::get_or_create_slot(|cx| {
        Arc::new(DialogFocusState {
            content_handle: cx.focus_handle(),
            saved_focus: std::cell::RefCell::new(None),
            was_open: std::cell::RefCell::new(false),
        })
    });

    // Detect open/close transitions and schedule deferred focus moves. The
    // window is on the update stack during render, so `with_window` returns
    // `None` if called directly — deferring runs after the update cycle.
    let was_open = *state.was_open.borrow();
    if open && !was_open {
        *state.was_open.borrow_mut() = true;
        crate::reactive::with_root_cx(|cx| {
            let entity_id = cx.entity_id();
            let state = state.clone();
            cx.defer(move |cx| {
                cx.with_window(entity_id, |window, cx| {
                    *state.saved_focus.borrow_mut() = window.focused(cx);
                    window.focus(&state.content_handle, cx);
                });
            });
        });
    } else if !open && was_open {
        *state.was_open.borrow_mut() = false;
        crate::reactive::with_root_cx(|cx| {
            let entity_id = cx.entity_id();
            let state = state.clone();
            cx.defer(move |cx| {
                cx.with_window(entity_id, |window, cx| {
                    if let Some(saved) = state.saved_focus.borrow().as_ref() {
                        window.focus(saved, cx);
                    }
                });
            });
        });
    }

    if !open {
        let mut el = gpui::div();
        el.style().display = Some(Display::None);
        return el.into_any_element();
    }

    let on_close = Arc::new(on_close);
    let content_handle = state.content_handle.clone();

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
                        let content_handle = content_handle.clone();
                        move |event: &KeyDownEvent, window: &mut Window, cx: &mut gpui::App| {
                            if event.keystroke.key == "escape" {
                                on_close(cx);
                            } else if event.keystroke.key == "tab" {
                                // Focus trap: cycle within dialog content. Call
                                // focus_next/prev; if it escaped the content
                                // handle, refocus the content and wrap around.
                                if event.keystroke.modifiers.shift {
                                    window.focus_prev(cx);
                                    if !content_handle.contains_focused(window, cx) {
                                        content_handle.focus(window, cx);
                                        window.focus_prev(cx);
                                    }
                                } else {
                                    window.focus_next(cx);
                                    if !content_handle.contains_focused(window, cx) {
                                        content_handle.focus(window, cx);
                                        window.focus_next(cx);
                                    }
                                }
                                cx.stop_propagation();
                            }
                        }
                    })
                    .child(
                        gpui::div()
                            .id("vgui-dialog-content")
                            .track_focus(&content_handle)
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

/// Render a floating element anchored below a [`crate::NodeRef`].
///
/// Uses the bound element's previous-frame bounds. If the ref is not yet
/// bound (first frame), falls back to the origin.
pub fn floating_at(
    anchor: &crate::NodeRef,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    let position = if anchor.is_bound() {
        let bounds = anchor.bounds();
        Point {
            x: bounds.origin.x,
            y: bounds.origin.y + bounds.size.height,
        }
    } else {
        Point::default()
    };
    floating(position, content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        div, px, FocusHandle, InteractiveElement, IntoElement, KeyDownEvent, Keystroke,
        Modifiers, ParentElement, StatefulInteractiveElement, Styled, TestAppContext,
    };
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    /// Modal dialog focus management: opening moves focus off a background
    /// element, Escape requests close, and closing restores focus to the
    /// element focused before the dialog opened.
    #[test]
    fn dialog_moves_focus_in_and_restores_on_close() {
        let mut app = TestAppContext::single();
        let open = Rc::new(Cell::new(false));
        let closed = Rc::new(Cell::new(false));
        let outside_h: Rc<RefCell<Option<FocusHandle>>> = Rc::new(RefCell::new(None));

        let (o, oh, c_close) = (open.clone(), outside_h.clone(), closed.clone());
        let (_view, mut cx) = app.add_window_view(|window, cx| {
            let outside = cx.focus_handle();
            *oh.borrow_mut() = Some(outside.clone());
            crate::VguiRoot::new(cx, window.window_handle(), move || {
                let (outside, o, c_close) = (outside.clone(), o.clone(), c_close.clone());
                div()
                    .child(
                        div()
                            .id("background")
                            .track_focus(&outside)
                            .w(px(10.))
                            .h(px(10.)),
                    )
                    .child(dialog(
                        o.get(),
                        move |_| c_close.set(true),
                        div().id("dialog-body").w(px(10.)).h(px(10.)),
                    ))
                    .into_any_element()
            })
        });

        let draw = |cx: &mut gpui::VisualTestContext| {
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        };
        draw(&mut cx);

        let outside = outside_h.borrow().as_ref().unwrap().clone();

        cx.update(|window, cx| {
            outside.focus(window, cx);
            assert!(outside.is_focused(window), "precondition: background focused");
            open.set(true);
        });
        // Two passes plus a deferred-callback pump so the open transition's
        // `cx.defer` focus move executes.
        draw(&mut cx);
        cx.update(|_, _| {});
        draw(&mut cx);
        cx.update(|window, _cx| {
            assert!(
                !outside.is_focused(window),
                "opening the dialog must move focus off the background"
            );
        });

        // Escape while open requests dismissal through on_close.
        cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke {
                modifiers: Modifiers::default(),
                key: "escape".into(),
                key_char: None,
            },
            is_held: false,
            prefer_character_input: false,
        });
        assert!(closed.get(), "Escape must call on_close");

        // Closing restores focus to the saved background element.
        open.set(false);
        draw(&mut cx);
        cx.update(|_, _| {});
        draw(&mut cx);
        cx.update(|window, _cx| {
            assert!(
                outside.is_focused(window),
                "closing must restore focus to the previously focused element"
            );
        });
    }
}
