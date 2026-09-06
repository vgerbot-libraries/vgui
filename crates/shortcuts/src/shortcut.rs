//! Stateful shortcut sequence matcher.
//!
//! A [`Shortcut`] is a sequence of [`ShortcutSegment`]s (the comma-separated
//! steps of a combination like `"Ctrl+K, Ctrl+P"`). [`Shortcut::matches`] is
//! stateful: it advances an internal cursor each time the current segment
//! matches, auto-resets once full, and does **not** reset on mismatch (the
//! caller removes it from the partial set so the next attempt starts fresh
//! after an auto-reset).

use std::cell::Cell;

use crate::error::ShortcutsError;
use crate::event::ShortcutKeyboardEvent;
use crate::matcher::{Matcher, ModifierKind};
use crate::parser::parse_shortcut_key;
use crate::registry::MacroRegistry;

const KEY_DELIMITER: char = '+';
const COMBO_DELIMITER: char = ',';

/// One step of a shortcut sequence: a set of simultaneous matchers plus the
/// modifier flags extracted from them.
#[derive(Clone, Debug)]
pub struct ShortcutSegment {
    ctrl: bool,
    alt: bool,
    shift: bool,
    meta: bool,
    /// Non-modifier matchers (modifiers are extracted into the flags above).
    matchers: Vec<Box<dyn Matcher>>,
    /// All matchers in their original order, for display.
    all_matchers: Vec<Box<dyn Matcher>>,
}

impl ShortcutSegment {
    fn new(all: Vec<Box<dyn Matcher>>) -> Self {
        let mut ctrl = false;
        let mut alt = false;
        let mut shift = false;
        let mut meta = false;
        let mut matchers = Vec::with_capacity(all.len());
        for m in &all {
            match m.as_modifier() {
                Some(ModifierKind::Ctrl) => ctrl = true,
                Some(ModifierKind::Alt) => alt = true,
                Some(ModifierKind::Shift) => shift = true,
                Some(ModifierKind::Meta) => meta = true,
                None => matchers.push(m.clone_box()),
            }
        }
        Self {
            ctrl,
            alt,
            shift,
            meta,
            matchers,
            all_matchers: all,
        }
    }

    /// Exact modifier equality AND all non-modifier matchers pass.
    /// `Ctrl+K` does NOT match `Ctrl+Shift+K`.
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        if self.ctrl != event.ctrl_key {
            return false;
        }
        if self.shift != event.shift_key {
            return false;
        }
        if self.alt != event.alt_key {
            return false;
        }
        if self.meta != event.meta_key {
            return false;
        }
        self.matchers.iter().all(|m| m.matches(event))
    }

    fn display(&self) -> String {
        self.all_matchers
            .iter()
            .map(|m| m.display())
            .collect::<Vec<_>>()
            .join("+")
    }

    fn equals(&self, other: &ShortcutSegment) -> bool {
        self.ctrl == other.ctrl
            && self.alt == other.alt
            && self.shift == other.shift
            && self.meta == other.meta
            && self.all_matchers.len() == other.all_matchers.len()
            && self
                .all_matchers
                .iter()
                .zip(other.all_matchers.iter())
                .all(|(a, b)| a.display() == b.display())
    }
}

/// A parsed shortcut combination with stateful sequence matching.
#[derive(Debug)]
pub struct Shortcut {
    segments: Vec<ShortcutSegment>,
    match_times: Cell<usize>,
}

impl Shortcut {
    /// Parse `combination` (e.g. `"Ctrl+K, Ctrl+P"`) against `registry`.
    pub fn from(combination: &str, registry: &dyn MacroRegistry) -> Result<Self, ShortcutsError> {
        let groups = parse_shortcut_key(combination, registry, KEY_DELIMITER, COMBO_DELIMITER)?;
        let segments = groups.into_iter().map(ShortcutSegment::new).collect();
        Ok(Self {
            segments,
            match_times: Cell::new(0),
        })
    }

    /// Stateful match. Auto-resets when already full; advances the cursor on
    /// segment match; does NOT reset on mismatch.
    pub fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        if self.is_full_match() {
            self.reset();
        }
        let segment = match self.segments.get(self.match_times.get()) {
            Some(s) => s,
            None => return false,
        };
        if segment.matches(event) {
            self.match_times.set(self.match_times.get() + 1);
            true
        } else {
            false
        }
    }

    pub fn reset(&self) {
        self.match_times.set(0);
    }

    pub fn is_partial_match(&self) -> bool {
        self.match_times.get() != self.segments.len()
    }

    pub fn is_full_match(&self) -> bool {
        self.match_times.get() == self.segments.len()
    }

    /// Full combination label, segments joined by `,`.
    pub fn display(&self) -> String {
        self.segments
            .iter()
            .map(|s| s.display())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Matched prefix label, joined by `,`.
    pub fn partial_display(&self) -> String {
        self.segments[..self.match_times.get()]
            .iter()
            .map(|s| s.display())
            .collect::<Vec<_>>()
            .join(",")
    }

    /// Structural equality of all segments.
    pub fn equals(&self, other: &Shortcut) -> bool {
        if self.segments.len() != other.segments.len() {
            return false;
        }
        self.segments
            .iter()
            .zip(other.segments.iter())
            .all(|(a, b)| a.equals(b))
    }
}

impl Matcher for Shortcut {
    fn matches(&self, event: &ShortcutKeyboardEvent<'_>) -> bool {
        Shortcut::matches(self, event)
    }
    fn display(&self) -> String {
        Shortcut::display(self)
    }
    fn clone_box(&self) -> Box<dyn Matcher> {
        // A fresh, independent stateful copy.
        Box::new(Self {
            segments: self.segments.clone(),
            match_times: Cell::new(0),
        })
    }
}

impl Clone for Shortcut {
    fn clone(&self) -> Self {
        Self {
            segments: self.segments.clone(),
            match_times: Cell::new(0),
        }
    }
}
