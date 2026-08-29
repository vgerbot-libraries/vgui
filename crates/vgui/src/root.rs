use std::cell::RefCell;
use std::rc::Rc;

use gpui::{AnyElement, App, AppContext, Context, Entity, InteractiveElement, IntoElement, ParentElement, Render, WeakEntity, Window};

use crate::reactive::{enter_scope, exit_scope};

pub struct Scope {
    pub(crate) host: WeakEntity<VguiRoot>,
    pub(crate) slots: Vec<Slot>,
    pub(crate) index: usize,
    pub(crate) initialized: bool,
    pub(crate) subscriptions: Vec<gpui::Subscription>,
    pub(crate) memos: Vec<Rc<dyn Fn(&mut Context<VguiRoot>)>>,
    pub(crate) memo_deps: Vec<Vec<gpui::EntityId>>,
    pub(crate) effects: Vec<EffectSlot>,
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
        }));
        Self {
            scope,
            render: Box::new(move || render().into_any_element()),
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
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.scope.borrow_mut().index = 0;
        enter_scope(self.scope.clone(), cx);
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
