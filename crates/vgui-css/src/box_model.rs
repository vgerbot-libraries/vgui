use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::parse::unsupported;
use crate::value::{
    emit_as_definite, emit_as_length, expand_box_edges, parse_length, parse_lengths, parse_number,
};

pub(crate) fn emit(
    prop: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "width" => Ok(Some(size_field("width", tokens, span)?)),
        "height" => Ok(Some(size_field("height", tokens, span)?)),
        "min-width" => Ok(Some(size_min_max("min_size", "width", tokens, span)?)),
        "min-height" => Ok(Some(size_min_max("min_size", "height", tokens, span)?)),
        "max-width" => Ok(Some(size_min_max("max_size", "width", tokens, span)?)),
        "max-height" => Ok(Some(size_min_max("max_size", "height", tokens, span)?)),
        "padding" => Ok(Some(box_edges("padding", tokens, span, true)?)),
        "padding-top" => Ok(Some(edge("padding", "top", tokens, span, true)?)),
        "padding-right" => Ok(Some(edge("padding", "right", tokens, span, true)?)),
        "padding-bottom" => Ok(Some(edge("padding", "bottom", tokens, span, true)?)),
        "padding-left" => Ok(Some(edge("padding", "left", tokens, span, true)?)),
        "padding-inline" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! { s.padding.left = Some(#v); s.padding.right = Some(#v); }))
        }
        "padding-block" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! { s.padding.top = Some(#v); s.padding.bottom = Some(#v); }))
        }
        "margin" => Ok(Some(box_edges("margin", tokens, span, false)?)),
        "margin-top" => Ok(Some(edge("margin", "top", tokens, span, false)?)),
        "margin-right" => Ok(Some(edge("margin", "right", tokens, span, false)?)),
        "margin-bottom" => Ok(Some(edge("margin", "bottom", tokens, span, false)?)),
        "margin-left" => Ok(Some(edge("margin", "left", tokens, span, false)?)),
        "margin-inline" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(Some(quote! { s.margin.left = Some(#v); s.margin.right = Some(#v); }))
        }
        "margin-block" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(Some(quote! { s.margin.top = Some(#v); s.margin.bottom = Some(#v); }))
        }
        "inset" => Ok(Some(box_edges("inset", tokens, span, false)?)),
        "top" => Ok(Some(edge("inset", "top", tokens, span, false)?)),
        "right" => Ok(Some(edge("inset", "right", tokens, span, false)?)),
        "bottom" => Ok(Some(edge("inset", "bottom", tokens, span, false)?)),
        "left" => Ok(Some(edge("inset", "left", tokens, span, false)?)),
        "aspect-ratio" => Ok(Some(emit_aspect_ratio(tokens, span)?)),
        _ => Ok(None),
    }
}

pub(crate) fn emit_interp(
    prop: &str,
    expr: TokenStream2,
    _span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "width" => Ok(Some(
            quote! { s.size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "height" => Ok(Some(
            quote! { s.size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "min-width" => Ok(Some(
            quote! { s.min_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "min-height" => Ok(Some(
            quote! { s.min_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "max-width" => Ok(Some(
            quote! { s.max_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "max-height" => Ok(Some(
            quote! { s.max_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "aspect-ratio" => Ok(Some(quote! { s.aspect_ratio = Some(#expr as f32); })),
        _ => Ok(None),
    }
}

fn size_field(field: &str, tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let v = emit_as_length(&len, span)?;
    let ident = Ident::new(field, span);
    Ok(quote! { s.size.#ident = Some(#v); })
}

fn size_min_max(
    which: &str,
    field: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let v = emit_as_length(&len, span)?;
    let which = Ident::new(which, span);
    let field = Ident::new(field, span);
    Ok(quote! { s.#which.#field = Some(#v); })
}

fn box_edges(
    field: &str,
    tokens: &[TokenTree],
    span: Span,
    definite: bool,
) -> syn::Result<TokenStream2> {
    let lens = parse_lengths(tokens, field, span)?;
    let edges = expand_box_edges(&lens)?;
    let ident = Ident::new(field, span);
    let mut stmts = Vec::new();
    let names = ["top", "right", "bottom", "left"];
    for (name, len) in names.iter().zip(edges.iter()) {
        let edge = Ident::new(name, span);
        let v = if definite {
            emit_as_definite(len, field, span)?
        } else {
            emit_as_length(len, span)?
        };
        stmts.push(quote! { s.#ident.#edge = Some(#v); });
    }
    Ok(quote! { #(#stmts)* })
}

fn edge(
    field: &str,
    edge: &str,
    tokens: &[TokenTree],
    span: Span,
    definite: bool,
) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let ident = Ident::new(field, span);
    let edge = Ident::new(edge, span);
    let v = if definite {
        emit_as_definite(&len, field, span)?
    } else {
        emit_as_length(&len, span)?
    };
    Ok(quote! { s.#ident.#edge = Some(#v); })
}

fn emit_aspect_ratio(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(quote! { s.aspect_ratio = Some(#n as f32); });
        }
    }
    if tokens.len() == 3 {
        if let (Some(w), TokenTree::Punct(p), Some(h)) = (
            parse_number(&tokens[0]),
            &tokens[1],
            parse_number(&tokens[2]),
        ) {
            if p.as_char() == '/' && h != 0.0 {
                let n = w / h;
                return Ok(quote! { s.aspect_ratio = Some(#n as f32); });
            }
        }
    }
    Err(unsupported("aspect-ratio", tokens, span))
}
