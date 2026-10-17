//! Dropdown and context menus.
//!
//! Imperative widgets built on the same signal + deferred-floating-panel
//! pattern as vgui's `select`: an open signal drives whether the panel is
//! rendered, and a click outside or an item pick closes it.

use std::rc::Rc;

use gpui::{
    deferred, px, AnyElement, App, ClickEvent, InteractiveElement, IntoElement,
    MouseButton, MouseDownEvent, ParentElement, Point, Pixels, StatefulInteractiveElement, Styled,
    Window,
};
use vgui::{
    create_signal, css, floating, next_auto_id, WriteSignal, __dom_contextmenu,
};

/// One menu entry: `(label, callback run when picked)`.
pub type MenuItemData = (String, Rc<dyn Fn(&mut App)>);

type MenuItems = Rc<Vec<MenuItemData>>;

/// One clickable menu row.
fn menu_row(id: &'static str, label: String, action: Rc<dyn Fn(&mut App)>, dismiss: WriteSignal<bool>)
    -> gpui::AnyElement
{
    let row = gpui::div()
        .id((id, next_auto_id()))
        .w_full()
        .cursor_pointer()
        .on_click(move |_: &ClickEvent, _window: &mut Window, cx: &mut App| {
            action(cx);
            dismiss.set(cx, false);
        })
        .child(label);
    css! {
        display: flex;
        flex-direction: row;
        align-items: center;
        padding: 6px 12px;
        background: var(--popover);
        color: var(--popover-foreground);
        cursor: pointer;
        font-size: 14px;
        text-align: left;
    }
    .apply(row)
    .into_any_element()
}

/// Build the floating panel that lists `items`. When `anchored` is true it is
/// placed under the trigger (absolute child); otherwise the caller places it
/// through [`floating`] at a cursor position.
fn menu_panel(
    id: &'static str,
    items: MenuItems,
    dismiss: WriteSignal<bool>,
    anchored: bool,
) -> gpui::AnyElement {
    let dismiss_out = dismiss.clone();
    let mut panel = gpui::div().id((id, next_auto_id())).min_w(px(160.)).py_1();
    if anchored {
        panel = panel.absolute().top(px(36.)).left_0();
    }
    let mut panel = css! {
        background: var(--popover);
        color: var(--popover-foreground);
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
    }
    .apply(panel)
    .shadow_md()
    .on_mouse_down_out(
        move |_: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
            dismiss_out.set(cx, false);
            cx.stop_propagation();
        },
    );

    for (label, action) in items.iter() {
        panel = panel.child(menu_row(
            "menu-item",
            label.clone(),
            action.clone(),
            dismiss.clone(),
        ));
    }
    panel.into_any_element()
}

/// Dropdown menu opened by a left click on `trigger`. The menu is rendered in
/// a deferred panel anchored under the trigger; choosing an item invokes its
/// callback and closes the menu.
pub fn dropdown_menu(trigger: AnyElement, items: Vec<MenuItemData>) -> gpui::AnyElement {
    let items: MenuItems = Rc::new(items);
    let (open_read, open_write) = create_signal(false);

    let toggle_read = open_read.clone();
    let toggle_write = open_write.clone();
    let is_open = open_read.get();

    let mut root = gpui::div()
        .id(("dropdown-menu", next_auto_id()))
        .relative()
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                toggle_write.set(cx, !toggle_read.get());
                cx.stop_propagation();
            },
        );

    root = root.child(trigger);

    if is_open {
        root = root.child(deferred(menu_panel(
            "dropdown-panel",
            items.clone(),
            open_write.clone(),
            true,
        )));
    }
    root.into_any_element()
}

/// Wrap `content` so a secondary (right) click opens a context menu at the
/// cursor position with the given `items`.
pub fn context_menu(content: AnyElement, items: Vec<MenuItemData>) -> gpui::AnyElement {
    let items: MenuItems = Rc::new(items);
    let (open_read, open_write) = create_signal(false);
    let (pos_read, pos_write) = create_signal::<Point<Pixels>>(Point::new(px(0.), px(0.)));

    let is_open = open_read.get();
    let position = pos_read.get();
    let open_write_handler = open_write.clone();

    let mut root = gpui::div()
        .id(("context-menu", next_auto_id()))
        .relative()
        .on_aux_click(__dom_contextmenu(
            move |ev: &vgui::PointerEvent, _window: &mut Window, cx: &mut App| {
                pos_write.set(
                    cx,
                    Point::new(px(ev.client_x as f32), px(ev.client_y as f32)),
                );
                open_write_handler.set(cx, true);
                cx.stop_propagation();
            },
        ))
        .child(content);

    if is_open {
        root = root.child(floating(
            position,
            menu_panel("context-panel", items.clone(), open_write.clone(), false),
        ));
    }
    root.into_any_element()
}
