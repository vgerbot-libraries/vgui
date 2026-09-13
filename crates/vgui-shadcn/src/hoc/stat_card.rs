use vgui::prelude::*;

use crate::components::{Card, CardHeader};

pub struct StatCard {
    pub title: String,
    pub value: String,
    pub hint: String,
}

impl gpui::IntoElement for StatCard {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        Card {
            children: vec![
                CardHeader {
                    title: self.title,
                    description: self.hint,
                }
                .into_any_element(),
                view! {
                    <span style={css! { font-size: 28px; font-weight: 700; color: var(--foreground); }}>
                        {self.value}
                    </span>
                }
                .into_any_element(),
            ],
        }
        .into_any_element()
    }
}

pub fn stat_card(title: impl Into<String>, value: impl Into<String>, hint: impl Into<String>) -> StatCard {
    StatCard {
        title: title.into(),
        value: value.into(),
        hint: hint.into(),
    }
}
