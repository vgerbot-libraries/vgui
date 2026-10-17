//! Rotating activity indicator.
//!
//! gpui divs cannot rotate, so the spinner is drawn on a [`canvas_element`]:
//! a partial arc (the "blade") rotated by an angle advanced on a reactive
//! interval. Reading the angle signal in the render scope repaints the canvas
//! on every tick.
use std::f32::consts::PI;

use gpui::{px, hsla, IntoElement, Styled};
use vgui::{canvas_element, color, create_signal, use_interval, __var_color};

/// A rotating spinner of `size` logical pixels.
///
/// `speed_ms` is the frame period (~16ms ≈ 60fps); a larger value turns more
/// slowly. The spinner advances 30° per tick.
pub fn spinner(size: f32, speed_ms: u64) -> gpui::AnyElement {
    let (angle, set_angle) = create_signal(0f32);
    let (delay, _set_delay) = create_signal(speed_ms.max(1));
    let tick_angle = angle.clone();
    use_interval(
        move |cx| {
            let next = (tick_angle.get() + PI / 6.0) % (2.0 * PI);
            set_angle.set(cx, next);
        },
        delay,
    );

    let start = angle.get();
    let half = size / 2.0;
    let radius = half - 2.0;
    let primary = __var_color("--primary", Some(hsla(0.58, 0.7, 0.5, 1.0)));

    canvas_element(move |ctx| {
        // Faint full track.
        ctx.set_stroke_style(color("rgba(128,128,128,0.25)"));
        ctx.set_line_width(2.5);
        ctx.begin_path();
        ctx.arc(half, half, radius, 0.0, 2.0 * PI, false);
        ctx.stroke();

        // Rotated foreground blade (a 90° arc starting at the current angle).
        ctx.translate(half, half);
        ctx.rotate(start);
        ctx.translate(-half, -half);
        ctx.set_stroke_style(primary);
        ctx.set_line_width(2.5);
        ctx.begin_path();
        ctx.arc(half, half, radius, 0.0, PI / 2.0, false);
        ctx.stroke();
    })
    .w(px(size))
    .h(px(size))
    .into_any_element()
}
