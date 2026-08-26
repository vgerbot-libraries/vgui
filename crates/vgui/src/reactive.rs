use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, RwLock};

use gpui::{App, AppContext, Entity, EntityId};

use crate::root::{Scope, Slot, VguiRoot};

thread_local! {
    static CURRENT: RefCell<Option<Current>> = const { RefCell::new(None) };
    static TRACKING: RefCell<Option<Vec<EntityId>>> = const { RefCell::new(None) };
}

struct Current {
    scope: Rc<RefCell<Scope>>,
    cx: *mut gpui::Context<'static, VguiRoot>,
}

pub(crate) fn enter_scope(scope: Rc<RefCell<Scope>>, cx: &mut gpui::Context<VguiRoot>) {
    CURRENT.with(|c| {
        *c.borrow_mut() = Some(Current {
            scope,
            cx: unsafe {
                std::mem::transmute::<&mut gpui::Context<VguiRoot>, *mut gpui::Context<'static, VguiRoot>>(
                    cx,
                )
            },
        });
    });
}

pub(crate) fn exit_scope() {
    CURRENT.with(|c| {
        if let Some(cur) = c.borrow().as_ref() {
            let mut scope = cur.scope.borrow_mut();
            if scope.initialized && scope.index != scope.slots.len() {
                panic!(
                    "vgui hook order changed: used {} slots, stored {}",
                    scope.index,
                    scope.slots.len()
                );
            }
            scope.initialized = true;
        }
        *c.borrow_mut() = None;
    });
}

fn current() -> Current {
    CURRENT.with(|c| {
        c.borrow()
            .as_ref()
            .cloned()
            .expect("create_signal must run inside VguiRoot / Scope::enter")
    })
}

impl Clone for Current {
    fn clone(&self) -> Self {
        Self {
            scope: self.scope.clone(),
            cx: self.cx,
        }
    }
}

pub struct SignalCell<T>(pub T);

#[derive(Clone)]
pub struct ReadSignal<T> {
    entity: Entity<SignalCell<T>>,
    cache: Arc<RwLock<T>>,
}

#[derive(Clone)]
pub struct WriteSignal<T> {
    entity: Entity<SignalCell<T>>,
    cache: Arc<RwLock<T>>,
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        TRACKING.with(|t| {
            if let Some(list) = t.borrow_mut().as_mut() {
                list.push(self.entity.entity_id());
            }
        });
        self.cache.read().expect("signal cache poisoned").clone()
    }

    pub fn get_with(&self, cx: &App) -> T {
        let value = self.entity.read(cx).0.clone();
        *self.cache.write().expect("signal cache poisoned") = value.clone();
        value
    }
}

impl<T: Clone + PartialEq + 'static> WriteSignal<T> {
    pub fn set<C: AppContext>(&self, cx: &mut C, value: T) {
        {
            let cache = self.cache.read().expect("signal cache poisoned");
            if *cache == value {
                return;
            }
        }
        *self.cache.write().expect("signal cache poisoned") = value.clone();
        let _ = self.entity.update(cx, |cell, cx| {
            cell.0 = value;
            cx.notify();
        });
    }

    pub fn update<C: AppContext, R>(&self, cx: &mut C, f: impl FnOnce(&mut T) -> R) -> C::Result<R> {
        let cache = self.cache.clone();
        self.entity.update(cx, |cell, cx| {
            let old = cell.0.clone();
            let r = f(&mut cell.0);
            if old != cell.0 {
                *cache.write().expect("signal cache poisoned") = cell.0.clone();
                cx.notify();
            }
            r
        })
    }
}

pub(crate) fn should_notify<T: PartialEq>(old: &T, new: &T) -> bool {
    old != new
}

fn with_tracking<R>(f: impl FnOnce() -> R) -> (R, Vec<EntityId>) {
    TRACKING.with(|t| *t.borrow_mut() = Some(Vec::new()));
    let result = f();
    let deps = TRACKING.with(|t| t.borrow_mut().take().unwrap_or_default());
    (result, deps)
}

#[derive(Clone)]
pub(crate) struct TypedCell<T> {
    pub entity: Entity<SignalCell<T>>,
    pub cache: Arc<RwLock<T>>,
}

struct MemoRuntime<T: Clone + PartialEq + 'static> {
    cell: TypedCell<T>,
    compute: Arc<dyn Fn() -> T>,
    deps: Vec<EntityId>,
}



pub fn create_signal<T: Clone + PartialEq + 'static>(
    initial: T,
) -> (ReadSignal<T>, WriteSignal<T>) {
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            let typed = match &scope.slots[index] {
                Slot::Signal(stored) => stored
                    .downcast_ref::<TypedCell<T>>()
                    .cloned()
                    .unwrap_or_else(|| panic!("vgui signal slot {index} changed type")),
                _ => panic!("vgui signal slot {index} changed type"),
            };
            scope.index += 1;
            return (
                ReadSignal {
                    entity: typed.entity.clone(),
                    cache: typed.cache.clone(),
                },
                WriteSignal {
                    entity: typed.entity,
                    cache: typed.cache,
                },
            );
        }
    }

    let cx = unsafe { &mut *cur.cx };
    let entity = cx.new(|_| SignalCell(initial.clone()));
    let cache = Arc::new(RwLock::new(initial));
    let sub = cx.observe(&entity, |this, observed, cx| {
        this.notify_dep(observed.entity_id(), cx);
    });
    let typed = TypedCell {
        entity: entity.clone(),
        cache: cache.clone(),
    };
    {
        let mut scope = cur.scope.borrow_mut();
        scope.slots.push(Slot::Signal(Arc::new(typed.clone())));
        scope.subscriptions.push(sub);
        scope.index += 1;
    }
    (
        ReadSignal {
            entity: entity.clone(),
            cache: cache.clone(),
        },
        WriteSignal { entity, cache },
    )
}

pub fn create_memo<T: Clone + PartialEq + 'static>(f: impl Fn() -> T + 'static) -> ReadSignal<T> {
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            let typed = match &scope.slots[index] {
                Slot::Memo(stored) => stored
                    .downcast_ref::<TypedCell<T>>()
                    .cloned()
                    .unwrap_or_else(|| panic!("vgui signal slot {index} changed type")),
                _ => panic!("vgui signal slot {index} changed type"),
            };
            scope.index += 1;
            return ReadSignal {
                entity: typed.entity,
                cache: typed.cache,
            };
        }
    }

    let compute = Arc::new(f);
    let (value, deps) = with_tracking(|| compute());
    let cx = unsafe { &mut *cur.cx };
    let entity = cx.new(|_| SignalCell(value.clone()));
    let cache = Arc::new(RwLock::new(value));
    let typed = TypedCell {
        entity: entity.clone(),
        cache: cache.clone(),
    };
    let runtime = Rc::new(MemoRuntime {
        cell: typed.clone(),
        compute: compute.clone(),
        deps: deps.clone(),
    });
    let sub = cx.observe(&entity, |this, observed, cx| {
        this.notify_dep(observed.entity_id(), cx);
    });
    {
        let mut scope = cur.scope.borrow_mut();
        scope.slots.push(Slot::Memo(Arc::new(typed.clone())));
        scope.subscriptions.push(sub);
        scope.memos.push(Rc::new(move |cx: &mut gpui::Context<VguiRoot>| {
            let new_value = (runtime.compute)();
            let old = runtime.cell.cache.read().expect("signal cache poisoned").clone();
            if old != new_value {
                *runtime.cell.cache.write().expect("signal cache poisoned") = new_value.clone();
                let _ = runtime.cell.entity.update(cx, |cell, cx| {
                    cell.0 = new_value;
                    cx.notify();
                });
            }
        }));
        scope.memo_deps.push(deps);
        scope.index += 1;
    }
    ReadSignal { entity, cache }
}

pub fn create_effect(f: impl Fn() + 'static) {
    // Runs at registration (during render), not after paint. Re-entrancy from the
    // first effect calling setters is user-visible and not deferred.
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            match &scope.slots[index] {
                Slot::Effect(_) => {}
                _ => panic!("vgui signal slot {index} changed type"),
            }
            scope.index += 1;
            return;
        }
    }

    let run = Arc::new(f);
    let ((), deps) = with_tracking(|| run());
    {
        let mut scope = cur.scope.borrow_mut();
        scope.slots.push(Slot::Effect(Arc::new(()) as Arc<dyn Any>));
        scope.effects.push(crate::root::EffectSlot {
            run: run.clone(),
            deps,
        });
        scope.index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::should_notify;

    #[test]
    fn equal_values_do_not_notify() {
        assert!(!should_notify(&1, &1));
        assert!(should_notify(&1, &2));
    }
}
