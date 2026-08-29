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
                s.text.font_size = Some(#v);
            }))
        }
        "font-weight" => {
            let v = emit_font_weight(tokens, span)?;
            Ok(Some(quote! {
                s.text.font_weight = Some(#v);
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
                s.text.font_style = Some(#v);
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
                s.text.text_align = Some(#v);
            }))
        }
        "text-decoration" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            Ok(Some(match kw.as_str() {
                "underline" => Ok(quote! {
                    s.text.underline = Some(::gpui::UnderlineStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "line-through" => Ok(quote! {
                    s.text.strikethrough = Some(::gpui::StrikethroughStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "none" => Ok(quote! {
                    s.text.underline = None;
                    s.text.strikethrough = None;
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
                s.text.white_space = Some(#v);
            }))
        }
        "line-height" => {
            if tokens.len() == 1 {
                if let TokenTree::Literal(_) = &tokens[0] {
                    if parse_suffixed_length(&tokens[0]).is_none() {
                        if let Some(n) = parse_number(&tokens[0]) {
                            return Ok(Some(quote! {
                                s.text.line_height = Some(::gpui::relative(#n as f32));
                            }));
                        }
                    }
                }
            }
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(Some(quote! {
                s.text.line_height = Some(#v);
            }))
        }
"font-family" => {
    let fam = if let Some(kw) = keyword(tokens) {
        kw
    } else if let Some(expr) = is_interp(tokens) {
        return Ok(Some(quote! {
            s.text.font_family =
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
        s.text.font_family =
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
        s.text.text_overflow = Some(#v);
    }))
}
"text-decoration-color" => {
    let c = crate::color::emit_color(tokens, span)?;
    Ok(Some(quote! {
        s.text
            .underline.get_or_insert_with(::core::default::Default::default).color = Some((#c).into());
    }))
}
"text-decoration-thickness" => {
    let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
    let v = emit_length(&len);
    Ok(Some(quote! {
        s.text
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
        s.text
            .underline.get_or_insert_with(::core::default::Default::default).wavy = #wavy;
    }))
}
"text-background" | "text-background-color" => {
    let c = crate::color::emit_color(tokens, span)?;
    Ok(Some(quote! {
        s.text.background_color = Some((#c).into());
    }))
}
        "line-clamp" => {
            let n = number_value(tokens, prop, span)? as usize;
            Ok(Some(quote! {
                s.text.line_clamp = Some(#n);
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
            s.text.line_height = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#expr));
        })),
"font-family" => Ok(Some(quote! {
    s.text.font_family =
        ::std::option::Option::Some(::gpui::SharedString::from(#expr));
})),
"text-overflow" => Ok(Some(quote! {
    s.text.text_overflow = Some(#expr);
})),
"text-decoration-color" => Ok(Some(quote! {
    s.text
        .underline.get_or_insert_with(::core::default::Default::default).color = Some((#expr).into());
})),
"text-background" | "text-background-color" => Ok(Some(quote! {
    s.text.background_color = Some((#expr).into());
})),
        "line-clamp" => Ok(Some(quote! {
            s.text.line_clamp = Some(#expr as usize);
        })),
        _ => Ok(None),
    }
}

pub(crate) fn emit_var(
    prop: &str,
    name: &str,
    default_tokens: Option<&[TokenTree]>,
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    use crate::value::opt_default;
    // Helper: resolve default tokens to a keyword SharedString literal.
    let kw_default = |tokens: Option<&[TokenTree]>, prop: &str| -> syn::Result<Option<TokenStream2>> {
        match tokens {
            None => Ok(None),
            Some(t) => {
                if let Some(kw) = keyword(t).or_else(|| hyphen_keyword(t)) {
                    let kw_lit = kw.as_str();
                    Ok(Some(quote! { ::gpui::SharedString::from(#kw_lit) }))
                } else {
                    Err(syn::Error::new(
                        span,
                        format!("var(--{name}) default is not a valid keyword for '{prop}'"),
                    ))
                }
            }
        }
    };
    match prop {
        "font-size" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    if matches!(len.kind, LengthKind::Percent) {
                        return Err(syn::Error::new(span, "font-size cannot be a percentage"));
                    }
                    Some(emit_as_absolute(&len, prop, span)?)
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.font_size = Some(::vgui::__var_absolute(#name, #default));
            }))
        }
        "font-weight" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let v = crate::keywords::emit_font_weight(t, span)?;
                    Some(v)
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.font_weight = Some(::vgui::__var_font_weight(#name, #default));
            }))
        }
        "font-style" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.font_style = Some(::vgui::__resolve_font_style(::vgui::__var_keyword(#name, #default).as_str()));
            }))
        }
        "text-align" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.text_align = Some(::vgui::__resolve_text_align(::vgui::__var_keyword(#name, #default).as_str()));
            }))
        }
        "white-space" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.white_space = Some(::vgui::__resolve_white_space(::vgui::__var_keyword(#name, #default).as_str()));
            }))
        }
        "text-overflow" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.text_overflow = Some(::vgui::__resolve_text_overflow(::vgui::__var_keyword(#name, #default).as_str()));
            }))
        }
        "text-decoration" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                ::vgui::__apply_text_decoration(s, ::vgui::__var_keyword(#name, #default).as_str());
            }))
        }
        "text-decoration-style" => {
            let default = kw_default(default_tokens, prop)?;
            let default = opt_default(default);
            Ok(Some(quote! {
                ::vgui::__apply_text_decoration_style(s, ::vgui::__var_keyword(#name, #default).as_str());
            }))
        }
        "text-decoration-thickness" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    Some(emit_length(&len))
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.underline.get_or_insert_with(::core::default::Default::default).thickness =
                    ::vgui::__var_length(#name, #default);
            }))
        }
        "line-height" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    // Bare number → relative(n); else definite length.
                    if t.len() == 1 {
                        if let TokenTree::Literal(_) = &t[0] {
                            if parse_suffixed_length(&t[0]).is_none() {
                                if let Some(n) = parse_number(&t[0]) {
                                    return Ok(Some(quote! {
                                        s.text.line_height = Some(::vgui::__var_line_height(#name, ::std::option::Option::Some(::gpui::relative(#n as f32))));
                                    }));
                                }
                            }
                        }
                    }
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    Some(emit_as_definite(&len, prop, span)?)
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.line_height = Some(::vgui::__var_line_height(#name, #default));
            }))
        }
        "font-family" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    if let Some(kw) = keyword(t) {
                        Some(quote! { ::gpui::SharedString::from(#kw) })
                    } else if t.len() == 1 {
                        if let TokenTree::Literal(lit) = &t[0] {
                            if let Ok(s) = syn::parse2::<syn::LitStr>(TokenStream2::from(TokenTree::Literal(lit.clone()))) {
                                let v = s.value();
                                Some(quote! { ::gpui::SharedString::from(#v) })
                            } else {
                                return Err(unsupported(prop, t, span));
                            }
                        } else {
                            return Err(unsupported(prop, t, span));
                        }
                    } else {
                        return Err(unsupported(prop, t, span));
                    }
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.font_family = Some(::vgui::__var_font_family(#name, #default));
            }))
        }
        "line-clamp" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let n = crate::value::number_value(t, prop, span)?;
                    Some(quote! { #n as f32 })
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.line_clamp = Some(::vgui::__var_number(#name, #default) as usize);
            }))
        }
        "text-decoration-color" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let c = crate::color::emit_color(t, span)?;
                    Some(quote! { ::gpui::Hsla::from(#c) })
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                s.text.underline.get_or_insert_with(::core::default::Default::default).color =
                    Some(::vgui::__var_color(#name, #default).into());
            }))
        }
        _ => Ok(None),
    }
}
