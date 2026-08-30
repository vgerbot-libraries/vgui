//! SolidJS-style `ref` handle for vgui.
//!
//! gpui rebuilds its element tree every frame, so there is no persistent DOM
//! node to hand back like SolidJS's `ref`. Instead, `NodeRef` wraps the
//! persistent gpui handles (`FocusHandle` + `ScrollHandle`) that survive across
//! frames. The `view!` macro binds a `NodeRef` to an element via the
//! `ref={node_ref}` attribute; the macro-emitted code calls
//! [`crate::reactive::__bind_ref`] during render to populate the handles from
//! the reactive scope slot.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Bounds, FocusHandle, Pixels, Point, ScrollHandle, Window};

/// The pair of gpui handles cached in a reactive scope slot for a `NodeRef`.
///
/// Stored as `Arc` so it is `Clone + 'static` and fits the
/// `get_or_create_slot` caching mechanism. Created on first render and reused
/// on every subsequent render for the same logical element.
pub(crate) struct NodeRefHandles {
    pub(crate) focus: FocusHandle,
    pub(crate) scroll: ScrollHandle,
}

/// A persistent handle to a rendered element, inspired by SolidJS `ref`.
///
/// Created via [`NodeRef::new()`] (or obtained from a `view!` `ref=` attribute).
/// The handle wraps a [`FocusHandle`] and [`ScrollHandle`] that gpui populates
/// during layout/paint. Methods read state from the **previous frame** —
/// sufficient for layout calculations and imperative actions.
///
/// A `NodeRef` is an empty shell until it is bound to an element by the
/// `view!` macro during render. Calling any method before the first render
/// panics with a clear message, mirroring SolidJS where `ref` is `undefined`
/// until mount.
///
/// # Example
///
/// ```ignore
/// let my_ref = NodeRef::new();
/// view! {
///     <div ref={my_ref.clone()}>"hello"</div>
/// }
/// // later, inside an event handler:
/// my_ref.focus(window, cx);
/// let b = my_ref.bounds();
/// ```
#[derive(Clone)]
pub struct NodeRef {
    inner: Rc<RefCell<NodeRefInner>>,
}

struct NodeRefInner {
    focus_handle: Option<FocusHandle>,
    scroll_handle: Option<ScrollHandle>,
}

impl NodeRef {
    /// Create an unbound `NodeRef` shell.
    ///
    /// The handle remains unbound until the `view!` macro emits a
    /// `ref={self}` attribute, at which point [`__bind_ref`](crate::reactive::__bind_ref)
    /// populates the inner handles from the reactive scope slot.
    pub fn new() -> Self {
        NodeRef {
            inner: Rc::new(RefCell::new(NodeRefInner {
                focus_handle: None,
                scroll_handle: None,
            })),
        }
    }
    pub(crate) fn bind(&self, handles: &NodeRefHandles) {
        let mut inner = self.inner.borrow_mut();
        inner.focus_handle = Some(handles.focus.clone());
        inner.scroll_handle = Some(handles.scroll.clone());
    }

    fn with_focus<R>(&self, f: impl FnOnce(&FocusHandle) -> R, _name: &str) -> R {
        let inner = self.inner.borrow();
        match &inner.focus_handle {
            Some(h) => f(h),
            None => panic!(
                "NodeRef not yet bound to an element; it must be used as \
                 `ref={{...}}` on an element in `view!` before calling methods"
            ),
        }
    }

    fn with_scroll<R>(&self, f: impl FnOnce(&ScrollHandle) -> R) -> R {
        let inner = self.inner.borrow();
        match &inner.scroll_handle {
            Some(h) => f(h),
            None => panic!(
                "NodeRef not yet bound to an element; it must be used as \
                 `ref={{...}}` on an element in `view!` before calling methods"
            ),
        }
    }

    /// Move keyboard focus to the bound element.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        self.with_focus(|h| h.focus(window, cx), "focus");
    }

    /// Whether the bound element is currently focused.
    pub fn is_focused(&self, window: &Window) -> bool {
        self.with_focus(|h| h.is_focused(window), "is_focused")
    }

    /// Whether the bound element contains the focused element (or is focused).
    pub fn contains_focused(&self, window: &Window, cx: &App) -> bool {
        self.with_focus(|h| h.contains_focused(window, cx), "contains_focused")
    }

    /// The painted bounds of the bound element from the previous frame.
    pub fn bounds(&self) -> Bounds<Pixels> {
        self.with_scroll(|h| h.bounds())
    }

    /// The current scroll offset (distance from the container's top-left to
    /// the first child's top-left; more negative as you scroll down).
    pub fn scroll_offset(&self) -> Point<Pixels> {
        self.with_scroll(|h| h.offset())
    }

    /// Scroll so that child `ix` is visible (minimal scroll).
    pub fn scroll_to(&self, ix: usize) {
        self.with_scroll(|h| h.scroll_to_item(ix))
    }

    /// Scroll so that child `ix` is the first visible element.
    pub fn scroll_to_top(&self, ix: usize) {
        self.with_scroll(|h| h.scroll_to_top_of_item(ix))
    }

    /// Scroll to the bottom of the scrollable content.
    pub fn scroll_to_bottom(&self) {
        self.with_scroll(|h| h.scroll_to_bottom())
    }

    /// Set the scroll offset explicitly.
    pub fn set_scroll_offset(&self, offset: Point<Pixels>) {
        self.with_scroll(|h| h.set_offset(offset))
    }

    /// The painted bounds of child `ix`, or `None` if out of range.
    pub fn child_bounds(&self, ix: usize) -> Option<Bounds<Pixels>> {
        self.with_scroll(|h| h.bounds_for_item(ix))
    }

    /// The number of tracked children.
    pub fn child_count(&self) -> usize {
        self.with_scroll(|h| h.children_count())
    }

    /// Clone the underlying `FocusHandle` (panics if unbound).
    pub fn focus_handle(&self) -> FocusHandle {
        self.with_focus(|h| h.clone(), "focus_handle")
    }

    /// Clone the underlying `ScrollHandle` (panics if unbound).
    pub fn scroll_handle(&self) -> ScrollHandle {
        self.with_scroll(|h| h.clone())
    }
}

impl Default for NodeRef {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for NodeRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.borrow();
        f.debug_struct("NodeRef")
            .field("bound", &inner.focus_handle.is_some())
            .finish()
    }
}
