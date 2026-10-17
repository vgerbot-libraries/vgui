//! Headless interaction tests for batch-5 widgets: drag-resizable split,
//! attachment remove, menu-bar open/pick, plus construct/draw smoke tests for
//! the canvas spinner/charts and the chat bubble scroller.
//!
//! Stateful widgets own signals and are driven through real pointer/keyboard
//! events inside a live `VguiRoot`, mirroring `menu_window.rs`.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, App, ClickEvent, hsla, InteractiveElement, IntoElement, Modifiers, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, StatefulInteractiveElement,
    Styled, TestAppContext, VisualTestContext, Window,
};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

fn down(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
}

fn up(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseUpEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
}

fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
    down(cx, x, y);
    up(cx, x, y);
}

fn drag(cx: &mut VisualTestContext, x0: f32, y0: f32, x1: f32, y1: f32) {
    down(cx, x0, y0);
    cx.simulate_event(MouseMoveEvent {
        position: point(px(x1), px(y1)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    up(cx, x1, y1);
}

/// A clickable marker placed at an absolute x inside a pane; it reports hits.
fn x_marker(x: f32, hit: Rc<Cell<u8>>) -> gpui::AnyElement {
    gpui::div()
        .id(("mk", x as usize))
        .absolute()
        .left(px(x))
        .top(px(8.))
        .w(px(10.))
        .h(px(10.))
        .on_click(move |_: &ClickEvent, _w: &mut Window, _cx: &mut App| hit.set(1))
        .into_any_element()
}

#[test]
fn h_resizable_drags_divider() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let hit = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let hit = hit.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                // Marker at x=120 inside the left pane; clipped while the pane
                // is 100px wide, visible once widened past 120.
                let left = gpui::div()
                    .relative()
                    .w_full()
                    .h(px(60.))
                    .child(x_marker(120., hit.clone()));
                let right = gpui::div().w_full().h(px(60.)).child("R");
                let split = h_resizable(
                    left.into_any_element(),
                    right.into_any_element(),
                    100.,
                    400.,
                );
                vgui::view! { <div>{split}</div> }
            })
        }
    });
    draw(&mut cx);

    // Marker is clipped before dragging.
    click(&mut cx, 125., 14.);
    draw(&mut cx);
    assert_eq!(hit.get(), 0, "marker outside the 100px pane is not clickable");

    // Grab the divider strip (~x100) and drag +60px.
    drag(&mut cx, 103., 20., 163., 20.);
    draw(&mut cx);

    // Marker now lies inside the widened (~160px) pane.
    click(&mut cx, 125., 14.);
    draw(&mut cx);
    assert_eq!(hit.get(), 1, "dragging the divider reveals the marker");
}

#[test]
fn attachment_remove_fires() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let removed = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let removed = removed.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let card = attachment_view(
                    Attachment::new("doc.pdf", 2048),
                    None,
                    Rc::new({
                        let removed = removed.clone();
                        move |_cx| removed.set(removed.get() + 1)
                    }),
                );
                vgui::view! { <div>{card}</div> }
            })
        }
    });
    draw(&mut cx);
    click(&mut cx, 256., 20.);
    draw(&mut cx);
    assert_eq!(removed.get(), 1, "remove button fires on_remove");
}

#[test]
fn menu_bar_opens_and_picks() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let ran = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let ran = ran.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let bar = menu_bar(vec![MenuEntry::new(
                    "File",
                    vec![(
                        "New".to_string(),
                        Rc::new({
                            let ran = ran.clone();
                            move |_cx: &mut App| ran.set(ran.get() + 1)
                        }) as Rc<dyn Fn(&mut App)>,
                    )],
                )]);
                vgui::view! { <div>{bar}</div> }
            })
        }
    });
    draw(&mut cx);
    click(&mut cx, 20., 16.);
    draw(&mut cx);
    click(&mut cx, 30., 52.);
    draw(&mut cx);
    assert_eq!(ran.get(), 1, "opening a menu and picking an item runs it");
}

#[test]
fn canvas_and_chat_widgets_draw() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            let spin = spinner(20., 1000);
            let bar = bar_chart(vec![3.0, 1.0, 4.0, 1.0, 5.0, 9.0, 2.0], 200., 120.);
            let line = line_chart(vec![1.0, 2.0, 3.0, 2.0, 4.0, 3.0], 200., 120.);
            let pie = pie_chart(
                vec![
                    PieSlice::new(2.0, hsla(0.58, 0.7, 0.5, 1.0)),
                    PieSlice::new(3.0, hsla(0.08, 0.7, 0.55, 1.0)),
                    PieSlice::new(5.0, hsla(0.33, 0.6, 0.45, 1.0)),
                ],
                120.,
            );
            let transcript = message_scroller(vec![
                Message::new("Ada", "hello", "10:01", BubbleSide::Received),
                Message::new("You", "hi", "10:02", BubbleSide::Sent),
            ]);
            let single = bubble(Message::new("You", "yo", "10:03", BubbleSide::Sent));
            vgui::view! {
                <div>
                    {spin}
                    {bar}
                    {line}
                    {pie}
                    {transcript}
                    {single}
                </div>
            }
        })
    });
    draw(&mut cx);
    draw(&mut cx);
}
