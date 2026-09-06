use shortcuts::{
    CaseInsensitiveKeyMatcher, CaseSensitiveKeyMatcher, KeyEventType, MacroMatcher, Matcher,
    ModifierKind, ModifierMatcher, OrMatcher, ShortcutKeyboardEvent, AndMatcher,
};

fn ev(key: &str, ctrl: bool, shift: bool, alt: bool, meta: bool) -> ShortcutKeyboardEvent<'_> {
    ShortcutKeyboardEvent::new(KeyEventType::KeyDown, key, "", shift, ctrl, alt, meta, false)
}

#[test]
fn case_insensitive_matches_both_cases() {
    let m = CaseInsensitiveKeyMatcher::new("a");
    assert!(m.matches(&ev("a", false, false, false, false)));
    assert!(m.matches(&ev("A", false, false, false, false)));
    assert!(!m.matches(&ev("b", false, false, false, false)));
}

#[test]
fn case_sensitive_matches_exact() {
    let m = CaseSensitiveKeyMatcher::new("A");
    assert!(m.matches(&ev("A", false, false, false, false)));
    assert!(!m.matches(&ev("a", false, false, false, false)));
}

#[test]
fn modifier_matcher_as_modifier_returns_kind() {
    assert_eq!(
        ModifierMatcher::new(ModifierKind::Ctrl).as_modifier(),
        Some(ModifierKind::Ctrl)
    );
    assert_eq!(
        ModifierMatcher::new(ModifierKind::Alt).as_modifier(),
        Some(ModifierKind::Alt)
    );
    assert_eq!(
        ModifierMatcher::new(ModifierKind::Shift).as_modifier(),
        Some(ModifierKind::Shift)
    );
    assert_eq!(
        ModifierMatcher::new(ModifierKind::Meta).as_modifier(),
        Some(ModifierKind::Meta)
    );
}

#[test]
fn modifier_matcher_checks_boolean() {
    let ctrl = ModifierMatcher::new(ModifierKind::Ctrl);
    assert!(ctrl.matches(&ev("x", true, false, false, false)));
    assert!(!ctrl.matches(&ev("x", false, false, false, false)));
}

#[test]
fn and_matcher_empty_rejects() {
    let empty = AndMatcher::new(vec![]);
    assert!(!empty.matches(&ev("a", false, false, false, false)));
}

#[test]
fn and_matcher_all_must_pass() {
    let m = AndMatcher::new(vec![
        Box::new(CaseInsensitiveKeyMatcher::new("k")),
        Box::new(ModifierMatcher::new(ModifierKind::Ctrl)),
    ]);
    assert!(m.matches(&ev("k", true, false, false, false)));
    assert!(!m.matches(&ev("k", false, false, false, false)));
}

#[test]
fn or_matcher_any_passes() {
    let m = OrMatcher::new(vec![
        Box::new(CaseInsensitiveKeyMatcher::new("a")),
        Box::new(CaseInsensitiveKeyMatcher::new("b")),
    ]);
    assert!(m.matches(&ev("a", false, false, false, false)));
    assert!(m.matches(&ev("b", false, false, false, false)));
    assert!(!m.matches(&ev("c", false, false, false, false)));
}

#[test]
fn macro_matcher_delegates_as_modifier() {
    let inner = ModifierMatcher::new(ModifierKind::Ctrl);
    let macro_m = MacroMatcher::new("MyCtrl", Box::new(inner));
    assert_eq!(macro_m.as_modifier(), Some(ModifierKind::Ctrl));
    assert_eq!(macro_m.display(), "MyCtrl");
    assert!(macro_m.matches(&ev("x", true, false, false, false)));
}
