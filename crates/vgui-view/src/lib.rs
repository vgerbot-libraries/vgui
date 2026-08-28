extern crate proc_macro;

mod parse;
mod emit;
mod control;
mod component;
mod builtin;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::quote;

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    match expand_view(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

pub(crate) enum Node {
    Element(Element),
    Fragment(Vec<Node>),
    Interp(TokenStream2),
    Text(syn::LitStr),
}

pub(crate) struct Element {
    pub(crate) tag: Ident,
    pub(crate) attrs: Vec<Attr>,
    pub(crate) children: Vec<Node>,
    pub(crate) self_closing: bool,
}

pub(crate) struct Attr {
    pub(crate) kind: AttrKind,
    pub(crate) value: AttrValue,
    pub(crate) span: Span,
}

pub(crate) enum AttrKind {
    Ident(Ident),
    On(Ident),
    Style,
    Hover,
    Active,
    Focus,
    Id,
    Src,
    Class,
    Type,
    Tabindex,
}

pub(crate) enum AttrValue {
    Expr(TokenStream2),
    Lit(TokenStream2),
}

fn expand_view(input: TokenStream2) -> syn::Result<TokenStream2> {
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == Delimiter::Brace {
                tokens = g.stream().into_iter().collect();
            }
        }
    }
    let mut i = 0;
    let node = parse::parse_node(&tokens, &mut i)?;
    parse::skip_ws(&tokens, &mut i);
    if i < tokens.len() {
        return Err(syn::Error::new(
            tokens[i].span(),
            "unexpected tokens after view root",
        ));
    }
    let expr = emit::emit_node(&node)?;
    Ok(quote! {{ let el = #expr; el }})
}

pub(crate) fn emit_element(el: &Element) -> syn::Result<TokenStream2> {
    let name = el.tag.to_string();
    if name == "Show" {
        return control::emit_show(el);
    }
    if name == "For" {
        return control::emit_for(el);
    }
    if name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        return component::emit_component(el);
    }
    builtin::emit_builtin(el)
}
