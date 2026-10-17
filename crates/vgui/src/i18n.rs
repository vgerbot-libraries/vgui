//! Minimal reactive localization.
//!
//! A tiny, dependency-free gettext-style core: register message tables with
//! the [`i18n!`](crate::i18n) macro, read the active locale inside a render
//! with [`use_i18n`] + [`t!`](crate::t), and switch language at runtime with
//! [`set_locale`]. Switching bumps a global revision and notifies every
//! component that subscribed through [`use_i18n`], so translated text updates
//! without a manual rebuild.
//!
//! ```
//! use vgui::{i18n, translate};
//! i18n! {
//!     "en" => { "greeting" => "Hello", "farewell" => "Goodbye" },
//!     "zh" => { "greeting" => "你好", "farewell" => "再见" }
//! }
//! assert_eq!(translate("greeting"), "Hello");
//! ```
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use gpui::App;

use crate::reactive::{create_signal, on_cleanup, WriteSignal};

thread_local! {
    static LOCALE: RefCell<String> = const { RefCell::new(String::new()) };
    static MESSAGES: RefCell<HashMap<(String, String), String>> = RefCell::new(HashMap::new());
    static REVISION: Cell<u64> = const { Cell::new(0) };
    static NEXT_SUB_ID: Cell<u64> = const { Cell::new(1) };
    // Subscribers keyed by a per-subscription id so a component can detach on
    // cleanup; values are the revision writers owned by each `use_i18n`.
    static SUBSCRIBERS: RefCell<Vec<(u64, WriteSignal<u64>)>> = RefCell::new(Vec::new());
}

/// The default locale used before the first [`set_locale`].
pub const DEFAULT_LOCALE: &str = "en";

/// Register a flat message table for one locale. Prefer the [`i18n!`] macro.
pub fn register_messages(locale: &str, pairs: &[(&str, &str)]) {
    MESSAGES.with(|m| {
        let mut map = m.borrow_mut();
        for (key, value) in pairs {
            map.insert((locale.to_string(), key.to_string()), value.to_string());
        }
    });
}

/// Current locale, falling back to [`DEFAULT_LOCALE`].
pub fn locale() -> String {
    LOCALE.with(|l| {
        let cur = l.borrow();
        if cur.is_empty() {
            DEFAULT_LOCALE.to_string()
        } else {
            cur.clone()
        }
    })
}

/// Translate `key` in the active locale; returns the key unchanged when no
/// translation is registered.
pub fn translate(key: &str) -> String {
    let loc = locale();
    MESSAGES.with(|m| {
        m.borrow()
            .get(&(loc, key.to_string()))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    })
}

/// Switch the active locale and notify every [`use_i18n`] subscriber.
pub fn set_locale(cx: &mut App, locale: &str) {
    let changed = LOCALE.with(|l| {
        let mut cur = l.borrow_mut();
        if *cur == locale {
            false
        } else {
            *cur = locale.to_string();
            true
        }
    });
    if !changed {
        return;
    }
    let rev = REVISION.with(|r| {
        let next = r.get() + 1;
        r.set(next);
        next
    });
    SUBSCRIBERS.with(|s| {
        for (_, writer) in s.borrow().iter() {
            writer.set(cx, rev);
        }
    });
}

/// Subscribe the current render scope to locale changes.
///
/// Returns a revision signal: read it (`.get()`) before calling [`translate`]
/// (or the [`t!`] macro) so this subtree re-renders on [`set_locale`]. Pair
/// with [`on_cleanup`] automatically — the subscription detaches when the
/// owning scope is disposed.
pub fn use_i18n() -> crate::reactive::ReadSignal<u64> {
    let rev = REVISION.with(|r| r.get());
    let (reader, writer) = create_signal(rev);
    let id = NEXT_SUB_ID.with(|n| {
        let id = n.get();
        n.set(id + 1);
        id
    });
    SUBSCRIBERS.with(|s| s.borrow_mut().push((id, writer)));
    on_cleanup(move || {
        SUBSCRIBERS.with(|s| s.borrow_mut().retain(|(sub_id, _)| *sub_id != id));
    });
    reader
}

/// Translate a key while tracking locale changes in the current component.
///
/// Expands to a read of the locale-revision signal followed by a lookup, so
/// callers re-render when [`set_locale`] runs.
#[macro_export]
macro_rules! t {
    ($key:expr) => {{
        let __vgui_i18n_rev = $crate::use_i18n();
        let __vgui_i18n_n = __vgui_i18n_rev.get();
        let _ = __vgui_i18n_n;
        $crate::translate($key)
    }};
}

/// Register message tables for one or more locales.
///
/// ```
/// vgui::i18n! {
///     "en" => { "ok" => "OK" },
///     "fr" => { "ok" => "D'accord" }
/// }
/// ```
#[macro_export]
macro_rules! i18n {
    ( $( $locale:expr => { $( $key:expr => $value:expr ),* $(,)? } ),* $(,)? ) => {
        $(
            $crate::register_messages($locale, &[ $( ($key, $value) ),* ]);
        )*
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_default_locale_and_falls_back_to_key() {
        // The default locale before any `set_locale` is [`DEFAULT_LOCALE`]
        // ("en"), so this stays free of any application context.
        assert_eq!(locale(), DEFAULT_LOCALE);
        register_messages(DEFAULT_LOCALE, &[("ok", "OK"), ("cancel", "Cancel")]);
        assert_eq!(translate("ok"), "OK");
        assert_eq!(translate("no.such.key"), "no.such.key");
    }
}
