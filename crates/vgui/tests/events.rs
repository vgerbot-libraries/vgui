//! Integration test for the web-aligned DOM event attributes.
//!
//! Builds a `view!` tree exercising every new `on:` event name
//! (`keydown`, `keyup`, `pointerdown`, `pointerup`, `pointermove`, `resize`)
//! outside a real gpui render scope. `on:resize` no-ops when no scope is
//! active (`try_current` is `None`), so it must not panic. The other events
//! wire into gpui `InteractiveElement` listeners and must compile and build
//! without panicking.

use gpui::{Element, ElementId};
use vgui::prelude::*;
use vgui::view;

/// Extracts the `ElementId` from any element via the `Element::id` trait
/// method (avoids the `Styled::id` name clash).
fn element_id<E: Element>(el: &E) -> Option<ElementId> {
    <E as Element>::id(el)
}

/// A render scope guard mirroring `enter_scope`/`exit_scope` id-counter
/// behavior, copied from `tests/element_id.rs`.
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

#[test]
fn dom_events_compile_and_produce_element() {
    let _scope = RenderScope;
    // keydown/keyup/pointerdown/pointerup/pointermove are on InteractiveElement
    // (not Stateful), so they don't force an auto-id. The proof here is that
    // the macro wiring compiles and the element builds without panicking.
    let el = view! {
        <div
            on:keydown={move |_e: &KeyboardEvent, _w, _cx| {}}
            on:keyup={move |_e: &KeyboardEvent, _w, _cx| {}}
            on:pointerdown={move |_e: &PointerEvent, _w, _cx| {}}
            on:pointerup={move |_e: &PointerEvent, _w, _cx| {}}
            on:pointermove={move |_e: &PointerEvent, _w, _cx| {}}
            on:resize={move |_e: &ResizeEvent, _w, _cx| {}}
        >
            {"events"}
        </div>
    };
    // Converts to AnyElement without panic.
    let _any = el.into_any_element();
}

#[test]
fn resize_alone_does_not_force_id() {
    // `on:resize` is not an element event; it registers into the scope and
    // returns the element unchanged, so a bare `<div on:resize=…>` stays a
    // plain `Div` with no id.
    let _scope = RenderScope;
    let el = view! {
        <div on:resize={move |_e: &ResizeEvent, _w, _cx| {}}>
            {"resize only"}
        </div>
    };
    assert!(element_id(&el).is_none());
}
