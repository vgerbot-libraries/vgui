//! Integration tests for the `ref={node_ref}` attribute and `NodeRef` handle.
//!
//! These tests exercise the `view!` macro end-to-end (without a gpui window)
//! to verify that `ref=` forces an auto-id on elements, preserves explicit
//! ids, and that `NodeRef::new()` panics on method calls before binding.

use std::panic::AssertUnwindSafe;

use gpui::{Element, ElementId, SharedString};
use vgui::prelude::*;
use vgui::{view, NodeRef};

/// Extracts the `ElementId` from any element — same helper as `element_id.rs`.
fn element_id<E: Element>(el: &E) -> Option<ElementId> {
    <E as Element>::id(el)
}

/// A render scope guard for tests — same as `element_id.rs`.
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
fn ref_forces_auto_id_on_div() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <div ref={r.clone()}>{"hello"}</div>
    };
    let id = element_id(&el).expect("ref= should force an auto id");
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("div"), 0),
        "first ref div should get id ('div', 0)"
    );
}

#[test]
fn ref_with_explicit_id_preserved() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <div id="my-id" ref={r.clone()}>{"hello"}</div>
    };
    let id = element_id(&el).expect("element should have an id");
    assert_eq!(
        id,
        ElementId::Name(SharedString::from("my-id")),
        "explicit id should be preserved when ref= is also present"
    );
}

#[test]
fn ref_on_button_works() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <button ref={r.clone()}>{"click"}</button>
    };
    let id = element_id(&el).expect("ref= on button should force an auto id");
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("button"), 0),
    );
}

#[test]
fn ref_on_checkbox_works() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <input type="checkbox" ref={r.clone()} />
    };
    let id = element_id(&el).expect("ref= on checkbox should force an auto id");
    // checkbox() internally consumes one auto-id, so the wrapper gets 1.
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("input"), 1),
    );
}

#[test]
fn ref_on_label_works() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <label ref={r.clone()}>{"text"}</label>
    };
    let id = element_id(&el).expect("ref= on label should force an auto id");
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("input"), 0),
    );
}

#[test]
fn ref_with_click_event_still_works() {
    let _scope = RenderScope::new();
    let r = NodeRef::new();
    let el = view! {
        <button ref={r.clone()} on:click={vgui::click(|_cx| {})}>{"x"}</button>
    };
    let id = element_id(&el).expect("element should have an id");
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("button"), 0),
    );
}

#[test]
fn noderef_new_unbound_panics() {
    let r = NodeRef::new();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        r.bounds();
    }));
    assert!(result.is_err(), "NodeRef::new() should panic on method calls before binding");
    let msg = result.unwrap_err();
    let msg = msg.downcast_ref::<String>().map(|s| s.as_str())
        .or_else(|| msg.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        msg.contains("not yet bound"),
        "panic message should mention 'not yet bound', got: {msg}"
    );
}

#[test]
fn noderef_new_focus_handle_panics() {
    let r = NodeRef::new();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let _ = r.focus_handle();
    }));
    assert!(result.is_err(), "focus_handle() should panic before binding");
}

#[test]
fn noderef_clone_shares_state() {
    let r1 = NodeRef::new();
    let r2 = r1.clone();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        r2.bounds();
    }));
    assert!(result.is_err(), "cloned NodeRef should also be unbound");
}

#[test]
fn noderef_default_is_unbound() {
    let r = NodeRef::default();
    let result = std::panic::catch_unwind(AssertUnwindSafe(|| {
        let _ = r.scroll_handle();
    }));
    assert!(result.is_err(), "NodeRef::default() should be unbound");
}

#[test]
fn ref_outside_render_scope_still_gets_id() {
    // Outside a render scope, __bind_ref returns false and the element
    // still gets a fallback id. The ref itself stays unbound.
    let r = NodeRef::new();
    let el = view! {
        <div ref={r.clone()}>{"hello"}</div>
    };
    let id = element_id(&el).expect("ref= should force an id even outside render scope");
    assert!(matches!(id, ElementId::NamedInteger(_, _)));
}
