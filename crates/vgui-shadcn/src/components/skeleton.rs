use vgui::prelude::*;

#[vgui_component]
pub fn skeleton(
    width: f32,
    height: f32,
) -> impl gpui::IntoElement {
    let style = css! {
        background: var(--muted);
        border-radius: var(--radius);
    };
    let el = gpui::div()
        .w(gpui::px(width))
        .h(gpui::px(height));
    style.apply(el)
}
