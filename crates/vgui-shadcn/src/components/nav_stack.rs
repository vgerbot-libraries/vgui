//! Push/pop navigation stack.
//!
//! Unlike the URL-based [`vgui::Router`], a [`NavStack`] keeps an in-memory
//! stack of pages (history). [`NavStack::push`] opens a new page,
//! [`NavStack::pop`] returns to the previous one, and [`nav_stack`] renders the
//! page on top. Reads of the stack subscribe the render scope, so pushing or
//! popping re-renders automatically.

use gpui::{AnyElement, App, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled};
use vgui::{create_signal, ReadSignal, WriteSignal};

/// Handle to a reactive page stack. Cloneable; all clones share one stack.
#[derive(Clone)]
pub struct NavStack<T: Clone + PartialEq + 'static> {
    read: ReadSignal<Vec<T>>,
    write: WriteSignal<Vec<T>>,
}

impl<T: Clone + PartialEq + 'static> NavStack<T> {
    /// Start a stack rooted at `root`.
    pub fn new(root: T) -> Self {
        let (read, write) = create_signal(vec![root]);
        Self { read, write }
    }

    /// Number of pages currently on the stack (always at least one).
    pub fn depth(&self) -> usize {
        self.read.get().len()
    }

    /// Whether there is a page to go back to.
    pub fn can_pop(&self) -> bool {
        self.read.get().len() > 1
    }

    /// The top page, if any.
    pub fn current(&self) -> Option<T> {
        self.read.get().last().cloned()
    }

    /// Push a new page on top.
    pub fn push(&self, cx: &mut App, page: T) {
        let mut pages = self.read.get();
        pages.push(page);
        self.write.set(cx, pages);
    }

    /// Pop the top page. The root page is never removed; returns the popped
    /// page when one was removed.
    pub fn pop(&self, cx: &mut App) -> Option<T> {
        let mut pages = self.read.get();
        if pages.len() <= 1 {
            return None;
        }
        let popped = pages.pop();
        self.write.set(cx, pages);
        popped
    }

    /// Collapse the stack back to a single root page.
    pub fn reset(&self, cx: &mut App, root: T) {
        self.write.set(cx, vec![root]);
    }
}

/// Render the page on top of `stack`.
///
/// `render(page, depth)` builds the element for a page; `depth` is the current
/// stack depth (1 at the root), useful for showing a back button.
pub fn nav_stack<T, F>(stack: NavStack<T>, render: F) -> AnyElement
where
    T: Clone + PartialEq + 'static,
    F: Fn(&T, usize) -> AnyElement + 'static,
{
    let pages = stack.read.get();
    let depth = pages.len();
    match pages.last() {
        Some(page) => render(page, depth),
        None => gpui::Empty.into_any_element(),
    }
}

/// Convenience: a back button that pops `stack`. Renders nothing (an empty
/// element) when the stack is at its root, so callers can always mount it.
pub fn nav_back_button<T: Clone + PartialEq + 'static>(
    stack: NavStack<T>,
    label: impl Into<String>,
) -> AnyElement {
    let label: String = label.into();
    if !stack.can_pop() {
        return gpui::Empty.into_any_element();
    }
    let for_click = stack.clone();
    let btn = gpui::div()
        .id(("nav-back", vgui::next_auto_id()))
        .cursor_pointer()
        .on_click(
            move |_: &gpui::ClickEvent, _w: &mut gpui::Window, cx: &mut App| {
                for_click.pop(cx);
            },
        )
        .child(label);
    vgui::css! {
        color: var(--muted-foreground);
        font-size: 13px;
        padding: 4px 8px;
    }
    .apply(btn)
    .into_any_element()
}
