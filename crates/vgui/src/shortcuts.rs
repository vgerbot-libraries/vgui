//! vgui integration for the `shortcuts` crate.
//!
//! [`use_shortcuts`] creates a [`Shortcuts`] handle backed by a
//! [`shortcuts::Keyboard`] and wires its `fire` dispatch into vgui's
//! [`use_key_down`]/[`use_key_up`] hooks. Matched commands call
//! [`KeyboardEvent::stop_propagation`] (the vgui equivalent of the DOM
//! `preventDefault`) when `prevent_default` is `true` on the command.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gpui::{App, Window};

use crate::event::KeyboardEvent;
use crate::reactive::{use_key_down, use_key_up};

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

/// Handle returned by [`use_shortcuts`]. Wraps a [`shortcuts::Keyboard`] and
/// the gpui-aware command handlers registered via [`Shortcuts::on`].
#[derive(Clone)]
pub struct Shortcuts {
    keyboard: Rc<Keyboard>,
    handlers: Rc<RefCell<HashMap<String, GpuiHandler>>>,
}

impl Shortcuts {
    /// Register a keymap. Delegates to [`Keyboard::keymap`].
    pub fn keymap(&self, opts: KeymapOptions) -> &Self {
        self.keyboard.keymap(opts);
        self
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

    /// Register a partial-match change listener.
    pub fn on_partial_change(&self, handler: impl Fn(&[String]) + 'static) {
        self.keyboard.on_partial_change(handler);
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
/// [`use_key_down`]). On each key event, the dispatcher calls
/// [`Keyboard::fire`]; for each matched command with `prevent_default == true`,
/// it calls [`KeyboardEvent::stop_propagation`], then invokes the registered
/// gpui handler.
pub fn use_shortcuts() -> Shortcuts {
    let keyboard = Rc::new(Keyboard::new());
    let handlers: Rc<RefCell<HashMap<String, GpuiHandler>>> = Rc::new(RefCell::new(HashMap::new()));

    // keydown dispatcher
    {
        let (kb, hs) = (keyboard.clone(), handlers.clone());
        use_key_down(move |e: &KeyboardEvent, w: &mut Window, cx: &mut App| {
            let ev = to_event(e, KeyEventType::KeyDown);
            let matched = kb.fire(&ev);
            let should_stop = matched.iter().any(|m| m.prevent_default);
            if should_stop {
                e.stop_propagation();
            }
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
        let (kb, hs) = (keyboard.clone(), handlers.clone());
        use_key_up(move |e: &KeyboardEvent, w: &mut Window, cx: &mut App| {
            let ev = to_event(e, KeyEventType::KeyUp);
            let matched = kb.fire(&ev);
            let should_stop = matched.iter().any(|m| m.prevent_default);
            if should_stop {
                e.stop_propagation();
            }
            for m in &matched {
                if let Some(h) = hs.borrow().get(m.command.as_str()) {
                    let h = h.clone();
                    h(&m.event, w, cx);
                }
            }
        });
    }

    Shortcuts { keyboard, handlers }
}
