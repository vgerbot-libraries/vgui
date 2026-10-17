//! Headless draw test for the native Markdown renderer.
//!
//! Parsing is unit-tested in the component module; this verifies the full
//! block/inline tree renders through a real gpui window without panicking.

#![cfg(not(target_family = "wasm"))]

use gpui::{TestAppContext, VisualTestContext};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

#[test]
fn markdown_document_draws() {
    apply_theme(false);
    let src = "# Title\n\nA paragraph with **bold**, *italic* and `code`, plus [a link](https://example.com).\n\n- one\n- two\n\n1. first\n2. second\n\n> quoted text\n\n---\n\n```\nfn main() {}\n```";
    let doc = src.to_string();
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        let doc = doc.clone();
        vgui::VguiRoot::new(cx, window.window_handle(), move || markdown(doc.clone()))
    });
    draw(&mut cx);
}
