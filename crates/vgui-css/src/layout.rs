use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::keywords::{
    emit_align_content, emit_align_items, emit_flex_direction, emit_justify, emit_overflow,
};
use crate::parse::{hyphen_keyword, keyword, unsupported};
use crate::value::{
    emit_as_definite, emit_as_length, number_value, parse_length, parse_lengths, parse_number,
    split_values,
};

pub(crate) fn emit(
    prop: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "display" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "flex" => quote! { ::gpui::Display::Flex },
                "block" => quote! { ::gpui::Display::Block },
                "none" => quote! { ::gpui::Display::None },
                "grid" => quote! { ::gpui::Display::Grid },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'display': {other}"),
                    ))
                }
            };
            Ok(Some(quote! { s.display = Some(#v); }))
        }
        "visibility" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "hidden" => quote! { ::gpui::Visibility::Hidden },
                "visible" => quote! { ::gpui::Visibility::Visible },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'visibility': {other}"),
                    ))
                }
            };
            Ok(Some(quote! { s.visibility = Some(#v); }))
        }
        "overflow" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(Some(quote! { s.overflow.x = Some(#v); s.overflow.y = Some(#v); }))
        }
        "overflow-x" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(Some(quote! { s.overflow.x = Some(#v); }))
        }
        "overflow-y" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(Some(quote! { s.overflow.y = Some(#v); }))
        }
        "position" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "relative" => quote! { ::gpui::Position::Relative },
                "absolute" => quote! { ::gpui::Position::Absolute },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'position': {other}"),
                    ))
                }
            };
            Ok(Some(quote! { s.position = Some(#v); }))
        }
        "flex-direction" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_flex_direction(&kw, span)?;
            Ok(Some(quote! { s.flex_direction = Some(#v); }))
        }
        "flex-wrap" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "nowrap" => quote! { ::gpui::FlexWrap::NoWrap },
                "wrap" => quote! { ::gpui::FlexWrap::Wrap },
                "wrap-reverse" => quote! { ::gpui::FlexWrap::WrapReverse },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'flex-wrap': {other}"),
                    ))
                }
            };
            Ok(Some(quote! { s.flex_wrap = Some(#v); }))
        }
        "flex" => Ok(Some(emit_flex(tokens, span)?)),
        "flex-grow" => {
            let n = number_value(tokens, prop, span)?;
            Ok(Some(quote! { s.flex_grow = Some(#n as f32); }))
        }
        "flex-shrink" => {
            let n = number_value(tokens, prop, span)?;
            Ok(Some(quote! { s.flex_shrink = Some(#n as f32); }))
        }
        "flex-basis" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(Some(quote! { s.flex_basis = Some(#v); }))
        }
        "justify-content" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_justify(&kw, span)?;
            Ok(Some(quote! { s.justify_content = Some(#v); }))
        }
        "align-items" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_items(&kw, span)?;
            Ok(Some(quote! { s.align_items = Some(#v); }))
        }
        "align-self" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_items(&kw, span)?;
            Ok(Some(quote! { s.align_self = Some(#v); }))
        }
        "align-content" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_content(&kw, span)?;
            Ok(Some(quote! { s.align_content = Some(#v); }))
        }
        "gap" => {
            let lens = parse_lengths(tokens, prop, span)?;
            Ok(Some(match lens.len() {
                1 => {
                    let v = emit_as_definite(&lens[0], prop, span)?;
                    quote! { s.gap.width = Some(#v); s.gap.height = Some(#v); }
                }
                2 => {
                    let h = emit_as_definite(&lens[0], prop, span)?;
                    let w = emit_as_definite(&lens[1], prop, span)?;
                    quote! { s.gap.height = Some(#h); s.gap.width = Some(#w); }
                }
                _ => return Err(unsupported(prop, tokens, span)),
            }))
        }
        "row-gap" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! { s.gap.height = Some(#v); }))
        }
        "column-gap" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! { s.gap.width = Some(#v); }))
        }
        "grid-template-columns" => {
            let n = number_value(tokens, prop, span)?;
            let n = n as u16;
            Ok(Some(quote! { s.grid_cols = Some(#n); }))
        }
        "grid-template-rows" => {
            let n = number_value(tokens, prop, span)?;
            let n = n as u16;
            Ok(Some(quote! { s.grid_rows = Some(#n); }))
        }
"scrollbar-width" => {
    let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
    let v = match kw.as_str() {
        "thin" => quote! { ::std::option::Option::Some(::gpui::AbsoluteLength::Pixels(::gpui::px(4.))) },
        "none" => quote! { ::std::option::Option::Some(::gpui::AbsoluteLength::Pixels(::gpui::px(0.))) },
        "auto" | _ => quote! { ::std::option::Option::Some(::gpui::AbsoluteLength::Pixels(::gpui::px(12.))) },
    };
    Ok(Some(quote! { s.scrollbar_width = #v; }))
}
"grid-column" => Ok(Some(emit_grid_span(tokens, "column", span)?)),
"grid-column-start" => Ok(Some(emit_grid_line(tokens, "column", "start", span)?)),
"grid-column-end" => Ok(Some(emit_grid_line(tokens, "column", "end", span)?)),
"grid-row" => Ok(Some(emit_grid_span(tokens, "row", span)?)),
"grid-row-start" => Ok(Some(emit_grid_line(tokens, "row", "start", span)?)),
"grid-row-end" => Ok(Some(emit_grid_line(tokens, "row", "end", span)?)),
        _ => Ok(None),
    }
}

pub(crate) fn emit_interp(
    prop: &str,
    expr: TokenStream2,
    _span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "flex-grow" => Ok(Some(quote! { s.flex_grow = Some(#expr as f32); })),
        "flex-shrink" => Ok(Some(quote! { s.flex_shrink = Some(#expr as f32); })),
        "flex-basis" => Ok(Some(
            quote! { s.flex_basis = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        )),
        "gap" => Ok(Some(quote! {
            let __gap = ::core::convert::Into::<::gpui::DefiniteLength>::into(#expr);
            s.gap.width = Some(__gap);
            s.gap.height = Some(__gap);
        })),
        "grid-template-columns" => Ok(Some(quote! { s.grid_cols = Some(#expr as u16); })),
        "grid-template-rows" => Ok(Some(quote! { s.grid_rows = Some(#expr as u16); })),
        _ => Ok(None),
    }
}

fn emit_flex(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(kw) = keyword(tokens) {
        return match kw.as_str() {
            "none" => Ok(quote! {
                s.flex_grow = Some(0f32);
                s.flex_shrink = Some(0f32);
                s.flex_basis = Some(::gpui::Length::Auto);
            }),
            "auto" => Ok(quote! {
                s.flex_grow = Some(1f32);
                s.flex_shrink = Some(1f32);
                s.flex_basis = Some(::gpui::Length::Auto);
            }),
            _ => Err(unsupported("flex", tokens, span)),
        };
    }
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(quote! {
                s.flex_grow = Some(#n as f32);
                s.flex_shrink = Some(1f32);
                s.flex_basis = Some(::core::convert::Into::<::gpui::Length>::into(::gpui::px(0.)));
            });
        }
    }
    Err(unsupported("flex", tokens, span))
}
/// Parse `grid-column` / `grid-row` shorthand: "span N", "A / B", or "N".
fn emit_grid_span(tokens: &[TokenTree], axis: &str, span: Span) -> syn::Result<TokenStream2> {
    let parts = split_values(tokens);
    let (start, end) = match parts.len() {
        1 => {
            let p = &parts[0];
            if let Some(kw) = keyword(p) {
                if kw == "auto" {
                    (quote! { ::gpui::GridPlacement::Auto }, quote! { ::gpui::GridPlacement::Auto })
                } else {
                    return Err(unsupported("grid span", p, span));
                }
            } else if let Some(n) = parse_number(&p[0]) {
                (quote! { ::gpui::GridPlacement::Line(#n as i16) }, quote! { ::gpui::GridPlacement::Auto })
            } else {
                return Err(unsupported("grid span", p, span));
            }
        }
        2 => {
            let s = parse_grid_placement(&parts[0], span)?;
            let e = parse_grid_placement(&parts[1], span)?;
            (s, e)
        }
        _ => return Err(unsupported("grid span", tokens, span)),
    };
    Ok(quote! {
        s.grid_location.get_or_insert_with(::core::default::Default::default).#axis =
            ::core::ops::Range { start: #start, end: #end };
    })
}

/// Parse `grid-column-start` / `grid-row-start` etc.
fn emit_grid_line(tokens: &[TokenTree], axis: &str, end: &str, span: Span) -> syn::Result<TokenStream2> {
    let p = if let Some(kw) = keyword(tokens) {
        if kw == "auto" {
            quote! { ::gpui::GridPlacement::Auto }
        } else {
            return Err(unsupported("grid line", tokens, span));
        }
    } else if let Some(n) = parse_number(&tokens[0]) {
        quote! { ::gpui::GridPlacement::Line(#n as i16) }
    } else {
        return Err(unsupported("grid line", tokens, span));
    };
    Ok(quote! {
        s.grid_location.get_or_insert_with(::core::default::Default::default).#axis.#end = #p;
    })
}

fn parse_grid_placement(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(kw) = keyword(tokens) {
        if kw == "auto" {
            return Ok(quote! { ::gpui::GridPlacement::Auto });
        }
    }
    // Check for "span N"
    if tokens.len() >= 2 {
        if let TokenTree::Ident(id) = &tokens[0] {
            if id.to_string() == "span" {
                if let Some(n) = parse_number(&tokens[1]) {
                    return Ok(quote! { ::gpui::GridPlacement::Span(#n as u16) });
                }
            }
        }
    }
    if let Some(n) = parse_number(&tokens[0]) {
        return Ok(quote! { ::gpui::GridPlacement::Line(#n as i16) });
    }
    Err(unsupported("grid placement", tokens, span))
}
