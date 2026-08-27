use proc_macro2::{Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::color::emit_color;
use crate::keywords::{emit_cursor, emit_shadow};
use crate::parse::{hyphen_keyword, keyword, unsupported};
use crate::value::{emit_as_absolute, number_value, parse_length, split_values};

pub(crate) fn emit(
    prop: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "background" | "background-color" => {
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! { s.background = Some((#c).into()); }))
        }
        "color" => {
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).color = Some((#c).into());
            }))
        }
        "opacity" => {
            let n = number_value(tokens, prop, span)?;
            Ok(Some(quote! { s.opacity = Some(#n as f32); }))
        }
        "border-color" => {
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! { s.border_color = Some((#c).into()); }))
        }
        "border-style" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "solid" => quote! { ::gpui::BorderStyle::Solid },
                "dashed" => quote! { ::gpui::BorderStyle::Dashed },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'border-style': {other}"),
                    ))
                }
            };
            Ok(Some(quote! { s.border_style = Some(#v); }))
        }
        "border-width" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(Some(quote! {
                s.border_widths.top = Some(#v);
                s.border_widths.right = Some(#v);
                s.border_widths.bottom = Some(#v);
                s.border_widths.left = Some(#v);
            }))
        }
        "border" => Ok(Some(emit_border(tokens, span)?)),
        "border-radius" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(Some(quote! {
                s.corner_radii.top_left = Some(#v);
                s.corner_radii.top_right = Some(#v);
                s.corner_radii.bottom_right = Some(#v);
                s.corner_radii.bottom_left = Some(#v);
            }))
        }
        "border-top-left-radius" => Ok(Some(corner("top_left", tokens, span)?)),
        "border-top-right-radius" => Ok(Some(corner("top_right", tokens, span)?)),
        "border-bottom-right-radius" => Ok(Some(corner("bottom_right", tokens, span)?)),
        "border-bottom-left-radius" => Ok(Some(corner("bottom_left", tokens, span)?)),
        "cursor" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_cursor(&kw, span)?;
            Ok(Some(quote! { s.mouse_cursor = Some(#v); }))
        }
        "box-shadow" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_shadow(&kw, span)?;
            Ok(Some(quote! { s.box_shadow = Some(#v); }))
        }
        _ => Ok(None),
    }
}

pub(crate) fn emit_interp(
    prop: &str,
    expr: TokenStream2,
    _span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "opacity" => Ok(Some(quote! { s.opacity = Some(#expr as f32); })),
        "background" | "background-color" => Ok(Some(quote! { s.background = Some((#expr).into()); })),
        "color" => Ok(Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).color = Some((#expr).into());
        })),
        _ => Ok(None),
    }
}

fn emit_border(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let parts = split_values(tokens);
    let mut width = None;
    let mut style = None;
    let mut color = None;
    for part in parts {
        if let Some(len) = parse_length(&part) {
            width = Some(len);
            continue;
        }
        if let Some(kw) = hyphen_keyword(&part) {
            match kw.as_str() {
                "solid" => {
                    style = Some(quote! { ::gpui::BorderStyle::Solid });
                    continue;
                }
                "dashed" => {
                    style = Some(quote! { ::gpui::BorderStyle::Dashed });
                    continue;
                }
                _ => {}
            }
        }
        if emit_color(&part, span).is_ok() {
            color = Some(emit_color(&part, span)?);
            continue;
        }
        return Err(unsupported("border", tokens, span));
    }
    let width = width.ok_or_else(|| unsupported("border", tokens, span))?;
    let w = emit_as_absolute(&width, "border", span)?;
    let style = style.unwrap_or(quote! { ::gpui::BorderStyle::Solid });
    let mut out = quote! {
        s.border_widths.top = Some(#w);
        s.border_widths.right = Some(#w);
        s.border_widths.bottom = Some(#w);
        s.border_widths.left = Some(#w);
        s.border_style = Some(#style);
    };
    if let Some(c) = color {
        out.extend(quote! { s.border_color = Some((#c).into()); });
    }
    Ok(out)
}

fn corner(name: &str, tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported("border-radius", tokens, span))?;
    let v = emit_as_absolute(&len, "border-radius", span)?;
    let ident = Ident::new(name, span);
    Ok(quote! { s.corner_radii.#ident = Some(#v); })
}
