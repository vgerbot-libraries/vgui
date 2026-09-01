use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, ClickEvent, Div, Window};

pub(crate) type FormHandler = Rc<RefCell<Option<Box<dyn FnMut(&mut App)>>>>;

#[derive(Clone)]
struct FormCtx {
    on_submit: Option<FormHandler>,
    on_reset: Option<FormHandler>,
}

thread_local! {
    static FORM_STACK: RefCell<Vec<FormCtx>> = const { RefCell::new(Vec::new()) };
}

fn box_handler(h: Option<Box<dyn FnMut(&mut App)>>) -> Option<FormHandler> {
    h.map(|f| Rc::new(RefCell::new(Some(f))))
}

fn invoke(handler: &Option<FormHandler>, cx: &mut App) {
    if let Some(cb) = handler {
        if let Some(f) = cb.borrow_mut().as_mut() {
            f(cx);
        }
    }
}

/// Push a form context, build `content` while it is current, then pop.
///
/// Children of `<form>` must be constructed inside `content` so submit/reset
/// buttons and text inputs can snapshot the enclosing handlers.
pub fn form_scope(
    on_submit: Option<Box<dyn FnMut(&mut App)>>,
    on_reset: Option<Box<dyn FnMut(&mut App)>>,
    content: impl FnOnce() -> Div,
) -> Div {
    FORM_STACK.with(|s| {
        s.borrow_mut().push(FormCtx {
            on_submit: box_handler(on_submit),
            on_reset: box_handler(on_reset),
        });
    });
    let div = content();
    FORM_STACK.with(|s| {
        s.borrow_mut().pop();
    });
    div
}

fn top() -> Option<FormCtx> {
    FORM_STACK.with(|s| s.borrow().last().cloned())
}

/// Invoke the enclosing form's `on:submit`. No-op when no form is on the stack.
pub fn __form_submit(cx: &mut App) {
    if let Some(ctx) = top() {
        invoke(&ctx.on_submit, cx);
    }
}

/// Invoke the enclosing form's `on:reset`. No-op when no form is on the stack.
pub fn __form_reset(cx: &mut App) {
    if let Some(ctx) = top() {
        invoke(&ctx.on_reset, cx);
    }
}

pub(crate) fn current_form_submit() -> Option<FormHandler> {
    top().and_then(|ctx| ctx.on_submit)
}

/// Click listener bound to the form that is current during construction.
pub fn __form_bind_submit() -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let handler = current_form_submit();
    move |_, _, cx| invoke(&handler, cx)
}

/// Click listener bound to the form that is current during construction.
pub fn __form_bind_reset() -> impl Fn(&ClickEvent, &mut Window, &mut App) + 'static {
    let handler = top().and_then(|ctx| ctx.on_reset);
    move |_, _, cx| invoke(&handler, cx)
}
