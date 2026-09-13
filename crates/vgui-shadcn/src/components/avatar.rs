use vgui::prelude::*;

#[vgui_component]
pub fn avatar(
    initials: String,
    size: f32,
) -> impl gpui::IntoElement {
    let style = css! {
        border-radius: 999px;
        background: var(--muted);
        color: var(--muted-foreground);
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 12px;
        font-weight: 600;
    };
    let el = gpui::div()
        .w(gpui::px(size))
        .h(gpui::px(size))
        .child(initials);
    style.apply(el)
}
