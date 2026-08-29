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
mod cssvalue;

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    match expand_css(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[proc_macro]
pub fn theme(input: TokenStream) -> TokenStream {
    match expand_theme(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

pub(crate) struct Decl {
    pub(crate) name: String,
    pub(crate) span: Span,
    pub(crate) value: Vec<TokenTree>,
}

pub(crate) struct VarRef {
    pub(crate) name: String,
    pub(crate) fallback: Option<Vec<TokenTree>>,
}

fn expand_css(input: TokenStream2) -> syn::Result<TokenStream2> {
    let decls = parse::parse_decls(input)?;
    if decls.is_empty() {
        return Ok(quote! { ::vgui::Css::new(|_| {}) });
    }
    // Partition into custom-property definitions (--name: value) and style decls.
    // Custom props become compile-time defaults for var() references; they emit
    // no runtime code themselves.
    let mut local_vars: std::collections::HashMap<String, Vec<TokenTree>> =
        std::collections::HashMap::new();
    let mut style_decls = Vec::new();
    for decl in decls {
        if decl.name.starts_with('&') {
            return Err(syn::Error::new(
                decl.span,
                "pseudo selectors belong on the element: hover={css! { ... }}",
            ));
        }
        if let Some(custom) = decl.name.strip_prefix("--") {
            local_vars.insert(custom.to_string(), decl.value);
            continue;
        }
        style_decls.push(decl);
    }
    let mut stmts = Vec::new();
    for decl in &style_decls {
        stmts.push(emit_decl(decl, &local_vars)?);
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

fn expand_theme(input: TokenStream2) -> syn::Result<TokenStream2> {
    let decls = parse::parse_decls(input)?;
    let mut set_calls = Vec::new();
    for decl in decls {
        let name = decl
            .name
            .strip_prefix("--")
            .ok_or_else(|| {
                syn::Error::new(
                    decl.span,
                    "theme! only accepts custom properties (--name: value)",
                )
            })?
            .to_string();
        let value = cssvalue::emit_css_value(&decl.value, decl.span)?;
        set_calls.push(quote! { __t.set(#name, #value); });
    }
    Ok(quote! {
        {
            let mut __t = ::vgui::Theme::new();
            #(#set_calls)*
            __t
        }
    })
}

fn emit_decl(
    decl: &Decl,
    local_vars: &std::collections::HashMap<String, Vec<TokenTree>>,
) -> syn::Result<TokenStream2> {
    let prop = decl.name.as_str();
    let tokens = &decl.value;
    let span = decl.span;
    if let Some(expr) = parse::is_interp(tokens) {
        return emit_interp_prop(prop, expr, span);
    }
    if let Some(vref) = parse::is_var(tokens) {
        return emit_var_prop(prop, &vref, local_vars, span);
    }
    // Check for linear-gradient(...) containing var() color args — handled
    // specially because it's not a sole var() but contains var() inside.
    if prop == "background" {
        if let Some(ts) = visual::try_emit_var_gradient(tokens, local_vars, span)? {
            return Ok(ts);
        }
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

fn emit_var_prop(
    prop: &str,
    vref: &VarRef,
    local_vars: &std::collections::HashMap<String, Vec<TokenTree>>,
    span: Span,
) -> syn::Result<TokenStream2> {
    // Resolve default tokens: local var definition takes priority, then the
    // fallback in the var() call, else None (runtime panic if theme lacks it).
    let default_tokens: Option<&[TokenTree]> = local_vars
        .get(&vref.name)
        .or(vref.fallback.as_ref())
        .map(|v| v.as_slice());
    macro_rules! try_cat {
        ($e:expr) => {
            if let Some(ts) = $e? {
                return Ok(ts);
            }
        };
    }
    try_cat!(layout::emit_var(prop, &vref.name, default_tokens, span));
    try_cat!(box_model::emit_var(prop, &vref.name, default_tokens, span));
    try_cat!(visual::emit_var(prop, &vref.name, default_tokens, span));
    try_cat!(text::emit_var(prop, &vref.name, default_tokens, span));
    Err(syn::Error::new(
        span,
        format!("property '{prop}' does not support var()"),
    ))
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
