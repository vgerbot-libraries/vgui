//! Variable-height virtualised lists.
//!
//! [`v_virtual_list`] renders only the rows intersecting a fixed-height
//! viewport, offset by spacers so the scroll track reflects the full content
//! height. Row heights come from a [`VirtualListState`] (uniform estimates by
//! default, per-item measurements supported), so unlike gpui's fixed-row
//! `uniform_list` the rows may differ in height. Wheel events move a scroll
//! offset clamped to the content; the window is recomputed each render from
//! [`VirtualListState::visible_range`].

use std::rc::Rc;

use gpui::{
    px, AnyElement, App, InteractiveElement, IntoElement, ParentElement, ScrollDelta,
    ScrollWheelEvent, Styled, Window,
};
use vgui::{create_signal, next_auto_id, VirtualListState};

/// A vertically scrolling, variable-height virtual list.
///
/// - `count` rows, each assumed `row_height` px unless later measured.
/// - The viewport is `viewport_height` px tall and clips overflow.
/// - `make_row(index)` builds the element for one row; it is called only for
///   the visible window (plus overscan).
pub fn v_virtual_list(
    count: usize,
    row_height: f32,
    viewport_height: f32,
    make_row: Rc<dyn Fn(usize) -> AnyElement>,
) -> AnyElement {
    virtual_list(count, row_height, viewport_height, make_row, false)
}

/// A horizontally scrolling, variable-width virtual list. Widths/windowing
/// use the same per-item measurement model rotated across the x axis.
pub fn h_virtual_list(
    count: usize,
    item_width: f32,
    viewport_width: f32,
    make_item: Rc<dyn Fn(usize) -> AnyElement>,
) -> AnyElement {
    virtual_list(count, item_width, viewport_width, make_item, true)
}

fn virtual_list(
    count: usize,
    item_extent: f32,
    viewport: f32,
    make_item: Rc<dyn Fn(usize) -> AnyElement>,
    horizontal: bool,
) -> AnyElement {
    let state = VirtualListState::new(count, item_extent).with_overscan(item_extent);
    let (scroll, set_scroll) = create_signal(0.0_f32);

    let offset = scroll.get();
    let total = state.total_height();
    let max_offset = (total - viewport).max(0.0);
    let clamped = offset.clamp(0.0, max_offset);
    let (first, last) = state.visible_range(clamped, viewport);
    let before = state.top_spacer(first);
    let after = state.bottom_spacer(last.saturating_sub(1));

    let read_scroll = scroll.clone();
    let set_scroll_on_wheel = set_scroll.clone();
    let total_for_wheel = total;
    let viewport_for_wheel = viewport;
    let horiz = horizontal;

    let mut viewport_el = gpui::div()
        .id(("virtual-list", next_auto_id()))
        .w(if horizontal { px(viewport) } else { px(320.) })
        .h(if horizontal { px(120.) } else { px(viewport) })
        .overflow_hidden()
        .on_scroll_wheel(move |ev: &ScrollWheelEvent, _w: &mut Window, cx: &mut App| {
            let dy = match &ev.delta {
                ScrollDelta::Pixels(p) => {
                    if horiz {
                        f32::from(p.x)
                    } else {
                        f32::from(p.y)
                    }
                }
                ScrollDelta::Lines(p) => {
                    let lines = if horiz { p.x } else { p.y };
                    lines * item_extent
                }
            };
            let cur = read_scroll.get();
            let max = (total_for_wheel - viewport_for_wheel).max(0.0);
            set_scroll_on_wheel.set(cx, (cur + dy).clamp(0.0, max));
        });

    if horizontal {
        let mut track = gpui::div().flex().flex_row();
        if before > 0.0 {
            track = track.child(gpui::div().w(px(before)).h(px(1.)));
        }
        for i in first..last {
            track = track.child(make_item(i));
        }
        if after > 0.0 {
            track = track.child(gpui::div().w(px(after)).h(px(1.)));
        }
        viewport_el = viewport_el.child(track);
    } else {
        let mut track = gpui::div().flex().flex_col();
        if before > 0.0 {
            track = track.child(gpui::div().w(px(1.)).h(px(before)));
        }
        for i in first..last {
            track = track.child(make_item(i));
        }
        if after > 0.0 {
            track = track.child(gpui::div().w(px(1.)).h(px(after)));
        }
        viewport_el = viewport_el.child(track);
    }

    vgui::css! {
        background: var(--card);
        border-width: 1px; border-style: solid; border-color: var(--border);
        border-radius: var(--radius);
    }
    .apply(viewport_el)
    .into_any_element()
}
