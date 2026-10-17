//! Scientific / math plots.
//!
//! Unlike the business [`chart`](crate::chart) widgets (bar/line/pie),
//! [`Plot`] draws a Cartesian coordinate system: labelled axes with tick
//! marks, a light grid, and one or more scatter series rendered in a fixed
//! palette. Data is auto-scaled to the plotting area (with an explicit
//! domain override), so arbitrary math/sample data lands in frame.

use gpui::{hsla, px, Hsla, IntoElement, Styled};
use vgui::{canvas_element, __var_color};

const MARGIN_L: f32 = 40.0;
const MARGIN_R: f32 = 12.0;
const MARGIN_T: f32 = 14.0;
const MARGIN_B: f32 = 28.0;
const TICKS: usize = 5;

fn palette(i: usize) -> Hsla {
    const HUES: [f32; 6] = [0.58, 0.08, 0.33, 0.82, 0.11, 0.75];
    hsla(HUES[i % HUES.len()], 0.65, 0.5, 1.0)
}

/// One named scatter series.
#[derive(Clone)]
pub struct PlotSeries {
    /// Legend label.
    pub label: String,
    /// `(x, y)` samples in data coordinates.
    pub points: Vec<(f32, f32)>,
}

impl PlotSeries {
    /// Create a series from samples.
    pub fn new(label: impl Into<String>, points: Vec<(f32, f32)>) -> Self {
        Self {
            label: label.into(),
            points,
        }
    }
}

/// A Cartesian scatter plot configuration.
pub struct Plot {
    width: f32,
    height: f32,
    series: Vec<PlotSeries>,
    x_label: Option<String>,
    y_label: Option<String>,
    /// Optional fixed `(min, max)` data window; otherwise auto-scaled.
    domain: Option<((f32, f32), (f32, f32))>,
}

impl Plot {
    /// Start an empty plot of the given canvas size.
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            series: Vec::new(),
            x_label: None,
            y_label: None,
            domain: None,
        }
    }

    /// Add a scatter series.
    pub fn series(mut self, series: PlotSeries) -> Self {
        self.series.push(series);
        self
    }

    /// Set the horizontal axis title.
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the vertical axis title.
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Pin the visible data window `((xmin, xmax), (ymin, ymax))`.
    pub fn domain(mut self, x: (f32, f32), y: (f32, f32)) -> Self {
        self.domain = Some((x, y));
        self
    }
}

fn bounds(series: &[PlotSeries]) -> ((f32, f32), (f32, f32)) {
    let mut xmin = f32::INFINITY;
    let mut xmax = f32::NEG_INFINITY;
    let mut ymin = f32::INFINITY;
    let mut ymax = f32::NEG_INFINITY;
    for s in series {
        for &(x, y) in &s.points {
            xmin = xmin.min(x);
            xmax = xmax.max(x);
            ymin = ymin.min(y);
            ymax = ymax.max(y);
        }
    }
    if !xmin.is_finite() {
        xmin = 0.0;
        xmax = 1.0;
        ymin = 0.0;
        ymax = 1.0;
    }
    if (xmax - xmin).abs() < f32::EPSILON {
        xmax = xmin + 1.0;
    }
    if (ymax - ymin).abs() < f32::EPSILON {
        ymax = ymin + 1.0;
    }
    ((xmin, xmax), (ymin, ymax))
}

fn nice_ticks(min: f32, max: f32) -> Vec<f32> {
    (0..=TICKS)
        .map(|i| min + (max - min) * (i as f32 / TICKS as f32))
        .collect()
}

/// Render a [`Plot`] to an animated-free canvas element.
pub fn plot(plot: Plot) -> gpui::AnyElement {
    let Plot {
        width,
        height,
        series,
        x_label,
        y_label,
        domain,
    } = plot;

    let ((xmin, xmax), (ymin, ymax)) = domain.unwrap_or_else(|| bounds(&series));

    let plot_w = (width - MARGIN_L - MARGIN_R).max(1.0);
    let plot_h = (height - MARGIN_T - MARGIN_B).max(1.0);

    let axis = __var_color("--foreground", Some(hsla(0.0, 0.0, 0.2, 1.0)));
    let grid_c = __var_color("--border", Some(hsla(0.0, 0.0, 0.85, 1.0)));
    let tick_c = __var_color("--muted-foreground", Some(hsla(0.0, 0.0, 0.45, 1.0)));

    canvas_element(move |ctx| {
        let to_x = |x: f32| MARGIN_L + (x - xmin) / (xmax - xmin) * plot_w;
        // Canvas y grows downward; flip data y.
        let to_y = |y: f32| MARGIN_T + (1.0 - (y - ymin) / (ymax - ymin)) * plot_h;

        // Grid + tick labels.
        ctx.set_stroke_style(grid_c);
        ctx.set_line_width(1.0);
        ctx.set_font("10px sans-serif");
        for (i, &v) in nice_ticks(xmin, xmax).iter().enumerate() {
            let x = to_x(v);
            ctx.begin_path();
            ctx.move_to(x, MARGIN_T);
            ctx.line_to(x, MARGIN_T + plot_h);
            ctx.stroke();
            if i % 2 == 0 {
                ctx.set_fill_style(tick_c);
                ctx.fill_text(&format!("{:.1}", v), x - 8.0, MARGIN_T + plot_h + 14.0);
            }
        }
        for (i, &v) in nice_ticks(ymin, ymax).iter().enumerate() {
            let y = to_y(v);
            ctx.begin_path();
            ctx.move_to(MARGIN_L, y);
            ctx.line_to(MARGIN_L + plot_w, y);
            ctx.stroke();
            if i % 2 == 0 {
                ctx.set_fill_style(tick_c);
                ctx.fill_text(&format!("{:.1}", v), 4.0, y + 3.0);
            }
        }

        // Axis frame.
        ctx.set_stroke_style(axis);
        ctx.set_line_width(1.2);
        ctx.stroke_rect(MARGIN_L, MARGIN_T, plot_w, plot_h);

        // Axis titles.
        ctx.set_fill_style(tick_c);
        if let Some(label) = &x_label {
            ctx.fill_text(label, MARGIN_L + plot_w / 2.0 - 20.0, height - 4.0);
        }
        if let Some(label) = &y_label {
            ctx.save();
            ctx.translate(10.0, MARGIN_T + plot_h / 2.0 + 20.0);
            ctx.fill_text(label, 0.0, 0.0);
            ctx.restore();
        }

        // Series points as filled circles.
        for (si, s) in series.iter().enumerate() {
            let color = palette(si);
            ctx.set_fill_style(color);
            for &(x, y) in &s.points {
                let px = to_x(x);
                let py = to_y(y);
                ctx.begin_path();
                ctx.arc(px, py, 3.0, 0.0, std::f32::consts::TAU, false);
                ctx.fill();
            }
        }
    })
    .w(px(width))
    .h(px(height))
    .into_any_element()
}
