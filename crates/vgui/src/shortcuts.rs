//! vgui integration for the `shortcuts` crate.
//!
//! [`use_shortcuts`] creates a [`Shortcuts`] handle backed by a
//! [`shortcuts::Keyboard`] and wires its `fire` dispatch into vgui's
//! [`use_key_down`]/[`use_key_up`] hooks. Matched commands call
//! [`KeyboardEvent::stop_propagation`] (the vgui equivalent of the DOM
//! `preventDefault`) when `prevent_default` is `true` on the command.
//!
//! The [`Shortcuts`] handle is persisted across re-renders via vgui's
//! slot mechanism, so the engine state (context stack, partial matches,
//! sequence cursors) survives signal-driven re-renders.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{App, Window};

use crate::event::KeyboardEvent;
use crate::reactive::{get_or_create_slot, use_key_down, use_key_up};

pub use shortcuts::{
    CommandOptions, ContextGuard, ContextOptions, Interceptor, InterceptorGuard, KeyEventType,
    KeymapOptions, MacroMatcher, MacroRegistry, Matcher, ModifierKind, Shortcut, ShortcutContext,
    ShortcutEvent, ShortcutKeyboardEvent, Keyboard,
};

/// Convert a vgui [`KeyboardEvent`] + known phase into the core event
/// (zero-copy).
fn to_event<'a>(e: &'a KeyboardEvent, ty: KeyEventType) -> ShortcutKeyboardEvent<'a> {
    ShortcutKeyboardEvent::new(
        ty,
        &e.key,
        &e.code,
        e.shift_key,
        e.ctrl_key,
        e.alt_key,
        e.meta_key,
        e.repeat,
    )
}

/// A handler bound to a command, receiving the gpui window/app context.
type GpuiHandler = Rc<dyn Fn(&ShortcutEvent<'_>, &mut Window, &mut App)>;

/// A partial-match change listener receiving the gpui app context.
type GpuiPartialHandler = Rc<dyn Fn(&[String], &mut App)>;

/// Handle returned by [`use_shortcuts`]. Wraps a [`shortcuts::Keyboard`] and
/// the gpui-aware command handlers registered via [`Shortcuts::on`].
///
/// Cloning a `Shortcuts` shares the underlying engine state (via `Rc`).
#[derive(Clone)]
pub struct Shortcuts {
    keyboard: Rc<Keyboard>,
    handlers: Rc<RefCell<HashMap<String, GpuiHandler>>>,
    partial_handlers: Rc<RefCell<Vec<GpuiPartialHandler>>>,
}

impl Shortcuts {
    /// Register a keymap. Delegates to [`Keyboard::keymap`].
    pub fn keymap(&self, opts: KeymapOptions) -> &Self {
        self.keyboard.keymap(opts);
        self
    }

    /// Whether any commands have been registered via [`keymap`](Self::keymap).
    /// Use this to guard one-time initialization in a reactive render.
    pub fn has_keymap(&self) -> bool {
        self.keyboard.has_keymap()
    }

    /// Register a handler for `command`. The handler receives the
    /// [`ShortcutEvent`] plus the gpui `Window` and `App` contexts.
    pub fn on(
        &self,
        command: &str,
        handler: impl Fn(&ShortcutEvent<'_>, &mut Window, &mut App) + 'static,
    ) -> &Self {
        self.handlers
            .borrow_mut()
            .insert(command.to_string(), Rc::new(handler));
        self
    }

    /// Set the active context, replacing the entire context stack.
    /// Unlike [`switch_context`](Self::switch_context), this does not
    /// return a guard — the stack is cleared first.
    pub fn set_context(&self, name: &str) {
        self.keyboard
            .set_context(name)
            .expect("set_context: context not registered")
    }

    /// Switch the active context. Returns a guard that pops the context on
    /// drop.
    pub fn switch_context(&self, name: &str) -> ContextGuard {
        self.keyboard
            .switch_context(name)
            .expect("switch_context: context not registered")
    }

    /// Current active context name (top of the stack).
    pub fn current_context(&self) -> Option<String> {
        self.keyboard.current_context()
    }

    /// Pause shortcut dispatch.
    pub fn pause(&self) {
        self.keyboard.pause()
    }

    /// Resume shortcut dispatch.
    pub fn resume(&self) {
        self.keyboard.resume()
    }

    /// Register a partial-match change listener. The listener is called
    /// after each `fire()` with the current set of partially-matched
    /// command names and the gpui `App` context, so it can update signals.
    pub fn on_partial_change(&self, handler: impl Fn(&[String], &mut App) + 'static) {
        self.partial_handlers.borrow_mut().push(Rc::new(handler));
    }

    /// Access the underlying [`Keyboard`] for advanced use.
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }
}

/// Create a [`Shortcuts`] handle and wire it into the current render scope's
/// `keydown`/`keyup` hooks.
///
/// Must be called inside a `VguiRoot` render scope (same requirement as
/// [`use_key_down`]). The handle is persisted across re-renders via vgui's
/// slot mechanism — the underlying [`Keyboard`] state (context stack,
/// partial matches, sequence cursors) survives signal-driven re-renders.
///
/// On each key event, the dispatcher calls [`Keyboard::fire`]; for each
/// matched command with `prevent_default == true`, it calls
/// [`KeyboardEvent::stop_propagation`], then invokes the registered
/// gpui handler. Partial-match listeners are called with the current
/// partial set and the `App` context.
pub fn use_shortcuts() -> Shortcuts {
    // Persist the Shortcuts handle across re-renders. On first render the
    // factory creates a fresh Keyboard; on subsequent renders the stored
    // handle (sharing the same Rc<Keyboard>) is returned.
    let sc = get_or_create_slot(|_cx| Shortcuts {
        keyboard: Rc::new(Keyboard::new()),
        handlers: Rc::new(RefCell::new(HashMap::new())),
        partial_handlers: Rc::new(RefCell::new(Vec::new())),
    });

    // keydown dispatcher — re-registered on every render (handlers are
    // cleared by reset_render_state), but captures the persisted Keyboard.
    {
        let (kb, hs, phs) = (
            sc.keyboard.clone(),
            sc.handlers.clone(),
            sc.partial_handlers.clone(),
        );
        use_key_down(move |e: &KeyboardEvent, w: &mut Window, cx: &mut App| {
            let ev = to_event(e, KeyEventType::KeyDown);
            let matched = kb.fire(&ev);
            let should_stop = matched.iter().any(|m| m.prevent_default);
            if should_stop {
                e.stop_propagation();
            }
            // Notify partial-match listeners with cx so they can update signals.
            let partials = kb.partial_matches();
            for h in phs.borrow().iter() {
                h(&partials, cx);
            }
            // Invoke matched command handlers.
            for m in &matched {
                if let Some(h) = hs.borrow().get(m.command.as_str()) {
                    let h = h.clone();
                    h(&m.event, w, cx);
                }
            }
        });
    }

    // keyup dispatcher
    {
        let (kb, hs, phs) = (
            sc.keyboard.clone(),
            sc.handlers.clone(),
            sc.partial_handlers.clone(),
        );
        use_key_up(move |e: &KeyboardEvent, w: &mut Window, cx: &mut App| {
            let ev = to_event(e, KeyEventType::KeyUp);
            let matched = kb.fire(&ev);
            let should_stop = matched.iter().any(|m| m.prevent_default);
            if should_stop {
                e.stop_propagation();
            }
            let partials = kb.partial_matches();
            for h in phs.borrow().iter() {
                h(&partials, cx);
            }
            for m in &matched {
                if let Some(h) = hs.borrow().get(m.command.as_str()) {
                    let h = h.clone();
                    h(&m.event, w, cx);
                }
            }
        });
    }

    sc
}
