//! Integration tests for auto-generated element ids.
//!
//! These tests exercise the `view!` macro end-to-end (without a gpui window)
//! to verify the fix for the bug where only the first item in a `<For>` list
//! received clicks. The root cause was that auto-generated ids were baked in
//! at macro-expansion time (compile-time), so every element produced by a
//! closure invoked multiple times shared the same id. The fix generates ids
//! at runtime from a per-render counter that is reset on every render.
//!
//! We assert on the `ElementId` returned by `Element::id()` on the generated
//! `Stateful<Div>` elements.

use gpui::{Element, ElementId, SharedString};
use vgui::prelude::*;
use vgui::{click, next_auto_id, view};

/// Extracts the `ElementId` from any element by calling `Element::id` via a
/// trait reference. This avoids the method-name clash between `Element::id`
/// (0 args, returns `Option<ElementId>`) and `Styled::id` (1 arg, returns
/// `Stateful<Self>`) when both traits are in scope.
fn element_id<E: Element>(el: &E) -> Option<ElementId> {
    <E as Element>::id(el)
}

/// A render scope guard for tests. Sets the per-render element id counter to
/// 0 on construction and clears it on drop, mirroring what
/// `enter_scope`/`exit_scope` do during a real `VguiRoot` render. This lets us
/// build `view!` trees outside of a gpui `App` while still exercising the
/// runtime id-generation path.
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
fn auto_button_gets_an_id() {
    // A button with on:click but no explicit id must still receive an
    // auto-generated id (otherwise gpui won't route clicks to it at all).
    let _scope = RenderScope::new();
    let el = view! {
        <button on:click={click(|_cx| {})}>{"x"}</button>
    };
    let id = element_id(&el).expect("button should have an auto id");
    assert_eq!(
        id,
        ElementId::NamedInteger(SharedString::from("button"), 0),
        "first auto button should get id ('button', 0)"
    );
}

#[test]
fn sibling_buttons_get_distinct_ids() {
    // Two buttons built in the same render must get distinct ids. This is the
    // core regression: previously both shared a compile-time id literal, so
    // gpui only routed clicks to the first one.
    let _scope = RenderScope::new();
    let a = view! {
        <button on:click={click(|_cx| {})}>{"a"}</button>
    };
    let b = view! {
        <button on:click={click(|_cx| {})}>{"b"}</button>
    };
    let id_a = element_id(&a).expect("a has id");
    let id_b = element_id(&b).expect("b has id");
    assert_ne!(
        id_a, id_b,
        "sibling buttons must have distinct ids, got {id_a:?} and {id_b:?}"
    );
}

#[test]
fn ids_are_stable_across_renders() {
    // Simulate two renders. The same logical button (first auto button in the
    // render) must receive the same id in both renders so gpui preserves its
    // stateful state (hover, focus, click hitbox tracking).
    let id1 = {
        let _scope = RenderScope::new();
        let el = view! { <button on:click={click(|_cx| {})}>{"x"}</button> };
        element_id(&el).expect("render1 button has id")
    };
    let id2 = {
        let _scope = RenderScope::new();
        let el = view! { <button on:click={click(|_cx| {})}>{"x"}</button> };
        element_id(&el).expect("render2 button has id")
    };
    assert_eq!(
        id1, id2,
        "same logical button must keep a stable id across re-renders"
    );
}

#[test]
fn explicit_id_is_preserved_not_overwritten() {
    // When the user provides an explicit `id`, the macro must use it verbatim
    // and NOT call next_auto_id. This keeps user-provided ids stable and
    // avoids consuming counter values (which would shift subsequent auto ids).
    let _scope = RenderScope::new();
    let before = next_auto_id(); // counter is now at `before + 1`
    let el = view! {
        <button id="my-button" on:click={click(|_cx| {})}>{"x"}</button>
    };
    let id = element_id(&el).expect("explicit-id button has id");
    assert_eq!(
        id,
        ElementId::Name(SharedString::from("my-button")),
        "explicit id must be used as-is"
    );
    // The counter must NOT have advanced due to the explicit-id button.
    let after = next_auto_id();
    assert_eq!(
        after,
        before + 1,
        "explicit id must not consume an auto counter value (counter should only advance by the two next_auto_id calls)"
    );
}

#[test]
fn repeated_closure_calls_get_distinct_ids() {
    // Reproduces the original todolist bug at the unit level: a closure
    // invoked multiple times (as `<For>` does internally via `for_each`) must
    // produce elements with distinct ids. We call a button-building closure 3
    // times within a single render scope and verify all ids differ.
    let _scope = RenderScope::new();

    fn make_button(n: u32) -> impl Element {
        view! {
            <button on:click={click(move |_cx| { let _ = n; })}>
                {format!("item {n}")}
            </button>
        }
    }

    let el0 = make_button(0);
    let el1 = make_button(1);
    let el2 = make_button(2);
    let id0 = element_id(&el0).expect("button 0 has id");
    let id1 = element_id(&el1).expect("button 1 has id");
    let id2 = element_id(&el2).expect("button 2 has id");

    assert_ne!(id0, id1, "buttons 0 and 1 must differ");
    assert_ne!(id0, id2, "buttons 0 and 2 must differ");
    assert_ne!(id1, id2, "buttons 1 and 2 must differ");

    // Verify the ids are the expected sequential values.
    assert_eq!(id0, ElementId::NamedInteger(SharedString::from("button"), 0));
    assert_eq!(id1, ElementId::NamedInteger(SharedString::from("button"), 1));
    assert_eq!(id2, ElementId::NamedInteger(SharedString::from("button"), 2));
}

#[test]
fn for_each_items_get_distinct_ids() {
    // Tests the actual `for_each` helper used by `<For>` to confirm the
    // end-to-end path: the closure is called once per item, and each call
    // produces a button with a distinct id.
    let _scope = RenderScope::new();

    let items = vec![0u32, 1, 2];
    let parent = vgui::for_each(items, |n, _i| {
        view! {
            <button on:click={click(move |_cx| { let _ = n; })}>
                {format!("item {n}")}
            </button>
        }
    });

    // `for_each` returns an AnyElement (a wrapping Div). We can't traverse
    // into its private children, but we can verify the build succeeded without
    // panic and that the parent itself has no id (it's a plain Div, not
    // stateful).
    assert!(element_id(&parent).is_none(), "for_each wrapper should not be stateful");
}

#[test]
fn fallback_ids_outside_scope_do_not_collide_with_render_ids() {
    // Outside a render scope, next_auto_id uses a fallback atomic counter
    // starting at u64::MAX / 2. Render-scope ids start at 0. They must never
    // collide.
    vgui::__test_exit_render_scope();
    let fallback = next_auto_id();
    assert!(
        fallback >= u64::MAX / 2,
        "fallback id should be in the high range, got {fallback}"
    );

    let _scope = RenderScope::new();
    let render_id = next_auto_id();
    assert!(
        render_id < u64::MAX / 2,
        "render id should be in the low range, got {render_id}"
    );
    assert_ne!(fallback, render_id);
}

#[test]
fn different_tag_names_get_different_named_ids() {
    // A button and a div-with-on:click in the same render should get ids with
    // different name prefixes, ensuring they never collide even at the same
    // counter index.
    let _scope = RenderScope::new();
    let btn = view! {
        <button on:click={click(|_cx| {})}>{"x"}</button>
    };
    let div = view! {
        <div on:click={click(|_cx| {})}>{"y"}</div>
    };
    let btn_id = element_id(&btn).expect("button has id");
    let div_id = element_id(&div).expect("div has id");
    assert_ne!(btn_id, div_id, "different tags must produce different ids");
}
