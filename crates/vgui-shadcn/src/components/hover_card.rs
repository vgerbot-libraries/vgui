//! Rich hover preview card.
//!
//! Wraps a trigger element; hovering it (pointer enter/leave) opens a floating
//! card anchored to the trigger, built on the same [`floating_at`]/[`NodeRef`]
//! positioning as [`popover`](crate::components::popover) and
//! [`tooltip`](crate::components::tooltip).

use gpui::IntoElement;
use vgui::{create_signal, floating_at, for_each, NodeRef};
use vgui::prelude::*;

/// Show `card` in a floating panel whenever `trigger` is hovered.
///
/// Unlike a tooltip, the card can hold arbitrary rich content (title, body,
/// actions). It appears on pointer enter and hides on leave.
pub fn hover_card(
    trigger: gpui::AnyElement,
    card: Vec<gpui::AnyElement>,
) -> gpui::AnyElement {
    let (open, set_open) = create_signal(false);
    let anchor = NodeRef::new();
    let is_open = open.get();

    let mut root = view! {
        <div
            ref={anchor.clone()}
            style={css! { display: flex; position: relative; }}
            on:hover={move |hovered: bool, _w: &mut gpui::Window, cx: &mut gpui::App| {
                set_open.set(cx, hovered);
            }}
        >
            {trigger}
        </div>
    }
    .into_any_element();

    if is_open {
        let panel = floating_at(
            &anchor,
            view! {
                <div style={css! {
                    display: flex;
                    flex-direction: column;
                    gap: 6px;
                    width: 280px;
                    background: var(--popover);
                    color: var(--popover-foreground);
                    border-width: 1px;
                    border-style: solid;
                    border-color: var(--border);
                    border-radius: var(--radius);
                    padding: 12px;
                }}>
                    {for_each(card, |c, _| c)}
                </div>
            },
        );
        // Compose the floating panel as a sibling inside the positioned root.
        root = gpui::div()
            .relative()
            .child(root)
            .child(panel)
            .into_any_element();
    }

    root
}
