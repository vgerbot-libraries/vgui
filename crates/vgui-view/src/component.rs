use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, string_lit_static, emit_child};
use crate::{AttrKind, Element};

pub(crate) fn emit_component(el: &Element) -> syn::Result<TokenStream2> {
    let tag = &el.tag;
    let has_attrs = !el.attrs.is_empty();
    let children = &el.children;

    // No attrs and no children → plain function call (e.g. `Blinker()`)
    if !has_attrs && children.is_empty() {
        return Ok(quote! { #tag() });
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
            AttrKind::Ident(id) => {
                let id_str = id.to_string();
                if id_str.starts_with("on_") {
                    fields.push(quote! { #id: ::std::boxed::Box::new(#value) });
                } else {
                    fields.push(quote! { #id: #value });
                }
            }
            AttrKind::Id => fields.push(quote! { id: #value }),
            AttrKind::Src => fields.push(quote! { src: #value }),
            AttrKind::Style => fields.push(quote! { style: #value }),
            AttrKind::Hover => fields.push(quote! { hover: #value }),
            AttrKind::Active => fields.push(quote! { active: #value }),
            AttrKind::Focus => fields.push(quote! { focus: #value }),
            AttrKind::Class => {
                if string_lit_static(&attr.value).is_some() {
                    fields.push(quote! { class: ::std::option::Option::Some(#value) });
                } else {
                    fields.push(quote! { class: #value });
                }
            }
            AttrKind::Type => fields.push(quote! { r#type: #value }),
            AttrKind::Tabindex => fields.push(quote! { tabindex: #value }),
            AttrKind::For => fields.push(quote! { r#for: #value }),
            AttrKind::Ref => fields.push(quote! { r#ref: #value }),
            AttrKind::Animate => fields.push(quote! { animate: #value }),
            AttrKind::Drag => fields.push(quote! { drag: #value }),
            AttrKind::DragPreview => fields.push(quote! { drag_preview: #value }),
            AttrKind::CanDrop => fields.push(quote! { can_drop: #value }),
            AttrKind::Role => fields.push(quote! { role: ::std::option::Option::Some(::vgui::__resolve_aria_role(#value)) }),
            AttrKind::Aria(name) => {
                let n = name.to_string();
                let aria_key = Ident::new(&format!("aria_{n}"), name.span());
                fields.push(quote! { #aria_key: ::std::option::Option::Some(#value) });
            }
            AttrKind::On(ev) => {
                let name = Ident::new(&format!("on_{ev}"), ev.span());
                fields.push(quote! { #name: ::std::boxed::Box::new(#value) });
            }
        }
    }
    if !children.is_empty() {
        let kids: Vec<TokenStream2> = children.iter().map(emit_child).collect::<Result<_, _>>()?;
        fields.push(quote! { children: ::std::vec![#(::gpui::IntoElement::into_any_element(#kids)),*] });
    }
    if let Some(spread) = spread {
        Ok(quote! { #tag { #(#fields,)* ..#spread } })
    } else {
        Ok(quote! { #tag { #(#fields),* } })
    }
}
