//! Web-aligned keyboard event view consumed by all matchers.
//!
//! This is a borrowed, zero-copy projection of the host's keyboard event
//! (vgui constructs it from [`vgui::KeyboardEvent`]). It carries the web
//! `key`/`code` strings and the four modifier booleans plus `repeat`.

/// Whether a shortcut fires on key press or release.
///
/// Mirrors the TypeScript library's `Opportunity` (`"keydown" | "keyup"`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KeyEventType {
    KeyDown,
    KeyUp,
}

/// Borrowed, zero-copy view of a keyboard event.
///
/// Built by the host from its own event type. All matchers read this.
#[derive(Clone, Copy, Debug)]
pub struct ShortcutKeyboardEvent<'a> {
    pub event_type: KeyEventType,
    /// Web `key`: the logical key value (`"a"`, `"Enter"`, `" "`, `"ArrowUp"`…).
    pub key: &'a str,
    /// Web `code`: the physical key identifier (`"KeyA"`, `"Digit5"`,
    /// `"Space"`…). Empty string if unknown.
    pub code: &'a str,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    pub repeat: bool,
}

impl<'a> ShortcutKeyboardEvent<'a> {
    pub fn new(
        event_type: KeyEventType,
        key: &'a str,
        code: &'a str,
        shift_key: bool,
        ctrl_key: bool,
        alt_key: bool,
        meta_key: bool,
        repeat: bool,
    ) -> Self {
        Self {
            event_type,
            key,
            code,
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
            repeat,
        }
    }
}
