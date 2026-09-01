use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, emit_child};
use crate::{AttrKind, Element};

pub(crate) fn emit_component(el: &Element) -> syn::Result<TokenStream2> {
    let tag = &el.tag;
    let has_attrs = !el.attrs.is_empty();
    let children = &el.children;
    if !has_attrs && children.is_empty() {
        return Ok(quote! { #tag() });
    }
    if !has_attrs {
        if children.len() == 1 {
            let c = emit_child(&children[0])?;
            return Ok(quote! { #tag(#c) });
        }
        let kids: Vec<TokenStream2> = children.iter().map(emit_child).collect::<Result<_, _>>()?;
        return Ok(quote! { #tag(::std::vec![#(#kids),*]) });
    }
    let mut fields = Vec::new();
    let mut spread: Option<TokenStream2> = None;
    for attr in &el.attrs {
        let value = attr_tokens(&attr.value);
        match &attr.kind {
            AttrKind::Spread => {
                if spread.is_some() {
                    return Err(syn::Error::new(
                        attr.span,
                        "only one `{..props}` spread is allowed per component",
                    ));
                }
                spread = Some(value);
            }
            AttrKind::Ident(id) => fields.push(quote! { #id: #value }),
            AttrKind::Id => fields.push(quote! { id: #value }),
            AttrKind::Src => fields.push(quote! { src: #value }),
            AttrKind::Style => fields.push(quote! { style: #value }),
            AttrKind::Hover => fields.push(quote! { hover: #value }),
            AttrKind::Active => fields.push(quote! { active: #value }),
            AttrKind::Focus => fields.push(quote! { focus: #value }),
            AttrKind::Class => fields.push(quote! { class: #value }),
            AttrKind::Type => fields.push(quote! { r#type: #value }),
            AttrKind::Tabindex => fields.push(quote! { tabindex: #value }),
            AttrKind::For => fields.push(quote! { r#for: #value }),
            AttrKind::Ref => fields.push(quote! { r#ref: #value }),
            AttrKind::Animate => fields.push(quote! { animate: #value }),
            AttrKind::Role => fields.push(quote! { role: ::std::option::Option::Some(::vgui::__resolve_aria_role(#value)) }),
            AttrKind::Aria(name) => {
                let n = name.to_string();
                let aria_key = Ident::new(&format!("aria_{n}"), name.span());
                fields.push(quote! { #aria_key: ::std::option::Option::Some(#value) });
            }
            AttrKind::On(ev) => {
                let name = Ident::new(&format!("on_{ev}"), ev.span());
                fields.push(quote! { #name: #value });
            }
        }
    }
    if !children.is_empty() {
        let kids: Vec<TokenStream2> = children.iter().map(emit_child).collect::<Result<_, _>>()?;
        fields.push(quote! { children: ::std::vec![#(#kids),*] });
    }
    if let Some(spread) = spread {
        Ok(quote! { #tag { #(#fields,)* ..#spread } })
    } else {
        Ok(quote! { #tag { #(#fields),* } })
    }
}
