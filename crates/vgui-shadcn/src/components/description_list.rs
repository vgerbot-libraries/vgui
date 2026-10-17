use vgui::for_each;
use vgui::prelude::*;

/// One `(term, detail)` row in a [`description_list`].
pub type DescriptionItem = (String, gpui::AnyElement);

/// Term/detail list for displaying structured key/value content, rendered as
/// a semantic description list with an optional title.
#[vgui_component]
pub fn description_list(
    title: Option<String>,
    items: Vec<DescriptionItem>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 0px;
        }}>
            {if let Some(title) = title {
                view! {
                    <span style={css! {
                        font-size: 14px;
                        font-weight: 600;
                        margin-bottom: 8px;
                    }}>
                        {title}
                    </span>
                }.into_any_element()
            } else {
                gpui::div().into_any_element()
            }}
            {for_each(items.into_iter().enumerate(), move |(_, (term, detail)), _| {
                view! {
                    <div style={css! {
                        display: flex;
                        flex-direction: row;
                        align-items: flex-start;
                        gap: 16px;
                        padding: 10px 0px;
                        border-bottom-width: 1px;
                        border-style: solid;
                        border-color: var(--border);
                    }}>
                        <span style={css! {
                            width: 160px;
                            flex-shrink: 0;
                            font-size: 13px;
                            font-weight: 500;
                            color: var(--muted-foreground);
                        }}>
                            {term}
                        </span>
                        <span style={css! { flex: 1; font-size: 13px; color: var(--foreground); }}>
                            {detail}
                        </span>
                    </div>
                }
            })}
        </div>
    }
}
