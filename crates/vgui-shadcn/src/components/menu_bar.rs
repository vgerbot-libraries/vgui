//! Application menu bar.
//!
//! A horizontal bar of named top-level menus (File, Edit, View…), each opening
//! its [`crate::components::menu::dropdown_menu`] of items. Suitable as the
//! top chrome of a desktop-style window.

use gpui::{px, AnyElement, IntoElement, ParentElement, Styled};

use crate::components::menu::{dropdown_menu, MenuItemData};

/// One top-level menu: a title and its items.
pub struct MenuEntry {
    /// The menu's title shown on the bar (e.g. `"File"`).
    pub title: String,
    /// Items revealed when the menu is opened.
    pub items: Vec<MenuItemData>,
}

impl MenuEntry {
    /// Build a top-level menu entry.
    pub fn new(title: impl Into<String>, items: Vec<MenuItemData>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }
}

/// A horizontal application menu bar.
///
/// Each entry's title acts as the trigger for a [`dropdown_menu`]; pressing a
/// title opens its panel and choosing an item runs its callback.
pub fn menu_bar(menus: Vec<MenuEntry>) -> AnyElement {
    let mut bar = gpui::div()
        .flex()
        .flex_row()
        .items_center()
        .h(px(32.))
        .px_1()
        .w_full();
    bar = vgui::css! {
        background: var(--muted);
        border-bottom-width: 1px;
        border-style: solid;
        border-color: var(--border);
    }
    .apply(bar);

    for menu in menus {
        let trigger = vgui::css! {
            padding: 4px 10px;
            font-size: 13px;
            color: var(--foreground);
            cursor: pointer;
            border-radius: var(--radius);
        }
        .apply(gpui::div().child(menu.title));
        let dropdown = dropdown_menu(trigger.into_any_element(), menu.items);
        bar = bar.child(dropdown);
    }

    bar.into_any_element()
}
