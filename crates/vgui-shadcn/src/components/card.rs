use vgui::for_each;
use vgui::prelude::*;

pub struct Card {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Card {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
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
        .into_any_element()
    }
}

pub struct CardHeader {
    pub title: String,
    pub description: String,
}

impl gpui::IntoElement for CardHeader {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        view! {
            <div style={css! { display: flex; flex-direction: column; gap: 4px; }}>
                <span style={css! { font-size: 16px; font-weight: 600; color: var(--card-foreground); }}>
                    {self.title}
                </span>
                <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
                    {self.description}
                </span>
            </div>
        }
        .into_any_element()
    }
}
