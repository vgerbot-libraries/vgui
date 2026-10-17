//! Headless draw tests for the variable-height virtual list.
//!
//! Wheel events are not exposed by gpui's headless platform, so scroll-window
//! math is covered by the `vgui`-crate unit tests on `VirtualListState`; these
//! tests ensure the windowed renderer builds and draws both orientations for a
//! large item count without materialising/panicking.

#![cfg(not(target_family = "wasm"))]

use std::rc::Rc;

use gpui::{px, AnyElement, IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

fn row(label: &'static str, height: f32) -> AnyElement {
    gpui::div()
        .h(px(height))
        .w(px(300.))
        .child(label)
        .into_any_element()
}

#[test]
fn v_virtual_list_draws_large_count() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            let make = Rc::new(|_i: usize| row("item", 24.));
            vgui_shadcn::components::virtual_list::v_virtual_list(100_000, 24., 120., make)
        })
    });
    draw(&mut cx);
}

#[test]
fn h_virtual_list_draws_large_count() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            let make = Rc::new(|_i: usize| row("x", 40.));
            vgui_shadcn::components::virtual_list::h_virtual_list(100_000, 40., 200., make)
        })
    });
    draw(&mut cx);
}
