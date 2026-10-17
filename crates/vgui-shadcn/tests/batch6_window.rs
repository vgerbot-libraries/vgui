//! Headless tests for batch-6 widgets: validation-aware form, reactive i18n,
//! shimmer, and custom window chrome.
//!
//! Window-management caption buttons (minimize/maximize/close/drag) are wired
//! to the real `Window` APIs but the gpui *test* platform leaves those ops
//! `unimplemented!`, so this suite only draws the chrome; the form and i18n
//! paths are exercised through real pointer events.

#![cfg(not(target_family = "wasm"))]

use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    point, px, App, IntoElement, InteractiveElement, Modifiers, MouseButton, MouseDownEvent,
    MouseUpEvent, ParentElement, StatefulInteractiveElement, Styled, TestAppContext,
    VisualTestContext,
};
use vgui_shadcn::prelude::*;
use vgui::{i18n, set_locale, translate};
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

fn name_form(value: &str, submitted: Rc<Cell<u8>>) -> gpui::Div {
    let value = value.to_string();
    form(
        Rc::new(move |_cx: &mut App| submitted.set(1)),
        move || {
            let input = gpui::div()
                .w(px(160.))
                .h(px(28.))
                .child(vgui::view! { <input value={value.clone()} /> })
                .into_any_element();
            vgui::view! {
                <div style={vgui::css!{ display:flex; flex-direction:column; gap:8px; }}>
                    {form_field(
                        "Name",
                        input,
                        value.clone(),
                        required("Name is required"),
                    )}
                    {form_submit_button("Submit")}
                </div>
            }
        },
    )
}

#[test]
fn form_blocks_submit_when_required_field_empty() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let submitted = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let submitted = submitted.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                vgui::view! { <div>{name_form("", submitted.clone())}</div> }
            })
        }
    });
    draw(&mut cx);

    click(&mut cx, 30., 84.);
    draw(&mut cx);
    assert_eq!(
        submitted.get(),
        0,
        "empty required field must block the submit callback"
    );
    // Re-submitting still invalid: validation state is not latched true.
    click(&mut cx, 30., 84.);
    draw(&mut cx);
    assert_eq!(submitted.get(), 0);
}

#[test]
fn form_submits_when_valid() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let submitted = Rc::new(Cell::new(0u8));

    let (_view, mut cx) = app.add_window_view({
        let submitted = submitted.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                vgui::view! { <div>{name_form("Ada", submitted.clone())}</div> }
            })
        }
    });
    draw(&mut cx);

    click(&mut cx, 30., 84.);
    draw(&mut cx);
    assert_eq!(submitted.get(), 1, "a filled required field submits");
}

#[test]
fn imperative_validate_form_reports_validity() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let result = Rc::new(Cell::new(99u8));

    let (_view, mut cx) = app.add_window_view({
        let result = result.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let form = form(
                    Rc::new(|_cx: &mut App| {}),
                    || {
                        let field = |v: &str| {
                            let input = gpui::div()
                                .w(px(160.))
                                .h(px(28.))
                                .child(vgui::view! { <input value={v.to_string()} /> })
                                .into_any_element();
                            form_field("Email", input, v.to_string(), email("bad email"))
                        };
                        vgui::view! {
                            <div style={vgui::css!{ display:flex; flex-direction:column; }}>
                                {field("not-an-email")}
                            </div>
                        }
                    },
                );
                let probe_result = result.clone();
                let probe = gpui::div()
                    .id(("probe", vgui::next_auto_id()))
                    .w(px(40.))
                    .h(px(20.))
                    .on_click(move |_: &gpui::ClickEvent, _w, cx: &mut App| {
                        probe_result.set(if validate_form(cx) { 1 } else { 0 });
                    });
                vgui::view! {
                    <div>
                        {probe.into_any_element()}
                        {form}
                    </div>
                }
            })
        }
    });
    draw(&mut cx);
    click(&mut cx, 10., 14.);
    draw(&mut cx);
    assert_eq!(result.get(), 0, "malformed email fails imperative validation");
}

#[test]
fn locale_switch_translates_through_pointer_event() {
    apply_theme(false);
    i18n! {
        "en" => { "greeting" => "Hello" },
        "fr" => { "greeting" => "Bonjour" }
    }
    let mut app = TestAppContext::single();
    let seen = Rc::new(Cell::new(String::new()));

    let (_view, mut cx) = app.add_window_view({
        let seen = seen.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let switch_seen = seen.clone();
                let switch = gpui::div()
                    .id(("lang", vgui::next_auto_id()))
                    .w(px(60.))
                    .h(px(24.))
                    .on_click(move |_: &gpui::ClickEvent, _w, cx: &mut App| {
                        set_locale(cx, "fr");
                        switch_seen.set(translate("greeting"));
                    });
                vgui::view! {
                    <div>
                        {vgui::t!("greeting")}
                        {switch.into_any_element()}
                    </div>
                }
            })
        }
    });
    draw(&mut cx);
    assert_eq!(translate("greeting"), "Hello");

    click(&mut cx, 10., 30.);
    assert_eq!(seen.take(), "Bonjour", "set_locale changes subsequent translation");

    // Restore global locale so other tests start from the default.
    cx.update(|_window, cx| set_locale(cx, "en"));
}

#[test]
fn shimmer_and_window_chrome_draw() {
    apply_theme(false);
    let mut app = TestAppContext::single();

    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            let load = shimmer(120., 14., 900);
            let title = title_bar(TitleBarOptions::new("Editor"));
            let status = status_bar(
                StatusBar::new()
                    .item(status_item("Ln 3, Col 8"))
                    .item(status_item("Rust")),
            );
            let framed = window_border(
                vgui::view! {
                    <div style={vgui::css!{ display:flex; flex-direction:column; }}>
                        {title}
                        {status}
                    </div>
                }
                .into_any_element(),
            );
            vgui::view! {
                <div style={vgui::css!{ display:flex; flex-direction:column; gap:8px; }}>
                    {load}
                    {framed}
                </div>
            }
        })
    });
    draw(&mut cx);
    // Two frames exercise the interval-driven shimmer repaint without panic.
    draw(&mut cx);
}
