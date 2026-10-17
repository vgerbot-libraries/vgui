//! Drag-resizable split panels.
//!
//! Two panels share a draggable divider. Pressing the divider anchors the
//! pointer; subsequent moves resize the first pane by the pointer delta
//! (clamped to a minimum), so the behaviour is independent of where the split
//! sits in the window. The second pane fills the remainder. Both a horizontal
//! ([`h_resizable`]) and a vertical ([`v_resizable`]) split are provided.

use gpui::{
    px, AnyElement, App, InteractiveElement, IntoElement, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use vgui::{create_signal, next_auto_id};

const MIN_PANE: f32 = 48.0;
const HANDLE: f32 = 6.0;

/// Horizontal split: `left` starts at `initial_left` px, `right` fills the rest.
pub fn h_resizable(
    left: AnyElement,
    right: AnyElement,
    initial_left: f32,
    total_width: f32,
) -> AnyElement {
    split(left, right, initial_left, total_width, true)
}

/// Vertical split: `top` starts at `initial_top` px, `bottom` fills the rest.
pub fn v_resizable(
    top: AnyElement,
    bottom: AnyElement,
    initial_top: f32,
    total_height: f32,
) -> AnyElement {
    split(top, bottom, initial_top, total_height, false)
}

fn split(
    first: AnyElement,
    second: AnyElement,
    initial: f32,
    total: f32,
    horizontal: bool,
) -> AnyElement {
    let (pane, set_pane) = create_signal(initial.max(MIN_PANE));
    // `None` when not dragging; otherwise the last pointer position on the
    // active axis, so each move contributes only its delta.
    let (anchor, set_anchor) = create_signal(Option::<f32>::None);

    let max_pane = (total - MIN_PANE).max(MIN_PANE);

    // The root spans the whole split and owns down/move/up (like gpui's
    // range input, whose large track captures the drag). A press begins a
    // resize only when it lands on the handle strip; subsequent moves resize
    // by pointer delta regardless of whether the cursor stays on the strip.
    let down_pane = pane.clone();
    let down_anchor = set_anchor.clone();
    let root_anchor = anchor.clone();
    let root_pane = pane.clone();
    let move_pane = set_pane.clone();
    let move_anchor = set_anchor.clone();
    let up_anchor = set_anchor.clone();

    let axis = move |ev_p: gpui::Point<gpui::Pixels>| -> f32 {
        if horizontal {
            f32::from(ev_p.x)
        } else {
            f32::from(ev_p.y)
        }
    };

    let mut root = gpui::div()
        .id(("resizable-root", next_auto_id()))
        .relative()
        .w(px(total))
        .flex()
        .on_mouse_down(
            MouseButton::Left,
            move |ev: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                let p = axis(ev.position);
                // Begin only on the handle strip around the current divider.
                if (p - down_pane.get()).abs() <= HANDLE {
                    down_anchor.set(cx, Some(p));
                }
            },
        )
        .on_mouse_move(move |ev: &MouseMoveEvent, _w: &mut Window, cx: &mut App| {
            if ev.pressed_button != Some(MouseButton::Left) {
                return;
            }
            let Some(last) = root_anchor.get() else { return };
            let p = axis(ev.position);
            let next = (root_pane.get() + (p - last)).clamp(MIN_PANE, max_pane);
            move_pane.set(cx, next);
            move_anchor.set(cx, Some(p));
        })
        .on_mouse_up(
            MouseButton::Left,
            move |_ev: &MouseUpEvent, _w: &mut Window, cx: &mut App| {
                up_anchor.set(cx, None);
            },
        );
    root = if horizontal {
        root.flex_row().h_full()
    } else {
        root.flex_col().w_full().h(px(total))
    };

    // First (resizable) pane.
    let mut first_box = gpui::div().overflow_hidden().flex_shrink_0();
    first_box = if horizontal {
        first_box.w(px(pane.get())).h_full()
    } else {
        first_box.h(px(pane.get())).w_full()
    };
    root = root.child(first_box.child(first));

    // Visual divider (pointer logic lives on the root strip).
    let mut handle = gpui::div().flex_shrink_0();
    handle = if horizontal {
        handle.w(px(HANDLE)).h_full().cursor_col_resize()
    } else {
        handle.h(px(HANDLE)).w_full().cursor_row_resize()
    };
    let handle = vgui::css! {
        background: var(--border);
    }
    .apply(handle);
    root = root.child(handle);

    // Second pane fills remaining space.
    let mut second_box = gpui::div().flex_1().overflow_hidden().min_w(px(0.));
    second_box = if horizontal {
        second_box.h_full()
    } else {
        second_box.w_full()
    };
    root = root.child(second_box.child(second));

    root.into_any_element()
}
