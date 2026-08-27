use proc_macro2::{Delimiter, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned};

use crate::parse::{is_interp, unsupported};
use crate::value::{parse_int, parse_number};

pub(crate) fn parse_hex_digits(raw: &str) -> Result<(u32, bool), ()> {
    let hex: String = raw.chars().filter(|c| *c != '_').collect();
    let hex = hex.strip_prefix('#').unwrap_or(&hex);
    match hex.len() {
        3 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            u32::from_str_radix(&out, 16)
                .map(|v| (v, false))
                .map_err(|_| ())
        }
        4 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            u32::from_str_radix(&out, 16)
                .map(|v| (v, true))
                .map_err(|_| ())
        }
        6 => u32::from_str_radix(hex, 16)
            .map(|v| (v, false))
            .map_err(|_| ()),
        8 => u32::from_str_radix(hex, 16)
            .map(|v| (v, true))
            .map_err(|_| ()),
        _ => Err(()),
    }
}

pub(crate) fn emit_hex(raw: &str, span: Span) -> syn::Result<TokenStream2> {
    match parse_hex_digits(raw) {
        Ok((v, true)) => Ok(quote_spanned! {span=> ::gpui::rgba(#v) }),
        Ok((v, false)) => Ok(quote_spanned! {span=> ::gpui::rgb(#v) }),
        Err(()) => Err(syn::Error::new(span, "invalid hex color")),
    }
}

pub(crate) fn named_color(name: &str) -> Option<TokenStream2> {
    match name {
        "black" => Some(quote! { ::gpui::black() }),
        "white" => Some(quote! { ::gpui::white() }),
        "red" => Some(quote! { ::gpui::red() }),
        "green" => Some(quote! { ::gpui::green() }),
        "blue" => Some(quote! { ::gpui::blue() }),
        "yellow" => Some(quote! { ::gpui::yellow() }),
        "cyan" => Some(quote! { ::gpui::rgb(0x00ffff) }),
        "magenta" => Some(quote! { ::gpui::rgb(0xff00ff) }),
        "orange" => Some(quote! { ::gpui::rgb(0xffa500) }),
        "purple" => Some(quote! { ::gpui::rgb(0x800080) }),
        "gray" | "grey" => Some(quote! { ::gpui::rgb(0x808080) }),
        _ => None,
    }
}

pub(crate) fn emit_color(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(expr) = is_interp(tokens) {
        return Ok(expr);
    }
    if tokens.len() == 1 {
        match &tokens[0] {
            TokenTree::Ident(id) => {
                let name = id.to_string();
                if let Some(c) = named_color(&name) {
                    return Ok(c);
                }
            }
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') {
                    let inner = &s[1..s.len() - 1];
                    if inner.starts_with('#') {
                        return emit_hex(inner, span);
                    }
                }
            }
            _ => {}
        }
    }
    if !tokens.is_empty() {
        if let TokenTree::Punct(p) = &tokens[0] {
            if p.as_char() == '#' && tokens.len() >= 2 {
                let raw = match &tokens[1] {
                    TokenTree::Ident(id) => id.to_string(),
                    TokenTree::Literal(lit) => lit.to_string().replace('_', ""),
                    _ => return Err(syn::Error::new(span, "invalid hex color")),
                };
                return emit_hex(&raw, span);
            }
        }
    }
    if let TokenTree::Ident(id) = &tokens[0] {
        let name = id.to_string();
        if name == "rgb" || name == "rgba" {
            return emit_rgb_fn(tokens, span);
        }
    }
    Err(syn::Error::new(span, "invalid hex color").combine_or(unsupported("color", tokens, span)))
}

trait CombineErr {
    fn combine_or(self, other: syn::Error) -> syn::Error;
}

impl CombineErr for syn::Error {
    fn combine_or(self, other: syn::Error) -> syn::Error {
        let _ = other;
        self
    }
}

pub(crate) fn emit_rgb_fn(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if tokens.len() != 2 {
        return Err(unsupported("color", tokens, span));
    }
    let TokenTree::Ident(id) = &tokens[0] else {
        return Err(unsupported("color", tokens, span));
    };
    let TokenTree::Group(g) = &tokens[1] else {
        return Err(unsupported("color", tokens, span));
    };
    if g.delimiter() != Delimiter::Parenthesis {
        return Err(unsupported("color", tokens, span));
    }
    let args: Vec<TokenTree> = g.stream().into_iter().collect();
    let mut nums = Vec::new();
    let mut cur = Vec::new();
    for tt in args {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !cur.is_empty() {
                    nums.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(tt),
        }
    }
    if !cur.is_empty() {
        nums.push(cur);
    }
    let name = id.to_string();
    if name == "rgb" {
        if nums.len() != 3 {
            return Err(unsupported("color", tokens, span));
        }
        let r = parse_int(&nums[0][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let g = parse_int(&nums[1][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let b = parse_int(&nums[2][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let packed = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        Ok(quote! { ::gpui::rgb(#packed) })
    } else {
        if nums.len() != 4 {
            return Err(unsupported("color", tokens, span));
        }
        let r = parse_int(&nums[0][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let g = parse_int(&nums[1][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let b = parse_int(&nums[2][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let a = parse_number(&nums[3][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
        let packed = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | alpha;
        Ok(quote! { ::gpui::rgba(#packed) })
    }
}
