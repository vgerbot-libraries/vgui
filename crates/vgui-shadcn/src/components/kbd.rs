use vgui::prelude::*;

pub struct Kbd {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Kbd {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <kbd style={css! {
                background: var(--muted);
                color: var(--muted-foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: 4px;
                padding: 1px 6px;
                font-size: 12px;
            }}>
                {vgui::for_each(children, |c, _| c)}
            </kbd>
        }
        .into_any_element()
    }
}

pub fn kbd(label: impl Into<String>) -> Kbd {
    Kbd {
        children: vec![label.into().into_any_element()],
    }
}
