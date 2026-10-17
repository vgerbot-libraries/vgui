//! Canvas price chart: a filled area + line over a grid, with a last-price
//! marker and a right-hand price axis.
//!
//! The canvas is a fixed-size immediate-mode surface (vgui's `<canvas>` does
//! not expose laid-out bounds to `paint`), so the chart renders into a
//! constant logical size.

use gpui::IntoElement;
use vgui::prelude::*;

use crate::market::{pct, Instrument};

pub const CHART_W: f32 = 760.0;
pub const CHART_H: f32 = 320.0;

const PAD_L: f32 = 8.0;
const PAD_R: f32 = 74.0;
const PAD_T: f32 = 14.0;
const PAD_B: f32 = 22.0;

// Longbridge / Asian-market convention: red up, green down.
const UP: &str = "#f23645";
const DOWN: &str = "#2ebd85";
const GRID: &str = "#262936";
const AXIS: &str = "#7c8295";
const PANEL_BG: &str = "#13151c";

/// Render the price chart for `inst`, showing the last `window` points.
pub fn price_chart(inst: &Instrument, window: usize) -> gpui::AnyElement {
    let series: Vec<f64> = inst
        .history
        .iter()
        .rev()
        .take(window)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    let up = inst.change_pct() >= 0.0;
    let stroke = if up { UP } else { DOWN };
    let label_bg = stroke;

    let n = series.len().max(2) as f32;
    let (lo, hi) = series.iter().fold((f64::MAX, f64::MIN), |(lo, hi), &p| {
        (lo.min(p), hi.max(p))
    });
    let span_raw = (hi - lo).max(inst.price * 0.001);
    let lo = lo - span_raw * 0.08;
    let hi = hi + span_raw * 0.08;
    let span = hi - lo;

    let plot_w = CHART_W - PAD_L - PAD_R;
    let plot_h = CHART_H - PAD_T - PAD_B;

    let x = move |i: f32| PAD_L + plot_w * (i / (n - 1.0));
    let y = move |p: f64| PAD_T + plot_h * (1.0 - ((p - lo) / span) as f32);

    let last = inst.price;
    let last_y = y(last);

    canvas_element(move |ctx: &mut Context2D| {
        // Panel background.
        ctx.set_fill_style(color(PANEL_BG));
        ctx.fill_rect(0.0, 0.0, CHART_W, CHART_H);

        // Horizontal grid + right-hand price axis.
        ctx.set_font("11px sans-serif");
        let rows = 5;
        for r in 0..=rows {
            let gy = PAD_T + plot_h * (r as f32 / rows as f32);
            ctx.set_stroke_style(color(GRID));
            ctx.set_line_width(1.0);
            ctx.begin_path();
            ctx.move_to(PAD_L, gy);
            ctx.line_to(PAD_L + plot_w, gy);
            ctx.stroke();

            let price = hi - span * (r as f64 / rows as f64);
            ctx.set_fill_style(color(AXIS));
            ctx.set_text_align(CanvasTextAlign::Left);
            ctx.fill_text(&format!("{:.2}", price), PAD_L + plot_w + 8.0, gy + 4.0);
        }

        // Filled area under the line.
        ctx.begin_path();
        ctx.move_to(x(0.0), y(series[0]));
        for (i, p) in series.iter().enumerate().skip(1) {
            ctx.line_to(x(i as f32), y(*p));
        }
        ctx.line_to(x(n - 1.0), PAD_T + plot_h);
        ctx.line_to(x(0.0), PAD_T + plot_h);
        ctx.close_path();
        ctx.set_global_alpha(0.16);
        ctx.set_fill_style(color(stroke));
        ctx.fill();
        ctx.set_global_alpha(1.0);

        // Price line.
        ctx.begin_path();
        ctx.move_to(x(0.0), y(series[0]));
        for (i, p) in series.iter().enumerate().skip(1) {
            ctx.line_to(x(i as f32), y(*p));
        }
        ctx.set_stroke_style(color(stroke));
        ctx.set_line_width(1.8);
        ctx.stroke();

        // Last-price dashed marker across the plot.
        ctx.set_stroke_style(color(stroke));
        ctx.set_line_width(1.0);
        ctx.set_global_alpha(0.7);
        ctx.begin_path();
        ctx.move_to(PAD_L, last_y);
        ctx.line_to(PAD_L + plot_w, last_y);
        ctx.stroke();
        ctx.set_global_alpha(1.0);

        // Last-price tag on the axis.
        ctx.set_fill_style(color(label_bg));
        ctx.fill_rect(PAD_L + plot_w + 2.0, last_y - 9.0, PAD_R - 6.0, 18.0);
        ctx.set_fill_style(color("#ffffff"));
        ctx.set_text_align(CanvasTextAlign::Center);
        ctx.fill_text(
            &format!("{:.2}", last),
            PAD_L + plot_w + 4.0 + (PAD_R - 6.0) / 2.0,
            last_y + 4.0,
        );
    })
    .w(gpui::px(CHART_W))
    .h(gpui::px(CHART_H))
    .rounded_lg()
    .into_any_element()
}

/// Compact percentage chip colored by direction (red up / green down).
pub fn change_chip(change: f64) -> gpui::AnyElement {
    let up = change >= 0.0;
    let bg = if up { UP } else { DOWN };
    css! {
        display: flex;
        align-items: center;
        padding: 2px 8px;
        border-radius: 6px;
        font-size: 12px;
        font-weight: 600;
        color: #ffffff;
    }
    .apply(gpui::div().bg(color(bg)).child(pct(change)))
    .into_any_element()
}
