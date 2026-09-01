use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::ToTokens;

use crate::{Attr, AttrKind, AttrValue, Element, Node};

pub(crate) fn skip_ws(_tokens: &[TokenTree], _i: &mut usize) {}

pub(crate) fn parse_node(tokens: &[TokenTree], i: &mut usize) -> syn::Result<Node> {
    if *i >= tokens.len() {
        return Err(syn::Error::new(Span::call_site(), "expected view node"));
    }
    match &tokens[*i] {
        TokenTree::Punct(p) if p.as_char() == '<' => parse_element_or_fragment(tokens, i),
        TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
            let stream = g.stream();
            *i += 1;
            Ok(Node::Interp(stream))
        }
        TokenTree::Literal(lit) => {
            let lit_ts = TokenStream2::from(TokenTree::Literal(lit.clone()));
            *i += 1;
            if let Ok(s) = syn::parse2::<syn::LitStr>(lit_ts.clone()) {
                Ok(Node::Text(s))
            } else {
                Ok(Node::Interp(lit_ts))
            }
        }
        other => Err(syn::Error::new(
            other.span(),
            "expected element, fragment, or `{expr}`",
        )),
    }
}

pub(crate) fn parse_element_or_fragment(
    tokens: &[TokenTree],
    i: &mut usize,
) -> syn::Result<Node> {
    let span = tokens[*i].span();
    *i += 1; // <
    if *i >= tokens.len() {
        return Err(syn::Error::new(span, "expected tag after `<`"));
    }
    if is_punct(&tokens[*i], '>') {
        *i += 1;
        let mut children = Vec::new();
        loop {
            if *i >= tokens.len() {
                return Err(syn::Error::new(span, "unclosed fragment"));
            }
            if is_punct(&tokens[*i], '<') && *i + 1 < tokens.len() && is_punct(&tokens[*i + 1], '/')
            {
                *i += 2;
                if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                    return Err(syn::Error::new(span, "expected `</>`"));
                }
                *i += 1;
                break;
            }
            children.push(parse_node(tokens, i)?);
        }
        return Ok(Node::Fragment(children));
    }
    let tag = match &tokens[*i] {
        TokenTree::Ident(id) => id.clone(),
        other => return Err(syn::Error::new(other.span(), "expected tag name")),
    };
    *i += 1;
    let mut attrs = Vec::new();
    loop {
        if *i >= tokens.len() {
            return Err(syn::Error::new(tag.span(), "unclosed tag"));
        }
        if is_punct(&tokens[*i], '/') {
            *i += 1;
            if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                return Err(syn::Error::new(tag.span(), "expected `/>`"));
            }
            *i += 1;
            return Ok(Node::Element(Element {
                tag,
                attrs,
                children: Vec::new(),
                self_closing: true,
            }));
        }
        if is_punct(&tokens[*i], '>') {
            // Void/self-closing element: `<input>`, `<br>`, `<hr>`, `<wbr>` without `/>` — treat as self-closing.
            if matches!(tag.to_string().as_str(), "input" | "br" | "hr" | "wbr") {
                *i += 1;
                return Ok(Node::Element(Element {
                    tag,
                    attrs,
                    children: Vec::new(),
                    self_closing: true,
                }));
            }
            *i += 1;
            break;
        }
        // Detect spread syntax: `{..expr}` or `{...expr}` — a brace group
        // whose inner token stream begins with two or more `.` puncts.
        if let TokenTree::Group(g) = &tokens[*i] {
            if g.delimiter() == Delimiter::Brace {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                if let Some(expr) = extract_spread(&inner) {
                    let span = g.span();
                    *i += 1;
                    if expr.is_empty() {
                        return Err(syn::Error::new(
                            span,
                            "spread requires an expression after `..`",
                        ));
                    }
                    attrs.push(Attr {
                        kind: AttrKind::Spread,
                        value: AttrValue::Expr(expr),
                        span,
                    });
                    continue;
                }
            }
        }
        attrs.push(parse_attr(tokens, i)?);
    }
    let mut children = Vec::new();
    loop {
        if *i >= tokens.len() {
            return Err(syn::Error::new(tag.span(), "unclosed tag"));
        }
        if is_punct(&tokens[*i], '<') && *i + 1 < tokens.len() && is_punct(&tokens[*i + 1], '/') {
            *i += 2;
            if *i >= tokens.len() {
                return Err(syn::Error::new(tag.span(), "expected closing tag"));
            }
            let close = match &tokens[*i] {
                TokenTree::Ident(id) => id.clone(),
                other => return Err(syn::Error::new(other.span(), "expected closing tag name")),
            };
            *i += 1;
            if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                return Err(syn::Error::new(
                    close.span(),
                    "expected `>` after closing tag",
                ));
            }
            *i += 1;
            if close.to_string() != tag.to_string() {
                return Err(syn::Error::new(close.span(), "mismatched closing tag"));
            }
            break;
        }
        children.push(parse_node(tokens, i)?);
    }
    let _ = span;
    Ok(Node::Element(Element {
        tag,
        attrs,
        children,
        self_closing: false,
    }))
}

pub(crate) fn is_punct(tt: &TokenTree, ch: char) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ch)
}

/// Check whether `inner` begins with two or more `.` puncts (spread syntax
/// `{..expr}` or `{...expr}`). Returns `Some(expr_tokens)` — the tokens after
/// the leading dots — when the dot count is ≥ 2, `None` otherwise.
fn extract_spread(inner: &[TokenTree]) -> Option<TokenStream2> {
    let mut dots = 0;
    let mut start = 0;
    for (idx, tt) in inner.iter().enumerate() {
        if is_punct(tt, '.') {
            dots += 1;
            start = idx + 1;
        } else {
            break;
        }
    }
    if dots >= 2 {
        let expr: TokenStream2 = inner[start..].iter().cloned().collect();
        Some(expr)
    } else {
        None
    }
}

pub(crate) fn parse_attr(tokens: &[TokenTree], i: &mut usize) -> syn::Result<Attr> {
    let span = tokens[*i].span();
    let kind = match &tokens[*i] {
        TokenTree::Ident(id) => {
            let name = id.to_string();
            *i += 1;
            match name.as_str() {
                "style" => AttrKind::Style,
                "hover" => AttrKind::Hover,
                "active" => AttrKind::Active,
                "focus" => AttrKind::Focus,
                "id" => AttrKind::Id,
                "src" => AttrKind::Src,
                "class" => AttrKind::Class,
                "type" => AttrKind::Type,
                "for" => AttrKind::For,
                "ref" => AttrKind::Ref,
                "animate" => AttrKind::Animate,
                "tabindex" => AttrKind::Tabindex,
                "role" => AttrKind::Role,
                "aria" => {
                    if *i >= tokens.len() || !is_punct(&tokens[*i], ':') {
                        return Err(syn::Error::new(span, "expected `aria:name`"));
                    }
                    *i += 1;
                    match tokens.get(*i) {
                        Some(TokenTree::Ident(name)) => {
                            let name = name.clone();
                            *i += 1;
                            AttrKind::Aria(name)
                        }
                        _ => return Err(syn::Error::new(span, "expected aria attribute name after `aria:`")),
                    }
                }
                "on" => {
                    if *i >= tokens.len() || !is_punct(&tokens[*i], ':') {
                        return Err(syn::Error::new(span, "expected `on:event`"));
                    }
                    *i += 1;
                    match tokens.get(*i) {
                        Some(TokenTree::Ident(ev)) => {
                            let ev = ev.clone();
                            *i += 1;
                            AttrKind::On(ev)
                        }
                        _ => return Err(syn::Error::new(span, "expected event name after `on:`")),
                    }
                }
                _ => AttrKind::Ident(id.clone()),
            }
        }
        other => return Err(syn::Error::new(other.span(), "expected attribute name")),
    };
    if *i >= tokens.len() || !is_punct(&tokens[*i], '=') {
        return Err(syn::Error::new(span, "expected `=` after attribute"));
    }
    *i += 1;
    if *i >= tokens.len() {
        return Err(syn::Error::new(span, "expected attribute value"));
    }
    let value = match &tokens[*i] {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
            let stream = g.stream();
            *i += 1;
            AttrValue::Expr(stream)
        }
        TokenTree::Literal(lit) => {
            let ts = TokenStream2::from(TokenTree::Literal(lit.clone()));
            *i += 1;
            AttrValue::Lit(ts)
        }
        TokenTree::Ident(id) => {
            let name = id.to_string();
            *i += 1;
            if name == "true" || name == "false" {
                AttrValue::Lit(Ident::new(&name, id.span()).to_token_stream())
            } else {
                AttrValue::Expr(id.to_token_stream())
            }
        }
        other => return Err(syn::Error::new(other.span(), "expected attribute value")),
    };
    Ok(Attr { kind, value, span })
}
