use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use gpui::{App, AppContext, Entity, EntityId, IntoElement, ParentElement};

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
    /// Current viewport width in pixels, set during render for breakpoint
    /// resolution. `None` outside a render scope.
    static VIEWPORT_WIDTH: RefCell<Option<f32>> = const { RefCell::new(None) };
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
    VIEWPORT_WIDTH.with(|w| *w.borrow_mut() = None);
}

/// Set the current viewport width for breakpoint resolution. Called by
/// `VguiRoot::render` before entering the scope.
pub fn enter_child_scope(key: &str) {
    let cur = current();
    let child = {
        let mut scope = cur.scope.borrow_mut();
        let host = scope.host.clone();
        let parent = cur.scope.clone();
        scope
            .children
            .entry(key.to_string())
            .or_insert_with(|| {
                Rc::new(RefCell::new(Scope {
                    host,
                    slots: Vec::new(),
                    index: 0,
                    initialized: false,
                    subscriptions: Vec::new(),
                    memos: Vec::new(),
                    memo_deps: Vec::new(),
                    effects: Vec::new(),
                    cleanups: Vec::new(),
                    resize_handlers: Vec::new(),
                    children: std::collections::HashMap::new(),
                    parent: Some(parent),
                }))
            })
            .clone()
    };
    {
        let mut child_scope = child.borrow_mut();
        child_scope.index = 0;
        child_scope.resize_handlers.clear();
    }
    CURRENT.with(|c| {
        *c.borrow_mut() = Some(Current {
            scope: child,
            cx: cur.cx,
        });
    });
}

pub fn exit_child_scope() {
    let cur = current();
    let parent = {
        let mut scope = cur.scope.borrow_mut();
        if scope.initialized && scope.index != scope.slots.len() {
            panic!(
                "vgui hook order changed in child scope: used {} slots, stored {}",
                scope.index, scope.slots.len()
            );
        }
        scope.initialized = true;
        scope.parent.clone()
    };
    let parent = parent.expect("exit_child_scope called without a matching enter_child_scope");
    CURRENT.with(|c| {
        *c.borrow_mut() = Some(Current {
            scope: parent,
            cx: cur.cx,
        });
    });
}

pub(crate) fn set_viewport_width(width: f32) {
    VIEWPORT_WIDTH.with(|w| *w.borrow_mut() = Some(width));
}

/// Get the current viewport width, or `None` if outside a render scope.
#[doc(hidden)]
pub fn get_viewport_width() -> Option<f32> {
    VIEWPORT_WIDTH.with(|w| w.borrow().clone())
}

fn try_current() -> Option<Current> {
    CURRENT.with(|c| c.borrow().as_ref().cloned())
}

/// Public wrapper for macro-emitted `<Index>` code. Returns `true` when a
/// reactive scope is active so the macro can decide whether to call
/// `enter_child_scope` / `exit_child_scope`.
#[doc(hidden)]
pub fn __try_current() -> bool {
    try_current().is_some()
}

/// Register an `on:resize` handler into the current render scope. Called by
/// macro-emitted `on:resize` code. No-op outside a real render scope (tests),
/// so `on:resize` in test `view!`s won't panic.
#[doc(hidden)]
pub fn __register_resize_handler(
    handler: impl Fn(&crate::event::ResizeEvent, &mut gpui::Window, &mut gpui::App) + 'static,
) {
    if let Some(cur) = try_current() {
        cur.scope
            .borrow_mut()
            .resize_handlers
            .push(Rc::new(handler));
    }
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

// ---------------------------------------------------------------------------
// createStore — SolidJS-style fine-grained reactive store.
//
// Unlike `create_signal` (a single flat value), a store wraps an aggregate
// state tree. Writes always notify — the store does not require `PartialEq`
// on `T` — and fine-grained reactivity is achieved through `Store::select`,
// which creates a memo that recomputes when the store changes but only
// propagates downstream when the selected slice differs (via `PartialEq` on
// the slice type `U`). This mirrors SolidJS path-level tracking: reading a
// store field through `select` subscribes to that slice alone.
//
// Rust adaptation: instead of JS Proxy-based deep reactivity (intercepting
// property access at runtime), the user explicitly selects the slice they
// care about. This is type-safe, zero-magic, and idiomatic — the closure
// `|state| state.user.name.clone()` is a lens, not a string path.
// ---------------------------------------------------------------------------
//
/// Read handle for a reactive store. Cheap to clone.
///
/// Created by [`create_store`]. Use [`Store::get`] to read the whole state
/// (tracks the entire store as a dependency), [`Store::with`] to borrow
/// without cloning, or [`Store::select`] to derive a fine-grained signal
/// that only updates when a specific slice changes.
#[derive(Clone)]
pub struct Store<T: Clone + 'static> {
    entity: Entity<SignalCell<T>>,
    cache: Rc<RefCell<T>>,
}

/// Write handle for a reactive store. Cheap to clone.
///
/// Created by [`create_store`]. Use [`SetStore::set`] to replace the entire
/// state or [`SetStore::update`] to mutate in place. Both always notify —
/// use [`Store::select`] downstream to filter reactivity to the slices that
/// actually changed.
#[derive(Clone)]
pub struct SetStore<T: Clone + 'static> {
    entity: Entity<SignalCell<T>>,
    cache: Rc<RefCell<T>>,
}

impl<T: Clone + 'static> Store<T> {
    /// Read a clone of the current state, tracking the entire store as a
    /// dependency of the enclosing memo/effect.
    pub fn get(&self) -> T {
        track_entity(&self.entity);
        self.cache.borrow().clone()
    }

    /// Borrow the state through a closure without cloning, tracking the
    /// entire store as a dependency.
    pub fn with<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        track_entity(&self.entity);
        f(&self.cache.borrow())
    }

    /// Read the fresh value directly from the gpui entity, refreshing the
    /// local cache. Use outside a render scope when the cache may be stale
    /// (e.g. from an event handler that ran after the last render).
    pub fn get_with(&self, cx: &App) -> T {
        let value = self.entity.read(cx).0.clone();
        *self.cache.borrow_mut() = value.clone();
        value
    }

    /// Derive a fine-grained signal from a slice of the store state.
    ///
    /// The returned [`ReadSignal`] recomputes whenever the store changes but
    /// only notifies its own subscribers when the selected value differs
    /// (requiring `U: PartialEq`). This is the Rust-idiomatic equivalent of
    /// SolidJS path-level tracking: the closure acts as a lens.
    ///
    /// ```ignore
    /// let (store, set_store) = create_store(AppState::default());
    /// let name = store.select(|s| s.user.name.clone());
    /// // name: ReadSignal<String> — only updates when user.name changes.
    /// ```
    pub fn select<U: Clone + PartialEq + 'static>(
        &self,
        f: impl Fn(&T) -> U + 'static,
    ) -> ReadSignal<U> {
        let entity = self.entity.clone();
        let cache = self.cache.clone();
        create_memo(move || {
            track_entity(&entity);
            f(&cache.borrow())
        })
    }
}

impl<T: Clone + 'static> SetStore<T> {
    /// Replace the entire state. Always notifies — callers that need
    /// equality-skipping should use [`create_signal`] instead.
    pub fn set<C: AppContext>(&self, cx: &mut C, value: T) {
        *self.cache.borrow_mut() = value.clone();
        let _ = self.entity.update(cx, |cell, cx| {
            cell.0 = value;
            cx.notify();
        });
    }

    /// Mutate the state in place through a closure. Always notifies after
    /// the closure returns, even if the mutation was a no-op — downstream
    /// [`Store::select`] signals filter out unchanged slices.
    pub fn update<C: AppContext, R>(
        &self,
        cx: &mut C,
        f: impl FnOnce(&mut T) -> R,
    ) -> R {
        let cache = self.cache.clone();
        self.entity.update(cx, |cell, cx| {
            let r = f(&mut cell.0);
            *cache.borrow_mut() = cell.0.clone();
            cx.notify();
            r
        })
    }
}

/// Track a gpui entity as a dependency of the enclosing memo/effect.
fn track_entity<T: 'static>(entity: &Entity<T>) {
    TRACKING.with(|t| {
        if let Some(list) = t.borrow_mut().as_mut() {
            list.push(entity.entity_id());
        }
    });
}

/// Create a reactive store wrapping an aggregate state tree.
///
/// Returns a (`[`Store`]`, `[`SetStore`]`) pair, analogous to
/// [`create_signal`] but designed for state that is:
///
/// - **Aggregate** — a single struct or enum holding multiple fields, rather
///   than one signal per field.
/// - **Fine-grained** — use [`Store::select`] to derive signals for
///   individual slices; only the slices that actually changed propagate.
/// - **Not `PartialEq`** — the store does not compare old vs new state on
///   write (writes always notify). Filtering happens at the `select` level
///   via `PartialEq` on the slice type.
///
/// Like other reactive primitives, this must be called inside a
/// `VguiRoot` render scope. The store is cached in a scope slot and
/// persists across re-renders.
pub fn create_store<T: Clone + 'static>(initial: T) -> (Store<T>, SetStore<T>) {
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            let typed = match &scope.slots[index] {
                Slot::Signal(stored) => stored
                    .downcast_ref::<TypedCell<T>>()
                    .cloned()
                    .unwrap_or_else(|| panic!("vgui store slot {index} changed type")),
                _ => panic!("vgui store slot {index} changed type"),
            };
            scope.index += 1;
            return (
                Store {
                    entity: typed.entity.clone(),
                    cache: typed.cache.clone(),
                },
                SetStore {
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
        Store {
            entity: entity.clone(),
            cache: cache.clone(),
        },
        SetStore {
            entity,
            cache,
        },
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

/// Like `get_or_create_slot`, but returns `None` when no reactive scope is
/// active (e.g. constructing elements in standalone tests). The factory is
/// only called when a scope is active.
pub(crate) fn try_get_or_create_slot<T: Clone + 'static>(
    factory: impl FnOnce(&mut gpui::Context<VguiRoot>) -> T,
) -> Option<T> {
    let cur = try_current()?;
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
            .map(Some)
            .unwrap_or_else(|| panic!("vgui state slot {index} changed type"));
    }
    drop(scope);
    let cx = unsafe { &mut *cur.cx };
    let value = factory(cx);
    let mut scope = cur.scope.borrow_mut();
    scope.slots.push(Slot::Widget(std::sync::Arc::new(value.clone())));
    scope.index += 1;
    Some(value)
}

/// Bind a [`crate::ref_handle::NodeRef`] to the current reactive scope slot,
/// creating the `FocusHandle` and `ScrollHandle` on first render and returning
/// them on subsequent renders. Called by `view!` macro-emitted code when a
/// `ref={node_ref}` attribute is present.
///
/// Returns `true` when the handles were bound (a reactive scope is active),
/// or `false` when no scope is active (e.g. constructing elements in
/// standalone tests). In the `false` case the `NodeRef` remains unbound and
/// the caller should skip `track_focus`/`track_scroll`.
#[doc(hidden)]
pub fn __bind_ref(node_ref: &crate::ref_handle::NodeRef) -> bool {
    if try_current().is_none() {
        return false;
    }
    let handles: std::sync::Arc<crate::ref_handle::NodeRefHandles> = get_or_create_slot(|cx| {
        std::sync::Arc::new(crate::ref_handle::NodeRefHandles {
            focus: cx.focus_handle(),
            scroll: gpui::ScrollHandle::new(),
        })
    });
    node_ref.bind(&handles);
    true
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

/// Register a cleanup callback that runs when the current scope is disposed.
///
/// Uses the same slot-caching pattern as `create_effect`: on re-renders the
/// slot is recognised by position and the callback is not re-registered. The
/// callback runs once when the scope is disposed via `dispose_scope` (e.g.
/// when a `<Switch>` branch becomes inactive or an `<Index>` item is removed).
///
/// No-op when no reactive scope is active (e.g. in standalone tests), so
/// `on_cleanup` in test `view!`s won't panic.
pub fn on_cleanup(f: impl Fn() + 'static) {
    if try_current().is_none() {
        return;
    }
    let cur = current();
    {
        let mut scope = cur.scope.borrow_mut();
        let index = scope.index;
        if index < scope.slots.len() {
            match &scope.slots[index] {
                Slot::Cleanup(_) => {}
                _ => panic!("vgui slot {index} changed type"),
            }
            scope.index += 1;
            return;
        }
    }
    {
        let mut scope = cur.scope.borrow_mut();
        scope.slots.push(Slot::Cleanup(Arc::new(()) as Arc<dyn Any>));
        scope.cleanups.push(Rc::new(f));
        scope.index += 1;
    }
}

/// Dispose child scopes for inactive `<Switch>` branches. Called by
/// macro-emitted code before entering the active branch. No-op without a
/// reactive scope (test compatibility).
#[doc(hidden)]
pub fn __switch_dispose_inactive(switch_id: u64, active: Option<usize>, branch_count: usize) {
    if try_current().is_none() {
        return;
    }
    let cur = current();
    let to_dispose: Vec<Rc<RefCell<Scope>>> = {
        let mut scope = cur.scope.borrow_mut();
        (0..branch_count)
            .filter(|&b| active != Some(b))
            .filter_map(|b| {
                let key = format!("switch:{}:{}", switch_id, b);
                scope.children.remove(&key)
            })
            .collect()
    };
    for child in to_dispose {
        crate::root::dispose_scope(&child);
    }
}

/// Enter the child scope for a `<Switch>` branch. No-op without a reactive
/// scope (test compatibility).
#[doc(hidden)]
pub fn __switch_enter_branch(switch_id: u64, branch: usize) {
    if try_current().is_none() {
        return;
    }
    let key = format!("switch:{}:{}", switch_id, branch);
    enter_child_scope(&key);
}

/// Exit the current `<Switch>` branch child scope. No-op without a reactive
/// scope (test compatibility).
#[doc(hidden)]
pub fn __switch_exit_branch() {
    if try_current().is_none() {
        return;
    }
    exit_child_scope();
}

/// Dispose child scopes for `<Index>` positions >= `active_count`. Called by
/// macro-emitted code after rendering all current items. No-op without a
/// reactive scope (test compatibility).
#[doc(hidden)]
pub fn __index_dispose_excess(list_id: u64, active_count: usize) {
    if try_current().is_none() {
        return;
    }
    let cur = current();
    let prefix = format!("index:{}:", list_id);
    let to_dispose: Vec<Rc<RefCell<Scope>>> = {
        let mut scope = cur.scope.borrow_mut();
        let keys: Vec<String> = scope
            .children
            .keys()
            .filter(|k| {
                k.starts_with(&prefix)
                    && k[prefix.len()..]
                        .parse::<usize>()
                        .map(|idx| idx >= active_count)
                        .unwrap_or(false)
            })
            .cloned()
            .collect();
        keys.into_iter()
            .filter_map(|k| scope.children.remove(&k))
            .collect()
    };
    for child in to_dispose {
        crate::root::dispose_scope(&child);
    }
}

/// Keyed-by-position list primitive. Each item gets its own persistent child
/// scope (keyed by `index:{list_id}:{position}`), so state created inside the
/// closure (signals, memos, effects) survives re-renders as long as the item
/// remains at the same position. When the list shrinks, excess scopes are
/// disposed and their `on_cleanup` callbacks run.
pub fn index_list<T, E: gpui::IntoElement>(
    items: impl IntoIterator<Item = T>,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let list_id = next_auto_id();
    let has_scope = try_current().is_some();
    let mut parent = gpui::div();
    let mut n = 0;
    for (i, item) in items.into_iter().enumerate() {
        if has_scope {
            let key = format!("index:{}:{}", list_id, i);
            enter_child_scope(&key);
        }
        let el = child(item, i);
        if has_scope {
            exit_child_scope();
        }
        parent = parent.child(el);
        n += 1;
    }
    if has_scope {
        __index_dispose_excess(list_id, n);
    }
    if n == 0 {
        gpui::Empty.into_any_element()
    } else {
        parent.into_any_element()
    }
}

/// Like `index_list` but renders `fallback` when the iterator is empty.
pub fn index_list_or<T, E: gpui::IntoElement, F: gpui::IntoElement>(
    items: impl IntoIterator<Item = T>,
    fallback: F,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let list_id = next_auto_id();
    let has_scope = try_current().is_some();
    let mut parent = gpui::div();
    let mut n = 0;
    for (i, item) in items.into_iter().enumerate() {
        if has_scope {
            let key = format!("index:{}:{}", list_id, i);
            enter_child_scope(&key);
        }
        let el = child(item, i);
        if has_scope {
            exit_child_scope();
        }
        parent = parent.child(el);
        n += 1;
    }
    if has_scope {
        __index_dispose_excess(list_id, n);
    }
    if n == 0 {
        fallback.into_any_element()
    } else {
        parent.into_any_element()
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
