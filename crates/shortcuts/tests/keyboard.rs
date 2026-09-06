use std::collections::HashMap;
use std::rc::Rc;

use shortcuts::{CommandOptions, ContextOptions, KeyEventType, Keyboard, KeymapOptions, ShortcutEvent, ShortcutKeyboardEvent};

fn ev(ty: KeyEventType, key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> ShortcutKeyboardEvent<'_> {
    ShortcutKeyboardEvent::new(ty, key, "", shift, ctrl, alt, meta, false)
}

fn make_keymap() -> KeymapOptions {
    let mut commands = HashMap::new();
    commands.insert(
        "save".to_string(),
        CommandOptions {
            shortcut: "Ctrl+S".to_string(),
            event: None,
            prevent_default: None,
            interceptors: None,
        },
    );
    let mut contexts = HashMap::new();
    contexts.insert(
        "global".to_string(),
        ContextOptions {
            commands: vec!["save".to_string()],
            abstract_ctx: None,
            fallbacks: None,
        },
    );
    KeymapOptions { commands, contexts }
}

#[test]
fn fire_returns_matched_command() {
    let kb = Keyboard::new();
    kb.keymap(make_keymap());
    let _guard = kb.switch_context("global").unwrap();

    let fired = Rc::new(std::cell::RefCell::new(false));
    let fired_clone = fired.clone();
    kb.on("save", move |_sev: &ShortcutEvent<'_>| {
        *fired_clone.borrow_mut() = true;
    });

    let e = ev(KeyEventType::KeyDown, "s", true, false, false, false);
    let results = kb.fire(&e);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "save");
    assert!(*fired.borrow());
}

#[test]
fn fire_no_context_returns_empty() {
    let kb = Keyboard::new();
    kb.keymap(make_keymap());
    let e = ev(KeyEventType::KeyDown, "s", true, false, false, false);
    let results = kb.fire(&e);
    assert!(results.is_empty());
}

#[test]
fn pause_returns_empty() {
    let kb = Keyboard::new();
    kb.keymap(make_keymap());
    let _guard = kb.switch_context("global").unwrap();
    kb.pause();
    let e = ev(KeyEventType::KeyDown, "s", true, false, false, false);
    let results = kb.fire(&e);
    assert!(results.is_empty());
}

#[test]
fn context_fallback_inherits_commands() {
    let kb = Keyboard::new();
    let mut opts = make_keymap();
    opts.contexts.insert(
        "palette".to_string(),
        ContextOptions {
            commands: vec![],
            abstract_ctx: None,
            fallbacks: Some(vec!["global".to_string()]),
        },
    );
    kb.keymap(opts);
    let _guard = kb.switch_context("palette").unwrap();

    let e = ev(KeyEventType::KeyDown, "s", true, false, false, false);
    let results = kb.fire(&e);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].command, "save");
}

#[test]
fn partial_match_tracking() {
    let kb = Keyboard::new();
    let mut commands = HashMap::new();
    commands.insert(
        "palette".to_string(),
        CommandOptions {
            shortcut: "Ctrl+K,Ctrl+P".to_string(),
            event: None,
            prevent_default: None,
            interceptors: None,
        },
    );
    let mut contexts = HashMap::new();
    contexts.insert(
        "global".to_string(),
        ContextOptions {
            commands: vec!["palette".to_string()],
            abstract_ctx: None,
            fallbacks: None,
        },
    );
    kb.keymap(KeymapOptions { commands, contexts });
    let _guard = kb.switch_context("global").unwrap();

    let partials = Rc::new(std::cell::RefCell::new(Vec::new()));
    let partials_clone = partials.clone();
    kb.on_partial_change(move |names: &[String]| {
        *partials_clone.borrow_mut() = names.to_vec();
    });

    // First Ctrl+K → partial
    let e1 = ev(KeyEventType::KeyDown, "k", true, false, false, false);
    let r1 = kb.fire(&e1);
    assert!(r1.is_empty());
    assert_eq!(*partials.borrow(), vec!["palette".to_string()]);

    // Second Ctrl+P → full match
    let e2 = ev(KeyEventType::KeyDown, "p", true, false, false, false);
    let r2 = kb.fire(&e2);
    assert_eq!(r2.len(), 1);
    assert_eq!(r2[0].command, "palette");
    assert!(partials.borrow().is_empty());
}
