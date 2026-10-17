use vgui::for_each;
use vgui::prelude::*;

/// Horizontal action bar. Lays out controls with a uniform gap and optional
/// vertical separators between groups.
#[vgui_component]
pub fn toolbar(children: Vec<gpui::AnyElement>) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
            padding: 4px;
            border-radius: var(--radius);
            background: var(--background);
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

/// Vertical divider used to group controls inside a [`toolbar`].
#[vgui_component]
pub fn toolbar_separator() -> impl gpui::IntoElement {
    view! {
        <span style={css! {
            width: 1px;
            align-self: stretch;
            margin: 4px 4px;
            background: var(--border);
        }} />
    }
}
