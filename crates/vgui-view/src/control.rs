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

pub(crate) fn emit_switch(el: &Element) -> syn::Result<TokenStream2> {
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "fallback" => {
                fallback = Some(attr_tokens(&attr.value))
            }
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <Switch>"))
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <Switch>; allowed: `fallback`",
                ))
            }
        }
    }

    // Parse <Match> children.
    let mut branches: Vec<(TokenStream2, TokenStream2)> = Vec::new();
    for child in &el.children {
        match child {
            Node::Element(match_el) if match_el.tag.to_string() == "Match" => {
                let mut when = None;
                for attr in &match_el.attrs {
                    match &attr.kind {
                        AttrKind::Ident(id) if id == "when" => {
                            when = Some(attr_tokens(&attr.value))
                        }
                        AttrKind::Ref => {
                            return Err(syn::Error::new(
                                attr.span,
                                "ref is not supported on <Match>",
                            ))
                        }
                        _ => {
                            return Err(syn::Error::new(
                                attr.span,
                                "unsupported attribute on <Match>; allowed: `when`",
                            ))
                        }
                    }
                }
                let when = when.ok_or_else(|| {
                    syn::Error::new(match_el.tag.span(), "missing `when` on <Match>")
                })?;
                let body = wrap_children_element(&match_el.children)?;
                branches.push((when, body));
            }
            _ => {
                return Err(syn::Error::new(
                    el.tag.span(),
                    "<Switch> children must be <Match> elements",
                ))
            }
        }
    }

    let branch_count = branches.len();

    // Build if-else chain (short-circuit, first match wins).
    let mut active_check = quote! { ::core::option::Option::None };
    for (i, (when, _)) in branches.iter().enumerate().rev() {
        active_check = quote! {
            if #when { ::core::option::Option::Some(#i) } else { #active_check }
        };
    }

    // Build match arms with child scope enter/exit. Each arm wraps its body
    // in `into_any_element` so all arms share the type `AnyElement`.
    let mut arms = Vec::new();
    for (i, (_, body)) in branches.iter().enumerate() {
        arms.push(quote! {
            ::core::option::Option::Some(#i) => {
                ::vgui::__switch_enter_branch(__switch_id, #i);
                let __el = { #body };
                ::vgui::__switch_exit_branch();
                ::gpui::IntoElement::into_any_element(__el)
            }
        });
    }

    let fallback_arm = match &fallback {
        Some(fb) => quote! { _ => ::gpui::IntoElement::into_any_element(#fb) },
        None => quote! { _ => ::gpui::IntoElement::into_any_element(::gpui::Empty) },
    };

    Ok(quote! { {
        let __switch_id = ::vgui::next_auto_id();
        let __active: ::core::option::Option<usize> = #active_check;
        ::vgui::__switch_dispose_inactive(__switch_id, __active, #branch_count);
        match __active {
            #(#arms)*
            #fallback_arm
        }
    } })
}

pub(crate) fn emit_index(el: &Element) -> syn::Result<TokenStream2> {
    let mut each = None;
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "each" => each = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "fallback" => {
                fallback = Some(attr_tokens(&attr.value))
            }
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <Index>"))
            }
            _ => return Err(syn::Error::new(attr.span, "unsupported attribute on <Index>")),
        }
    }
    let each =
        each.ok_or_else(|| syn::Error::new(el.tag.span(), "missing `each` on <Index>"))?;
    if el.children.len() != 1 {
        return Err(syn::Error::new(
            el.tag.span(),
            "<Index> child must be a closure",
        ));
    }
    let closure = match &el.children[0] {
        Node::Interp(expr) if looks_like_closure(expr) => expr.clone(),
        _ => {
            return Err(syn::Error::new(
                el.tag.span(),
                "<Index> child must be a closure",
            ))
        }
    };
    if let Some(fallback) = fallback {
        Ok(quote! { ::vgui::index_list_or(#each, #fallback, #closure) })
    } else {
        Ok(quote! { ::vgui::index_list(#each, #closure) })
    }
}
