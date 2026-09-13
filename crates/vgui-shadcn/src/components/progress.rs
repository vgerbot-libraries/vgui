use vgui::prelude::*;

#[vgui_component]
pub fn progress(
    value: f64,
    max: f64,
) -> impl gpui::IntoElement {
    let pct = if max <= 0.0 {
        0.0
    } else {
        (value / max).clamp(0.0, 1.0)
    };
    let outer = css! {
        width: 100%;
        height: 8px;
        background: var(--muted);
        border-radius: 999px;
        overflow: hidden;
    };
    let inner_style = css! {
        height: 100%;
        background: var(--primary);
        border-radius: 999px;
    };
    let inner = inner_style.apply(
        gpui::div().w(gpui::relative(pct as f32))
    );
    outer.apply(
        gpui::div().child(inner)
    )
}
