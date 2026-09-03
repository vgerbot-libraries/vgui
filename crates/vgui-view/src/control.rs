use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, emit_children_into, wrap_children_element};
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

// ── Inlined variants ────────────────────────────────────────────────
//
// When a control-flow element is a *direct child of a parent element*,
// `emit_child_into` dispatches to these functions instead of the standalone
// `emit_*` functions. They emit statements that add children directly to the
// parent, avoiding any wrapper `div` and preserving the parent's layout model.

pub(crate) fn emit_show_into(el: &Element, parent: &Ident) -> syn::Result<TokenStream2> {
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
    let then_stmts = emit_children_into(parent, &el.children)?;
    if let Some(fallback) = fallback {
        Ok(quote! {
            if #when {
                #then_stmts
            } else {
                #parent = #parent.child(#fallback);
            }
        })
    } else {
        Ok(quote! {
            if #when {
                #then_stmts
            }
        })
    }
}

pub(crate) fn emit_for_into(el: &Element, parent: &Ident) -> syn::Result<TokenStream2> {
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
        Ok(quote! {{
            let mut __for_closure = #closure;
            let mut __for_count = 0usize;
            for (__i, __item) in #each.into_iter().enumerate() {
                #parent = #parent.child(__for_closure(__item, __i));
                __for_count += 1;
            }
            if __for_count == 0 {
                #parent = #parent.child(#fallback);
            }
        }})
    } else {
        Ok(quote! {{
            let mut __for_closure = #closure;
            for (__i, __item) in #each.into_iter().enumerate() {
                #parent = #parent.child(__for_closure(__item, __i));
            }
        }})
    }
}

pub(crate) fn emit_switch_into(el: &Element, parent: &Ident) -> syn::Result<TokenStream2> {
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

    // Parse <Match> children — store (when, children) pairs.
    let mut branches: Vec<(TokenStream2, Vec<Node>)> = Vec::new();
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
                branches.push((when, match_el.children.clone()));
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

    // Build match arms — each arm inlines its children into the parent.
    let mut arms = Vec::new();
    for (i, (_, children)) in branches.iter().enumerate() {
        let child_stmts = emit_children_into(parent, children)?;
        arms.push(quote! {
            ::core::option::Option::Some(#i) => {
                ::vgui::__switch_enter_branch(__switch_id, #i);
                #child_stmts
                ::vgui::__switch_exit_branch();
            }
        });
    }

    let fallback_arm = match &fallback {
        Some(fb) => quote! { _ => { #parent = #parent.child(#fb); } },
        None => quote! { _ => {} },
    };

    Ok(quote! {{
        let __switch_id = ::vgui::next_auto_id();
        let __active: ::core::option::Option<usize> = #active_check;
        ::vgui::__switch_dispose_inactive(__switch_id, __active, #branch_count);
        match __active {
            #(#arms)*
            #fallback_arm
        }
    }})
}

pub(crate) fn emit_index_into(el: &Element, parent: &Ident) -> syn::Result<TokenStream2> {
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
        Ok(quote! {{
            let __index_id = ::vgui::next_auto_id();
            let __has_scope = ::vgui::__try_current();
            let mut __index_closure = #closure;
            let mut __index_count = 0usize;
            for (__i, __item) in #each.into_iter().enumerate() {
                if __has_scope {
                    ::vgui::enter_child_scope(&::std::format!("index:{}:{}", __index_id, __i));
                }
                #parent = #parent.child(__index_closure(__item, __i));
                if __has_scope {
                    ::vgui::exit_child_scope();
                }
                __index_count += 1;
            }
            if __has_scope {
                ::vgui::__index_dispose_excess(__index_id, __index_count);
            }
            if __index_count == 0 {
                #parent = #parent.child(#fallback);
            }
        }})
    } else {
        Ok(quote! {{
            let __index_id = ::vgui::next_auto_id();
            let __has_scope = ::vgui::__try_current();
            let mut __index_closure = #closure;
            let mut __index_count = 0usize;
            for (__i, __item) in #each.into_iter().enumerate() {
                if __has_scope {
                    ::vgui::enter_child_scope(&::std::format!("index:{}:{}", __index_id, __i));
                }
                #parent = #parent.child(__index_closure(__item, __i));
                if __has_scope {
                    ::vgui::exit_child_scope();
                }
                __index_count += 1;
            }
            if __has_scope {
                ::vgui::__index_dispose_excess(__index_id, __index_count);
            }
        }})
    }
}
