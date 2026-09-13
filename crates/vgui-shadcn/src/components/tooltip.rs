use vgui::floating_at;
use vgui::prelude::*;
use vgui::show;

#[vgui_component]
pub fn tooltip(
    open: bool,
    anchor: NodeRef,
    text: String,
) -> impl gpui::IntoElement {
    show(
        open,
        floating_at(
            &anchor,
            view! {
                <div style={css! {
                    background: var(--foreground);
                    color: var(--background);
                    border-radius: 6px;
                    padding: 4px 8px;
                    font-size: 12px;
                }}>
                    {text}
                </div>
            },
        ),
        gpui::Empty,
    )
}
