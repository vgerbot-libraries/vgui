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
        "background" => {
            // Try linear-gradient(...) first, fall back to solid color
            if let Some(ts) = try_parse_linear_gradient(tokens, span)? {
                return Ok(Some(ts));
            }
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! { s.background = Some((#c).into()); }))
        }
        "background-color" => {
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! { s.background = Some((#c).into()); }))
        }
        "color" => {
            let c = emit_color(tokens, span)?;
            Ok(Some(quote! {
                s.text.color = Some((#c).into());
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
            s.text.color = Some((#expr).into());
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

/// Try to parse `linear-gradient(angle_or_dir, color1, color2)`.
/// Returns `None` if tokens don't start with `linear-gradient(`.
fn try_parse_linear_gradient(tokens: &[TokenTree], span: Span) -> syn::Result<Option<TokenStream2>> {
    // Must start with "linear-gradient(...)" or "linear_gradient(...)"
    // In token stream, "linear-gradient" is: Ident("linear"), Punct('-'), Ident("gradient"), Group('(...)')
    if tokens.is_empty() {
        return Ok(None);
    }
    // Check for "linear-gradient" pattern (3 tokens: ident, punct, ident)
    let group_idx = if tokens.len() >= 4
        && matches!(&tokens[0], TokenTree::Ident(id) if id.to_string() == "linear")
        && matches!(&tokens[1], TokenTree::Punct(p) if p.as_char() == '-')
        && matches!(&tokens[2], TokenTree::Ident(id) if id.to_string() == "gradient")
    {
        3
    } else if tokens.len() >= 2
        && matches!(&tokens[0], TokenTree::Ident(id) if id.to_string() == "linear_gradient")
    {
        1
    } else {
        return Ok(None);
    };
    let TokenTree::Group(g) = &tokens[group_idx] else { return Ok(None); };
    if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
        return Ok(None);
    }
    let args: Vec<TokenTree> = g.stream().into_iter().collect();
    // Split args by commas at top level
    let parts = split_values(&args);
    if parts.len() < 3 {
        return Err(syn::Error::new(span, "linear-gradient requires at least 3 arguments: angle/dir, from-color, to-color"));
    }
    // Parse angle/direction from first arg
    let angle = parse_gradient_angle(&parts[0], span)?;
    // Parse two colors
    let from = emit_color(&parts[1], span)?;
    let to = emit_color(&parts[2], span)?;
    Ok(Some(quote! {
        s.background = ::std::option::Option::Some(::gpui::linear_gradient(
            #angle,
            ::gpui::linear_color_stop(::gpui::Hsla::from(#from), 0.0),
            ::gpui::linear_color_stop(::gpui::Hsla::from(#to), 1.0),
        ).into());
    }))
}

/// Parse gradient angle: "90deg", "to right", "to left", "to top", "to bottom"
fn parse_gradient_angle(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    // "to right" / "to left" / "to top" / "to bottom"
    if tokens.len() == 2 {
        if let TokenTree::Ident(id) = &tokens[0] {
            if id.to_string() == "to" {
                if let TokenTree::Ident(dir) = &tokens[1] {
                    return Ok(match dir.to_string().as_str() {
                        "right" => quote! { 90f32 },
                        "left" => quote! { 270f32 },
                        "top" => quote! { 0f32 },
                        "bottom" => quote! { 180f32 },
                        _ => return Err(syn::Error::new(span, format!("unsupported gradient direction: to {}", dir))),
                    });
                }
            }
        }
    }
    // "90deg"
    if tokens.len() == 1 {
        if let TokenTree::Literal(lit) = &tokens[0] {
            let s = lit.to_string();
            if let Some(deg) = s.strip_suffix("deg") {
                if let Ok(n) = deg.parse::<f32>() {
                    return Ok(quote! { #n });
                }
            }
        }
    }
    Err(syn::Error::new(span, "expected gradient angle (e.g. 90deg) or direction (e.g. to right)"))
}

/// Try to parse `linear-gradient(angle, var(--a), var(--b))` where color args
/// may be `var()` references. Returns `Ok(None)` if tokens aren't a gradient.
/// Called from `emit_decl` when the value contains `var()` inside a gradient.
pub(crate) fn try_emit_var_gradient(
    tokens: &[TokenTree],
    local_vars: &std::collections::HashMap<String, Vec<TokenTree>>,
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    use crate::value::opt_default;
    // Reuse the same detection logic as try_parse_linear_gradient.
    if tokens.is_empty() {
        return Ok(None);
    }
    let group_idx = if tokens.len() >= 4
        && matches!(&tokens[0], TokenTree::Ident(id) if id == "linear")
        && matches!(&tokens[1], TokenTree::Punct(p) if p.as_char() == '-')
        && matches!(&tokens[2], TokenTree::Ident(id) if id == "gradient")
    {
        3
    } else if tokens.len() >= 2
        && matches!(&tokens[0], TokenTree::Ident(id) if id == "linear_gradient")
    {
        1
    } else {
        return Ok(None);
    };
    let TokenTree::Group(g) = &tokens[group_idx] else { return Ok(None); };
    if g.delimiter() != proc_macro2::Delimiter::Parenthesis {
        return Ok(None);
    }
    let args: Vec<TokenTree> = g.stream().into_iter().collect();
    // Split args by top-level commas only (keep var(...) groups intact).
    let parts: Vec<Vec<TokenTree>> = split_top_level_commas(&args);
    if parts.len() < 3 {
        return Err(syn::Error::new(span, "linear-gradient requires at least 3 arguments: angle/dir, from-color, to-color"));
    }
    let angle = parse_gradient_angle(&parts[0], span)?;
    // Each color arg: either a literal color (emit_color) or a var() reference.
    let resolve_color_arg = |arg: &[TokenTree]| -> syn::Result<TokenStream2> {
        if let Some(vref) = crate::parse::is_var(arg) {
            let vname = vref.name.clone();
            let default_tokens = local_vars.get(&vref.name).or(vref.fallback.as_ref()).map(|v| v.as_slice());
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let c = emit_color(t, span)?;
                    Some(quote! { ::gpui::Hsla::from(#c) })
                }
            };
            let default = opt_default(default);
            Ok(quote! { ::vgui::__var_color(#vname, #default) })
        } else {
            let c = emit_color(arg, span)?;
            Ok(quote! { ::gpui::Hsla::from(#c) })
        }
    };
    let from = resolve_color_arg(&parts[1])?;
    let to = resolve_color_arg(&parts[2])?;
    Ok(Some(quote! {
        s.background = ::std::option::Option::Some(::gpui::linear_gradient(
            #angle,
            ::gpui::linear_color_stop(#from, 0.0),
            ::gpui::linear_color_stop(#to, 1.0),
        ).into());
    }))
}

pub(crate) fn emit_var(
    prop: &str,
    name: &str,
    default_tokens: Option<&[TokenTree]>,
    span: Span,
) -> syn::Result<Option<TokenStream2>> {
    use crate::value::opt_default;
    match prop {
        "background" | "background-color" | "color" | "border-color" | "text-background"
        | "text-background-color" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let c = crate::color::emit_color(t, span)?;
                    Some(quote! { ::gpui::Hsla::from(#c) })
                }
            };
            let default = opt_default(default);
            let field = match prop {
                "background" | "background-color" | "text-background" | "text-background-color" => {
                    if prop == "text-background" || prop == "text-background-color" {
                        quote! { s.text.background_color = Some(::vgui::__var_color(#name, #default).into()); }
                    } else {
                        quote! { s.background = Some(::vgui::__var_color(#name, #default).into()); }
                    }
                }
                "color" => quote! { s.text.color = Some(::vgui::__var_color(#name, #default).into()); },
                "border-color" => quote! { s.border_color = Some(::vgui::__var_color(#name, #default).into()); },
                _ => unreachable!(),
            };
            Ok(Some(field))
        }
        "opacity" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let n = crate::value::number_value(t, prop, span)?;
                    Some(quote! { #n as f32 })
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! { s.opacity = Some(::vgui::__var_number(#name, #default)); }))
        }
        "border-width" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    Some(emit_as_absolute(&len, prop, span)?)
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                let __v = ::vgui::__var_absolute(#name, #default);
                s.border_widths.top = Some(__v);
                s.border_widths.right = Some(__v);
                s.border_widths.bottom = Some(__v);
                s.border_widths.left = Some(__v);
            }))
        }
        "border-radius" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    Some(emit_as_absolute(&len, prop, span)?)
                }
            };
            let default = opt_default(default);
            Ok(Some(quote! {
                let __v = ::vgui::__var_absolute(#name, #default);
                s.corner_radii.top_left = Some(__v);
                s.corner_radii.top_right = Some(__v);
                s.corner_radii.bottom_right = Some(__v);
                s.corner_radii.bottom_left = Some(__v);
            }))
        }
        "border-top-left-radius" | "border-top-right-radius"
        | "border-bottom-right-radius" | "border-bottom-left-radius" => {
            let default = match default_tokens {
                None => None,
                Some(t) => {
                    let len = parse_length(t).ok_or_else(|| unsupported(prop, t, span))?;
                    Some(emit_as_absolute(&len, prop, span)?)
                }
            };
            let default = opt_default(default);
            let corner = match prop {
                "border-top-left-radius" => "top_left",
                "border-top-right-radius" => "top_right",
                "border-bottom-right-radius" => "bottom_right",
                "border-bottom-left-radius" => "bottom_left",
                _ => unreachable!(),
            };
            let ident = Ident::new(corner, span);
            Ok(Some(quote! { s.corner_radii.#ident = Some(::vgui::__var_absolute(#name, #default)); }))
        }
        "border" => Err(syn::Error::new(
            span,
            "border shorthand does not support var(); use border-width / border-color / border-style with var()",
        )),
        _ => Ok(None),
    }
}

/// Split tokens by top-level commas only, preserving nested groups (e.g.
/// `var(--a)`) as single parts.
fn split_top_level_commas(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    for tt in tokens {
        match tt {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(tt.clone()),
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}
