use proc_macro2::{Delimiter, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned};

use crate::parse::{is_interp, unsupported};

pub(crate) fn parse_number(tt: &TokenTree) -> Option<f32> {
    if let TokenTree::Literal(lit) = tt {
        let s = lit.to_string().replace('_', "");
        if s.ends_with("f32") || s.ends_with("f64") {
            return s
                .trim_end_matches("f32")
                .trim_end_matches("f64")
                .parse()
                .ok();
        }
        if s.contains('.') {
            return s.parse().ok();
        }
        if let Ok(n) = s.parse::<i64>() {
            return Some(n as f32);
        }
        s.parse().ok()
    } else {
        None
    }
}

pub(crate) fn parse_suffixed_length(tt: &TokenTree) -> Option<LengthVal> {
    let TokenTree::Literal(lit) = tt else {
        return None;
    };
    let s = lit.to_string().replace('_', "");
    let span = tt.span();
    if let Some(num) = s.strip_suffix("px") {
        let n: f32 = num.parse().ok()?;
        return Some(LengthVal {
            kind: LengthKind::Px,
            n,
            span,
        });
    }
    if let Some(num) = s.strip_suffix("rem") {
        let n: f32 = num.parse().ok()?;
        return Some(LengthVal {
            kind: LengthKind::Rem,
            n,
            span,
        });
    }
    None
}

pub(crate) fn parse_int(tt: &TokenTree) -> Option<i64> {
    if let TokenTree::Literal(lit) = tt {
        let s = lit.to_string().replace('_', "");
        s.parse().ok()
    } else {
        None
    }
}

pub(crate) enum LengthKind {
    Px,
    Rem,
    Percent,
    Auto,
    Interp(TokenStream2),
}

pub(crate) struct LengthVal {
    pub(crate) kind: LengthKind,
    pub(crate) n: f32,
    pub(crate) span: Span,
}

pub(crate) fn split_values(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                i += 1;
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Literal(_) => {
                if parse_suffixed_length(&tokens[i]).is_some() {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                    cur.push(tokens[i].clone());
                    out.push(std::mem::take(&mut cur));
                    i += 1;
                    continue;
                }
                if i + 1 < tokens.len() {
                    if let TokenTree::Ident(id) = &tokens[i + 1] {
                        let name = id.to_string();
                        if name == "px" || name == "rem" || name == "rems" {
                            if !cur.is_empty() {
                                out.push(std::mem::take(&mut cur));
                            }
                            cur.push(tokens[i].clone());
                            cur.push(tokens[i + 1].clone());
                            out.push(std::mem::take(&mut cur));
                            i += 2;
                            continue;
                        }
                    }
                    if let TokenTree::Punct(p) = &tokens[i + 1] {
                        if p.as_char() == '%' {
                            if !cur.is_empty() {
                                out.push(std::mem::take(&mut cur));
                            }
                            cur.push(tokens[i].clone());
                            cur.push(tokens[i + 1].clone());
                            out.push(std::mem::take(&mut cur));
                            i += 2;
                            continue;
                        }
                    }
                }
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Ident(_) => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                if i + 1 < tokens.len() {
                    cur.push(tokens[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
                out.push(std::mem::take(&mut cur));
            }
            _ => {
                cur.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub(crate) fn parse_length(tokens: &[TokenTree]) -> Option<LengthVal> {
    if tokens.is_empty() {
        return None;
    }
    if let Some(expr) = is_interp(tokens) {
        return Some(LengthVal {
            kind: LengthKind::Interp(expr),
            n: 0.0,
            span: tokens[0].span(),
        });
    }
    if tokens.len() == 1 {
        if let TokenTree::Ident(id) = &tokens[0] {
            if id.to_string() == "auto" {
                return Some(LengthVal {
                    kind: LengthKind::Auto,
                    n: 0.0,
                    span: id.span(),
                });
            }
        }
        if let Some(len) = parse_suffixed_length(&tokens[0]) {
            return Some(len);
        }
        if let Some(n) = parse_number(&tokens[0]) {
            return Some(LengthVal {
                kind: LengthKind::Px,
                n,
                span: tokens[0].span(),
            });
        }
    }
    if tokens.len() == 2 {
        if let Some(n) = parse_number(&tokens[0]) {
            match &tokens[1] {
                TokenTree::Ident(id) => {
                    let name = id.to_string();
                    if name == "px" {
                        return Some(LengthVal {
                            kind: LengthKind::Px,
                            n,
                            span: tokens[0].span(),
                        });
                    }
                    if name == "rem" || name == "rems" {
                        return Some(LengthVal {
                            kind: LengthKind::Rem,
                            n,
                            span: tokens[0].span(),
                        });
                    }
                }
                TokenTree::Punct(p) if p.as_char() == '%' => {
                    return Some(LengthVal {
                        kind: LengthKind::Percent,
                        n,
                        span: tokens[0].span(),
                    });
                }
                _ => {}
            }
        }
    }
    None
}

pub(crate) fn emit_length(len: &LengthVal) -> TokenStream2 {
    match &len.kind {
        LengthKind::Px => {
            let n = len.n;
            quote! { ::gpui::px(#n as f32) }
        }
        LengthKind::Rem => {
            let n = len.n;
            quote! { ::gpui::rems(#n as f32) }
        }
        LengthKind::Percent => {
            let frac = len.n / 100.0;
            quote! { ::gpui::relative(#frac as f32) }
        }
        LengthKind::Auto => quote! { ::gpui::Length::Auto },
        LengthKind::Interp(expr) => expr.clone(),
    }
}

pub(crate) fn emit_as_length(len: &LengthVal, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Ok(quote! { ::gpui::Length::Auto }),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::Length>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote_spanned! {span=> ::core::convert::Into::<::gpui::Length>::into(#inner) })
        }
    }
}

pub(crate) fn emit_as_definite(len: &LengthVal, prop: &str, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Err(syn::Error::new(span, format!("{prop} cannot be auto"))),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#inner) })
        }
    }
}

pub(crate) fn emit_as_absolute(len: &LengthVal, prop: &str, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Err(syn::Error::new(span, format!("{prop} cannot be auto"))),
        LengthKind::Percent => Err(syn::Error::new(
            span,
            format!("{prop} cannot be a percentage"),
        )),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#inner) })
        }
    }
}

pub(crate) fn expand_box_edges(values: &[LengthVal]) -> syn::Result<[LengthVal; 4]> {
    match values.len() {
        1 => Ok([
            clone_len(&values[0]),
            clone_len(&values[0]),
            clone_len(&values[0]),
            clone_len(&values[0]),
        ]),
        2 => Ok([
            clone_len(&values[0]),
            clone_len(&values[1]),
            clone_len(&values[0]),
            clone_len(&values[1]),
        ]),
        4 => Ok([
            clone_len(&values[0]),
            clone_len(&values[1]),
            clone_len(&values[2]),
            clone_len(&values[3]),
        ]),
        n => Err(syn::Error::new(
            values
                .get(0)
                .map(|v| v.span)
                .unwrap_or_else(Span::call_site),
            format!("expected 1, 2, or 4 values, got {n}"),
        )),
    }
}

pub(crate) fn clone_len(len: &LengthVal) -> LengthVal {
    LengthVal {
        kind: match &len.kind {
            LengthKind::Px => LengthKind::Px,
            LengthKind::Rem => LengthKind::Rem,
            LengthKind::Percent => LengthKind::Percent,
            LengthKind::Auto => LengthKind::Auto,
            LengthKind::Interp(e) => LengthKind::Interp(e.clone()),
        },
        n: len.n,
        span: len.span,
    }
}

pub(crate) fn parse_lengths(tokens: &[TokenTree], prop: &str, span: Span) -> syn::Result<Vec<LengthVal>> {
    if let Some(len) = parse_length(tokens) {
        return Ok(vec![len]);
    }
    let parts = split_values(tokens);
    let mut out = Vec::new();
    for part in parts {
        let len = parse_length(&part).ok_or_else(|| unsupported(prop, tokens, span))?;
        out.push(len);
    }
    Ok(out)
}

pub(crate) fn number_value(tokens: &[TokenTree], prop: &str, span: Span) -> syn::Result<f32> {
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(n);
        }
    }
    Err(unsupported(prop, tokens, span))
}

/// Wrap an optional default expression as `Some(expr)` or `None`.
pub(crate) fn opt_default(expr: Option<TokenStream2>) -> TokenStream2 {
    match expr {
        Some(e) => quote! { ::std::option::Option::Some(#e) },
        None => quote! { ::std::option::Option::None },
    }
}
