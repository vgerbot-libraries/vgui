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
    /// Cleanup callbacks registered via `on_cleanup`. Run when the scope is
    /// disposed (e.g. when a `<Switch>` branch becomes inactive or an `<Index>`
    /// item is removed). Stored alongside `slots` so the slot-caching index
    /// stays consistent.
    pub(crate) cleanups: Vec<Rc<dyn Fn()>>,
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
    Cleanup(std::sync::Arc<dyn std::any::Any>),
    Widget(std::sync::Arc<dyn std::any::Any>),
}

/// Recursively collects a scope and all its descendants (depth-first, root
/// first). Used by `notify_dep` and the resize handler dispatcher so that
/// memos/effects/handlers living in nested child scopes (created by
/// `<Switch>`/`<Index>`) are reached.
pub(crate) fn collect_all_scopes(scope: &Rc<RefCell<Scope>>) -> Vec<Rc<RefCell<Scope>>> {
    let mut result = vec![scope.clone()];
    let children: Vec<_> = scope.borrow().children.values().cloned().collect();
    for child in children {
        result.extend(collect_all_scopes(&child));
    }
    result
}

/// Recursively disposes a scope: disposes children first (depth-first), runs
/// own cleanups, then clears all state. After disposal the scope is empty and
/// can be re-entered as if freshly created.
pub(crate) fn dispose_scope(scope: &Rc<RefCell<Scope>>) {
    // Collect children first to avoid holding a borrow during recursion.
    let children: Vec<_> = scope.borrow().children.values().cloned().collect();
    for child in children {
        dispose_scope(&child);
    }
    // Run cleanups (own only — children already ran theirs).
    let cleanups: Vec<Rc<dyn Fn()>> = scope.borrow().cleanups.iter().cloned().collect();
    for cleanup in cleanups {
        cleanup();
    }
    // Clear everything.
    let mut s = scope.borrow_mut();
    s.children.clear();
    s.slots.clear();
    s.subscriptions.clear();
    s.memos.clear();
    s.memo_deps.clear();
    s.effects.clear();
    s.cleanups.clear();
    s.resize_handlers.clear();
    s.index = 0;
    s.initialized = false;
}

/// Recursively collects resize handlers from a scope and all descendants.
fn collect_all_resize_handlers(
    scope: &Rc<RefCell<Scope>>,
) -> Vec<Rc<dyn Fn(&crate::event::ResizeEvent, &mut gpui::Window, &mut gpui::App)>> {
    let s = scope.borrow();
    let mut result: Vec<_> = s.resize_handlers.iter().cloned().collect();
    let children: Vec<_> = s.children.values().cloned().collect();
    drop(s);
    for child in children {
        result.extend(collect_all_resize_handlers(&child));
    }
    result
}

/// Recursively resets per-render state (`index`, `resize_handlers`) on a
/// scope and all its descendants. Called at the start of every render so that
/// each persistent child scope begins with a clean slot sequence.
fn reset_render_state(scope: &Rc<RefCell<Scope>>) {
    let children: Vec<_> = scope.borrow().children.values().cloned().collect();
    {
        let mut s = scope.borrow_mut();
        s.index = 0;
        s.resize_handlers.clear();
    }
    for child in children {
        reset_render_state(&child);
    }
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
            effects: Vec::new(),
            cleanups: Vec::new(),
            resize_handlers: Vec::new(),
            index: 0,
            initialized: false,
            subscriptions: Vec::new(),
            memos: Vec::new(),
            memo_deps: Vec::new(),
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
        let scopes = collect_all_scopes(&self.scope);
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
        // Reset per-render state on the root scope and all descendants. Child
        // scopes are persistent (keyed by name), so their slots survive route
        // switches, but their `index` and `resize_handlers` must be reset every
        // render. Nested child scopes (from <Switch>/<Index>) are reset
        // recursively.
        reset_render_state(&self.scope);
        // Register the window-bounds observer once; it dispatches the current
        // render's resize handlers (root + all children) on viewport-size change.
        if self.resize_sub.is_none() {
            self.resize_sub = Some(cx.observe_window_bounds(window, |root, window, cx| {
                let ev = ::vgui::event::ResizeEvent::from_window(window);
                let handlers = collect_all_resize_handlers(&root.scope);
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
