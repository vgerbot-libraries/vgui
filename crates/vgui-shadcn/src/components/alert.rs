use vgui::for_each;
use vgui::prelude::*;

variants! {
    Alert {
        base => css! {
            display: flex;
            flex-direction: column;
            gap: 4px;
            border-width: 1px;
            border-style: solid;
            border-radius: var(--radius);
            padding: 12px 16px;
        },
        variant {
            default => css! {
                background: var(--card);
                color: var(--card-foreground);
                border-color: var(--border);
            },
            destructive => css! {
                background: var(--card);
                color: var(--destructive);
                border-color: var(--destructive);
            },
        },
    }
}

#[vgui_component]
pub fn alert(
    variant: AlertVariant,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    let variants = AlertVariants::default().variant(variant);
    view! {
        <div style={variants}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn alert_title(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <span style={css! { font-size: 14px; font-weight: 600; }}>
            {for_each(children, |c, _| c)}
        </span>
    }
}

#[vgui_component]
pub fn alert_description(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
            {for_each(children, |c, _| c)}
        </span>
    }
}
