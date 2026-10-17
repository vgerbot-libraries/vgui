//! Lightweight code editor view with lexical syntax highlighting.
//!
//! A small dependency-free lexer ([`tokenize`]) classifies source into
//! keywords, strings, comments, numbers, function calls and types for a few
//! common languages, and [`code_editor`] renders every line through gpui's
//! [`StyledText`] highlight runs (with per-token color and italic comments).
//! Block comments carry across lines.
//!
//! Scope: this is a single-pass lexical highlighter for display, not a
//! parser or semantic analyzer — it deliberately makes no claim of semantic
//! or language-server-level analysis.

use gpui::{
    div, hsla, px, AnyElement, FontStyle, HighlightStyle, IntoElement, ParentElement, Styled,
    StyledText,
};
use vgui::__var_color;

/// Languages the lexer has keyword/comment rules for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    /// `//` and `/* */` comments, Rust keyword set.
    Rust,
    /// `//` and `/* */` comments, JS/TS keyword set.
    JavaScript,
    /// `#` comments, Python keyword set.
    Python,
}

impl Language {
    fn line_comment(self) -> &'static str {
        match self {
            Language::Python => "#",
            _ => "//",
        }
    }

    fn keywords(self) -> &'static [&'static str] {
        match self {
            Language::Rust => &[
                "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else",
                "enum", "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop",
                "match", "mod", "move", "mut", "pub", "ref", "return", "self", "Self", "static",
                "struct", "super", "trait", "true", "type", "unsafe", "use", "where", "while",
            ],
            Language::JavaScript => &[
                "async", "await", "break", "case", "catch", "class", "const", "continue",
                "debugger", "default", "delete", "do", "else", "export", "extends", "false",
                "finally", "for", "from", "function", "if", "import", "in", "instanceof", "let",
                "new", "null", "of", "return", "static", "super", "switch", "this", "throw",
                "true", "try", "typeof", "undefined", "var", "void", "while", "yield",
            ],
            Language::Python => &[
                "False", "None", "True", "and", "as", "assert", "async", "await", "break",
                "class", "continue", "def", "del", "elif", "else", "except", "finally", "for",
                "from", "global", "if", "import", "in", "is", "lambda", "nonlocal", "not", "or",
                "pass", "raise", "return", "try", "while", "with", "yield",
            ],
        }
    }
}

/// Syntactic class of a token.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    /// Language keyword / reserved word.
    Keyword,
    /// String or character literal.
    String,
    /// Line or block comment.
    Comment,
    /// Numeric literal.
    Number,
    /// Identifier immediately followed by `(`.
    Function,
    /// Capitalised identifier (type / constructor).
    Type,
}

/// One lexed token with its byte range inside a line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// Byte range within the line.
    pub range: std::ops::Range<usize>,
    /// Syntactic class.
    pub kind: TokenKind,
}

fn highlight(kind: TokenKind) -> HighlightStyle {
    let color = match kind {
        TokenKind::Keyword => hsla(0.73, 0.55, 0.62, 1.0),
        TokenKind::String => hsla(0.36, 0.5, 0.38, 1.0),
        TokenKind::Comment => hsla(0.0, 0.0, 0.5, 1.0),
        TokenKind::Number => hsla(0.07, 0.7, 0.5, 1.0),
        TokenKind::Function => hsla(0.58, 0.65, 0.45, 1.0),
        TokenKind::Type => hsla(0.5, 0.5, 0.42, 1.0),
    };
    HighlightStyle {
        color: Some(color),
        font_style: if kind == TokenKind::Comment {
            Some(FontStyle::Italic)
        } else {
            None
        },
        ..Default::default()
    }
}

/// Stateful lexer. Block comments (`/* */`, for non-Python languages) span
/// lines, so one highlighter is fed successive lines.
pub struct Highlighter {
    lang: Language,
    in_block_comment: bool,
}

impl Highlighter {
    /// Create a highlighter for `lang`.
    pub fn new(lang: Language) -> Self {
        Self {
            lang,
            in_block_comment: false,
        }
    }

    /// Whether the lexer is currently inside an unterminated block comment.
    pub fn in_block_comment(&self) -> bool {
        self.in_block_comment
    }
    /// Tokenize one line. Byte ranges are relative to this line only.
    pub fn tokenize_line(&mut self, line: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;

        // Continuation of a block comment from a previous line.
        if self.in_block_comment {
            if let Some(rel) = line.find("*/") {
                let end = rel + 2;
                tokens.push(Token {
                    range: 0..end,
                    kind: TokenKind::Comment,
                });
                self.in_block_comment = false;
                i = end;
            } else {
                tokens.push(Token {
                    range: 0..line.len(),
                    kind: TokenKind::Comment,
                });
                return tokens;
            }
        }

        let lc = self.lang.line_comment();
        let block = !matches!(self.lang, Language::Python);

        while i < bytes.len() {
            let b = bytes[i];

            if b.is_ascii_whitespace() {
                i += 1;
                continue;
            }

            // Line comment: rest of the line.
            if line[i..].starts_with(lc) {
                tokens.push(Token {
                    range: i..line.len(),
                    kind: TokenKind::Comment,
                });
                break;
            }

            // Block comment start.
            if block && line[i..].starts_with("/*") {
                let after = i + 2;
                if let Some(rel) = line[after..].find("*/") {
                    tokens.push(Token {
                        range: i..after + rel + 2,
                        kind: TokenKind::Comment,
                    });
                    i = after + rel + 2;
                } else {
                    tokens.push(Token {
                        range: i..line.len(),
                        kind: TokenKind::Comment,
                    });
                    self.in_block_comment = true;
                    break;
                }
                continue;
            }

            // String literal: " ' or ` (backtick not used by Python).
            if b == b'"' || b == b'\'' || (b == b'`' && !matches!(self.lang, Language::Python)) {
                let quote = b;
                let start = i;
                i += 1;
                let mut closed = false;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == quote {
                        i += 1;
                        closed = true;
                        break;
                    }
                    i += 1;
                }
                tokens.push(Token {
                    range: start..if closed { i } else { line.len() },
                    kind: TokenKind::String,
                });
                if !closed {
                    break;
                }
                continue;
            }

            // Numeric literal (incl. 0x hex and simple floats).
            if b.is_ascii_digit() {
                let start = i;
                while i < bytes.len()
                    && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'.' || bytes[i] == b'_')
                {
                    i += 1;
                }
                tokens.push(Token {
                    range: start..i,
                    kind: TokenKind::Number,
                });
                continue;
            }

            // Identifier.
            if b.is_ascii_alphabetic() || b == b'_' {
                let start = i;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let word = &line[start..i];
                let kind = if self.lang.keywords().contains(&word) {
                    TokenKind::Keyword
                } else if word.chars().next().is_some_and(|c| c.is_uppercase()) {
                    TokenKind::Type
                } else {
                    // Function if the next non-space token is '(' (or '!('
                    // for a Rust macro invocation such as `println!`).
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    let is_call = bytes.get(j) == Some(&b'(')
                        || (bytes.get(j) == Some(&b'!')
                            && bytes.get(j + 1) == Some(&b'('));
                    if is_call {
                        TokenKind::Function
                    } else {
                        continue;
                    }
                };
                tokens.push(Token {
                    range: start..i,
                    kind,
                });
                continue;
            }

            // Punctuation / operator: unhighlighted.
            i += 1;
        }

        tokens
    }
}

/// Tokenize a full (possibly multi-line) source string. Returns one token
/// vector per input line, suitable for feeding a renderer.
pub fn tokenize(source: &str, lang: Language) -> Vec<Vec<Token>> {
    let mut hl = Highlighter::new(lang);
    source
        .lines()
        .map(|line| hl.tokenize_line(line))
        .collect()
}

/// Render source as a read-only, highlighted code view.
pub fn code_editor(source: impl Into<String>, lang: Language) -> AnyElement {
    let source = source.into();
    let per_line = tokenize(&source, lang);

    let border = __var_color("--border", Some(hsla(0., 0., 0.85, 1.)));
    let bg = __var_color("--muted", Some(hsla(0., 0., 0.97, 1.)));
    let fg = __var_color("--foreground", Some(hsla(0., 0., 0.12, 1.)));
    let gutter = __var_color("--muted-foreground", Some(hsla(0., 0., 0.5, 1.)));

    let text_style = gpui::TextStyle {
        color: fg,
        font_family: "monospace".into(),
        font_size: px(13.).into(),
        ..Default::default()
    };

    let mut body = div().flex().flex_col().w_full().py_2().bg(bg);

    for (idx, (line, tokens)) in source.lines().zip(per_line.iter()).enumerate() {
        let highlights = tokens
            .iter()
            .map(|t| (t.range.clone(), highlight(t.kind)))
            .collect::<Vec<_>>();

        let rendered = if line.is_empty() {
            StyledText::new(" ").with_default_highlights(&text_style, None)
        } else {
            StyledText::new(line.to_string())
                .with_default_highlights(&text_style, highlights)
        };

        let row = div()
            .flex()
            .flex_row()
            .w_full()
            .child(
                div()
                    .w(px(40.))
                    .flex_shrink_0()
                    .font_family("monospace")
                    .text_size(px(13.))
                    .text_color(gutter)
                    .child(format!("{}", idx + 1)),
            )
            .child(div().flex_1().child(rendered));
        body = body.child(row);
    }

    div()
        .w_full()
        .border_1()
        .border_color(border)
        .rounded(px(vgui::__var_number("--radius", Some(6.))))
        .child(body)
        .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_rust_line() {
        let src = "let x = 42; // hi";
        let toks = Highlighter::new(Language::Rust).tokenize_line(src);
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Keyword && &src[t.range.clone()] == "let"));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Number && &src[t.range.clone()] == "42"));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Comment && &src[t.range.clone()] == "// hi"));
    }

    #[test]
    fn lexes_string_and_function() {
        let src = "println!(\"hi\")";
        let toks = Highlighter::new(Language::Rust).tokenize_line(src);
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::String && &src[t.range.clone()] == "\"hi\""));
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Function && &src[t.range.clone()] == "println"));
    }

    #[test]
    fn block_comment_spans_lines() {
        let mut hl = Highlighter::new(Language::Rust);
        let first = hl.tokenize_line("let a = 1; /* start");
        assert!(first.iter().any(|t| t.kind == TokenKind::Comment));
        assert!(hl.in_block_comment());
        let mid = hl.tokenize_line("still comment");
        assert!(mid.iter().all(|t| t.kind == TokenKind::Comment));
        let last = hl.tokenize_line("end */ let b = 2;");
        assert!(!hl.in_block_comment());
        assert!(last.iter().any(|t| t.kind == TokenKind::Keyword));
    }

    #[test]
    fn python_uses_hash_comments() {
        let src = "x = 1  # note";
        let toks = Highlighter::new(Language::Python).tokenize_line(src);
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Comment && &src[t.range.clone()] == "# note"));
        // `//` is not a Python comment.
        let no_comment = Highlighter::new(Language::Python).tokenize_line("a // b");
        assert!(!no_comment.iter().any(|t| t.kind == TokenKind::Comment));
    }

    #[test]
    fn uppercase_identifier_is_a_type() {
        let src = "let s = String::new();";
        let toks = Highlighter::new(Language::Rust).tokenize_line(src);
        assert!(toks
            .iter()
            .any(|t| t.kind == TokenKind::Type && &src[t.range.clone()] == "String"));
    }

    #[test]
    fn tokenize_produces_one_vec_per_line() {
        let lines = tokenize("fn a() {}\nfn b() {}", Language::Rust);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].iter().any(|t| t.kind == TokenKind::Keyword));
        assert!(lines[1].iter().any(|t| t.kind == TokenKind::Function));
    }
}
