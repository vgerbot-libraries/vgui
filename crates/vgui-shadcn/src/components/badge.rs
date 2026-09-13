use vgui::for_each;
use vgui::prelude::*;

variants! {
    Badge {
        base => css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            border-radius: var(--radius);
            padding: 2px 8px;
            font-size: 12px;
            font-weight: 500;
        },
        variant {
            default => css! { background: var(--primary); color: var(--primary-foreground); },
            secondary => css! { background: var(--secondary); color: var(--secondary-foreground); },
            outline => css! {
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
            },
            destructive => css! { background: var(--destructive); color: var(--primary-foreground); },
        },
    }
}

pub struct Badge {
    pub variant: BadgeVariant,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Badge {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let variants = BadgeVariants::default().variant(self.variant);
        let children = self.children;
        view! {
            <span style={variants}>
                {for_each(children, |c, _| c)}
            </span>
        }
        .into_any_element()
    }
}

pub fn badge(variant: BadgeVariant, label: impl Into<String>) -> Badge {
    Badge {
        variant,
        children: vec![label.into().into_any_element()],
    }
}
