use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn page_header(
    title: String,
    description: String,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: space-between;
            gap: 12px;
            padding: 8px 0px;
        }}>
            <div style={css! { display: flex; flex-direction: column; gap: 4px; }}>
                <span style={css! { font-size: 22px; font-weight: 700; color: var(--foreground); }}>
                    {title}
                </span>
                <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
                    {description}
                </span>
            </div>
            <div style={css! { display: flex; flex-direction: row; gap: 8px; }}>
                {for_each(children, |c, _| c)}
            </div>
        </div>
    }
}
