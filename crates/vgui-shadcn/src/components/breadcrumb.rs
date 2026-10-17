use vgui::for_each;
use vgui::prelude::*;

use crate::icons;

/// One breadcrumb entry: `(label, optional click callback)`. The last entry
/// is rendered as the current page (non-interactive); earlier entries are
/// clickable links when they carry a callback.
pub type BreadcrumbItem = (String, Option<Box<dyn Fn(&mut gpui::App) + 'static>>);

#[vgui_component]
pub fn breadcrumb(items: Vec<BreadcrumbItem>) -> impl gpui::IntoElement {
    let last = items.len().saturating_sub(1);
    view! {
        <nav role="navigation" style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            flex-wrap: wrap;
            gap: 4px;
            font-size: 14px;
            color: var(--muted-foreground);
        }}>
            {for_each(items.into_iter().enumerate(), move |(i, (label, on_click)), _| {
                let current = i == last;
                view! {
                    <span style={css! {
                        display: flex;
                        flex-direction: row;
                        align-items: center;
                        gap: 4px;
                    }}>
                        {if i > 0 {
                            view! { <span>{icons::CHEVRON_RIGHT}</span> }
                                .into_any_element()
                        } else {
                            gpui::div().into_any_element()
                        }}
                        {if current {
                            view! {
                                <span style={css! { color: var(--foreground); font-weight: 500; }}>
                                    {label}
                                </span>
                            }.into_any_element()
                        } else if let Some(on_click) = on_click {
                            view! {
                                <button
                                    style={css! {
                                        border-width: 0px;
                                        color: var(--muted-foreground);
                                        cursor: pointer;
                                        padding: 0px;
                                        font-size: 14px;
                                    }}
                                    hover={css! { color: var(--foreground); }}
                                    on:click={click(move |cx| on_click(cx))}
                                >
                                    {label}
                                </button>
                            }.into_any_element()
                        } else {
                            view! { <span>{label}</span> }.into_any_element()
                        }}
                    </span>
                }
            })}
        </nav>
    }
}
