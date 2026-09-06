//! Unified error type for the shortcuts engine.

#[derive(Debug)]
pub enum ShortcutsError {
    /// A shortcut token did not resolve to any registered macro.
    MacroNotFound(String),
    /// A multi-char delimiter was supplied to the parser.
    InvalidDelimiter(String),
    /// `alias` was given an origin pattern that is not registered.
    AliasOriginNotFound(String),
    /// `switch_context` was given an unregistered context name.
    ContextNotRegistered(String),
    /// `on` was given a command name that has no keymap entry.
    CommandNotRegistered(String),
}

impl std::fmt::Display for ShortcutsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MacroNotFound(t) => write!(f, "Macro not found: {t}"),
            Self::InvalidDelimiter(d) => write!(f, "The delimiter should be a single character: {d}"),
            Self::AliasOriginNotFound(t) => {
                write!(f, "Cannot set an alias whose origin pattern does not exist: {t}")
            }
            Self::ContextNotRegistered(c) => {
                write!(f, "Cannot switch to a non-existent context: {c}")
            }
            Self::CommandNotRegistered(c) => {
                write!(f, "Command has not been registered: {c}")
            }
        }
    }
}

impl std::error::Error for ShortcutsError {}
