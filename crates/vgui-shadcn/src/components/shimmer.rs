//! Animated shimmer loading surface.
//!
//! A placeholder block with a light band that sweeps across it on a loop,
//! signalling in-flight content. As with the spinner, gpui divs cannot
//! transform, so the surface and its moving highlight are painted on a
//! [`vgui::canvas_element`] and advanced by a reactive interval.

use gpui::{px, hsla, Hsla, IntoElement, Styled};
use vgui::{canvas_element, create_signal, use_interval, __var_color};

/// A rectangular shimmer placeholder.
///
/// - `width`/`height` — surface size in logical pixels.
/// - `period_ms` — full sweep duration; the highlight loops at this rate.
pub fn shimmer(width: f32, height: f32, period_ms: u64) -> gpui::AnyElement {
    let (t, set_t) = create_signal(0f32);
    let (delay, _set_delay) = create_signal(16u64);
    let tick = t.clone();
    use_interval(
        move |cx| {
            // Advance phase by one ~16ms frame; wrap over a full period.
            let step = 16.0 / (period_ms.max(1) as f32);
            set_t.set(cx, (tick.get() + step) % 1.0);
        },
        delay,
    );

    let phase = t.get();
    let track = __var_color("--muted", Some(hsla(0.0, 0.0, 0.9, 1.0)));
    let highlight: Hsla = hsla(0.0, 0.0, 1.0, 0.6);

    canvas_element(move |ctx| {
        // Base surface.
        ctx.set_fill_style(track);
        round_rect_path(ctx, 0.0, 0.0, width, height, height.min(8.0));
        ctx.fill();

        // Sweeping highlight: a vertical translucent band whose centre
        // crosses the surface once per period. Its extent is clamped inside
        // the surface (the canvas offers no clip path).
        let band_w = (width * 0.32).max(12.0);
        let centre = -band_w + phase * (width + 2.0 * band_w);
        let x0 = (centre - band_w / 2.0).clamp(0.0, width);
        let x1 = (centre + band_w / 2.0).clamp(0.0, width);
        if x1 > x0 {
            ctx.set_global_alpha(0.8);
            ctx.set_fill_style(highlight);
            ctx.fill_rect(x0, 0.0, x1 - x0, height);
            ctx.set_global_alpha(1.0);
        }
    })
    .w(px(width))
    .h(px(height))
    .into_any_element()
}

fn round_rect_path(ctx: &mut vgui::Context2D, x: f32, y: f32, w: f32, h: f32, r: f32) {
    let r = r.min(w / 2.0).min(h / 2.0);
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.line_to(x + w - r, y);
    ctx.quadratic_curve_to(x + w, y, x + w, y + r);
    ctx.line_to(x + w, y + h - r);
    ctx.quadratic_curve_to(x + w, y + h, x + w - r, y + h);
    ctx.line_to(x + r, y + h);
    ctx.quadratic_curve_to(x, y + h, x, y + h - r);
    ctx.line_to(x, y + r);
    ctx.quadratic_curve_to(x, y, x + r, y);
    ctx.close_path();
}
