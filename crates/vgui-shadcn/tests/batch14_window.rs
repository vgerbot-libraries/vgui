//! Headless tests for the diff viewer: unified/split toggle drives the
//! returned signal; unified and conflict viewers draw.

#![cfg(not(target_family = "wasm"))]

use std::cell::RefCell;
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

#[test]
fn diff_editor_toggle_flips_split_signal() {
    apply_theme(false);
    let signal: Rc<RefCell<Option<vgui::ReadSignal<bool>>>> = Rc::new(RefCell::new(None));
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view({
        let signal = signal.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), {
                let signal = signal.clone();
                move || {
                    let (el, split) = DiffEditor::render(
                        "line one\nline two".to_string(),
                        "line one\nline TWO\nline three".to_string(),
                    );
                    *signal.borrow_mut() = Some(split);
                    gpui::div().w(px(400.)).child(el).into_any_element()
                }
            })
        }
    });
    draw(&mut cx);
    assert_eq!(signal.borrow().as_ref().unwrap().get(), false, "starts unified");

    // Toggle is the 96px control at the right of the 400px header (y center 15).
    click(&mut cx, 344., 15.);
    draw(&mut cx);
    assert_eq!(
        signal.borrow().as_ref().unwrap().get(),
        true,
        "clicking the mode toggle enables split view"
    );

    click(&mut cx, 344., 15.);
    draw(&mut cx);
    assert_eq!(signal.borrow().as_ref().unwrap().get(), false, "toggles back");
}

#[test]
fn unified_diff_viewer_draws() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            gpui::div()
                .w(px(420.))
                .child(diff_viewer("a\nb\nc", "a\nB\nc"))
                .into_any_element()
        })
    });
    draw(&mut cx);
}

#[test]
fn conflict_viewer_draws_both_sides() {
    apply_theme(false);
    let text = "context\n<<<<<<< HEAD\nours\n=======\ntheirs\n>>>>>>> branch\nend";
    let text = text.to_string();
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(move |window, cx| {
        let text = text.clone();
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            gpui::div()
                .w(px(420.))
                .child(conflict_viewer(&text))
                .into_any_element()
        })
    });
    draw(&mut cx);
}
