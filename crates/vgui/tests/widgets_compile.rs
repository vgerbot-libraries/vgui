//! Compile/smoke test for button tab, img alt, textarea rows, and meter.

use gpui::{Element, IntoElement};
use vgui::prelude::*;
use vgui::view;

struct RenderScope;

impl RenderScope {
    fn new() -> Self {
        vgui::__test_enter_render_scope();
        RenderScope
    }
}

impl Drop for RenderScope {
    fn drop(&mut self) {
        vgui::__test_exit_render_scope();
    }
}

fn assert_into_any<E: IntoElement>(_: fn() -> E) {}

#[test]
fn widgets_compile_and_produce_elements() {
    let _scope = RenderScope::new();

    let button = view! { <button>{"Go"}</button> };
    let _ = button.into_any_element();

    let img = view! { <img src={"x.png"} alt={"logo"} /> };
    let _ = img.into_any_element();

    // `text_area` needs a VguiRoot slot; type-check `rows` without constructing.
    assert_into_any(|| view! { <textarea rows={4u32} /> });

    let meter = view! {
        <meter value={0.3f64} min={0f64} max={1f64} low={0.2f64} high={0.8f64} optimum={0.5f64} />
    };
    let _ = meter.into_any_element();
}
