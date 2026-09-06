//! Built-in token table, registered against vgui's web key values.
//!
//! Mirrors the TypeScript `locales/default` but collapses all browser-sniff
//! branches to the modern web key-name path — vgui already normalizes to
//! modern web values (see `crates/vgui/src/event.rs`). Numeric `keyCode`
//! matchers are replaced by `key`/`code` matchers.

use crate::matchers::{
    CaseInsensitiveKeyMatcher, MacroMatcher, ModifierMatcher, OrMatcher,
};
use crate::matcher::ModifierKind;
use crate::macro_api::{alias, key_macro, key_macro_ins};
use crate::registry::MacroRegistry;

/// Register the built-in token→matcher table against `registry`.
pub fn register_defaults(registry: &dyn MacroRegistry) {
    register_modifiers(registry);
    register_whitespace(registry);
    register_navigation(registry);
    register_editing(registry);
    register_function_keys(registry);
    register_alphabet(registry);
    register_digits(registry);
    register_symbols(registry);
}

fn register_modifiers(registry: &dyn MacroRegistry) {
    // "Control" / "Ctrl" / "Ctl" → Ctrl modifier.
    registry.register(
        "Control",
        Box::new(ModifierMatcher::new(ModifierKind::Ctrl)),
    );
    let _ = alias(registry, "Ctrl", "Control", "Ctrl");
    let _ = alias(registry, "Ctl", "Control", "Ctl");
    // "Alt" / "Option" / "⌥" → Alt modifier.
    registry.register("Alt", Box::new(ModifierMatcher::new(ModifierKind::Alt)));
    let _ = alias(registry, "Option", "Alt", "Option");
    let _ = alias(registry, "⌥", "Alt", "⌥");
    // "Shift" / "⇧" → Shift modifier.
    registry.register("Shift", Box::new(ModifierMatcher::new(ModifierKind::Shift)));
    let _ = alias(registry, "⇧", "Shift", "⇧");
    // "Meta" / "Cmd" / "Command" / "⌘" → Meta modifier.
    registry.register("Meta", Box::new(ModifierMatcher::new(ModifierKind::Meta)));
    let _ = alias(registry, "Cmd", "Meta", "Cmd");
    let _ = alias(registry, "Command", "Meta", "Command");
    let _ = alias(registry, "⌘", "Meta", "⌘");
    // "⌃" aliases to Ctrl (symbol alias).
    let _ = alias(registry, "⌃", "Control", "⌃");
}

fn register_whitespace(registry: &dyn MacroRegistry) {
    key_macro_ins(registry, "Enter", "Enter", "Enter");
    let _ = alias(registry, "Return", "Enter", "Return");
    key_macro_ins(registry, "Tab", "Tab", "Tab");
    // vgui emits key " " for space; some sources use "Space".
    let space = OrMatcher::new(vec![
        Box::new(CaseInsensitiveKeyMatcher::new(" ")),
        Box::new(CaseInsensitiveKeyMatcher::new("Space")),
    ]);
    registry.register(
        "Space",
        Box::new(MacroMatcher::new("Space", Box::new(space))),
    );
}

fn register_navigation(registry: &dyn MacroRegistry) {
    key_macro_ins(registry, "ArrowUp", "ArrowUp", "ArrowUp");
    let _ = alias(registry, "Up", "ArrowUp", "Up");
    key_macro_ins(registry, "ArrowDown", "ArrowDown", "ArrowDown");
    let _ = alias(registry, "Down", "ArrowDown", "Down");
    key_macro_ins(registry, "ArrowLeft", "ArrowLeft", "ArrowLeft");
    let _ = alias(registry, "Left", "ArrowLeft", "Left");
    key_macro_ins(registry, "ArrowRight", "ArrowRight", "ArrowRight");
    let _ = alias(registry, "Right", "ArrowRight", "Right");
    key_macro_ins(registry, "Home", "Home", "Home");
    key_macro_ins(registry, "End", "End", "End");
    key_macro_ins(registry, "PageUp", "PageUp", "PageUp");
    let _ = alias(registry, "PgUp", "PageUp", "PgUp");
    key_macro_ins(registry, "PageDown", "PageDown", "PageDown");
    let _ = alias(registry, "PgDn", "PageDown", "PgDn");
}

fn register_editing(registry: &dyn MacroRegistry) {
    key_macro_ins(registry, "Backspace", "Backspace", "Backspace");
    key_macro_ins(registry, "Delete", "Delete", "Delete");
    key_macro_ins(registry, "Insert", "Insert", "Insert");
    key_macro_ins(registry, "Escape", "Escape", "Escape");
    let _ = alias(registry, "Esc", "Escape", "Esc");
    key_macro_ins(registry, "CapsLock", "CapsLock", "CapsLock");
}

fn register_function_keys(registry: &dyn MacroRegistry) {
    for n in 1..=24 {
        let name = format!("F{n}");
        key_macro_ins(registry, &name, &name, &name);
    }
}

fn register_alphabet(registry: &dyn MacroRegistry) {
    for c in 'A'..='Z' {
        let s = c.to_string();
        key_macro_ins(registry, &s, &s, &s);
    }
}

fn register_digits(registry: &dyn MacroRegistry) {
    for n in 0..=9 {
        let s = n.to_string();
        // vgui emits "0".."9" as the key; case-sensitive is exact.
        key_macro(registry, &s, &s, &s);
    }
}

fn register_symbols(registry: &dyn MacroRegistry) {
    const SYMBOLS: &[&str] = &[
        "`", "~", "!", "@", "#", "$", "%", "^", "&", "*", "(", ")", "-", "_", "=", "+", "[", "]",
        "{", "}", "\\", "|", ";", ":", "'", "\"", ",", ".", "<", ">", "?", "/",
    ];
    for &s in SYMBOLS {
        key_macro(registry, s, s, s);
    }
}
