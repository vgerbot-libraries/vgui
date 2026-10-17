//! Cycling content carousel.
//!
//! Holds a ring of slide elements with an internal index signal. Prev/next
//! buttons and clickable dots move between slides (wrapping). When
//! `auto_advance_ms` is non-zero the carousel advances automatically on that
//! interval; manual navigation resets the schedule.
use gpui::{px, AnyElement, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window};
use vgui::{create_signal, use_interval, ReadSignal, WriteSignal};

/// A horizontally cycling carousel over `slides`.
///
/// - `slides` — the content ring; a single slide is shown as-is.
/// - `auto_advance_ms` — autoplay period; `0` disables autoplay.
pub fn carousel(slides: Vec<AnyElement>, auto_advance_ms: u64) -> gpui::AnyElement {
    let count = slides.len();
    let (index, set_index) = create_signal(0usize);
    let (delay, _set_delay) = create_signal(auto_advance_ms);

    // Autoplay: tick the index forward on the reactive interval. A zero delay
    // pauses the timer, so a static carousel never advances.
    if count > 1 {
        let idx_read = index.clone();
        let idx_write = set_index.clone();
        use_interval(
            move |cx| {
                let cur = idx_read.get();
                idx_write.set(cx, (cur + 1) % count);
            },
            delay,
        );
    } else {
        let _ = delay;
    }

    let current = if count == 0 { 0 } else { index.get() % count };

    let mut root = gpui::div()
        .id(("carousel", vgui::next_auto_id()))
        .relative()
        .w_full();

    if let Some(slide) = slides.into_iter().nth(current) {
        root = root.child(slide);
    }

    if count > 1 {
        // Prev / next arrow buttons.
        let prev = nav_arrow("\u{2039}", "carousel-prev", index.clone(), set_index.clone(), count, true);
        let next = nav_arrow("\u{203A}", "carousel-next", index.clone(), set_index.clone(), count, false);
        root = root.child(prev).child(next);

        // Clickable indicator dots.
        let mut dots = gpui::div()
            .absolute()
            .bottom_2()
            .left_0()
            .right_0()
            .flex()
            .flex_row()
            .justify_center()
            .gap_1();
        for i in 0..count {
            let dot_write = set_index.clone();
            let active = i == current;
            let mut dot = gpui::div()
                .id(("carousel-dot", vgui::next_auto_id()))
                .w(px(8.))
                .h(px(8.))
                .rounded_full()
                .cursor_pointer()
                .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                    dot_write.set(cx, i);
                });
            dot = if active {
                dot.bg(gpui::white())
            } else {
                dot.bg(gpui::hsla(0., 0., 1., 0.4))
            };
            dots = dots.child(dot);
        }
        root = root.child(dots);
    }

    root.into_any_element()
}

fn nav_arrow(
    glyph: &'static str,
    id: &'static str,
    index: ReadSignal<usize>,
    set_index: WriteSignal<usize>,
    count: usize,
    backwards: bool,
) -> AnyElement {
    let arrow = gpui::div()
        .id((id, vgui::next_auto_id()))
        .absolute()
        .top_1_2()
        .w(px(28.))
        .h(px(28.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .text_color(gpui::white())
        .bg(gpui::hsla(0., 0., 0., 0.35))
        .rounded_full();
    let arrow = if backwards { arrow.left_2() } else { arrow.right_2() };
    arrow
        .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
            let cur = index.get();
            let next = if backwards {
                (cur + count - 1) % count
            } else {
                (cur + 1) % count
            };
            set_index.set(cx, next);
        })
        .child(glyph)
        .into_any_element()
}
