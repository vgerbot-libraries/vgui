//! Headless tests for the bundled icon set: a grid of real SVGs paints
//! through gpui's SVG renderer, and unknown names render non-fatally.

#![cfg(not(target_family = "wasm"))]

use gpui::{IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

#[test]
fn icon_grid_draws_real_svgs() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_v, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            // A spread of genuine bundled icons, drawn as a wrapped row.
            let names = [
                "bell", "check", "chevron_right", "close", "file", "folder", "settings", "star",
                "archive", "bookmark",
            ];
            let mut row = gpui::div().flex().flex_row().flex_wrap();
            for name in names {
                assert!(has_icon(name), "expected bundled icon {name}");
                row = row.child(gpui::div().p_1().child(icon(name).size_px(18.)));
            }
            row.into_any_element()
        })
    });
    draw(&mut cx);
}

#[test]
fn unknown_name_is_safe_placeholder() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_v, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            gpui::div().child(icon("no-such-icon")).into_any_element()
        })
    });
    draw(&mut cx);
    assert!(!has_icon("no-such-icon"));
}
