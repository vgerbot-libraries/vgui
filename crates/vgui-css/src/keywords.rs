use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned};

use crate::parse::{is_interp, keyword, unsupported};
use crate::value::parse_int;

pub(crate) fn emit_overflow(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "hidden" => Ok(quote_spanned! {span=> ::gpui::Overflow::Hidden }),
        "scroll" => Ok(quote_spanned! {span=> ::gpui::Overflow::Scroll }),
        "visible" => Ok(quote_spanned! {span=> ::gpui::Overflow::Visible }),
        "clip" => Ok(quote_spanned! {span=> ::gpui::Overflow::Clip }),
        "auto" => Ok(quote_spanned! {span=> ::gpui::Overflow::Scroll }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'overflow': {other}"),
        )),
    }
}

pub(crate) fn emit_flex_direction(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "row" => Ok(quote! { ::gpui::FlexDirection::Row }),
        "column" => Ok(quote! { ::gpui::FlexDirection::Column }),
        "row-reverse" => Ok(quote! { ::gpui::FlexDirection::RowReverse }),
        "column-reverse" => Ok(quote! { ::gpui::FlexDirection::ColumnReverse }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'flex-direction': {other}"),
        )),
    }
}

pub(crate) fn emit_justify(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::JustifyContent::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::JustifyContent::FlexEnd }),
        "center" => Ok(quote! { ::gpui::JustifyContent::Center }),
        "space-between" => Ok(quote! { ::gpui::JustifyContent::SpaceBetween }),
        "space-around" => Ok(quote! { ::gpui::JustifyContent::SpaceAround }),
        "space-evenly" => Ok(quote! { ::gpui::JustifyContent::SpaceEvenly }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'justify-content': {other}"),
        )),
    }
}

pub(crate) fn emit_align_items(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::AlignItems::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::AlignItems::FlexEnd }),
        "center" => Ok(quote! { ::gpui::AlignItems::Center }),
        "baseline" => Ok(quote! { ::gpui::AlignItems::Baseline }),
        "stretch" => Ok(quote! { ::gpui::AlignItems::Stretch }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'align-items': {other}"),
        )),
    }
}

pub(crate) fn emit_align_content(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::AlignContent::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::AlignContent::FlexEnd }),
        "center" => Ok(quote! { ::gpui::AlignContent::Center }),
        "space-between" => Ok(quote! { ::gpui::AlignContent::SpaceBetween }),
        "space-around" => Ok(quote! { ::gpui::AlignContent::SpaceAround }),
        "stretch" => Ok(quote! { ::gpui::AlignContent::Stretch }),
        "space-evenly" => Ok(quote! { ::gpui::AlignContent::SpaceEvenly }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'align-content': {other}"),
        )),
    }
}

pub(crate) fn emit_font_weight(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(expr) = is_interp(tokens) {
        return Ok(expr);
    }
    if let Some(kw) = keyword(tokens) {
        return match kw.as_str() {
            "thin" => Ok(quote! { ::gpui::FontWeight::THIN }),
            "extra-light" => Ok(quote! { ::gpui::FontWeight::EXTRA_LIGHT }),
            "light" => Ok(quote! { ::gpui::FontWeight::LIGHT }),
            "normal" => Ok(quote! { ::gpui::FontWeight::NORMAL }),
            "medium" => Ok(quote! { ::gpui::FontWeight::MEDIUM }),
            "semibold" => Ok(quote! { ::gpui::FontWeight::SEMIBOLD }),
            "bold" => Ok(quote! { ::gpui::FontWeight::BOLD }),
            "extrabold" | "extra-bold" => Ok(quote! { ::gpui::FontWeight::EXTRA_BOLD }),
            "black" => Ok(quote! { ::gpui::FontWeight::BLACK }),
            other => Err(syn::Error::new(
                span,
                format!("unsupported CSS value for 'font-weight': {other}"),
            )),
        };
    }
    if tokens.len() == 1 {
        if let Some(n) = parse_int(&tokens[0]) {
            return match n {
                100 => Ok(quote! { ::gpui::FontWeight::THIN }),
                200 => Ok(quote! { ::gpui::FontWeight::EXTRA_LIGHT }),
                300 => Ok(quote! { ::gpui::FontWeight::LIGHT }),
                400 => Ok(quote! { ::gpui::FontWeight::NORMAL }),
                500 => Ok(quote! { ::gpui::FontWeight::MEDIUM }),
                600 => Ok(quote! { ::gpui::FontWeight::SEMIBOLD }),
                700 => Ok(quote! { ::gpui::FontWeight::BOLD }),
                800 => Ok(quote! { ::gpui::FontWeight::EXTRA_BOLD }),
                900 => Ok(quote! { ::gpui::FontWeight::BLACK }),
                _ => Err(unsupported("font-weight", tokens, span)),
            };
        }
    }
    Err(unsupported("font-weight", tokens, span))
}

pub(crate) fn emit_cursor(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "pointer" => Ok(quote! { ::gpui::CursorStyle::PointingHand }),
        "default" => Ok(quote! { ::gpui::CursorStyle::Arrow }),
        "text" => Ok(quote! { ::gpui::CursorStyle::IBeam }),
        "crosshair" => Ok(quote! { ::gpui::CursorStyle::Crosshair }),
        "not-allowed" => Ok(quote! { ::gpui::CursorStyle::OperationNotAllowed }),
        "grab" => Ok(quote! { ::gpui::CursorStyle::OpenHand }),
        "grabbing" => Ok(quote! { ::gpui::CursorStyle::ClosedHand }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'cursor': {other}"),
        )),
    }
}

pub(crate) fn emit_shadow(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "none" => Ok(quote! { ::std::vec::Vec::new() }),
        "sm" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(3.),
                    spread_radius: ::gpui::px(0.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(2.),
                    spread_radius: ::gpui::px(-1.),
                }
            ]
        }),
        "md" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-1.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(2.)),
                    blur_radius: ::gpui::px(4.),
                    spread_radius: ::gpui::px(-2.),
                }
            ]
        }),
        "lg" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(10.)),
                    blur_radius: ::gpui::px(15.),
                    spread_radius: ::gpui::px(-3.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-4.),
                }
            ]
        }),
        "xl" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(20.)),
                    blur_radius: ::gpui::px(25.),
                    spread_radius: ::gpui::px(-5.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(8.)),
                    blur_radius: ::gpui::px(10.),
                    spread_radius: ::gpui::px(-6.),
                }
            ]
        }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'box-shadow': {other}"),
        )),
    }
}
