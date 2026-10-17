//! Headless tests for the dock: tab switching, divider resize, nested draw.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, App, ClickEvent, InteractiveElement, IntoElement, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, StatefulInteractiveElement, Styled,
    TestAppContext, VisualTestContext, Window,
};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
}

fn drag(cx: &mut VisualTestContext, x0: f32, y0: f32, x1: f32, y1: f32) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x0), px(y0)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(px(x1), px(y1)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    cx.simulate_event(MouseUpEvent {
        position: point(px(x1), px(y1)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
}

/// Marker at a fixed spot inside a panel body; records which panel was hit.
fn body_marker(value: u8, seen: Rc<Cell<u8>>) -> gpui::AnyElement {
    gpui::div()
        .relative()
        .w_full()
        .h_full()
        .child(
            gpui::div()
                .id(("dock-mk", value as usize))
                .absolute()
                .left(px(50.))
                .top(px(40.))
                .w(px(12.))
                .h(px(12.))
                .on_click(move |_: &ClickEvent, _w: &mut Window, _cx: &mut App| seen.set(value)),
        )
        .into_any_element()
}

#[test]
fn dock_tab_switch_changes_body() {
    apply_theme(false);
    let seen = Rc::new(Cell::new(0u8));
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view({
        let seen = seen.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), {
                let seen = seen.clone();
                move || {
                    let s1 = seen.clone();
                    let s2 = seen.clone();
                    let leaf = DockNode::leaf(vec![
                        PanelView::new("A", Rc::new(move || body_marker(1, s1.clone()))),
                        PanelView::new("B", Rc::new(move || body_marker(2, s2.clone()))),
                    ]);
                    dock(dock_area(leaf, 400., 200.))
                }
            })
        }
    });
    draw(&mut cx);

    // Initially panel A is active: its marker (value 1) is hittable.
    click(&mut cx, 56., 76.);
    draw(&mut cx);
    assert_eq!(seen.get(), 1, "active panel A marker should be hit");

    // Switch to tab B (second 108px tab, centre ~x162).
    click(&mut cx, 162., 15.);
    draw(&mut cx);
    click(&mut cx, 56., 76.);
    draw(&mut cx);
    assert_eq!(seen.get(), 2, "after switching, panel B marker is hit");
}

#[test]
fn dock_divider_drag_resizes_pane() {
    apply_theme(false);
    let hit = Rc::new(Cell::new(0u8));
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view({
        let hit = hit.clone();
        move |window, cx| {
            let hit = hit.clone();
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                // Marker at x=130 inside the left pane: clipped at ratio .25
                // (100px), revealed once widened past 130.
                let hit_l = hit.clone();
                let left_body: Rc<dyn Fn() -> gpui::AnyElement> = Rc::new(move || {
                    gpui::div()
                        .relative()
                        .w_full()
                        .h_full()
                        .child(
                            gpui::div()
                                .id(("edge-mk", 7usize))
                                .absolute()
                                .left(px(130.))
                                .top(px(40.))
                                .w(px(12.))
                                .h(px(12.))
                                .on_click({
                                    let hit_l = hit_l.clone();
                                    move |_: &ClickEvent, _w: &mut Window, _cx: &mut App| {
                                        hit_l.set(1)
                                    }
                                }),
                        )
                        .into_any_element()
                });
                let right_body: Rc<dyn Fn() -> gpui::AnyElement> =
                    Rc::new(|| gpui::div().w_full().h_full().child("R").into_any_element());
                let split = DockNode::split_horizontal(
                    DockNode::leaf(vec![PanelView::new("L", left_body)]),
                    DockNode::leaf(vec![PanelView::new("R", right_body)]),
                    0.25,
                );
                dock(dock_area(split, 400., 200.))
            })
        }
    });
    draw(&mut cx);

    // Marker outside the 100px pane is not clickable.
    click(&mut cx, 136., 76.);
    draw(&mut cx);
    assert_eq!(hit.get(), 0, "marker clipped before resize");

    // Divider at ~x100; drag +80px -> ratio .45 -> first pane 180px.
    drag(&mut cx, 103., 100., 183., 100.);
    draw(&mut cx);

    click(&mut cx, 136., 76.);
    draw(&mut cx);
    assert_eq!(hit.get(), 1, "marker revealed after divider drag");
}

#[test]
fn dock_nested_splits_draw() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            let pane = |t: &'static str| {
                DockNode::leaf(vec![PanelView::new(
                    t,
                    Rc::new(move || gpui::div().child(t).into_any_element()),
                )])
            };
            let left = DockNode::split_vertical(pane("TL"), pane("BL"), 0.6);
            let root = DockNode::split_horizontal(left, pane("Right"), 0.4);
            dock(dock_area(root, 480., 320.))
        })
    });
    draw(&mut cx);
}
