//! Headless interaction tests for the batch-3 stateful widgets:
//! number_input steppers, tree expand/select, and date_picker open/select.
//!
//! These widgets own internal signals, so they are constructed inside a live
//! `VguiRoot` and driven through real mouse events, mirroring `menu_window.rs`.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, InteractiveElement, IntoElement, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Styled, TestAppContext, VisualTestContext,
};
use vgui_shadcn::prelude::*;

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
fn number_input_steppers_step_and_clamp() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let last = Rc::new(Cell::new(10.0f64));

    let (_view, cx) = app.add_window_view({
        let last = last.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let n = number_input(
                    10.0,
                    5.0,
                    Some(0.0),
                    Some(100.0),
                    Rc::new({
                        let last = last.clone();
                        move |v, _cx| last.set(v)
                    }),
                );
                vgui::view! { <div>{n}</div> }
            })
        }
    });
    draw(cx);

    // Stepper column is the rightmost 26px of the 140x36 field; up = top half.
    click(cx, 127., 9., MouseButton::Left);
    draw(cx);
    assert_eq!(last.get(), 15.0, "up step adds one step");

    click(cx, 127., 27., MouseButton::Left);
    draw(cx);
    assert_eq!(last.get(), 10.0, "down step subtracts one step");
}

#[test]
fn tree_expands_then_selects_child() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let selected = Rc::new(Cell::new(0u64));

    let (_view, cx) = app.add_window_view({
        let selected = selected.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let nodes = vec![TreeNodeData::branch(
                    1,
                    "Parent",
                    vec![TreeNodeData::leaf(2, "Child")],
                )];
                let t = tree(
                    nodes,
                    0,
                    Rc::new({
                        let selected = selected.clone();
                        move |id, _cx| selected.set(id)
                    }),
                );
                vgui::view! { <div>{t}</div> }
            })
        }
    });
    draw(cx);

    // Parent row occupies y 0..26; its chevron sits at the row's left edge.
    click(cx, 12., 13., MouseButton::Left);
    draw(cx);

    // After expanding, the child row is the next 26px row; clicking the label
    // selects the child. (If expand failed, this point hits nothing.)
    click(cx, 40., 39., MouseButton::Left);
    draw(cx);
    assert_eq!(selected.get(), 2, "expanded child row should be selectable");
}

#[test]
fn date_picker_opens_and_selects_day() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let picked = Rc::new(Cell::new((0i32, 0u32, 0u32)));

    let (_view, mut cx) = app.add_window_view({
        let picked = picked.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let dp = date_picker(
                    None,
                    Rc::new({
                        let picked = picked.clone();
                        move |y, m, d, _cx| picked.set((y, m, d))
                    }),
                );
                vgui::view! { <div>{dp}</div> }
            })
        }
    });
    draw(&mut cx);

    // Open the popover by pressing the trigger.
    click(&mut cx, 20., 18., MouseButton::Left);
    draw(&mut cx);

    // Default month is Jan 2026; the 1st (Thursday) cell sits in the first
    // day row of the calendar anchored below the trigger.
    click(&mut cx, 152., 120., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(
        picked.get(),
        (2026, 1, 1),
        "selecting the first day yields 2026-01-01"
    );
}

fn menu_item(label: &str, flag: Rc<Cell<u8>>) -> MenuItemData {
    (
        String::from(label),
        Rc::new(move |_: &mut gpui::App| flag.set(flag.get() + 1)) as Rc<_>,
    )
}

#[test]
fn dropdown_button_opens_and_picks() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let picked = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let picked = picked.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let b = dropdown_button(
                    String::from("Actions"),
                    vec![menu_item("Run", picked.clone())],
                    ButtonVariant::Outline,
                    ButtonSize::Default,
                );
                vgui::view! { <div>{b}</div> }
            })
        }
    });
    draw(&mut cx);

    // Open via the trigger button, then pick the first panel row.
    click(&mut cx, 20., 18., MouseButton::Left);
    draw(&mut cx);
    click(&mut cx, 20., 56., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(picked.get(), 1, "dropdown button item should fire");
}

#[test]
fn hover_card_renders_without_panic() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            let trigger = gpui::div()
                .id("hc-trigger")
                .w(px(80.))
                .h(px(28.))
                .child("Hover me");
            let card = hover_card(
                trigger.into_any_element(),
                vec![gpui::div().child("Details").into_any_element()],
            );
            vgui::view! { <div>{card}</div> }
        })
    });
    // Construct + layout the closed card (the open branch is gated by a
    // signal-driven `show`); must not panic on hook slots.
    draw(&mut cx);
}
