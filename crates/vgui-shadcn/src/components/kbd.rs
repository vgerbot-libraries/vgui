use vgui::prelude::*;

#[vgui_component]
pub fn kbd(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <kbd style={css! {
            background: var(--muted);
            color: var(--muted-foreground);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: 4px;
            padding: 1px 6px;
            font-size: 12px;
        }}>
            {vgui::for_each(children, |c, _| c)}
        </kbd>
    }
}
