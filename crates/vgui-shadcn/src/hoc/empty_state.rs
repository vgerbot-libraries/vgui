use vgui::prelude::*;

#[vgui_component]
pub fn empty_state(
    title: String,
    description: String,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 32px;
            color: var(--muted-foreground);
        }}>
            <span style={css! { font-size: 16px; font-weight: 600; color: var(--foreground); }}>
                {title}
            </span>
            <span style={css! { font-size: 13px; }}>
                {description}
            </span>
        </div>
    }
}
