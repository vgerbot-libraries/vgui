use shortcuts::{default_macro_registry, parse_shortcut_key, ShortcutsError};

#[test]
fn parses_two_steps_one_token() {
    let reg = default_macro_registry();
    let groups = parse_shortcut_key("Ctrl+K,Ctrl+P", &reg, '+', ',').unwrap();
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].len(), 2); // Ctrl + K
    assert_eq!(groups[1].len(), 2); // Ctrl + P
}

#[test]
fn parses_multi_token_step() {
    let reg = default_macro_registry();
    let groups = parse_shortcut_key("Ctrl+Shift+S", &reg, '+', ',').unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0].len(), 3);
}

#[test]
fn unknown_token_errors() {
    let reg = default_macro_registry();
    let err = parse_shortcut_key("Ctrl+XYZ", &reg, '+', ',').unwrap_err();
    assert!(matches!(err, ShortcutsError::MacroNotFound(t) if t == "XYZ"));
}

#[test]
fn escaped_comma_is_literal() {
    let reg = default_macro_registry();
    // \, is a literal comma in a token, not a delimiter. The split produces
    // one group with one token "a,b" — which isn't registered, so we get
    // MacroNotFound("a,b"). If the comma had been treated as a delimiter,
    // we'd get MacroNotFound("a") instead.
    let err = parse_shortcut_key(r"a\,b", &reg, '+', ',').unwrap_err();
    assert!(matches!(err, ShortcutsError::MacroNotFound(t) if t == "a,b"));
}
