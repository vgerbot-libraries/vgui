//! vgui-css — the `css!` proc-macro.
//!
//! Parses CSS declarations (`color: #fff; padding: 8px;`) and compiles them
//! into gpui `StyleRefinement` closures.
extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

mod parse;
mod value;
mod color;
mod keywords;
mod layout;
mod box_model;
mod visual;
mod text;

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    match expand_css(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

pub(crate) struct Decl {
    pub(crate) name: String,
    pub(crate) span: Span,
    pub(crate) value: Vec<TokenTree>,
}

fn expand_css(input: TokenStream2) -> syn::Result<TokenStream2> {
    let decls = parse::parse_decls(input)?;
    if decls.is_empty() {
        return Ok(quote! { ::vgui::Css::new(|_| {}) });
    }
    let mut stmts = Vec::new();
    for decl in decls {
        if decl.name.starts_with('&') {
            return Err(syn::Error::new(
                decl.span,
                "pseudo selectors belong on the element: hover={css! { ... }}",
            ));
        }
        stmts.push(emit_decl(&decl)?);
    }
    Ok(quote! {
        {
            #[allow(unused_imports)]
            use ::vgui as _;
            ::vgui::Css::new(|s| {
                #(#stmts)*
            })
        }
    })
}

fn emit_decl(decl: &Decl) -> syn::Result<TokenStream2> {
    let prop = decl.name.as_str();
    let tokens = &decl.value;
    let span = decl.span;
    if let Some(expr) = parse::is_interp(tokens) {
        return emit_interp_prop(prop, expr, span);
    }
    macro_rules! try_cat {
        ($e:expr) => {
            if let Some(ts) = $e? {
                return Ok(ts);
            }
        };
    }
    try_cat!(layout::emit(prop, tokens, span));
    try_cat!(box_model::emit(prop, tokens, span));
    try_cat!(visual::emit(prop, tokens, span));
    try_cat!(text::emit(prop, tokens, span));
    Err(parse::unknown_prop(prop, span))
}

fn emit_interp_prop(prop: &str, expr: TokenStream2, span: Span) -> syn::Result<TokenStream2> {
    macro_rules! try_cat {
        ($e:expr) => {
            if let Some(ts) = $e? {
                return Ok(ts);
            }
        };
    }
    try_cat!(layout::emit_interp(prop, expr.clone(), span));
    try_cat!(box_model::emit_interp(prop, expr.clone(), span));
    try_cat!(visual::emit_interp(prop, expr.clone(), span));
    try_cat!(text::emit_interp(prop, expr, span));
    Err(syn::Error::new(
        span,
        format!("unsupported CSS value for '{prop}': interpolation"),
    ))
}
