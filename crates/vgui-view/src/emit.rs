use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::{emit_element, AttrValue, Node};

pub(crate) fn emit_node(node: &Node) -> syn::Result<TokenStream2> {
    match node {
        Node::Fragment(children) => wrap_children_element(children),
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

/// Emit statements that add `node` as a child of the parent element `parent`.
///
/// For regular nodes this produces `parent = parent.child(expr);`.
/// For control-flow nodes (`<Show>`, `<For>`, `<Switch>`, `<Index>`,
/// `<Provider>`) and fragments, the logic is **inlined** — no wrapper `div`
/// is introduced. Each iteration / branch adds its children directly to
/// `parent`, preserving the parent's layout model (flex, grid, etc.).
pub(crate) fn emit_child_into(parent: &Ident, node: &Node) -> syn::Result<TokenStream2> {
    match node {
        Node::Element(el) => {
            let name = el.tag.to_string();
            match name.as_str() {
                "Show" => crate::control::emit_show_into(el, parent),
                "For" => crate::control::emit_for_into(el, parent),
                "Switch" => crate::control::emit_switch_into(el, parent),
                "Index" => crate::control::emit_index_into(el, parent),
                "Provider" => crate::provider::emit_provider_into(el, parent),
                _ => {
                    let expr = emit_element(el)?;
                    Ok(quote! { #parent = #parent.child(#expr); })
                }
            }
        }
        Node::Fragment(children) => emit_children_into(parent, children),
        Node::Interp(expr) => Ok(quote! { #parent = #parent.child(::vgui::into_child(#expr)); }),
        Node::Text(s) => Ok(quote! { #parent = #parent.child(::vgui::into_child(#s)); }),
    }
}

/// Emit statements that add all `children` to the parent element `parent`.
/// Control-flow children are inlined; regular children produce `.child()` calls.
pub(crate) fn emit_children_into(parent: &Ident, children: &[Node]) -> syn::Result<TokenStream2> {
    let mut stmts = Vec::with_capacity(children.len());
    for node in children {
        stmts.push(emit_child_into(parent, node)?);
    }
    Ok(quote! { #(#stmts)* })
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
