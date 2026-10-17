//! Native Markdown rendering.
//!
//! A small, dependency-free CommonMark subset is parsed to [`Block`]s /
//! [`Span`]s ([`Markdown::parse`]) and rendered to gpui elements
//! ([`Markdown::render`], or the [`markdown`] convenience function).
//!
//! Supported syntax:
//! - headings `#`..`######`
//! - paragraphs and hard line breaks inside a paragraph
//! - fenced code blocks ```` ``` ````
//! - bullet (`-`/`*`/`+`) and numbered (`1.`) lists
//! - blockquote `>`
//! - thematic break `---`
//! - inline `**bold**`, `*italic*`/`_italic_`, `` `code` ``, `[link](url)`

use gpui::{
    div, px, AnyElement, FontWeight, IntoElement, ParentElement, Styled,
};
use vgui::__var_color;

/// Inline content.
#[derive(Clone, Debug, PartialEq)]
pub enum Span {
    /// Plain text.
    Text(String),
    /// `**bold**`
    Bold(Vec<Span>),
    /// `*italic*`
    Italic(Vec<Span>),
    /// `` `code` ``
    Code(String),
    /// `[label](url)`
    Link { label: String, url: String },
}

/// Block-level content.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    /// `#`..`######` heading (level 1..=6).
    Heading(u8, Vec<Span>),
    /// A paragraph of inline spans.
    Paragraph(Vec<Span>),
    /// A fenced code block (raw, unhighlighted source).
    CodeBlock(String),
    /// `>` quoted blocks.
    Quote(Vec<Block>),
    /// `ordered` and a list of item bodies (each a list of spans).
    List { ordered: bool, items: Vec<Vec<Span>> },
    /// A `---` horizontal rule.
    Divider,
}

/// Parsed Markdown document.
#[derive(Clone, Debug, PartialEq)]
pub struct Markdown {
    blocks: Vec<Block>,
}

impl Markdown {
    /// Parse a Markdown source string.
    pub fn parse(src: &str) -> Self {
        Self {
            blocks: parse_blocks(&split_lines(src)),
        }
    }

    /// The parsed block tree.
    pub fn blocks(&self) -> &[Block] {
        &self.blocks
    }

    /// Number of top-level blocks.
    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Whether the document has no blocks.
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    /// Render to a gpui element.
    pub fn render(&self) -> AnyElement {
        render_blocks(&self.blocks, 0)
    }
}

/// Parse and render Markdown in one call.
pub fn markdown(src: impl Into<String>) -> AnyElement {
    Markdown::parse(&src.into()).render()
}

fn split_lines(src: &str) -> Vec<&str> {
    src.lines().collect()
}

fn parse_blocks(lines: &[&str]) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim_start();

        // Fenced code block: consume until the closing fence.
        if let Some(lang_rest) = fence(trimmed) {
            let _ = lang_rest;
            let mut code = String::new();
            i += 1;
            while i < lines.len() && fence(lines[i].trim_start()).is_none() {
                if !code.is_empty() {
                    code.push('\n');
                }
                code.push_str(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // closing fence
            }
            blocks.push(Block::CodeBlock(code));
            continue;
        }

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(Block::Divider);
            i += 1;
            continue;
        }

        if let Some(level) = heading_level(trimmed) {
            let text = trimmed[level as usize + 1..].trim();
            blocks.push(Block::Heading(level, parse_inline(text)));
            i += 1;
            continue;
        }

        // Blockquote: gather consecutive `>` lines, then recursively parse.
        if trimmed.starts_with('>') {
            let mut quoted: Vec<&str> = Vec::new();
            while i < lines.len() && lines[i].trim_start().starts_with('>') {
                let rest = lines[i].trim_start().trim_start_matches('>');
                let rest = rest.strip_prefix(' ').unwrap_or(rest);
                quoted.push(rest);
                i += 1;
            }
            blocks.push(Block::Quote(parse_blocks(&quoted)));
            continue;
        }

        if list_marker(trimmed).is_some() {
            let ordered = matches!(list_marker(trimmed), Some((_, true)));
            let mut items = Vec::new();
            while i < lines.len() {
                let t = lines[i].trim_start();
                match list_marker(t) {
                    Some((content, ord)) if ord == ordered => {
                        items.push(parse_inline(content.trim()));
                        i += 1;
                    }
                    _ => break,
                }
            }
            blocks.push(Block::List { ordered, items });
            continue;
        }

        // Paragraph: consume consecutive non-empty, non-special lines.
        let start = i;
        while i < lines.len() {
            let t = lines[i].trim_start();
            if t.is_empty()
                || heading_level(t).is_some()
                || fence(t).is_some()
                || t.starts_with('>')
                || list_marker(t).is_some()
                || t == "---"
            {
                break;
            }
            i += 1;
        }
        let joined = lines[start..i].join(" ");
        blocks.push(Block::Paragraph(parse_inline(&joined)));
    }
    blocks
}

/// Returns Some(rest_after_fence) for a ``` / ~~~ fence line.
fn fence(line: &str) -> Option<&str> {
    if line.starts_with("```") {
        Some(line[3..].trim())
    } else if line.starts_with("~~~") {
        Some(line[3..].trim())
    } else {
        None
    }
}

/// Heading level 1..=6 for a `#`-prefixed line, requiring a trailing space/eol.
fn heading_level(line: &str) -> Option<u8> {
    let hashes = line.bytes().take_while(|b| *b == b'#').count();
    if (1..=6).contains(&hashes) && line.as_bytes().get(hashes) == Some(&b' ') {
        Some(hashes as u8)
    } else {
        None
    }
}

/// Returns Some((content_after_marker, ordered)) for a list-item line.
fn list_marker(line: &str) -> Option<(&str, bool)> {
    if let Some(rest) = line.strip_prefix("- ").or_else(|| line.strip_prefix("* ")) {
        return Some((rest, false));
    }
    if let Some(rest) = line.strip_prefix("+ ") {
        return Some((rest, false));
    }
    // "1. " style: digits followed by ". "
    let digits = line.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits > 0 && line.as_bytes().get(digits) == Some(&b'.') {
        if let Some(rest) = line[digits + 1..].strip_prefix(' ') {
            return Some((rest, true));
        }
    }
    None
}

// ── Inline parsing ─────────────────────────────────────────────────────────

fn parse_inline(src: &str) -> Vec<Span> {
    let bytes = src.as_bytes();
    let mut spans = Vec::new();
    let mut text = String::new();
    let mut i = 0;

    let flush = |text: &mut String, spans: &mut Vec<Span>| {
        if !text.is_empty() {
            spans.push(Span::Text(std::mem::take(text)));
        }
    };

    while i < bytes.len() {
        let c = bytes[i] as char;

        // `code`
        if c == '`' {
            if let Some(end) = src[i + 1..].find('`') {
                flush(&mut text, &mut spans);
                spans.push(Span::Code(src[i + 1..i + 1 + end].to_string()));
                i += 2 + end;
                continue;
            }
        }

        // [label](url)
        if c == '[' {
            if let Some(close) = src[i + 1..].find(']') {
                let after = i + 1 + close + 1;
                if src.as_bytes().get(after) == Some(&b'(') {
                    if let Some(pclose) = src[after + 1..].find(')') {
                        flush(&mut text, &mut spans);
                        spans.push(Span::Link {
                            label: src[i + 1..i + 1 + close].to_string(),
                            url: src[after + 1..after + 1 + pclose].to_string(),
                        });
                        i = after + 1 + pclose + 1;
                        continue;
                    }
                }
            }
        }

        // **bold**
        if c == '*' && bytes.get(i + 1) == Some(&b'*') {
            if let Some(end) = src[i + 2..].find("**") {
                flush(&mut text, &mut spans);
                spans.push(Span::Bold(parse_inline(&src[i + 2..i + 2 + end])));
                i += 4 + end;
                continue;
            }
        }

        // *italic* or _italic_ (single delimiter, not part of **)
        if c == '*' || c == '_' {
            if let Some(end) = src[i + 1..].find(c) {
                flush(&mut text, &mut spans);
                spans.push(Span::Italic(parse_inline(&src[i + 1..i + 1 + end])));
                i += 2 + end;
                continue;
            }
        }

        text.push(c);
        i += 1;
    }
    flush(&mut text, &mut spans);
    spans
}

// ── Rendering ──────────────────────────────────────────────────────────────

fn render_blocks(blocks: &[Block], depth: usize) -> AnyElement {
    let mut col = div().flex().flex_col().gap_1();
    if depth == 0 {
        col = col.w_full();
    }
    for block in blocks {
        col = col.child(render_block(block, depth));
    }
    col.into_any_element()
}

fn render_block(block: &Block, depth: usize) -> AnyElement {
    let foreground = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));
    let muted = __var_color("--muted-foreground", Some(gpui::hsla(0., 0., 0.4, 1.)));
    let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));

    match block {
        Block::Heading(level, spans) => {
            let size = match level {
                1 => px(26.),
                2 => px(22.),
                3 => px(19.),
                4 => px(17.),
                _ => px(15.),
            };
            let mut h = div()
                .text_size(size)
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(foreground)
                .mt_1();
            h = render_inline_into(h, spans, true);
            h.into_any_element()
        }
        Block::Paragraph(spans) => {
            let mut p = div().flex().flex_wrap().text_size(px(14.)).text_color(foreground);
            p = render_inline_into(p, spans, false);
            p.into_any_element()
        }
        Block::CodeBlock(code) => div()
            .w_full()
            .bg(__var_color("--muted", Some(gpui::hsla(0., 0., 0.95, 1.))))
            .border_1()
            .border_color(border)
            .rounded(px(vgui::__var_number("--radius", Some(6.))))
            .px_3()
            .py_2()
            .text_size(px(13.))
            .text_color(foreground)
            .font_family("monospace")
            .child(code.clone())
            .into_any_element(),
        Block::Quote(children) => div()
            .border_l_2()
            .border_color(border)
            .pl_3()
            .text_color(muted)
            .child(render_blocks(children, depth + 1))
            .into_any_element(),
        Block::List { ordered, items } => {
            let mut list = div().flex().flex_col().gap_1().pl_1();
            for (idx, item) in items.iter().enumerate() {
                let bullet = if *ordered {
                    format!("{}. ", idx + 1)
                } else {
                    "• ".to_string()
                };
                let mut row = div().flex().flex_row().text_size(px(14.)).text_color(foreground);
                row = row
                    .child(div().text_color(muted).child(bullet))
                    .child(render_spans(item, false));
                list = list.child(row);
            }
            list.into_any_element()
        }
        Block::Divider => div()
            .w_full()
            .h(px(1.))
            .bg(border)
            .my_1()
            .into_any_element(),
    }
}

fn render_inline_into<E: ParentElement + Styled>(parent: E, spans: &[Span], _strong: bool) -> E {
    let mut el = parent;
    for span in spans {
        el = el.child(render_span(span));
    }
    el
}

fn render_spans(spans: &[Span], _strong: bool) -> AnyElement {
    let mut row = div().flex().flex_row().flex_wrap().items_center();
    for span in spans {
        row = row.child(render_span(span));
    }
    row.into_any_element()
}

fn render_span(span: &Span) -> AnyElement {
    let foreground = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));
    let primary = __var_color("--primary", Some(gpui::hsla(0.6, 0.6, 0.45, 1.)));
    let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));

    match span {
        Span::Text(t) => div().text_color(foreground).child(t.clone()).into_any_element(),
        Span::Bold(inner) => div()
            .font_weight(FontWeight::BOLD)
            .text_color(foreground)
            .child(render_spans(inner, true))
            .into_any_element(),
        Span::Italic(inner) => div()
            .italic()
            .text_color(foreground)
            .child(render_spans(inner, false))
            .into_any_element(),
        Span::Code(code) => div()
            .font_family("monospace")
            .text_size(px(12.5))
            .bg(__var_color("--muted", Some(gpui::hsla(0., 0., 0.95, 1.))))
            .border_1()
            .border_color(border)
            .rounded(px(4.))
            .px_1()
            .text_color(foreground)
            .child(code.clone())
            .into_any_element(),
        Span::Link { label, url } => div()
            .text_color(primary)
            .underline()
            .child(format!("{label} <{url}>"))
            .into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_heading_and_paragraph() {
        let md = Markdown::parse("# Title\n\nHello **world**");
        assert_eq!(md.blocks().len(), 2);
        assert_eq!(md.blocks()[0], Block::Heading(1, vec![Span::Text("Title".into())]));
        match &md.blocks()[1] {
            Block::Paragraph(spans) => assert!(spans.contains(&Span::Bold(vec![Span::Text(
                "world".into()
            )]))),
            other => panic!("expected paragraph, got {other:?}"),
        }
    }

    #[test]
    fn parses_lists_and_code() {
        let md = Markdown::parse("- a\n- b\n\n```\nlet x = 1;\n```\n1. first\n2. second");
        assert!(matches!(
            md.blocks()[0],
            Block::List { ordered: false, .. }
        ));
        assert_eq!(md.blocks()[1], Block::CodeBlock("let x = 1;".into()));
        match &md.blocks()[2] {
            Block::List { ordered, items } => {
                assert!(ordered);
                assert_eq!(items.len(), 2);
            }
            other => panic!("expected ordered list, got {other:?}"),
        }
    }

    #[test]
    fn parses_inline_variants() {
        assert_eq!(
            parse_inline("a *b* c"),
            vec![
                Span::Text("a ".into()),
                Span::Italic(vec![Span::Text("b".into())]),
                Span::Text(" c".into()),
            ]
        );
        assert_eq!(
            parse_inline("`x` and [l](u)"),
            vec![
                Span::Code("x".into()),
                Span::Text(" and ".into()),
                Span::Link {
                    label: "l".into(),
                    url: "u".into()
                },
            ]
        );
    }

    #[test]
    fn parses_quote_and_divider() {
        let md = Markdown::parse("> quoted\n\n---\n");
        assert!(matches!(md.blocks()[0], Block::Quote(_)));
        assert_eq!(md.blocks()[1], Block::Divider);
    }
}
