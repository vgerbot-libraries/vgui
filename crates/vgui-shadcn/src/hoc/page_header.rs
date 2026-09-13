use vgui::for_each;
use vgui::prelude::*;

pub struct PageHeader {
    pub title: String,
    pub description: String,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for PageHeader {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
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
                        {self.title}
                    </span>
                    <span style={css! { font-size: 13px; color: var(--muted-foreground); }}>
                        {self.description}
                    </span>
                </div>
                <div style={css! { display: flex; flex-direction: row; gap: 8px; }}>
                    {for_each(children, |c, _| c)}
                </div>
            </div>
        }
        .into_any_element()
    }
}
