use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::emit::{attr_tokens, wrap_children_element};
use crate::{AttrKind, Element, Node};

pub(crate) fn emit_show(el: &Element) -> syn::Result<TokenStream2> {
    let mut when = None;
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "when" => when = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "fallback" => fallback = Some(attr_tokens(&attr.value)),
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <Show>"))
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <Show>",
                ))
            }
        }
    }
    let when = when.ok_or_else(|| syn::Error::new(el.tag.span(), "missing `when` on <Show>"))?;
    let children = wrap_children_element(&el.children)?;
    if let Some(fallback) = fallback {
        Ok(quote! { ::vgui::show(#when, { #children }, #fallback) })
    } else {
        Ok(quote! { ::vgui::show_when(#when, { #children }) })
    }
}

pub(crate) fn emit_for(el: &Element) -> syn::Result<TokenStream2> {
    let mut each = None;
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "each" => each = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "fallback" => fallback = Some(attr_tokens(&attr.value)),
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <For>"))
            }
            _ => return Err(syn::Error::new(attr.span, "unsupported attribute on <For>")),
        }
    }
    let each = each.ok_or_else(|| syn::Error::new(el.tag.span(), "missing `each` on <For>"))?;
    if el.children.len() != 1 {
        return Err(syn::Error::new(
            el.tag.span(),
            "<For> child must be a closure",
        ));
    }
    let closure = match &el.children[0] {
        Node::Interp(expr) => {
            if looks_like_closure(expr) {
                expr.clone()
            } else {
                return Err(syn::Error::new(
                    el.tag.span(),
                    "<For> child must be a closure",
                ));
            }
        }
        _ => {
            return Err(syn::Error::new(
                el.tag.span(),
                "<For> child must be a closure",
            ))
        }
    };
    if let Some(fallback) = fallback {
        Ok(quote! { ::vgui::for_each_or(#each, #fallback, #closure) })
    } else {
        Ok(quote! { ::vgui::for_each(#each, #closure) })
    }
}

pub(crate) fn looks_like_closure(expr: &TokenStream2) -> bool {
    let s = expr.to_string();
    s.contains('|') || s.contains("move")
}
