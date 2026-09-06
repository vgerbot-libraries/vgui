use shortcuts::{KeyEventType, Shortcut, ShortcutKeyboardEvent};
use shortcuts::default_macro_registry;

fn ev(key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> ShortcutKeyboardEvent<'_> {
    ShortcutKeyboardEvent::new(KeyEventType::KeyDown, key, "", shift, ctrl, alt, meta, false)
}

#[test]
fn ctrl_k_requires_exact_modifiers() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("Ctrl+K", &reg).unwrap();
    // Ctrl+K matches
    assert!(sc.matches(&ev("k", true, false, false, false)));
    assert!(sc.is_full_match());
    // Ctrl+Shift+K does NOT match Ctrl+K (exact modifier equality)
    sc.reset();
    assert!(!sc.matches(&ev("k", true, true, false, false)));
}

#[test]
fn sequence_g_g_partial_then_full() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("G,G", &reg).unwrap();
    // First g → partial
    assert!(sc.matches(&ev("g", false, false, false, false)));
    assert!(sc.is_partial_match());
    assert!(!sc.is_full_match());
    // Second g → full
    assert!(sc.matches(&ev("g", false, false, false, false)));
    assert!(sc.is_full_match());
}

#[test]
fn mismatched_event_keeps_partial_progress() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("G,G", &reg).unwrap();
    // First g → partial
    assert!(sc.matches(&ev("g", false, false, false, false)));
    assert!(sc.is_partial_match());
    // Mismatched event → does not match, cursor stays
    assert!(!sc.matches(&ev("h", false, false, false, false)));
    assert!(sc.is_partial_match());
    // Next g does NOT advance because cursor is at segment 1 and this g
    // matches segment 1 → advances to full
    assert!(sc.matches(&ev("g", false, false, false, false)));
    assert!(sc.is_full_match());
}

#[test]
fn auto_reset_after_full() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("G,G", &reg).unwrap();
    // Complete the sequence
    sc.matches(&ev("g", false, false, false, false));
    sc.matches(&ev("g", false, false, false, false));
    assert!(sc.is_full_match());
    // Next match auto-resets and starts over
    assert!(sc.matches(&ev("g", false, false, false, false)));
    assert!(sc.is_partial_match());
}

#[test]
fn display_joins_segments() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("Ctrl+K,Ctrl+P", &reg).unwrap();
    assert_eq!(sc.display(), "Ctrl+K,Ctrl+P");
}

#[test]
fn partial_display_shows_prefix() {
    let reg = default_macro_registry();
    let sc = Shortcut::from("Ctrl+K,Ctrl+P", &reg).unwrap();
    sc.matches(&ev("k", true, false, false, false));
    assert_eq!(sc.partial_display(), "Ctrl+K");
}

#[test]
fn equals_compares_segments() {
    let reg = default_macro_registry();
    let a = Shortcut::from("Ctrl+K", &reg).unwrap();
    let b = Shortcut::from("Ctrl+K", &reg).unwrap();
    let c = Shortcut::from("Ctrl+P", &reg).unwrap();
    assert!(a.equals(&b));
    assert!(!a.equals(&c));
}
