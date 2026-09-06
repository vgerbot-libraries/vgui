//! The `Matcher` trait and modifier kind.
//!
//! Replaces the TypeScript `KeyboardEventMatcher` interface. Concrete

use crate::event::ShortcutKeyboardEvent;
/// Which modifier flag a [`ModifierMatcher`](crate::matchers::ModifierMatcher)
/// tests.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ModifierKind {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

/// Tests a keyboard event.
///
/// Every matcher is cloneable behind a trait object via [`Matcher::clone_box`]
/// so registries and shortcut segments can own independent copies.
pub trait Matcher: std::fmt::Debug {
    /// Whether this matcher accepts the event.
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool;

    /// User-facing label (for menus/help), mirrors the TypeScript `str()`.
    fn display(&self) -> String;

    /// Modifier matchers return their kind; non-modifier matchers return
    /// `None`. Used by [`ShortcutSegment`](crate::shortcut::ShortcutSegment) to
    /// extract modifier requirements (replaces the TypeScript `instanceof`
    /// checks). [`MacroMatcher`](crate::matchers::MacroMatcher) delegates to
    /// its origin so macro-wrapped modifiers are still detected.
    fn as_modifier(&self) -> Option<ModifierKind> {
        None
    }

    /// Clone this matcher into a boxed trait object.
    fn clone_box(&self) -> Box<dyn Matcher>;
}

impl Clone for Box<dyn Matcher> {
    fn clone(&self) -> Box<dyn Matcher> {
        self.clone_box()
    }
}
