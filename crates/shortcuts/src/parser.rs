//! Shortcut-string parser.
//!
//! Splits a combination (`"Ctrl+K, Ctrl+P"`) into a sequence of simultaneous
//! matcher groups, resolving each token through the macro registry. Supports
//! backslash-escaping of the `,` and `+` delimiters.

use crate::error::ShortcutsError;
use crate::matcher::Matcher;
use crate::registry::MacroRegistry;

/// Parse `combination` into `Vec<Vec<Box<dyn Matcher>>>` (outer = sequence
/// steps, inner = simultaneous matchers).
pub fn parse_shortcut_key(
    combination: &str,
    registry: &dyn MacroRegistry,
    key_delimiter: char,
    combo_delimiter: char,
) -> Result<Vec<Vec<Box<dyn Matcher>>>, ShortcutsError> {
    // Single-char delimiters only.
    let steps = split(combination, combo_delimiter)?;
    steps
        .into_iter()
        .map(|step| {
            let tokens = split(&step, key_delimiter)?;
            let matchers = tokens
                .into_iter()
                .filter(|t| !t.is_empty())
                .map(|token| {
                    registry
                        .get(&token)
                        .ok_or_else(|| ShortcutsError::MacroNotFound(token.clone()))
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(matchers)
        })
        .collect()
}

/// Backslash-escape-aware split on a single-char delimiter.
///
/// Mirrors the TS `split`: scan left-to-right; a delimiter not preceded by `\`
/// starts a new segment; a `\` immediately preceding the delimiter is stripped
/// from the token (so `\,` yields a literal `,`).
fn split(input: &str, delimiter: char) -> Result<Vec<String>, ShortcutsError> {
    let chars: Vec<char> = input.chars().collect();
    if chars.is_empty() {
        return Ok(vec![String::new()]);
    }
    let mut segments: Vec<String> = vec![String::new()];
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == delimiter {
            // Escaped delimiter: append literally, drop the backslash.
            if i > 0 && chars[i - 1] == '\\' {
                let last = segments.last_mut().unwrap();
                // Remove the trailing backslash we already appended.
                last.pop();
                last.push(c);
            } else {
                segments.push(String::new());
            }
        } else {
            segments.last_mut().unwrap().push(c);
        }
        i += 1;
    }
    Ok(segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_basic() {
        assert_eq!(split("a,b,c", ',').unwrap(), vec!["a", "b", "c"]);
    }

    #[test]
    fn split_escaped() {
        assert_eq!(split(r"a\,b", ',').unwrap(), vec!["a,b"]);
    }

    #[test]
    fn split_empty() {
        assert_eq!(split("", ',').unwrap(), vec![""]);
    }
}
