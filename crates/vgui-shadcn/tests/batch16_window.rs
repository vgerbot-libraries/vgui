//! Headless draw test for the lexical code editor view.
//!
//! Tokenization is unit-tested in the component module; this verifies the
//! per-line highlighted render tree (block comment, string, number, macro
//! call, type) lays out through a real gpui window without panicking.

#![cfg(not(target_family = "wasm"))]

use gpui::{
    div, px, IntoElement, ParentElement, Styled, TestAppContext, VisualTestContext,
};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

#[test]
fn code_editor_draws() {
    apply_theme(false);
    let src = "/* multi\n   line comment */\nfn main() {\n    let name = \"vgui\";\n    let count = 42;\n    let s = String::new();\n    println!(\"{}\", name);\n}";
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), move || {
            div()
                .w(px(480.))
                .child(code_editor(src, Language::Rust))
                .into_any_element()
        })
    });
    draw(&mut cx);
}
