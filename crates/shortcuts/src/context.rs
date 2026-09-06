//! Context and keymap configuration types (the user-facing config surface).
//!
//! Mirrors the TS `Keymap`/`CommandOptions`/`ContextOptions`. Parsed (resolved)
//! forms live in [`crate::keyboard`].

use std::collections::HashMap;

use crate::event::KeyEventType;
use crate::interceptor::Interceptor;

/// Options for a single command. `shortcut` is the combination string
/// (`"Ctrl+S"`); the rest are optional and default-filled on parse.
#[derive(Clone)]
pub struct CommandOptions {
    pub shortcut: String,
    /// Which event phases fire this command. Defaults to `[KeyDown]`.
    pub event: Option<Vec<KeyEventType>>,
    /// In vgui, mapped to `stop_propagation()` on match. Defaults to `true`.
    pub prevent_default: Option<bool>,
    /// Per-command interceptors (run after global interceptors).
    pub interceptors: Option<Vec<Interceptor>>,
}

/// Options for a context.
#[derive(Clone)]
pub struct ContextOptions {
    pub commands: Vec<String>,
    /// Renamed from the TS `abstract` (a Rust keyword). `true` marks a context
    /// that only exists to be inherited via `fallbacks`.
    pub abstract_ctx: Option<bool>,
    pub fallbacks: Option<Vec<String>>,
}

/// Top-level keymap configuration passed to [`Keyboard::keymap`](crate::keyboard::Keyboard::keymap).
#[derive(Clone, Default)]
pub struct KeymapOptions {
    pub commands: HashMap<String, CommandOptions>,
    pub contexts: HashMap<String, ContextOptions>,
}

/// A resolved context description (for introspection).
#[derive(Clone, Debug)]
pub struct ShortcutContext {
    pub name: String,
    pub fallbacks: Vec<String>,
    pub commands: Vec<String>,
}
