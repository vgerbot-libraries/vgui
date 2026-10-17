//! Mobile text-selection handles.
//!
//! [`TouchSelection`] renders two caret handles over a notional character
//! track (e.g. a text line). Dragging either handle updates the selected
//! `[start, end]` index range, clamped and kept ordered; the range is reported
//! through `on_change`. This models the platform touch-selection affordance
//! (two carets with knobs) without a platform text input.
//!
//! Pointer handling mirrors the range input / resizable split: the whole
//! track is one interactive surface; a press grabs the nearer caret and
//! subsequent moves resize the selection even when the cursor leaves it.

use std::rc::Rc;

use gpui::{
    px, App, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, Styled, Window,
};
use vgui::{create_signal, next_auto_id};

/// Width in logical pixels per character on the track.
const CHAR_W: f32 = 9.0;
/// Pointer grab radius around a caret.
const HIT: f32 = 18.0;
const KNOB_Y: f32 = 20.0;

/// Stateful touch-selection affordance.
pub struct TouchSelection {
    len: usize,
    start: usize,
    end: usize,
}

impl TouchSelection {
    /// Selection over a string of `len` characters, initially selecting all.
    pub fn new(len: usize) -> Self {
        Self {
            len,
            start: 0,
            end: len,
        }
    }

    /// Set the initial selected range (clamped/ordered).
    pub fn range(mut self, start: usize, end: usize) -> Self {
        self.start = start.min(end).min(self.len);
        self.end = end.max(start).min(self.len);
        self
    }
}

fn index_at(x: f32, len: usize) -> usize {
    ((x / CHAR_W).round() as isize)
        .clamp(0, len as isize) as usize
}

/// Render the selection handles. `on_change(start, end, cx)` fires as either
/// handle is dragged.
pub fn touch_selection(
    config: TouchSelection,
    on_change: Rc<dyn Fn(usize, usize, &mut App)>,
) -> gpui::AnyElement {
    let TouchSelection { len, start, end } = config;
    let (start_sig, set_start) = create_signal(start);
    let (end_sig, set_end) = create_signal(end);
    // Which handle is being dragged, if any: 0 = start, 1 = end.
    let (drag, set_drag) = create_signal(Option::<u8>::None);

    let width = (len as f32) * CHAR_W + HIT;
    let track_w = (len as f32) * CHAR_W;

    let s0 = start_sig.get();
    let e0 = end_sig.get();
    let sel_x = s0 as f32 * CHAR_W;
    let sel_w = (e0 as f32 - s0 as f32).max(0.0) * CHAR_W;

    let down_start = start_sig.clone();
    let down_end = end_sig.clone();
    let move_drag = drag.clone();
    let move_start = start_sig.clone();
    let move_end = end_sig.clone();
    let up_set_drag = set_drag.clone();

    let root = gpui::div()
        .id(("touch-sel", next_auto_id()))
        .relative()
        .w(px(width))
        .h(px(40.))
        .on_mouse_down(
            MouseButton::Left,
            move |ev: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                let x = f32::from(ev.position.x);
                let sx = down_start.get() as f32 * CHAR_W;
                let ex = down_end.get() as f32 * CHAR_W;
                if (x - sx).abs() <= HIT {
                    set_drag.set(cx, Some(0));
                } else if (x - ex).abs() <= HIT {
                    set_drag.set(cx, Some(1));
                }
            },
        )
        .on_mouse_move(move |ev: &MouseMoveEvent, _w: &mut Window, cx: &mut App| {
            if ev.pressed_button != Some(MouseButton::Left) {
                return;
            }
            let Some(which) = move_drag.get() else {
                return;
            };
            let i = index_at(f32::from(ev.position.x), len);
            match which {
                0 => {
                    let next = i.min(move_end.get());
                    set_start.set(cx, next);
                    on_change(next, move_end.get(), cx);
                }
                _ => {
                    let next = i.max(move_start.get());
                    set_end.set(cx, next);
                    on_change(move_start.get(), next, cx);
                }
            }
        })
        .on_mouse_up(
            MouseButton::Left,
            move |_ev: &MouseUpEvent, _w: &mut Window, cx: &mut App| {
                up_set_drag.set(cx, None);
            },
        );

    // Track line.
    let track = gpui::div()
        .absolute()
        .left(px(0.))
        .top(px(KNOB_Y))
        .w(px(track_w))
        .h(px(3.));
    let track = vgui::css! {
        background: var(--border);
        border-radius: 2px;
    }
    .apply(track);

    // Selected-highlight band.
    let band = gpui::div()
        .absolute()
        .left(px(sel_x))
        .top(px(KNOB_Y - 3.))
        .w(px(sel_w))
        .h(px(9.));
    let band = vgui::css! {
        background: var(--primary);
        opacity: 0.35;
        border-radius: 3px;
    }
    .apply(band);

    let caret = |xi: usize| {
        let x = xi as f32 * CHAR_W;
        let stem = vgui::css! {
            background: var(--primary);
        }
        .apply(
            gpui::div()
                .absolute()
                .left(px(HIT / 2.0 - 1.0))
                .top(px(8.))
                .w(px(2.))
                .h(px(14.)),
        );
        let knob = vgui::css! {
            background: var(--primary);
            border-radius: 999px;
        }
        .apply(
            gpui::div()
                .absolute()
                .left(px(HIT / 2.0 - 4.0))
                .top(px(KNOB_Y))
                .w(px(8.))
                .h(px(8.)),
        );
        // Visual only; pointer logic lives on the root.
        gpui::div()
            .absolute()
            .left(px(x - HIT / 2.0))
            .top(px(0.))
            .w(px(HIT))
            .h(px(32.))
            .child(stem)
            .child(knob)
    };

    root.child(track)
        .child(band)
        .child(caret(s0))
        .child(caret(e0))
        .into_any_element()
}
