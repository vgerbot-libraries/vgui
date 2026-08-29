//! Label association: maps input `id` strings to `FocusHandle`s so that
//! `<label for="id">` can focus the target control on click, and collects
//! focus handles from children of a wrapping `<label>` at render time.

use std::cell::RefCell;
use std::collections::HashMap;

use gpui::{App, FocusHandle, Window};

thread_local! {
    static REGISTRY: RefCell<HashMap<String, FocusHandle>> = RefCell::new(HashMap::new());
    static SCOPE_STACK: RefCell<Vec<Vec<FocusHandle>>> = RefCell::new(Vec::new());
}

/// Register a focus handle under an id string (called by text_input/range_input
/// when they have an `id` prop). Used by `<label for="id">` at click time.
pub fn register_label_target(id: &str, handle: FocusHandle) {
    REGISTRY.with(|r| {
        r.borrow_mut().insert(id.to_string(), handle);
    });
}

/// Look up a focus handle by id and focus it. Called from label's on_mouse_down.
pub fn focus_label_target(id: &str, window: &mut Window, cx: &mut App) {
    let handle = REGISTRY.with(|r| r.borrow().get(id).cloned());
    if let Some(handle) = handle {
        window.focus(&handle, cx);
    }
}

/// Push a new collection onto the scope stack. Called before rendering label
/// children.
#[doc(hidden)]
pub fn __label_scope_enter() {
    SCOPE_STACK.with(|s| s.borrow_mut().push(Vec::new()));
}

/// Pop the scope stack, return the first registered `FocusHandle` (first
/// focusable child). Called after rendering label children.
#[doc(hidden)]
pub fn label_scope_exit() -> Option<FocusHandle> {
    SCOPE_STACK.with(|s| s.borrow_mut().pop().and_then(|v| v.into_iter().next()))
}

/// If inside a label scope, push handle onto the current collection.
/// Called by text_input/range_input during entity creation.
pub(crate) fn register_in_label_scope(handle: FocusHandle) {
    SCOPE_STACK.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            top.push(handle);
        }
    });
}
