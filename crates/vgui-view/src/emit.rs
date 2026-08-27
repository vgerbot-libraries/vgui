use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::{emit_element, AttrValue, Node};

pub(crate) fn emit_node(node: &Node) -> syn::Result<TokenStream2> {
    match node {
        Node::Fragment(children) => {
            let kids = emit_children(children)?;
            Ok(quote! {{
                let mut el = ::gpui::div();
                #(el = el.child(#kids);)*
                el
            }})
        }
        Node::Interp(expr) => Ok(quote! { ::vgui::into_child(#expr) }),
        Node::Text(s) => Ok(quote! { ::vgui::into_child(#s) }),
        Node::Element(el) => emit_element(el),
    }
}

pub(crate) fn emit_children(children: &[Node]) -> syn::Result<Vec<TokenStream2>> {
    children.iter().map(emit_child).collect()
}

pub(crate) fn emit_child(node: &Node) -> syn::Result<TokenStream2> {
    match node {
        Node::Interp(expr) => Ok(quote! { ::vgui::into_child(#expr) }),
        Node::Text(s) => Ok(quote! { ::vgui::into_child(#s) }),
        other => emit_node(other),
    }
}

pub(crate) fn wrap_children_element(children: &[Node]) -> syn::Result<TokenStream2> {
    if children.is_empty() {
        return Ok(quote! { ::gpui::Empty });
    }
    if children.len() == 1 {
        return emit_node(&children[0]);
    }
    let kids = emit_children(children)?;
    Ok(quote! {{
        let mut el = ::gpui::div();
        #(el = el.child(#kids);)*
        el
    }})
}

pub(crate) fn attr_tokens(value: &AttrValue) -> TokenStream2 {
    match value {
        AttrValue::Expr(e) | AttrValue::Lit(e) => e.clone(),
    }
}

pub(crate) fn string_lit_static(value: &AttrValue) -> Option<TokenStream2> {
    match value {
        AttrValue::Lit(ts) => {
            if syn::parse2::<syn::LitStr>(ts.clone()).is_ok() {
                Some(ts.clone())
            } else {
                None
            }
        }
        _ => None,
    }
}
