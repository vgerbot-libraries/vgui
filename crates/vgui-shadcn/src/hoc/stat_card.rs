use vgui::prelude::*;

use crate::components::{Card, CardHeader};

#[vgui_component]
pub fn stat_card(
    title: String,
    value: String,
    hint: String,
) -> impl gpui::IntoElement {
    Card {
        children: vec![
            CardHeader {
                title,
                description: hint,
            }
            .into_any_element(),
            view! {
                <span style={css! { font-size: 28px; font-weight: 700; color: var(--foreground); }}>
                    {value}
                </span>
            }
            .into_any_element(),
        ],
    }
}
