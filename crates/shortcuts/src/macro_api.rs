//! Macro registration helpers — free functions mirroring the TS `macro.ts`.
//!
//! Each takes the registry to mutate. `label` is the user-facing display
//! string; when it differs from the matcher's own
//! [`display`](crate::matcher::Matcher::display), the matcher is wrapped in a
//! [`MacroMatcher`] that relabels it.

use crate::error::ShortcutsError;
use crate::matcher::Matcher;
use crate::matchers::{CaseInsensitiveKeyMatcher, CaseSensitiveKeyMatcher, CodeMatcher, MacroMatcher};
use crate::registry::MacroRegistry;
use crate::shortcut::Shortcut;

/// Wrap `matcher` in a relabeling [`MacroMatcher`] when its display differs
/// from `label`; otherwise return it unchanged.
fn wrap(label: &str, matcher: Box<dyn Matcher>) -> Box<dyn Matcher> {
    if matcher.display() == label {
        matcher
    } else {
        Box::new(MacroMatcher::new(label, matcher))
    }
}

/// Register `matcher` under `pattern`, relabeled to `label` if needed.
pub fn macro_register(
    registry: &dyn MacroRegistry,
    pattern: &str,
    matcher: Box<dyn Matcher>,
    label: &str,
) {
    registry.register(pattern, wrap(label, matcher));
}

/// Parse `combination` into a [`Shortcut`] and register it under `pattern`,
/// enabling recursive macro composition.
pub fn macro_from_str(
    registry: &dyn MacroRegistry,
    pattern: &str,
    combination: &str,
    label: &str,
) -> Result<(), ShortcutsError> {
    let shortcut = Shortcut::from(combination, registry)?;
    macro_register(registry, pattern, Box::new(shortcut), label);
    Ok(())
}

/// Register `alias_pattern` as an alias of the already-registered
/// `origin_pattern`.
pub fn alias(
    registry: &dyn MacroRegistry,
    alias_pattern: &str,
    origin_pattern: &str,
    label: &str,
) -> Result<(), ShortcutsError> {
    let matcher = registry
        .get(origin_pattern)
        .ok_or_else(|| ShortcutsError::AliasOriginNotFound(origin_pattern.to_string()))?;
    macro_register(registry, alias_pattern, matcher, label);
    Ok(())
}

/// Case-sensitive key macro.
pub fn key_macro(
    registry: &dyn MacroRegistry,
    pattern: &str,
    key: &str,
    label: &str,
) {
    macro_register(
        registry,
        pattern,
        Box::new(CaseSensitiveKeyMatcher::new(key)),
        label,
    );
}

/// Case-insensitive key macro.
pub fn key_macro_ins(
    registry: &dyn MacroRegistry,
    pattern: &str,
    key: &str,
    label: &str,
) {
    macro_register(
        registry,
        pattern,
        Box::new(CaseInsensitiveKeyMatcher::new(key)),
        label,
    );
}

/// Physical `code` macro.
pub fn code_macro(
    registry: &dyn MacroRegistry,
    pattern: &str,
    code: &str,
    label: &str,
) {
    macro_register(registry, pattern, Box::new(CodeMatcher::new(code)), label);
}
