use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::keywords::emit_font_weight;
use crate::parse::{hyphen_keyword, is_interp, keyword, unsupported};
use crate::value::{
    emit_as_absolute, emit_as_definite, emit_length, LengthKind, number_value, parse_length, parse_number,
    parse_suffixed_length,
};

pub(crate) fn emit(
    prop: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    match prop {
        "font-size" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            if matches!(len.kind, LengthKind::Percent) {
                return Err(syn::Error::new(span, "font-size cannot be a percentage"));
            }
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_size = Some(#v);
            }))
        }
        "font-weight" => {
            let v = emit_font_weight(tokens, span)?;
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(#v);
            }))
        }
        "font-style" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "italic" => quote! { ::gpui::FontStyle::Italic },
                "normal" => quote! { ::gpui::FontStyle::Normal },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'font-style': {other}"),
                    ))
                }
            };
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_style = Some(#v);
            }))
        }
        "text-align" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "left" => quote! { ::gpui::TextAlign::Left },
                "center" => quote! { ::gpui::TextAlign::Center },
                "right" => quote! { ::gpui::TextAlign::Right },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'text-align': {other}"),
                    ))
                }
            };
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(#v);
            }))
        }
        "text-decoration" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            Ok(Some(match kw.as_str() {
                "underline" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).underline = Some(::gpui::UnderlineStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "line-through" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).strikethrough = Some(::gpui::StrikethroughStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "none" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).underline = None;
                    s.text.get_or_insert_with(::core::default::Default::default).strikethrough = None;
                }),
                other => Err(syn::Error::new(
                    span,
                    format!("unsupported CSS value for 'text-decoration': {other}"),
                )),
            }?))
        }
        "white-space" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "nowrap" => quote! { ::gpui::WhiteSpace::Nowrap },
                "normal" => quote! { ::gpui::WhiteSpace::Normal },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'white-space': {other}"),
                    ))
                }
            };
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).white_space = Some(#v);
            }))
        }
        "line-height" => {
            if tokens.len() == 1 {
                if let TokenTree::Literal(_) = &tokens[0] {
                    if parse_suffixed_length(&tokens[0]).is_none() {
                        if let Some(n) = parse_number(&tokens[0]) {
                            return Ok(Some(quote! {
                                s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(::gpui::relative(#n as f32));
                            }));
                        }
                    }
                }
            }
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(#v);
            }))
        }
"font-family" => {
    let fam = if let Some(kw) = keyword(tokens) {
        kw
    } else if let Some(expr) = is_interp(tokens) {
        return Ok(Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).font_family =
                ::std::option::Option::Some(::gpui::SharedString::from(#expr));
        }));
    } else if tokens.len() == 1 {
        if let TokenTree::Literal(lit) = &tokens[0] {
            if let Ok(s) = syn::parse2::<syn::LitStr>(TokenStream2::from(TokenTree::Literal(lit.clone()))) {
                s.value()
            } else {
                return Err(unsupported(prop, tokens, span));
            }
        } else {
            return Err(unsupported(prop, tokens, span));
        }
    } else {
        return Err(unsupported(prop, tokens, span));
    };
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default).font_family =
            ::std::option::Option::Some(::gpui::SharedString::from(#fam));
    }))
}
"text-overflow" => {
    let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
    let v = match kw.as_str() {
        "ellipsis" => quote! { ::gpui::TextOverflow::Truncate(::gpui::SharedString::new_static("…")) },
        "clip" => quote! { ::gpui::TextOverflow::Truncate(::gpui::SharedString::new_static("")) },
        other => return Err(syn::Error::new(span, format!("unsupported text-overflow: {other}"))),
    };
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default).text_overflow = Some(#v);
    }))
}
"text-decoration-color" => {
    let c = crate::color::emit_color(tokens, span)?;
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default)
            .underline.get_or_insert_with(::core::default::Default::default).color = Some((#c).into());
    }))
}
"text-decoration-thickness" => {
    let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
    let v = emit_length(&len);
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default)
            .underline.get_or_insert_with(::core::default::Default::default).thickness = #v;
    }))
}
"text-decoration-style" => {
    let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
    let wavy = match kw.as_str() {
        "solid" => quote! { false },
        "wavy" => quote! { true },
        other => return Err(syn::Error::new(span, format!("unsupported text-decoration-style: {other}"))),
    };
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default)
            .underline.get_or_insert_with(::core::default::Default::default).wavy = #wavy;
    }))
}
"text-background" | "text-background-color" => {
    let c = crate::color::emit_color(tokens, span)?;
    Ok(Some(quote! {
        s.text.get_or_insert_with(::core::default::Default::default).background_color = Some((#c).into());
    }))
}
        "line-clamp" => {
            let n = number_value(tokens, prop, span)? as usize;
            Ok(Some(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).line_clamp = Some(#n);
            }))
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
        "line-height" => Ok(Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#expr));
        })),
"font-family" => Ok(Some(quote! {
    s.text.get_or_insert_with(::core::default::Default::default).font_family =
        ::std::option::Option::Some(::gpui::SharedString::from(#expr));
})),
"text-overflow" => Ok(Some(quote! {
    s.text.get_or_insert_with(::core::default::Default::default).text_overflow = Some(#expr);
})),
"text-decoration-color" => Ok(Some(quote! {
    s.text.get_or_insert_with(::core::default::Default::default)
        .underline.get_or_insert_with(::core::default::Default::default).color = Some((#expr).into());
})),
"text-background" | "text-background-color" => Ok(Some(quote! {
    s.text.get_or_insert_with(::core::default::Default::default).background_color = Some((#expr).into());
})),
        "line-clamp" => Ok(Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_clamp = Some(#expr as usize);
        })),
        _ => Ok(None),
    }
}
