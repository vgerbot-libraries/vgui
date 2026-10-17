//! Button that opens a dropdown menu.
//!
//! A thin, ergonomic wrapper over [`dropdown_menu`] that supplies a
//! shadcn-styled trigger button (label + chevron) so callers do not have to
//! build the trigger element themselves.

use vgui::prelude::*;

use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::menu::{dropdown_menu, MenuItemData};

/// A button showing `label` and a trailing chevron; clicking it toggles the
/// dropdown `items`. Picking an item runs its callback and closes the menu.
pub fn dropdown_button(
    label: String,
    items: Vec<MenuItemData>,
    variant: ButtonVariant,
    size: ButtonSize,
) -> gpui::AnyElement {
    let trigger = view! {
        <Button variant={variant} size={size} on:click={move |_cx| {}}>
            <span style={css! { display: flex; flex-direction: row; align-items: center; gap: 6px; }}>
                {label}
                <span style={css! { font-size: 10px; }}>{"\u{25BE}"}</span>
            </span>
        </Button>
    }
    .into_any_element();

    dropdown_menu(trigger, items)
}
