use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn card(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            background: var(--card);
            color: var(--card-foreground);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn card_header(
    title: String,
    description: String,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! { display: flex; flex-direction: column; gap: 4px; }}>
            <span style={css! { font-size: 16px; font-weight: 600; color: var(--card-foreground); }}>
                {title}
            </span>
            <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
                {description}
            </span>
        </div>
    }
}
