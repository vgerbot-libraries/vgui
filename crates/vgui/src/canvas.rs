//! Canvas 2D drawing context — a web-like `Context2D` backed by gpui paint
//! primitives.
//!
//! The [`canvas_element`] constructor produces a `gpui::Canvas` that can be
//! used inside `view!` via the `<canvas>` element.  Drawing is immediate-mode:
//! the `paint` closure runs every frame during gpui's paint phase.

use gpui::{
    px, App, Bounds, Canvas, FontStyle, FontWeight, Hsla, Pixels, Point,
    SharedString, TextAlign, Window, canvas, point,
};

// ── Transform2D ───────────────────────────────────────────────────────

/// 2-D affine transform stored as `| a c e | / | b d f | / | 0 0 1 |`.
#[derive(Clone, Copy, Debug)]
struct Transform2D {
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
}

impl Transform2D {
    const fn identity() -> Self {
        Transform2D { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: 0.0, f: 0.0 }
    }

    fn translate(tx: f32, ty: f32) -> Self {
        Transform2D { a: 1.0, b: 0.0, c: 0.0, d: 1.0, e: tx, f: ty }
    }

    fn rotate(angle: f32) -> Self {
        let (s, c) = angle.sin_cos();
        Transform2D { a: c, b: s, c: -s, d: c, e: 0.0, f: 0.0 }
    }

    fn scale(sx: f32, sy: f32) -> Self {
        Transform2D { a: sx, b: 0.0, c: 0.0, d: sy, e: 0.0, f: 0.0 }
    }

    /// Post-multiply: `self * other`.
    fn multiply(self, other: Transform2D) -> Transform2D {
        Transform2D {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    fn apply_point(&self, x: f32, y: f32) -> (f32, f32) {
        (self.a * x + self.c * y + self.e, self.b * x + self.d * y + self.f)
    }
}

// ── Canvas state ──────────────────────────────────────────────────────

#[derive(Clone, Debug)]
struct CanvasState {
    fill_style: Hsla,
    stroke_style: Hsla,
    line_width: f32,
    font: String,
    text_align: CanvasTextAlign,
    global_alpha: f32,
    transform: Transform2D,
}

impl Default for CanvasState {
    fn default() -> Self {
        CanvasState {
            fill_style: gpui::black(),
            stroke_style: gpui::black(),
            line_width: 1.0,
            font: String::from("10px sans-serif"),
            text_align: CanvasTextAlign::Left,
            global_alpha: 1.0,
            transform: Transform2D::identity(),
        }
    }
}

// ── Path commands ─────────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum PathCommand {
    MoveTo(f32, f32),
    LineTo(f32, f32),
    QuadraticCurveTo { cpx: f32, cpy: f32, x: f32, y: f32 },
    BezierCurveTo { cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32 },
    Arc { x: f32, y: f32, radius: f32, start: f32, end: f32, ccw: bool },
    ClosePath,
}

// ── Public types ──────────────────────────────────────────────────────

/// Text alignment for [`Context2D::fill_text`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CanvasTextAlign {
    Start,
    Left,
    Center,
    Right,
    End,
}

/// Metrics returned by [`Context2D::measure_text`].
pub struct TextMetrics {
    pub width: f32,
}

// ── Context2D ─────────────────────────────────────────────────────────

/// A web-like 2D drawing context backed by gpui paint primitives.
///
/// All coordinates are in canvas-local space; the current transform and the
/// element's bounds origin are applied automatically at paint time.
pub struct Context2D<'a> {
    bounds: Bounds<Pixels>,
    window: &'a mut Window,
    cx: &'a mut App,
    state: CanvasState,
    saved: Vec<CanvasState>,
    path: Vec<PathCommand>,
}

impl<'a> Context2D<'a> {
    fn new(bounds: Bounds<Pixels>, window: &'a mut Window, cx: &'a mut App) -> Self {
        let mut state = CanvasState::default();
        state.transform = Transform2D::translate(f32::from(bounds.origin.x), f32::from(bounds.origin.y));
        Context2D { bounds, window, cx, state, saved: Vec::new(), path: Vec::new() }
    }

    // ── State / styles ────────────────────────────────────────────────

    pub fn fill_style(&self) -> Hsla {
        self.state.fill_style
    }

    pub fn set_fill_style(&mut self, color: impl Into<Hsla>) {
        self.state.fill_style = color.into();
    }

    pub fn stroke_style(&self) -> Hsla {
        self.state.stroke_style
    }

    pub fn set_stroke_style(&mut self, color: impl Into<Hsla>) {
        self.state.stroke_style = color.into();
    }

    pub fn line_width(&self) -> f32 {
        self.state.line_width
    }

    pub fn set_line_width(&mut self, w: f32) {
        self.state.line_width = w;
    }

    pub fn font(&self) -> &str {
        &self.state.font
    }

    pub fn set_font(&mut self, font: impl Into<String>) {
        self.state.font = font.into();
    }

    pub fn text_align(&self) -> CanvasTextAlign {
        self.state.text_align
    }

    pub fn set_text_align(&mut self, align: CanvasTextAlign) {
        self.state.text_align = align;
    }

    pub fn global_alpha(&self) -> f32 {
        self.state.global_alpha
    }

    pub fn set_global_alpha(&mut self, a: f32) {
        self.state.global_alpha = a.clamp(0.0, 1.0);
    }

    // ── Transforms ────────────────────────────────────────────────────

    pub fn save(&mut self) {
        self.saved.push(self.state.clone());
    }

    pub fn restore(&mut self) {
        if let Some(state) = self.saved.pop() {
            self.state = state;
        }
    }

    pub fn translate(&mut self, x: f32, y: f32) {
        self.state.transform = self.state.transform.multiply(Transform2D::translate(x, y));
    }

    pub fn rotate(&mut self, angle_radians: f32) {
        self.state.transform = self.state.transform.multiply(Transform2D::rotate(angle_radians));
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.state.transform = self.state.transform.multiply(Transform2D::scale(sx, sy));
    }

    pub fn set_transform(&mut self, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) {
        let origin = Transform2D::translate(f32::from(self.bounds.origin.x), f32::from(self.bounds.origin.y));
        self.state.transform = origin.multiply(Transform2D { a, b, c, d, e, f });
    }

    pub fn reset_transform(&mut self) {
        self.state.transform =
            Transform2D::translate(f32::from(self.bounds.origin.x), f32::from(self.bounds.origin.y));
    }

    // ── Rectangles ────────────────────────────────────────────────────

    /// Fill a rectangle.  No-op if `w <= 0` or `h <= 0`.
    pub fn fill_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let corners = [
            (x, y),
            (x + w, y),
            (x + w, y + h),
            (x, y + h),
        ];
        let pts: Vec<Point<Pixels>> = corners
            .iter()
            .map(|&(cx, cy)| {
                let (tx, ty) = self.state.transform.apply_point(cx, cy);
                point(px(tx), px(ty))
            })
            .collect();
        let mut builder = gpui::PathBuilder::fill();
        builder.add_polygon(&pts, true);
        if let Ok(path) = builder.build() {
            let color = with_alpha(self.state.fill_style, self.state.global_alpha);
            self.window.paint_path(path, color);
        }
    }

    /// Stroke a rectangle outline.  No-op if `w <= 0` or `h <= 0`.
    pub fn stroke_rect(&mut self, x: f32, y: f32, w: f32, h: f32) {
        if w <= 0.0 || h <= 0.0 {
            return;
        }
        let corners = [(x, y), (x + w, y), (x + w, y + h), (x, y + h)];
        let pts: Vec<Point<Pixels>> = corners
            .iter()
            .map(|&(cx, cy)| {
                let (tx, ty) = self.state.transform.apply_point(cx, cy);
                point(px(tx), px(ty))
            })
            .collect();
        let mut builder = gpui::PathBuilder::stroke(px(self.state.line_width));
        builder.add_polygon(&pts, true);
        if let Ok(path) = builder.build() {
            let color = with_alpha(self.state.stroke_style, self.state.global_alpha);
            self.window.paint_path(path, color);
        }
    }

    /// No-op in immediate mode.  gpui repaints from a blank canvas every
    /// frame, so there is nothing to clear.
    pub fn clear_rect(&mut self, _x: f32, _y: f32, _w: f32, _h: f32) {}

    // ── Path operations ───────────────────────────────────────────────

    pub fn begin_path(&mut self) {
        self.path.clear();
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.path.push(PathCommand::MoveTo(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.path.push(PathCommand::LineTo(x, y));
    }

    pub fn quadratic_curve_to(&mut self, cpx: f32, cpy: f32, x: f32, y: f32) {
        self.path.push(PathCommand::QuadraticCurveTo { cpx, cpy, x, y });
    }

    pub fn bezier_curve_to(&mut self, cp1x: f32, cp1y: f32, cp2x: f32, cp2y: f32, x: f32, y: f32) {
        self.path.push(PathCommand::BezierCurveTo { cp1x, cp1y, cp2x, cp2y, x, y });
    }

    /// Add an arc.  `anticlockwise` = `false` draws clockwise (the web default).
    pub fn arc(&mut self, x: f32, y: f32, radius: f32, start_angle: f32, end_angle: f32, anticlockwise: bool) {
        self.path.push(PathCommand::Arc {
            x,
            y,
            radius,
            start: start_angle,
            end: end_angle,
            ccw: anticlockwise,
        });
    }

    pub fn close_path(&mut self) {
        self.path.push(PathCommand::ClosePath);
    }

    /// Fill the current path.
    pub fn fill(&mut self) {
        let color = with_alpha(self.state.fill_style, self.state.global_alpha);
        if let Some(path) = self.build_path(gpui::PathBuilder::fill()) {
            self.window.paint_path(path, color);
        }
    }

    /// Stroke the current path.
    pub fn stroke(&mut self) {
        let color = with_alpha(self.state.stroke_style, self.state.global_alpha);
        let builder = gpui::PathBuilder::stroke(px(self.state.line_width));
        if let Some(path) = self.build_path(builder) {
            self.window.paint_path(path, color);
        }
    }

    /// Replay `self.path` into a `PathBuilder`, applying the current transform
    /// to every point and flattening arcs into line segments.
    fn build_path(&self, mut builder: gpui::PathBuilder) -> Option<gpui::Path<Pixels>> {
        let t = self.state.transform;
        let mut started = false;

        for cmd in &self.path {
            match *cmd {
                PathCommand::MoveTo(x, y) => {
                    let (tx, ty) = t.apply_point(x, y);
                    builder.move_to(point(px(tx), px(ty)));
                    started = true;
                }
                PathCommand::LineTo(x, y) => {
                    let (tx, ty) = t.apply_point(x, y);
                    if !started {
                        builder.move_to(point(px(tx), px(ty)));
                        started = true;
                    } else {
                        builder.line_to(point(px(tx), px(ty)));
                    }
                }
                PathCommand::QuadraticCurveTo { cpx, cpy, x, y } => {
                    let (tcx, tcy) = t.apply_point(cpx, cpy);
                    let (tx, ty) = t.apply_point(x, y);
                    if !started {
                        builder.move_to(point(px(tcx), px(tcy)));
                        started = true;
                    }
                    builder.curve_to(point(px(tx), px(ty)), point(px(tcx), px(tcy)));
                }
                PathCommand::BezierCurveTo { cp1x, cp1y, cp2x, cp2y, x, y } => {
                    let (tc1x, tc1y) = t.apply_point(cp1x, cp1y);
                    let (tc2x, tc2y) = t.apply_point(cp2x, cp2y);
                    let (tx, ty) = t.apply_point(x, y);
                    if !started {
                        builder.move_to(point(px(tc1x), px(tc1y)));
                        started = true;
                    }
                    builder.cubic_bezier_to(
                        point(px(tx), px(ty)),
                        point(px(tc1x), px(tc1y)),
                        point(px(tc2x), px(tc2y)),
                    );
                }
                PathCommand::Arc { x, y, radius, start, end, ccw } => {
                    let (mut start, mut end) = (start, end);
                    if !ccw && end < start {
                        std::mem::swap(&mut start, &mut end);
                    }
                    let sweep = (end - start).abs();
                    let segments = ((sweep / (std::f32::consts::PI / 16.0)).ceil() as usize).max(2);
                    for i in 0..=segments {
                        let angle = start + (end - start) * (i as f32 / segments as f32);
                        let px_ = x + radius * angle.cos();
                        let py_ = y + radius * angle.sin();
                        let (tx, ty) = t.apply_point(px_, py_);
                        if i == 0 && !started {
                            builder.move_to(point(px(tx), px(ty)));
                            started = true;
                        } else {
                            builder.line_to(point(px(tx), px(ty)));
                        }
                    }
                }
                PathCommand::ClosePath => {
                    builder.close();
                }
            }
        }

        builder.build().ok()
    }

    // ── Text ──────────────────────────────────────────────────────────

    /// Draw text.  `y` is the text baseline, matching the web canvas spec.
    pub fn fill_text(&mut self, text: &str, x: f32, y: f32) {
        let (size, family, weight, style) = parse_font(&self.state.font);
        let color = with_alpha(self.state.fill_style, self.state.global_alpha);
        let run = gpui::TextRun {
            len: text.len(),
            font: gpui::Font {
                family: SharedString::from(family),
                features: Default::default(),
                fallbacks: None,
                weight,
                style,
            },
            color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = self.window.text_system().shape_line(
            SharedString::from(text),
            px(size),
            &[run],
            None,
        );
        let width = f32::from(line.width());
        let ascent = f32::from(line.ascent);
        let descent = f32::from(line.descent);
        let line_height = ascent + descent;

        let origin_x = match self.state.text_align {
            CanvasTextAlign::Left | CanvasTextAlign::Start => x,
            CanvasTextAlign::Center => x - width / 2.0,
            CanvasTextAlign::Right | CanvasTextAlign::End => x - width,
        };
        // y is the baseline; origin.y is the top of the line.
        let (tx, ty) = self.state.transform.apply_point(origin_x, y - ascent);
        let origin = point(px(tx), px(ty));
        let _ = line.paint(origin, px(line_height), TextAlign::Left, None, self.window, self.cx);
    }

    /// Not supported — gpui has no text-outline API.  Method exists for API
    /// compatibility but is a no-op.
    pub fn stroke_text(&mut self, _text: &str, _x: f32, _y: f32) {}

    /// Measure text using the current font.
    pub fn measure_text(&self, text: &str) -> TextMetrics {
        let (size, family, weight, style) = parse_font(&self.state.font);
        let run = gpui::TextRun {
            len: text.len(),
            font: gpui::Font {
                family: SharedString::from(family),
                features: Default::default(),
                fallbacks: None,
                weight,
                style,
            },
            color: self.state.fill_style,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let line = self.window.text_system().shape_line(
            SharedString::from(text),
            px(size),
            &[run],
            None,
        );
        TextMetrics { width: f32::from(line.width()) }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────

fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla { a: color.a * alpha, ..color }
}

/// Parse a CSS font shorthand string into `(size_px, family, weight, style)`.
fn parse_font(font: &str) -> (f32, String, FontWeight, FontStyle) {
    let tokens: Vec<&str> = font.split_whitespace().collect();
    let mut size = 10.0_f32;
    let mut weight = FontWeight::NORMAL;
    let mut style = FontStyle::Normal;
    let mut family_parts: Vec<&str> = Vec::new();

    for tok in &tokens {
        if tok.eq_ignore_ascii_case("italic") {
            style = FontStyle::Italic;
        } else if tok.eq_ignore_ascii_case("oblique") {
            style = FontStyle::Oblique;
        } else if tok.eq_ignore_ascii_case("bold") {
            weight = FontWeight::BOLD;
        } else if let Some(px_str) = tok.strip_suffix("px").or_else(|| tok.strip_suffix("PX")) {
            if let Ok(v) = px_str.parse::<f32>() {
                size = v;
            }
        } else if let Some(pt_str) = tok.strip_suffix("pt").or_else(|| tok.strip_suffix("PT")) {
            if let Ok(v) = pt_str.parse::<f32>() {
                size = v * 1.333;
            }
        } else {
            family_parts.push(tok);
        }
    }

    let family = if family_parts.is_empty() {
        String::from("sans-serif")
    } else {
        family_parts.join(" ")
    };

    (size, family, weight, style)
}

// ── canvas_element ────────────────────────────────────────────────────

/// Create a `<canvas>` element with a `paint` closure that receives a
/// [`Context2D`].
///
/// The closure is called every frame during gpui's paint phase (immediate
/// mode).  The returned `Canvas` implements `Styled` and `IntoElement`.
pub fn canvas_element(paint: impl 'static + FnOnce(&mut Context2D)) -> Canvas<Bounds<Pixels>> {
    canvas(
        |bounds, _, _| bounds,
        move |bounds, _, window, cx| {
            let mut ctx = Context2D::new(bounds, window, cx);
            paint(&mut ctx);
        },
    )
}

// ── Runtime CSS color parser ──────────────────────────────────────────

/// Parse a CSS color string at runtime into an [`Hsla`].
///
/// Supports `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`,
/// `hsl()`, `hsla()`, named CSS colors, and `"transparent"`.  Unrecognized
/// strings fall back to black.
pub fn color(s: &str) -> Hsla {
    let s = s.trim();
    if s.is_empty() {
        return Hsla::default();
    }
    if let Some(hex) = s.strip_prefix('#') {
        return parse_hex(hex);
    }
    let lower = s.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("rgba(").and_then(|r| r.strip_suffix(')')) {
        return parse_rgba(rest);
    }
    if let Some(rest) = lower.strip_prefix("rgb(").and_then(|r| r.strip_suffix(')')) {
        return parse_rgb(rest);
    }
    if let Some(rest) = lower.strip_prefix("hsla(").and_then(|r| r.strip_suffix(')')) {
        return parse_hsla(rest);
    }
    if let Some(rest) = lower.strip_prefix("hsl(").and_then(|r| r.strip_suffix(')')) {
        return parse_hsl(rest);
    }
    named_color(&lower).unwrap_or_default()
}

fn parse_hex(hex: &str) -> Hsla {
    let hex: String = hex.chars().filter(|c| *c != '_').collect();
    let (val, has_alpha) = match hex.len() {
        3 => {
            let out: String = hex.chars().flat_map(|c| [c, c]).collect();
            match u32::from_str_radix(&out, 16) {
                Ok(v) => (v, false),
                Err(_) => return Hsla::default(),
            }
        }
        4 => {
            let out: String = hex.chars().flat_map(|c| [c, c]).collect();
            match u32::from_str_radix(&out, 16) {
                Ok(v) => (v, true),
                Err(_) => return Hsla::default(),
            }
        }
        6 => match u32::from_str_radix(&hex, 16) {
            Ok(v) => (v, false),
            Err(_) => return Hsla::default(),
        },
        8 => match u32::from_str_radix(&hex, 16) {
            Ok(v) => (v, true),
            Err(_) => return Hsla::default(),
        },
        _ => return Hsla::default(),
    };
    if has_alpha {
        Hsla::from(gpui::rgba(val))
    } else {
        Hsla::from(gpui::rgb(val))
    }
}

fn parse_nums(s: &str) -> Vec<f32> {
    s.split([',', ' '])
        .filter(|p| !p.trim().is_empty())
        .filter_map(|p| p.trim().parse::<f32>().ok())
        .collect()
}

fn parse_rgb(s: &str) -> Hsla {
    let nums = parse_nums(s);
    if nums.len() < 3 {
        return Hsla::default();
    }
    let [r, g, b] = [nums[0], nums[1], nums[2]];
    rgba_from_components(r, g, b, 1.0)
}

fn parse_rgba(s: &str) -> Hsla {
    let nums = parse_nums(s);
    if nums.len() < 4 {
        return Hsla::default();
    }
    let [r, g, b, a] = [nums[0], nums[1], nums[2], nums[3]];
    rgba_from_components(r, g, b, a)
}

fn rgba_from_components(r: f32, g: f32, b: f32, a: f32) -> Hsla {
    let hex = ((r.round() as u32 & 0xFF) << 24)
        | ((g.round() as u32 & 0xFF) << 16)
        | ((b.round() as u32 & 0xFF) << 8)
        | ((a * 255.0).round() as u32 & 0xFF);
    Hsla::from(gpui::rgba(hex))
}

fn parse_hsl(s: &str) -> Hsla {
    let nums = parse_nums(s);
    if nums.len() < 3 {
        return Hsla::default();
    }
    hsla_from_components(nums[0], nums[1], nums[2], 1.0)
}

fn parse_hsla(s: &str) -> Hsla {
    let nums = parse_nums(s);
    if nums.len() < 4 {
        return Hsla::default();
    }
    hsla_from_components(nums[0], nums[1], nums[2], nums[3])
}

fn hsla_from_components(h: f32, s: f32, l: f32, a: f32) -> Hsla {
    Hsla { h: (h % 360.0) / 360.0, s: s / 100.0, l: l / 100.0, a }
}

/// Look up a CSS named color.  Returns `None` for unknown names.
fn named_color(name: &str) -> Option<Hsla> {
    // `transparent` — fully transparent black.
    if name == "transparent" {
        return Some(gpui::transparent_black());
    }
    // Returns `(hex, has_alpha)` for the named color.
    let (hex, has_alpha) = named_hex(name)?;
    if has_alpha {
        Some(Hsla::from(gpui::rgba(hex)))
    } else {
        Some(Hsla::from(gpui::rgb(hex)))
    }
}

/// CSS3 named color → `(hex, has_alpha)`.
#[rustfmt::skip]
fn named_hex(name: &str) -> Option<(u32, bool)> {
    let hex = match name {
        "aliceblue" => 0xf0f8ff, "antiquewhite" => 0xfaebd7, "aqua" => 0x00ffff,
        "aquamarine" => 0x7fffd4, "azure" => 0xf0ffff, "beige" => 0xf5f5dc,
        "bisque" => 0xffe4c4, "black" => 0x000000, "blanchedalmond" => 0xffebcd,
        "blue" => 0x0000ff, "blueviolet" => 0x8a2be2, "brown" => 0xa52a2a,
        "burlywood" => 0xdeb887, "cadetblue" => 0x5f9ea0, "chartreuse" => 0x7fff00,
        "chocolate" => 0xd2691e, "coral" => 0xff7f50, "cornflowerblue" => 0x6495ed,
        "cornsilk" => 0xfff8dc, "crimson" => 0xdc143c, "cyan" => 0x00ffff,
        "darkblue" => 0x00008b, "darkcyan" => 0x008b8b, "darkgoldenrod" => 0xb8860b,
        "darkgray" => 0xa9a9a9, "darkgrey" => 0xa9a9a9, "darkgreen" => 0x006400,
        "darkkhaki" => 0xbdb76b, "darkmagenta" => 0x8b008b, "darkolivegreen" => 0x556b2f,
        "darkorange" => 0xff8c00, "darkorchid" => 0x9932cc, "darkred" => 0x8b0000,
        "darksalmon" => 0xe9967a, "darkseagreen" => 0x8fbc8f, "darkslateblue" => 0x483d8b,
        "darkslategray" => 0x2f4f4f, "darkslategrey" => 0x2f4f4f, "darkturquoise" => 0x00ced1,
        "darkviolet" => 0x9400d3, "deeppink" => 0xff1493, "deepskyblue" => 0x00bfff,
        "dimgray" => 0x696969, "dimgrey" => 0x696969, "dodgerblue" => 0x1e90ff,
        "firebrick" => 0xb22222, "floralwhite" => 0xfffaf0, "forestgreen" => 0x228b22,
        "fuchsia" => 0xff00ff, "gainsboro" => 0xdcdcdc, "ghostwhite" => 0xf8f8ff,
        "gold" => 0xffd700, "goldenrod" => 0xdaa520, "gray" => 0x808080,
        "grey" => 0x808080, "green" => 0x008000, "greenyellow" => 0xadff2f,
        "honeydew" => 0xf0fff0, "hotpink" => 0xff69b4, "indianred" => 0xcd5c5c,
        "indigo" => 0x4b0082, "ivory" => 0xfffff0, "khaki" => 0xf0e68c,
        "lavender" => 0xe6e6fa, "lavenderblush" => 0xfff0f5, "lawngreen" => 0x7cfc00,
        "lemonchiffon" => 0xfffacd, "lightblue" => 0xadd8e6, "lightcoral" => 0xf08080,
        "lightcyan" => 0xe0ffff, "lightgoldenrodyellow" => 0xfafad2, "lightgray" => 0xd3d3d3,
        "lightgrey" => 0xd3d3d3, "lightgreen" => 0x90ee90, "lightpink" => 0xffb6c1,
        "lightsalmon" => 0xffa07a, "lightseagreen" => 0x20b2aa, "lightskyblue" => 0x87cefa,
        "lightslategray" => 0x778899, "lightslategrey" => 0x778899, "lightsteelblue" => 0xb0c4de,
        "lightyellow" => 0xffffe0, "lime" => 0x00ff00, "limegreen" => 0x32cd32,
        "linen" => 0xfaf0e6, "magenta" => 0xff00ff, "maroon" => 0x800000,
        "mediumaquamarine" => 0x66cdaa, "mediumblue" => 0x0000cd, "mediumorchid" => 0xba55d3,
        "mediumpurple" => 0x9370db, "mediumseagreen" => 0x3cb371, "mediumslateblue" => 0x7b68ee,
        "mediumspringgreen" => 0x00fa9a, "mediumturquoise" => 0x48d1cc, "mediumvioletred" => 0xc71585,
        "midnightblue" => 0x191970, "mintcream" => 0xf5fffa, "mistyrose" => 0xffe4e1,
        "moccasin" => 0xffe4b5, "navajowhite" => 0xffdead, "navy" => 0x000080,
        "oldlace" => 0xfdf5e6, "olive" => 0x808000, "olivedrab" => 0x6b8e23,
        "orange" => 0xffa500, "orangered" => 0xff4500, "orchid" => 0xda70d6,
        "palegoldenrod" => 0xeee8aa, "palegreen" => 0x98fb98, "paleturquoise" => 0xafeeee,
        "palevioletred" => 0xdb7093, "papayawhip" => 0xffefd5, "peachpuff" => 0xffdab9,
        "peru" => 0xcd853f, "pink" => 0xffc0cb, "plum" => 0xdda0dd,
        "powderblue" => 0xb0e0e6, "purple" => 0x800080, "rebeccapurple" => 0x663399,
        "red" => 0xff0000, "rosybrown" => 0xbc8f8f, "royalblue" => 0x4169e1,
        "saddlebrown" => 0x8b4513, "salmon" => 0xfa8072, "sandybrown" => 0xf4a460,
        "seagreen" => 0x2e8b57, "seashell" => 0xfff5ee, "sienna" => 0xa0522d,
        "silver" => 0xc0c0c0, "skyblue" => 0x87ceeb, "slateblue" => 0x6a5acd,
        "slategray" => 0x708090, "slategrey" => 0x708090, "snow" => 0xfffafa,
        "springgreen" => 0x00ff7f, "steelblue" => 0x4682b4, "tan" => 0xd2b48c,
        "teal" => 0x008080, "thistle" => 0xd8bfd8, "tomato" => 0xff6347,
        "turquoise" => 0x40e0d0, "violet" => 0xee82ee, "wheat" => 0xf5deb3,
        "white" => 0xffffff, "whitesmoke" => 0xf5f5f5, "yellow" => 0xffff00,
        "yellowgreen" => 0x9acd32,
        _ => return None,
    };
    Some((hex, false))
}
