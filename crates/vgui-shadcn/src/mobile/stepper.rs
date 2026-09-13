use vgui::prelude::*;

use crate::components::{Button, ButtonSize, ButtonVariant};
use crate::icons;

#[vgui_component]
pub fn stepper(
    value: i32,
    min: i32,
    max: i32,
    on_change: Box<dyn Fn(i32, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let on_change = std::rc::Rc::new(on_change);
    let dec = {
        let on_change = on_change.clone();
        move |cx: &mut gpui::App| {
            let next = (value - 1).max(min);
            on_change(next, cx);
        }
    };
    let inc = {
        let on_change = on_change.clone();
        move |cx: &mut gpui::App| {
            let next = (value + 1).min(max);
            on_change(next, cx);
        }
    };
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 8px;
        }}>
            {Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                on_click: Box::new(dec),
                children: vec![icons::CHEVRON_LEFT.to_string().into_any_element()],
            }}
            <span style={css! { min-width: 32px; text-align: center; color: var(--foreground); }}>
                {format!("{value}")}
            </span>
            {Button {
                variant: ButtonVariant::Outline,
                size: ButtonSize::Icon,
                on_click: Box::new(inc),
                children: vec![icons::CHEVRON_RIGHT.to_string().into_any_element()],
            }}
        </div>
    }
}
