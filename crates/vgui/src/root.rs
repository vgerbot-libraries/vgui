use std::cell::RefCell;
use std::collections::HashMap;
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
    /// Named child scopes (e.g. one per route). Persisted across renders so
    /// state created inside a child scope survives route switches; each child
    /// has its own independent slot sequence, so different branches can
    /// create signals/memos of different types without their slots colliding.
    pub(crate) children: HashMap<String, Rc<RefCell<Scope>>>,
    /// Parent scope. `None` for the root scope owned by `VguiRoot`.
    pub(crate) parent: Option<Rc<RefCell<Scope>>>,
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
            children: HashMap::new(),
            parent: None,
        }));
        Self {
            scope,
            render: Box::new(move || render().into_any_element()),
            resize_sub: None,
        }
    }

    pub(crate) fn notify_dep(&mut self, id: gpui::EntityId, cx: &mut Context<Self>) {
        // Collect this scope plus all descendant child scopes. Memos/effects
        // may live in a child scope (e.g. a route view), but the gpui observe
        // callback always fires on the root VguiRoot, so we must dispatch the
        // notification to every scope that might depend on `id`.
        let scopes: Vec<Rc<RefCell<Scope>>> = {
            let scope = self.scope.borrow();
            std::iter::once(self.scope.clone())
                .chain(scope.children.values().cloned())
                .collect()
        };
        for scope_rc in scopes {
            let (memos, memo_deps, effects) = {
                let scope = scope_rc.borrow();
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
        }
        cx.notify();
    }
}

impl Render for VguiRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Reset per-render state on the root scope and every child scope. Child
        // scopes are persistent (keyed by name), so their slots survive route
        // switches, but their `index` and `resize_handlers` must be reset every
        // render just like the root's.
        {
            let mut scope = self.scope.borrow_mut();
            scope.index = 0;
            scope.resize_handlers.clear();
            for child in scope.children.values() {
                let mut child_scope = child.borrow_mut();
                child_scope.index = 0;
                child_scope.resize_handlers.clear();
            }
        }
        // Register the window-bounds observer once; it dispatches the current
        // render's resize handlers (root + all children) on viewport-size change.
        if self.resize_sub.is_none() {
            self.resize_sub = Some(cx.observe_window_bounds(window, |root, window, cx| {
                let ev = ::vgui::event::ResizeEvent::from_window(window);
                let handlers: Vec<_> = {
                    let scope = root.scope.borrow();
                    let mut v: Vec<_> = scope.resize_handlers.iter().cloned().collect();
                    for c in scope.children.values() {
                        v.extend(c.borrow().resize_handlers.iter().cloned());
                    }
                    v
                };
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
