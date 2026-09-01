use std::cell::RefCell;
use std::rc::Rc;

use gpui::{AnyElement, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, WeakEntity, Window};

use crate::reactive::{enter_scope, exit_scope, set_viewport_width};

pub struct Scope {
    pub(crate) host: WeakEntity<VguiRoot>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) index: usize,
    pub(crate) initialized: bool,
    pub(crate) subscriptions: Vec<gpui::Subscription>,
    pub(crate) memos: Vec<Rc<dyn Fn(&mut Context<VguiRoot>)>>,
    pub(crate) memo_deps: Vec<Vec<gpui::EntityId>>,
    pub(crate) effects: Vec<EffectSlot>,
    /// `on:resize` handlers registered during the current render. Cleared and
    /// refilled on every render, then invoked by the window-bounds observer.
    pub(crate) resize_handlers:
        Vec<Rc<dyn Fn(&crate::event::ResizeEvent, &mut gpui::Window, &mut gpui::App)>>,
}

pub(crate) struct EffectSlot {
    pub deps: Vec<gpui::EntityId>,
    pub run: std::sync::Arc<dyn Fn()>,
}

pub(crate) enum Slot {
    Signal(std::sync::Arc<dyn std::any::Any>),
    Memo(std::sync::Arc<dyn std::any::Any>),
    Effect(std::sync::Arc<dyn std::any::Any>),
    Widget(std::sync::Arc<dyn std::any::Any>),
}

pub struct VguiRoot {
    pub(crate) scope: Rc<RefCell<Scope>>,
    render: Box<dyn FnMut() -> AnyElement>,
    /// Window-bounds observer subscription (registered once on first render,
    /// dispatches `on:resize` handlers on viewport-size change).
    resize_sub: Option<gpui::Subscription>,
}

impl VguiRoot {
    pub fn new<R: IntoElement + 'static>(
        cx: &mut Context<Self>,
        mut render: impl FnMut() -> R + 'static,
    ) -> Self {
        let scope = Rc::new(RefCell::new(Scope {
            host: cx.weak_entity(),
            slots: Vec::new(),
            index: 0,
            initialized: false,
            subscriptions: Vec::new(),
            memos: Vec::new(),
            memo_deps: Vec::new(),
            effects: Vec::new(),
            resize_handlers: Vec::new(),
        }));
        Self {
            scope,
            render: Box::new(move || render().into_any_element()),
            resize_sub: None,
        }
    }

    pub(crate) fn notify_dep(&mut self, id: gpui::EntityId, cx: &mut Context<Self>) {
        let (memos, memo_deps, effects) = {
            let scope = self.scope.borrow();
            (
                scope.memos.clone(),
                scope.memo_deps.clone(),
                scope
                    .effects
                    .iter()
                    .map(|e| (e.deps.clone(), e.run.clone()))
                    .collect::<Vec<_>>(),
            )
        };
        for (memo, deps) in memos.iter().zip(memo_deps.iter()) {
            if deps.contains(&id) {
                memo(cx);
            }
        }
        for (deps, run) in effects {
            if deps.contains(&id) {
                run();
            }
        }
        cx.notify();
    }
}

impl Render for VguiRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scope.borrow_mut().index = 0;
        // Refill `on:resize` handlers from this render's `view!` tree.
        self.scope.borrow_mut().resize_handlers.clear();
        // Register the window-bounds observer once; it dispatches the current
        // render's resize handlers on viewport-size change.
        if self.resize_sub.is_none() {
            self.resize_sub = Some(cx.observe_window_bounds(window, |root, window, cx| {
                let ev = ::vgui::event::ResizeEvent::from_window(window);
                let handlers = root.scope.borrow().resize_handlers.clone();
                for h in handlers {
                    h(&ev, window, cx);
                }
            }));
        }
        enter_scope(self.scope.clone(), cx);
        set_viewport_width(f32::from(window.viewport_size().width));
        let el = (self.render)();
        exit_scope();
        gpui::div()
            .on_key_down(|event: &gpui::KeyDownEvent, window: &mut Window, cx: &mut App| {
                if event.keystroke.key == "tab" {
                    if event.keystroke.modifiers.shift {
                        window.focus_prev(cx);
                    } else {
                        window.focus_next(cx);
                    }
                }
            })
            .child(el)
    }
}

pub fn mount<R: IntoElement + 'static>(
    cx: &mut App,
    render: impl FnMut() -> R + 'static,
) -> Entity<VguiRoot> {
    cx.new(|cx| VguiRoot::new(cx, render))
}
