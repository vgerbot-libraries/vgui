//! A faithful Rust port of `@vgerbot/shortcuts`: configurable keyboard
//! shortcuts with human-readable combination strings (`"Ctrl+K, Ctrl+P"`), a
//! macro registry that resolves token names to matchers, a stateful sequence
//! matcher, a context stack with fallbacks, and an interceptor middleware
//! chain.
//!
//! Pure (no gpui/vgui dependency). The host constructs
//! [`ShortcutKeyboardEvent`] from its own event type and drives
//! [`Keyboard::fire`].

mod context;
mod error;
mod event;
mod interceptor;
mod keyboard;
mod locale;
mod macro_api;
mod matcher;
mod matchers;
mod parser;
mod registry;
mod shortcut;

pub use context::{CommandOptions, ContextOptions, KeymapOptions, ShortcutContext};
pub use error::ShortcutsError;
pub use event::{KeyEventType, ShortcutKeyboardEvent};
pub use interceptor::{Interceptor, ShortcutEvent};
pub use keyboard::{ContextGuard, FireMatch, InterceptorGuard, Keyboard};
pub use macro_api::{alias, code_macro, key_macro, key_macro_ins, macro_from_str, macro_register};
pub use matcher::{Matcher, ModifierKind};
pub use matchers::{
    AndMatcher, CaseInsensitiveKeyMatcher, CaseSensitiveKeyMatcher, CodeMatcher, KeyMatcher,
    MacroMatcher, ModifierMatcher, OrMatcher,
};
pub use parser::parse_shortcut_key;
pub use registry::{default_macro_registry, EmptyMacroRegistry, MacroRegistry, MacroRegistryImpl};
pub use shortcut::Shortcut;

pub mod prelude {
    pub use crate::context::{CommandOptions, ContextOptions, KeymapOptions, ShortcutContext};
    pub use crate::event::{KeyEventType, ShortcutKeyboardEvent};
    pub use crate::interceptor::{Interceptor, ShortcutEvent};
    pub use crate::keyboard::{ContextGuard, FireMatch, InterceptorGuard, Keyboard};
    pub use crate::matcher::{Matcher, ModifierKind};
    pub use crate::matchers::{
        AndMatcher, CaseInsensitiveKeyMatcher, CaseSensitiveKeyMatcher, CodeMatcher, KeyMatcher,
        MacroMatcher, ModifierMatcher, OrMatcher,
    };
    pub use crate::registry::{default_macro_registry, MacroRegistry, MacroRegistryImpl};
    pub use crate::shortcut::Shortcut;
}
