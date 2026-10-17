//! Headless interaction tests for batch-4 widgets: nav_stack push/pop,
//! carousel wrap, combobox selection, command-palette hotkey/run, and a
//! construct/draw smoke test for the text-entry widgets (otp_input,
//! time_field).
//!
//! Stateful widgets own signals and are driven through real mouse/keyboard
//! events inside a live `VguiRoot`, mirroring `menu_window.rs`.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, App, ClickEvent, InteractiveElement, IntoElement, KeyDownEvent, Keystroke,
    Modifiers, MouseButton, MouseDownEvent, MouseUpEvent, ParentElement,
    StatefulInteractiveElement, Styled, TestAppContext, VisualTestContext, Window,
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

fn key(cx: &mut VisualTestContext, key: &str, control: bool) {
    cx.simulate_event(KeyDownEvent {
        keystroke: Keystroke {
            modifiers: Modifiers {
                control,
                ..Modifiers::default()
            },
            key: key.into(),
            key_char: None,
        },
        is_held: false,
        prefer_character_input: false,
    });
}

#[test]
fn nav_stack_push_and_pop() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let current = Rc::new(Cell::new(0u32));

    let (_view, mut cx) = app.add_window_view({
        let current = current.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let stack = NavStack::<u32>::new(0);
                let body = {
                    let stack = stack.clone();
                    let current = current.clone();
                    nav_stack(stack.clone(), move |page, _depth| {
                        current.set(*page);
                        let s = stack.clone();
                        let mut row = gpui::div()
                            .id(("nav-row", *page as usize))
                            .w(px(120.))
                            .h(px(30.))
                            .cursor_pointer()
                            .on_click(
                                move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                                    if s.can_pop() {
                                        s.pop(cx);
                                    } else {
                                        s.push(cx, 1);
                                    }
                                },
                            );
                        row = if *page == 0 {
                            row.child("open detail")
                        } else {
                            row.child("back")
                        };
                        row.into_any_element()
                    })
                };
                vgui::view! { <div>{body}</div> }
            })
        }
    });
    draw(&mut cx);
    assert_eq!(current.get(), 0, "starts at root page");

    // One click pushes to the detail page; a second pops back to root.
    click(&mut cx, 10., 15., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(current.get(), 1, "click pushes page 1");

    click(&mut cx, 10., 15., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(current.get(), 0, "click on detail pops back to root");
}

#[test]
fn carousel_prev_wraps_to_last() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let visible = Rc::new(Cell::new(99u8));

    let (_view, mut cx) = app.add_window_view({
        let visible = visible.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let slide = |n: u8| {
                    let visible = visible.clone();
                    gpui::div()
                        .id(("slide", n as usize))
                        .w(px(300.))
                        .h(px(80.))
                        .on_click(move |_: &ClickEvent, _w: &mut Window, _cx: &mut App| {
                            visible.set(n);
                        })
                        .child(format!("slide{}", n))
                        .into_any_element()
                };
                let c = carousel(vec![slide(0), slide(1), slide(2)], 0);
                vgui::view! { <div>{c}</div> }
            })
        }
    });
    draw(&mut cx);

    // Prev arrow (left edge, vertically centered) wraps index 0 -> 2.
    click(&mut cx, 14., 44., MouseButton::Left);
    draw(&mut cx);
    // Press the slide body; the marker reports which slide is mounted.
    click(&mut cx, 10., 10., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(visible.get(), 2, "prev on first slide wraps to last");
}

#[test]
fn combobox_filters_and_picks() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let picked = Rc::new(std::cell::RefCell::new(String::new()));

    let (_view, mut cx) = app.add_window_view({
        let picked = picked.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let cb = combobox(
                    vec!["Alpha".into(), "Beta".into(), "Gamma".into()],
                    "",
                    "choose",
                    Rc::new({
                        let picked = picked.clone();
                        move |v, _cx| *picked.borrow_mut() = v
                    }),
                );
                vgui::view! { <div>{cb}</div> }
            })
        }
    });
    draw(&mut cx);
    click(&mut cx, 20., 18., MouseButton::Left);
    draw(&mut cx);
    click(&mut cx, 30., 56., MouseButton::Left);
    draw(&mut cx);
    assert_eq!(*picked.borrow(), "Alpha", "picking first option fills value");
}

#[test]
fn command_palette_ctrl_k_runs_command() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let ran = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let ran = ran.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let cp = command_palette(vec![CommandItem::new("Do thing", {
                    let ran = ran.clone();
                    move |_cx| ran.set(ran.get() + 1)
                })]);
                vgui::view! { <div>{cp}</div> }
            })
        }
    });
    draw(&mut cx);
    assert_eq!(ran.get(), 0);

    key(&mut cx, "k", true);
    draw(&mut cx);
    key(&mut cx, "enter", false);
    draw(&mut cx);
    assert_eq!(ran.get(), 1, "Ctrl+K opens, Enter runs the highlighted command");
}

#[test]
fn text_entry_widgets_draw_without_panic() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            let otp = otp_input(
                4,
                Rc::new(|_code: String, _cx: &mut App| {}) as Rc<dyn Fn(String, &mut App)>,
            );
            let time = time_field(
                true,
                Rc::new(|_h: u32, _m: u32, _s: u32, _cx: &mut App| {}) as Rc<dyn Fn(u32, u32, u32, &mut App)>,
            );
            let back = nav_back_button(NavStack::<u32>::new(0), "Back");
            vgui::view! {
                <div>
                    {otp}
                    {time}
                    {back}
                </div>
            }
        })
    });
    draw(&mut cx);
}
