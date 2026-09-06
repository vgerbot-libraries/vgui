//! Macro registry: resolves token names to matchers, with a parent chain.
//!
//! Mirrors the TypeScript `MacroRegistry` / `MacroRegistryImpl`. A child
//! registry shadows its parent: local registrations override inherited tokens.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::locale::register_defaults;
use crate::matcher::Matcher;

/// Resolves shortcut tokens (`"Ctrl"`, `"Enter"`, a user macro…) to matchers.
pub trait MacroRegistry {
    fn register(&self, pattern: &str, matcher: Box<dyn Matcher>);
    fn exists(&self, pattern: &str) -> bool;
    /// Returns a fresh boxed clone of the matcher for `pattern`, if any.
    fn get(&self, pattern: &str) -> Option<Box<dyn Matcher>>;
    fn all_patterns(&self) -> Vec<String>;
}

/// No-op registry. The root parent when no real parent is supplied.
pub struct EmptyMacroRegistry;

impl MacroRegistry for EmptyMacroRegistry {
    fn register(&self, _pattern: &str, _matcher: Box<dyn Matcher>) {}
    fn exists(&self, _pattern: &str) -> bool {
        false
    }
    fn get(&self, _pattern: &str) -> Option<Box<dyn Matcher>> {
        None
    }
    fn all_patterns(&self) -> Vec<String> {
        Vec::new()
    }
}

/// A registry with an optional parent. Local entries shadow the parent.
pub struct MacroRegistryImpl {
    patterns: RefCell<HashMap<String, Box<dyn Matcher>>>,
    parent: Rc<dyn MacroRegistry>,
}

impl MacroRegistryImpl {
    pub fn new() -> Self {
        Self {
            patterns: RefCell::new(HashMap::new()),
            parent: Rc::new(EmptyMacroRegistry),
        }
    }

    pub fn with_parent(parent: Rc<dyn MacroRegistry>) -> Self {
        Self {
            patterns: RefCell::new(HashMap::new()),
            parent,
        }
    }
}

impl Default for MacroRegistryImpl {
    fn default() -> Self {
        Self::new()
    }
}

impl MacroRegistry for MacroRegistryImpl {
    fn register(&self, pattern: &str, matcher: Box<dyn Matcher>) {
        self.patterns.borrow_mut().insert(pattern.to_string(), matcher);
    }

    fn exists(&self, pattern: &str) -> bool {
        self.patterns.borrow().contains_key(pattern) || self.parent.exists(pattern)
    }

    fn get(&self, pattern: &str) -> Option<Box<dyn Matcher>> {
        if let Some(m) = self.patterns.borrow().get(pattern) {
            return Some(m.clone_box());
        }
        self.parent.get(pattern)
    }

    fn all_patterns(&self) -> Vec<String> {
        let mut all = self.parent.all_patterns();
        let local: Vec<String> = self.patterns.borrow().keys().cloned().collect();
        all.extend(local);
        // Dedup, keeping first occurrence (parent first).
        let mut seen = std::collections::HashSet::new();
        all.retain(|p| seen.insert(p.clone()));
        all
    }
}

/// A fresh registry preloaded with the built-in token table
/// (see [`register_defaults`]). Cheap to rebuild; no global singleton.
pub fn default_macro_registry() -> Rc<MacroRegistryImpl> {
    let r = MacroRegistryImpl::new();
    register_defaults(&r);
    Rc::new(r)
}

/// Blanket impl so `Rc<dyn MacroRegistry>` and `Rc<MacroRegistryImpl>` can be
/// passed where `&dyn MacroRegistry` is expected.
impl<T: MacroRegistry + ?Sized> MacroRegistry for Rc<T> {
    fn register(&self, pattern: &str, matcher: Box<dyn Matcher>) {
        (**self).register(pattern, matcher)
    }
    fn exists(&self, pattern: &str) -> bool {
        (**self).exists(pattern)
    }
    fn get(&self, pattern: &str) -> Option<Box<dyn Matcher>> {
        (**self).get(pattern)
    }
    fn all_patterns(&self) -> Vec<String> {
        (**self).all_patterns()
    }
}
