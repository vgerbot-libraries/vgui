//! Minimal data-visualization charts drawn on a [`vgui::canvas_element`].
//!
//! Three chart kinds are provided — [`bar_chart`], [`line_chart`] and
//! [`pie_chart`] — all deterministic and dependency-free. Each takes a fixed
//! pixel size and normalized series; axes/grid are drawn inside the plot area.

use gpui::{px, hsla, Hsla, IntoElement, Styled};
use vgui::{canvas_element, color, Context2D};

const PAD_L: f32 = 34.0;
const PAD_R: f32 = 10.0;
const PAD_T: f32 = 10.0;
const PAD_B: f32 = 22.0;
const GRID: &str = "rgba(128,128,128,0.18)";
const AXIS: &str = "rgba(128,128,128,0.7)";

/// A fixed-size canvas element of a given width/height.
fn sized(paint: impl 'static + FnOnce(&mut Context2D), w: f32, h: f32) -> gpui::AnyElement {
    canvas_element(paint).w(px(w)).h(px(h)).into_any_element()
}

fn series_bounds(values: &[f64]) -> (f64, f64) {
    let mut lo = 0.0f64;
    let mut hi = 0.0f64;
    for &v in values {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if (hi - lo).abs() < f64::EPSILON {
        hi = lo + 1.0;
    }
    (lo, hi)
}

/// Draw horizontal grid lines with a right-side value axis.
fn draw_grid(ctx: &mut Context2D, w: f32, h: f32, lo: f64, hi: f64) {
    let plot_h = h - PAD_T - PAD_B;
    ctx.set_font("10px sans-serif");
    for r in 0..=4 {
        let gy = PAD_T + plot_h * (r as f32 / 4.0);
        ctx.set_stroke_style(color(GRID));
        ctx.set_line_width(1.0);
        ctx.begin_path();
        ctx.move_to(PAD_L, gy);
        ctx.line_to(w - PAD_R, gy);
        ctx.stroke();

        let value = hi - (hi - lo) * (r as f64 / 4.0);
        ctx.set_fill_style(color(AXIS));
        ctx.fill_text(&format!("{:.0}", value), 2.0, gy + 3.0);
    }
}

/// Vertical bar chart of `values`.
pub fn bar_chart(values: Vec<f64>, width: f32, height: f32) -> gpui::AnyElement {
    let (lo, hi) = series_bounds(&values);
    let n = values.len().max(1) as f32;
    let plot_w = width - PAD_L - PAD_R;
    let plot_h = height - PAD_T - PAD_B;
    let slot = plot_w / n;
    let bar_w = (slot * 0.6).max(2.0);

    sized(
        move |ctx| {
            draw_grid(ctx, width, height, lo, hi);
            let primary = vgui::__var_color("--primary", Some(hsla(0.58, 0.7, 0.5, 1.0)));
            ctx.set_fill_style(primary);
            for (i, &v) in values.iter().enumerate() {
                let ratio = ((v - lo) / (hi - lo)) as f32;
                let bh = (ratio * plot_h).max(1.0);
                let x = PAD_L + slot * i as f32 + (slot - bar_w) / 2.0;
                let y = PAD_T + plot_h - bh;
                ctx.fill_rect(x, y, bar_w, bh);
            }
        },
        width,
        height,
    )
}

/// Line chart connecting `values`, with a translucent area fill.
pub fn line_chart(values: Vec<f64>, width: f32, height: f32) -> gpui::AnyElement {
    let (lo, hi) = series_bounds(&values);
    let n = values.len().max(2) as f32;
    let plot_w = width - PAD_L - PAD_R;
    let plot_h = height - PAD_T - PAD_B;
    let x = move |i: f32| PAD_L + plot_w * (i / (n - 1.0));
    let y = move |v: f64| PAD_T + plot_h * (1.0 - ((v - lo) / (hi - lo)) as f32);

    sized(
        move |ctx| {
            draw_grid(ctx, width, height, lo, hi);
            let primary = vgui::__var_color("--primary", Some(hsla(0.58, 0.7, 0.5, 1.0)));

            // Area fill.
            ctx.begin_path();
            ctx.move_to(x(0.0), y(values[0]));
            for (i, &v) in values.iter().enumerate().skip(1) {
                ctx.line_to(x(i as f32), y(v));
            }
            ctx.line_to(x(n - 1.0), PAD_T + plot_h);
            ctx.line_to(x(0.0), PAD_T + plot_h);
            ctx.close_path();
            ctx.set_global_alpha(0.16);
            ctx.set_fill_style(primary);
            ctx.fill();
            ctx.set_global_alpha(1.0);

            // Line.
            ctx.begin_path();
            ctx.move_to(x(0.0), y(values[0]));
            for (i, &v) in values.iter().enumerate().skip(1) {
                ctx.line_to(x(i as f32), y(v));
            }
            ctx.set_stroke_style(primary);
            ctx.set_line_width(1.8);
            ctx.stroke();
        },
        width,
        height,
    )
}

/// One pie slice `(label, value, color)`.
#[derive(Clone, Copy)]
pub struct PieSlice {
    /// Slice weight (must be non-negative).
    pub value: f64,
    /// Slice color.
    pub color: Hsla,
}

impl PieSlice {
    /// Build a slice.
    pub fn new(value: f64, color: Hsla) -> Self {
        Self { value, color }
    }
}

use std::f32::consts::PI;

/// Pie/donut chart of proportional `slices`.
pub fn pie_chart(slices: Vec<PieSlice>, size: f32) -> gpui::AnyElement {
    let total: f64 = slices.iter().map(|s| s.value.max(0.0)).sum::<f64>().max(f64::EPSILON);
    let cx = size / 2.0;
    let cy = size / 2.0;
    let radius = size / 2.0 - 4.0;

    sized(
        move |ctx| {
            let mut angle = -PI / 2.0;
            for slice in &slices {
                let frac = (slice.value.max(0.0) / total) as f32;
                if frac <= 0.0 {
                    continue;
                }
                let sweep = frac * 2.0 * PI;
                ctx.begin_path();
                ctx.move_to(cx, cy);
                ctx.arc(cx, cy, radius, angle, angle + sweep, false);
                ctx.close_path();
                ctx.set_fill_style(slice.color);
                ctx.fill();
                angle += sweep;
            }
        },
        size,
        size,
    )
}
