use vgui::prelude::*;

pub struct EmptyState {
    pub title: String,
    pub description: String,
}

impl gpui::IntoElement for EmptyState {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        view! {
            <div style={css! {
                display: flex;
                flex-direction: column;
                align-items: center;
                justify-content: center;
                gap: 8px;
                padding: 32px;
                color: var(--muted-foreground);
            }}>
                <span style={css! { font-size: 16px; font-weight: 600; color: var(--foreground); }}>
                    {self.title}
                </span>
                <span style={css! { font-size: 13px; }}>
                    {self.description}
                </span>
            </div>
        }
        .into_any_element()
    }
}

pub fn empty_state(title: impl Into<String>, description: impl Into<String>) -> EmptyState {
    EmptyState {
        title: title.into(),
        description: description.into(),
    }
}
