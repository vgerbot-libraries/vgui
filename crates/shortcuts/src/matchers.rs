//! Concrete matchers: keys, codes, modifiers, and combinators.

use crate::event::ShortcutKeyboardEvent;
use crate::matcher::{ModifierKind, Matcher};

/// Matches the web `key` value, optionally case-sensitive.
#[derive(Clone, Debug)]
pub struct KeyMatcher {
    pub key: String,
    pub case_sensitive: bool,
}

impl KeyMatcher {
    pub fn new(key: impl Into<String>, case_sensitive: bool) -> Self {
        Self {
            key: key.into(),
            case_sensitive,
        }
    }
}

impl Matcher for KeyMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        if self.case_sensitive {
            event.key == self.key
        } else {
            event.key.eq_ignore_ascii_case(&self.key)
        }
    }
    fn display(&self) -> String {
        self.key.clone()
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Case-insensitive key matcher. Stores the lowercased key (so its
/// [`display`](Matcher::display) is lowercased, matching the TS library).
#[derive(Clone, Debug)]
pub struct CaseInsensitiveKeyMatcher(pub String);

impl CaseInsensitiveKeyMatcher {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into().to_ascii_lowercase())
    }
}

impl Matcher for CaseInsensitiveKeyMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        event.key.eq_ignore_ascii_case(&self.0)
    }
    fn display(&self) -> String {
        self.0.clone()
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Case-sensitive key matcher.
#[derive(Clone, Debug)]
pub struct CaseSensitiveKeyMatcher(pub String);

impl CaseSensitiveKeyMatcher {
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }
}

impl Matcher for CaseSensitiveKeyMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        event.key == self.0
    }
    fn display(&self) -> String {
        self.0.clone()
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Matches the web `code` value (physical key identifier).
#[derive(Clone, Debug)]
pub struct CodeMatcher {
    pub code: String,
}

impl CodeMatcher {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
        }
    }
}

impl Matcher for CodeMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        event.code == self.code
    }
    fn display(&self) -> String {
        self.code.clone()
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Matches a modifier flag. Unlike the TS library (which matches the modifier
/// *key name*), this checks the event's modifier boolean — the right model for
/// vgui, where pressing `Ctrl+S` delivers an `S` event with `ctrl_key == true`.
#[derive(Clone, Debug)]
pub struct ModifierMatcher {
    pub kind: ModifierKind,
}

impl ModifierMatcher {
    pub fn new(kind: ModifierKind) -> Self {
        Self { kind }
    }
}

impl Matcher for ModifierMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        match self.kind {
            ModifierKind::Ctrl => event.ctrl_key,
            ModifierKind::Alt => event.alt_key,
            ModifierKind::Shift => event.shift_key,
            ModifierKind::Meta => event.meta_key,
        }
    }
    fn display(&self) -> String {
        match self.kind {
            ModifierKind::Ctrl => "Control".to_string(),
            ModifierKind::Alt => "Alt".to_string(),
            ModifierKind::Shift => "Shift".to_string(),
            ModifierKind::Meta => "Meta".to_string(),
        }
    }
    fn as_modifier(&self) -> Option<ModifierKind> {
        Some(self.kind)
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Logical AND. Empty rejects (mirrors TS `and`: `length > 0 && every`).
#[derive(Clone, Debug)]
pub struct AndMatcher(pub Vec<Box<dyn Matcher>>);

impl AndMatcher {
    pub fn new(matchers: Vec<Box<dyn Matcher>>) -> Self {
        Self(matchers)
    }
}

impl Matcher for AndMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        !self.0.is_empty() && self.0.iter().all(|m| m.matches(event))
    }
    fn display(&self) -> String {
        self.0
            .iter()
            .map(|m| m.display())
            .collect::<Vec<_>>()
            .join("+")
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Logical OR. Empty rejects (mirrors TS `or`: `some`).
#[derive(Clone, Debug)]
pub struct OrMatcher(pub Vec<Box<dyn Matcher>>);

impl OrMatcher {
    pub fn new(matchers: Vec<Box<dyn Matcher>>) -> Self {
        Self(matchers)
    }
}

impl Matcher for OrMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        self.0.iter().any(|m| m.matches(event))
    }
    fn display(&self) -> String {
        self.0
            .iter()
            .map(|m| m.display())
            .collect::<Vec<_>>()
            .join("|")
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}

/// Relabeling decorator: delegates [`matches`](Matcher::matches),
/// [`clone_box`](Matcher::clone_box) and [`as_modifier`](Matcher::as_modifier)
/// to `origin`, but reports `label` as its
/// [`display`](Matcher::display). Mirrors the TS `MacroKeyboardEventMatcher`.
#[derive(Clone, Debug)]
pub struct MacroMatcher {
    pub label: String,
    pub origin: Box<dyn Matcher>,
}

impl MacroMatcher {
    pub fn new(label: impl Into<String>, origin: Box<dyn Matcher>) -> Self {
        Self {
            label: label.into(),
            origin,
        }
    }
}

impl Matcher for MacroMatcher {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        self.origin.matches(event)
    }
    fn display(&self) -> String {
        self.label.clone()
    }
    fn as_modifier(&self) -> Option<ModifierKind> {
        self.origin.as_modifier()
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        Box::new(self.clone())
    }
}
