//! Context / Provider — SolidJS-equivalent dependency injection.
//!
//! vgui renders synchronously, depth-first, in a single flat per-render
//! `Scope`. There is no nested owner tree, so context hierarchy is
//! implemented with a **thread-local stack** of provided values — the same
//! pattern used by `<radiogroup>` (`__radiogroup_scope_enter`/`exit`) and
//! `<label>` (`__label_scope_enter`/`exit`): the `view!` macro emits `enter`
//! *before* evaluating children and `exit` *after*, so descendants
//! constructed between them see the pushed value.
//!
//! Keying is by `TypeId`: `Context<T>` is a zero-sized, `const`-constructable
//! typed marker stored in a plain `static`. One context per type; use newtype
//! wrappers for multiple contexts of the same logical type.

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::sync::Arc;

thread_local! {
    static CONTEXT_STACK: RefCell<Vec<(TypeId, Arc<dyn Any>)>> =
        const { RefCell::new(Vec::new()) };
}

/// Zero-sized, const-constructable typed marker identifying a context.
///
/// Keyed by `TypeId` of `T`; one context per type (use newtypes for multiple
/// contexts of the same logical type). Store in a plain `static`:
///
/// ```ignore
/// static THEME: Context<Theme> = Context::new();
/// ```
///
/// `PhantomData<fn() -> T>` is `Copy`/`Clone` and `Send + Sync` regardless of
/// `T`, so `Context<T>` needs no bounds on `T` to live in a `static`.
pub struct Context<T> {
    _marker: PhantomData<fn() -> T>,
}

impl<T: 'static> Context<T> {
    pub const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<T> Clone for Context<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for Context<T> {}
impl<T: 'static> Default for Context<T> {
    fn default() -> Self {
        Self::new()
    }
}

// Hidden macro-facing enter/exit — mirror `__radiogroup_scope_enter`/`exit`.
// The `<Provider>` builtin calls `enter` before evaluating children and `exit`
// after, so descendants constructed between them observe the pushed value.

#[doc(hidden)]
pub fn __provider_scope_enter<T: Clone + 'static>(_ctx: &Context<T>, value: T) {
    CONTEXT_STACK.with(|s| s.borrow_mut().push((TypeId::of::<T>(), Arc::new(value))));
}

#[doc(hidden)]
pub fn __provider_scope_exit() {
    CONTEXT_STACK.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Read the nearest ancestor `<Provider>` value for this context.
///
/// Walks the thread-local provider stack top-down; returns the nearest
/// matching entry, or `None` if no provider is active.
pub fn use_context<T: Clone + 'static>(_ctx: &Context<T>) -> Option<T> {
    CONTEXT_STACK.with(|s| {
        let stack = s.borrow();
        for (tid, val) in stack.iter().rev() {
            if *tid == TypeId::of::<T>() {
                return val.downcast_ref::<T>().cloned();
            }
        }
        None
    })
}

/// Like `use_context`, falling back to `default()` when no provider is active.
pub fn use_context_or<T: Clone + 'static>(_ctx: &Context<T>, default: impl FnOnce() -> T) -> T {
    use_context(_ctx).unwrap_or_else(default)
}

/// RAII guard returned by [`provide_context`]: pops the stack on drop.
pub struct ProviderGuard {
    _private: (),
}

impl Drop for ProviderGuard {
    fn drop(&mut self) {
        __provider_scope_exit();
    }
}

/// Programmatically provide a context value to code run while the guard is
/// alive. The `<Provider>` macro builtin is the primary mechanism; this is
/// for advanced manual use and tests.
pub fn provide_context<T: Clone + 'static>(ctx: &Context<T>, value: T) -> ProviderGuard {
    __provider_scope_enter(ctx, value);
    ProviderGuard { _private: () }
}

#[cfg(test)]
mod tests {
    use super::*;

    static C32: Context<i32> = Context::new();
    static CSTR: Context<String> = Context::new();

    #[test]
    fn no_provider_returns_none() {
        assert!(CONTEXT_STACK.with(|s| s.borrow().is_empty()));
        assert!(use_context(&C32).is_none());
    }

    #[test]
    fn provide_then_consume() {
        let _g = provide_context(&C32, 42);
        assert_eq!(use_context(&C32), Some(42));
        assert!(use_context(&CSTR).is_none());
    }

    #[test]
    fn nested_same_type_inner_shadows() {
        let _outer = provide_context(&C32, 1);
        assert_eq!(use_context(&C32), Some(1));
        {
            let _inner = provide_context(&C32, 2);
            assert_eq!(use_context(&C32), Some(2));
        }
        // Inner dropped — outer visible again.
        assert_eq!(use_context(&C32), Some(1));
    }

    #[test]
    fn different_types_coexist() {
        let _a = provide_context(&C32, 7);
        let _b = provide_context(&CSTR, String::from("hi"));
        assert_eq!(use_context(&C32), Some(7));
        assert_eq!(use_context(&CSTR), Some(String::from("hi")));
    }

    #[test]
    fn use_context_or_default() {
        assert_eq!(use_context_or(&C32, || 99), 99);
        let _g = provide_context(&C32, 5);
        assert_eq!(use_context_or(&C32, || 99), 5);
    }
}
