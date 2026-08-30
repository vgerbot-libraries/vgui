//! Label association: maps input `id` strings to `FocusHandle`s so that
//! `<label for="id">` can focus the target control on click, and collects
//! focus handles from children of a wrapping `<label>` at render time.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{App, FocusHandle, Window};

/// A target registered in a wrapping `<label>` scope.
///
/// `click_action` is `Some` for inputs whose primary interaction is more than
/// focusing — checkbox toggle, radio select, select cycle, file picker.  It is
/// `None` for text-like inputs where focusing alone is the desired behaviour.
pub struct LabelTarget {
    pub focus_handle: FocusHandle,
    pub click_action: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
}

thread_local! {
    static REGISTRY: RefCell<HashMap<String, FocusHandle>> = RefCell::new(HashMap::new());
    static SCOPE_STACK: RefCell<Vec<Vec<LabelTarget>>> = RefCell::new(Vec::new());
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

/// Pop the scope stack, return the first registered `LabelTarget` (first
/// focusable child). Called after rendering label children.
#[doc(hidden)]
pub fn label_scope_exit() -> Option<LabelTarget> {
    SCOPE_STACK.with(|s| s.borrow_mut().pop().and_then(|v| v.into_iter().next()))
}

/// If inside a label scope, push a handle with no click action (text-like
/// inputs where focusing is sufficient). Called by text_input/range_input.
pub(crate) fn register_in_label_scope(handle: FocusHandle) {
    register_in_label_scope_with_action(handle, None);
}

/// If inside a label scope, push a handle with an optional click action
/// (checkbox, radio, select, file). The click action is invoked by the
/// wrapping label's `on_mouse_down` when the user clicks the label text
/// rather than the input itself.
pub(crate) fn register_in_label_scope_with_action(
    handle: FocusHandle,
    action: Option<Rc<dyn Fn(&mut Window, &mut App)>>,
) {
    SCOPE_STACK.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            top.push(LabelTarget {
                focus_handle: handle,
                click_action: action,
            });
        }
    });
}
