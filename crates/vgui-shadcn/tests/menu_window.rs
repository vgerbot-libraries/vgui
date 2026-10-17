//! Headless interaction tests for imperative menus and the clipboard helper.
//!
//! Menus own an `open` signal, so they can only be constructed inside a live
//! `VguiRoot`. These tests drive an offscreen gpui window through real mouse
//! events: open the dropdown, pick an item, observe its callback, dismiss via
//! click-outside, and repeat for the right-click context menu anchored at the
//! cursor.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, InteractiveElement, IntoElement, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Styled, TestAppContext, VisualTestContext,
};
use vgui_shadcn::prelude::*;

fn item(label: &str, flag: Rc<Cell<u8>>) -> MenuItemData {
    (
        String::from(label),
        Rc::new(move |_: &mut gpui::App| flag.set(flag.get() + 1)) as Rc<_>,
    )
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

fn click(cx: &mut VisualTestContext, x: f32, y: f32, button: MouseButton) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button,
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button,
        click_count: 1,
    });
}

#[test]
fn dropdown_open_pick_and_dismiss() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let picked = Rc::new(Cell::new(0u8));

    let (_view, cx) = app.add_window_view({
        let picked = picked.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let trigger = gpui::div().id("trigger").w(px(100.)).h(px(28.)).child("Open");
                let dropdown =
                    dropdown_menu(trigger.into_any_element(), vec![item("Pick", picked.clone())]);
                vgui::view! { <div>{dropdown}</div> }
            })
        }
    });

    // Initial layout: menu closed.
    draw(cx);

    // Left press on the trigger toggles the panel open.
    click(cx, 10., 10., MouseButton::Left);
    draw(cx);

    // The panel rows sit below the trigger (top offset 36px). Picking a row
    // runs its action once and the menu closes.
    click(cx, 20., 50., MouseButton::Left);
    draw(cx);
    assert_eq!(picked.get(), 1, "menu item action should fire once");

    // Reopen, then dismiss by pressing outside the panel. The unmounted row
    // at its old coordinates must not fire again.
    click(cx, 10., 10., MouseButton::Left);
    draw(cx);
    click(cx, 300., 300., MouseButton::Left);
    draw(cx);
    click(cx, 20., 50., MouseButton::Left);
    assert_eq!(picked.get(), 1, "dismissed menu must not fire its item");
}

#[test]
fn context_menu_opens_at_cursor_and_picks() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let picked = Rc::new(Cell::new(0u8));

    let (_view, cx) = app.add_window_view({
        let picked = picked.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let content = gpui::div().id("content").w(px(100.)).h(px(28.)).child("Area");
                let menu =
                    context_menu(content.into_any_element(), vec![item("Pick", picked.clone())]);
                vgui::view! { <div>{menu}</div> }
            })
        }
    });

    draw(cx);

    // Secondary click on the content opens the menu anchored at the cursor.
    click(cx, 10., 10., MouseButton::Right);
    draw(cx);

    // Item row renders at the cursor anchor; picking it runs the action.
    click(cx, 20., 24., MouseButton::Left);
    assert_eq!(picked.get(), 1, "context menu item should fire");
}

#[test]
fn clipboard_round_trips_plain_text() {
    let app = TestAppContext::single();
    app.update(|cx| {
        vgui::write_text("hello vgui", cx);
        assert_eq!(vgui::read_text(cx).as_deref(), Some("hello vgui"));
    });
}
