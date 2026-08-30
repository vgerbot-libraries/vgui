use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use gpui::{App, AppContext, Entity, EntityId};

use crate::root::{Scope, Slot, VguiRoot};

thread_local! {
    static CURRENT: RefCell<Option<Current>> = const { RefCell::new(None) };
    static TRACKING: RefCell<Option<Vec<EntityId>>> = const { RefCell::new(None) };
    /// Per-render counter for auto-generated element ids. Set to `Some(0)` when
    /// a reactive scope is entered (i.e. at the start of every `VguiRoot`
    /// render) and cleared when the scope exits. While active, `next_auto_id`
    /// draws from this counter so that:
    /// - the same logical element gets a stable id across re-renders (counter
    ///   resets to 0 each render, call order is deterministic), preserving
    ///   gpui stateful state;
    /// - distinct elements — such as siblings produced by a `<For>` closure
    ///   invoked multiple times — receive distinct ids.
    static ELEMENT_ID_COUNTER: RefCell<Option<u64>> = const { RefCell::new(None) };
}

/// Fallback counter for auto-generated element ids when no reactive scope is
/// active (e.g. elements constructed in standalone tests). Uses a high starting
/// point to avoid collisions with the per-render counter (which starts at 0).
static FALLBACK_AUTO_ID: AtomicU64 = AtomicU64::new(u64::MAX / 2);

/// Returns a unique, deterministic id for an auto-generated stateful element.
///
/// Called by the `view!` macro for elements that need an id (those with
/// `on:click`, `hover`, `active`, `focus`, `class`, ...) but don't specify one
/// explicitly. Inside a `VguiRoot` render the id comes from a per-render counter
/// that is reset on every render, so the same logical element receives the same
/// id across re-renders (preserving gpui stateful state) while distinct elements
/// — such as siblings produced by a `<For>` closure invoked multiple times —
/// receive distinct ids.
pub fn next_auto_id() -> u64 {
    ELEMENT_ID_COUNTER.with(|c| {
        if let Some(counter) = c.borrow_mut().as_mut() {
            let id = *counter;
            *counter += 1;
            id
        } else {
            FALLBACK_AUTO_ID.fetch_add(1, Ordering::Relaxed)
        }
    })
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
                std::mem::transmute::<
                    &mut gpui::Context<VguiRoot>,
                    *mut gpui::Context<'static, VguiRoot>,
                >(cx)
            },
        });
    });
    // Reset the per-render element id counter. Each render starts from 0 so
    // that the same logical element gets a stable id across re-renders.
    ELEMENT_ID_COUNTER.with(|c| *c.borrow_mut() = Some(0));
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
    ELEMENT_ID_COUNTER.with(|c| *c.borrow_mut() = None);
}

fn current() -> Current {
    CURRENT.with(|c| {
        c.borrow()
            .as_ref()
            .cloned()
            .expect("create_signal must run inside VguiRoot / Scope::enter")
    })
}

/// Run a closure with the current VguiRoot context. Panics if not inside a
/// VguiRoot render (same precondition as `create_signal`).
pub(crate) fn with_root_cx<R>(f: impl FnOnce(&mut gpui::Context<VguiRoot>) -> R) -> R {
    let cur = current();
    let cx = unsafe { &mut *cur.cx };
    f(cx)
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
    cache: Rc<RefCell<T>>,
}

#[derive(Clone)]
pub struct WriteSignal<T> {
    entity: Entity<SignalCell<T>>,
    cache: Rc<RefCell<T>>,
}

impl<T: Clone + 'static> ReadSignal<T> {
    pub fn get(&self) -> T {
        TRACKING.with(|t| {
            if let Some(list) = t.borrow_mut().as_mut() {
                list.push(self.entity.entity_id());
            }
        });
        self.cache.borrow().clone()
    }

    pub fn get_with(&self, cx: &App) -> T {
        let value = self.entity.read(cx).0.clone();
        *self.cache.borrow_mut() = value.clone();
        value
    }
}

impl<T: Clone + PartialEq + 'static> WriteSignal<T> {
    pub fn set<C: AppContext>(&self, cx: &mut C, value: T) {
        {
            let cache = self.cache.borrow();
            if *cache == value {
                return;
            }
        }
        *self.cache.borrow_mut() = value.clone();
        let _ = self.entity.update(cx, |cell, cx| {
            cell.0 = value;
            cx.notify();
        });
    }

    pub fn update<C: AppContext, R>(
        &self,
        cx: &mut C,
        f: impl FnOnce(&mut T) -> R,
    ) -> R {
        let cache = self.cache.clone();
        self.entity.update(cx, |cell, cx| {
            let old = cell.0.clone();
            let r = f(&mut cell.0);
            if old != cell.0 {
                *cache.borrow_mut() = cell.0.clone();
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
    pub cache: Rc<RefCell<T>>,
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
    let cache = Rc::new(RefCell::new(initial));
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

/// Get-or-create a persistent gpui view entity, cached in a reactive scope
/// slot (mirroring `create_signal`). The same `Entity<T>` handle is returned
/// on every render so gpui keeps the view alive and its editing/drag state
/// persists across re-renders.
pub(crate) fn get_or_create_view<T: gpui::Render + 'static>(
    factory: impl FnOnce(&mut gpui::Context<VguiRoot>) -> gpui::Entity<T>,
) -> gpui::Entity<T> {
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            let stored = match &scope.slots[index] {
                Slot::Widget(stored) => stored.clone(),
                _ => panic!("vgui widget slot {index} changed type"),
            };
            scope.index += 1;
            drop(scope);
            return stored
                .downcast_ref::<gpui::Entity<T>>()
                .cloned()
                .unwrap_or_else(|| panic!("vgui widget slot {index} changed type"));
        }
    }
    let cx = unsafe { &mut *cur.cx };
    let entity = factory(cx);
    {
        let mut scope = cur.scope.borrow_mut();
        scope
            .slots
            .push(Slot::Widget(std::sync::Arc::new(entity.clone())));
        scope.index += 1;
    }
    entity
}

/// Get-or-create a persistent value cached in a reactive scope slot.
/// Same slot-caching mechanism as `get_or_create_view`, but for any
/// `Clone + 'static` type (e.g., `FocusHandle`, `Arc<DialogFocusState>`).
pub(crate) fn get_or_create_slot<T: Clone + 'static>(
    factory: impl FnOnce(&mut gpui::Context<VguiRoot>) -> T,
) -> T {
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            let stored = match &scope.slots[index] {
                Slot::Widget(stored) => stored.clone(),
                _ => panic!("vgui state slot {index} changed type"),
            };
            scope.index += 1;
            drop(scope);
            return stored
                .downcast_ref::<T>()
                .cloned()
                .unwrap_or_else(|| panic!("vgui state slot {index} changed type"));
        }
    }
    let cx = unsafe { &mut *cur.cx };
    let value = factory(cx);
    {
        let mut scope = cur.scope.borrow_mut();
        scope.slots.push(Slot::Widget(std::sync::Arc::new(value.clone())));
        scope.index += 1;
    }
    value
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
    let cache = Rc::new(RefCell::new(value));
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
        scope
            .memos
            .push(Rc::new(move |cx: &mut gpui::Context<VguiRoot>| {
                let new_value = (runtime.compute)();
                let old = runtime.cell.cache.borrow().clone();
                if old != new_value {
                    *runtime.cell.cache.borrow_mut() = new_value.clone();
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

#[doc(hidden)]
pub fn __test_enter_render_scope() {
    ELEMENT_ID_COUNTER.with(|c| *c.borrow_mut() = Some(0));
}

#[doc(hidden)]
pub fn __test_exit_render_scope() {
    ELEMENT_ID_COUNTER.with(|c| *c.borrow_mut() = None);
}

#[cfg(test)]
mod tests {
    use super::{__test_enter_render_scope as test_enter_render_scope, __test_exit_render_scope as test_exit_render_scope, next_auto_id, should_notify};

    #[test]
    fn equal_values_do_not_notify() {
        assert!(!should_notify(&1, &1));
        assert!(should_notify(&1, &2));
    }

    #[test]
    fn next_auto_id_is_sequential_within_a_render() {
        // Simulate entering a render scope (normally done by enter_scope).
        test_enter_render_scope();
        // Within one render, sequential calls produce sequential ids starting
        // at 0. This is what gives sibling elements (e.g. <For> items) distinct
        // ids.
        assert_eq!(next_auto_id(), 0);
        assert_eq!(next_auto_id(), 1);
        assert_eq!(next_auto_id(), 2);
        test_exit_render_scope();
    }

    #[test]
    fn next_auto_id_is_stable_across_renders() {
        // Simulate two consecutive renders. The counter resets to 0 at the
        // start of each render, so the same logical element (same call order)
        // receives the same id across re-renders. This preserves gpui stateful
        // state (focus, hover, etc.).
        test_enter_render_scope();
        let render1_a = next_auto_id();
        let render1_b = next_auto_id();
        test_exit_render_scope();

        test_enter_render_scope();
        let render2_a = next_auto_id();
        let render2_b = next_auto_id();
        test_exit_render_scope();

        assert_eq!(render1_a, render2_a);
        assert_eq!(render1_b, render2_b);
        assert_ne!(render1_a, render1_b);
    }

    #[test]
    fn next_auto_id_fallback_outside_scope_is_unique() {
        // Outside any render scope (e.g. elements built in standalone tests),
        // next_auto_id falls back to a global atomic counter. Two calls must
        // still produce distinct values, and they must not collide with the
        // per-render counter range (which starts at 0).
        test_exit_render_scope(); // ensure no scope is active
        let a = next_auto_id();
        let b = next_auto_id();
        assert_ne!(a, b);
        // Fallback ids start at u64::MAX / 2, well above the per-render range.
        assert!(a >= u64::MAX / 2);
        assert!(b >= u64::MAX / 2);
    }

    #[test]
    fn next_auto_id_does_not_leak_between_tests() {
        // Guards against a forgotten scope reset leaving the counter active for
        // unrelated tests. After exit, calls should use the fallback path.
        test_exit_render_scope();
        let outside = next_auto_id();
        assert!(outside >= u64::MAX / 2);
    }
}
