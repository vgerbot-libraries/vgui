use vgui::prelude::*;

#[vgui_component]
pub fn separator(
    vertical: bool,
) -> impl gpui::IntoElement {
    if vertical {
        view! {
            <div style={css! {
                width: 1px;
                height: 100%;
                background: var(--border);
            }} />
        }
    } else {
        view! {
            <div style={css! {
                width: 100%;
                height: 1px;
                background: var(--border);
            }} />
        }
    }
}
