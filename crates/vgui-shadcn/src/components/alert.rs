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

pub struct Alert {
    pub variant: AlertVariant,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Alert {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let variants = AlertVariants::default().variant(self.variant);
        let children = self.children;
        view! {
            <div style={variants}>
                {for_each(children, |c, _| c)}
            </div>
        }
        .into_any_element()
    }
}

pub struct AlertTitle {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for AlertTitle {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <span style={css! { font-size: 14px; font-weight: 600; }}>
                {for_each(children, |c, _| c)}
            </span>
        }
        .into_any_element()
    }
}

pub struct AlertDescription {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for AlertDescription {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
                {for_each(children, |c, _| c)}
            </span>
        }
        .into_any_element()
    }
}
