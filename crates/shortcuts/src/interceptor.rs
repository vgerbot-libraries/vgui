//! Interceptor middleware.
//!
//! Each interceptor receives a [`ShortcutEvent`] and a `next` continuation it
//! must call. The chain is built by right-folding `[global, per-command]` so
//! execution order is array order, with the terminal handler at the bottom.

use crate::event::ShortcutKeyboardEvent;
use crate::shortcut::Shortcut;
use std::rc::Rc;
/// A shortcut event delivered to handlers and interceptors.
#[derive(Clone, Debug)]
pub struct ShortcutEvent<'a> {
    pub shortcut: Rc<Shortcut>,
    pub native: &'a ShortcutKeyboardEvent<'a>,
}

impl<'a> ShortcutEvent<'a> {
    pub fn new(shortcut: Rc<Shortcut>, native: &'a ShortcutKeyboardEvent<'a>) -> Self {
        Self { shortcut, native }
    }
    pub fn display(&self) -> String {
        self.shortcut.display()
    }
    pub fn key(&self) -> &str {
        self.native.key
    }
    pub fn shift_key(&self) -> bool {
        self.native.shift_key
    }
    pub fn ctrl_key(&self) -> bool {
        self.native.ctrl_key
    }
    pub fn alt_key(&self) -> bool {
        self.native.alt_key
    }
    pub fn meta_key(&self) -> bool {
        self.native.meta_key
    }
    pub fn repeat(&self) -> bool {
        self.native.repeat
    }
    pub fn event_type(&self) -> crate::event::KeyEventType {
        self.native.event_type
    }
}

/// An interceptor: middleware wrapping the terminal handler.
///
/// `Rc<dyn Fn(&ShortcutEvent, &dyn Fn(&ShortcutEvent)) + 'static>` — no
/// `Send`/`Sync` (gpui is single-threaded). `Rc` so interceptor vectors can be
/// cloned cheaply when building the per-fire chain.
pub type Interceptor = Rc<dyn Fn(&ShortcutEvent<'_>, &dyn Fn(&ShortcutEvent<'_>)) + 'static>;
