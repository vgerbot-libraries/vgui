use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

use crate::color::emit_color;
use crate::parse::{hyphen_keyword, keyword, unsupported};
use crate::value::{parse_length, parse_number, LengthKind};

/// Infer the `CssValue` variant from literal tokens and emit the corresponding
/// `::vgui::CssValue::...` expression. Used by the `theme!` macro.
///
/// Tries in order: color, length, number, keyword.
pub(crate) fn emit_css_value(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    // 1. Color
    if let Ok(c) = emit_color(tokens, span) {
        return Ok(quote! { ::vgui::CssValue::Color(::gpui::Hsla::from(#c)) });
    }

    // 2. Length
    if let Some(len) = parse_length(tokens) {
        return match &len.kind {
            LengthKind::Px | LengthKind::Rem => {
                let inner = crate::value::emit_length(&len);
                Ok(quote! { ::vgui::CssValue::AbsoluteLength(::core::convert::Into::<::gpui::AbsoluteLength>::into(#inner)) })
            }
            LengthKind::Percent => {
                let inner = crate::value::emit_length(&len);
                Ok(quote! { ::vgui::CssValue::DefiniteLength(#inner) })
            }
            LengthKind::Auto => Ok(quote! { ::vgui::CssValue::Length(::gpui::Length::Auto) }),
            LengthKind::Interp(_) => Err(syn::Error::new(
                span,
                "theme! does not support interpolation; use Theme::set_* builders",
            )),
        };
    }

    // 3. Number
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(quote! { ::vgui::CssValue::Number(#n as f32) });
        }
    }

    // 4. Keyword
    if let Some(kw) = keyword(tokens).or_else(|| hyphen_keyword(tokens)) {
        let kw_lit = kw.as_str();
        return Ok(quote! { ::vgui::CssValue::Keyword(::gpui::SharedString::from(#kw_lit)) });
    }

    Err(unsupported("theme variable", tokens, span))
}
