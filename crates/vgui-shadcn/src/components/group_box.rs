use vgui::for_each;
use vgui::prelude::*;

/// Titled, bordered container grouping related controls.
#[vgui_component]
pub fn group_box(
    title: String,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <fieldset style={css! {
            display: flex;
            flex-direction: column;
            gap: 8px;
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 12px;
            margin: 0px;
        }}>
            <span style={css! {
                font-size: 12px;
                font-weight: 500;
                color: var(--muted-foreground);
            }}>
                {title}
            </span>
            <div style={css! { display: flex; flex-direction: column; gap: 8px; }}>
                {for_each(children, |c, _| c)}
            </div>
        </fieldset>
    }
}
