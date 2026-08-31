use gpui::{Display, IntoElement, ParentElement, Styled};

pub fn show(when: bool, then: impl IntoElement, fallback: impl IntoElement) -> gpui::AnyElement {
    if when {
        then.into_any_element()
    } else {
        fallback.into_any_element()
    }
}

pub fn show_when(when: bool, then: impl IntoElement) -> gpui::AnyElement {
    if when {
        then.into_any_element()
    } else {
        let mut el = gpui::div();
        el.style().display = Some(Display::None);
        el.into_any_element()
    }
}

pub fn for_each<T, E: IntoElement>(
    items: impl IntoIterator<Item = T>,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let mut parent = gpui::div();
    for (i, item) in items.into_iter().enumerate() {
        parent = parent.child(child(item, i));
    }
    parent.into_any_element()
}

pub fn for_each_or<T, E: IntoElement, F: IntoElement>(
    items: impl IntoIterator<Item = T>,
    fallback: F,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let mut parent = gpui::div();
    let mut n = 0;
    for (i, item) in items.into_iter().enumerate() {
        n += 1;
        parent = parent.child(child(item, i));
    }
    if n == 0 {
        fallback.into_any_element()
    } else {
        parent.into_any_element()
    }
}

/// Render a progress bar. `value` / `max` determines fill width.
pub fn progress(value: f64, max: f64) -> gpui::Div {
    let ratio = if max <= 0.0 { 0.0 } else { (value / max).clamp(0.0, 1.0) };
    let pct = ratio as f32;
    gpui::div()
        .w_full()
        .h(gpui::px(8.))
        .rounded(gpui::px(4.))
        .bg(gpui::hsla(0., 0., 0.85, 1.))
        .overflow_hidden()
        .child(
            gpui::div()
                .h_full()
                .w(gpui::relative(pct))
                .bg(gpui::hsla(0.6, 0.8, 0.5, 1.))
        )
}

/// Render a meter bar. Fill width is `value` between `min` and `max`.
/// When `low`, `high`, and `optimum` are all omitted the fill is the same
/// blue as [`progress`]. Otherwise values inside `[low.unwrap_or(min),
/// high.unwrap_or(max)]` are green and values outside that range are red.
pub fn meter(
    value: f64,
    min: f64,
    max: f64,
    low: Option<f64>,
    high: Option<f64>,
    optimum: Option<f64>,
) -> gpui::Div {
    let range = if max <= min { 1.0 } else { max - min };
    let pct = ((value - min) / range).clamp(0.0, 1.0) as f32;
    let good_low = low.unwrap_or(min);
    let good_high = high.unwrap_or(max);
    let fill = if low.is_none() && high.is_none() && optimum.is_none() {
        gpui::hsla(0.6, 0.8, 0.5, 1.)
    } else if value >= good_low && value <= good_high {
        gpui::hsla(0.33, 0.8, 0.45, 1.)
    } else {
        gpui::hsla(0., 0.8, 0.5, 1.)
    };
    gpui::div()
        .w_full()
        .h(gpui::px(8.))
        .rounded(gpui::px(4.))
        .bg(gpui::hsla(0., 0., 0.85, 1.))
        .overflow_hidden()
        .child(
            gpui::div()
                .h_full()
                .w(gpui::relative(pct))
                .bg(fill)
        )
}

/// Render a details/summary collapsible container.
/// `open` controls visibility of the content. The summary is always visible.
pub fn details(
    open: bool,
    summary: impl gpui::IntoElement,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    let mut el = gpui::div().flex_col();
    el = el.child(summary);
    if open {
        el = el.child(content);
    }
    el.into_any_element()
}
