use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, emit_children, string_lit_static};
use crate::{AttrKind, Element};

pub(crate) fn emit_builtin(el: &Element) -> syn::Result<TokenStream2> {
    let name = el.tag.to_string();
    let mut src = None;
    let mut id = None;
    let mut style = None;
    let mut hover = None;
    let mut active = None;
    let mut focus = None;
    let mut class = None;
    let mut events = Vec::new();
    let mut unknown = Vec::new();
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Src => src = Some(attr),
            AttrKind::Id => id = Some(attr),
            AttrKind::Style => style = Some(attr),
            AttrKind::Hover => hover = Some(attr),
            AttrKind::Active => active = Some(attr),
            AttrKind::Focus => focus = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::On(ev) => events.push((ev.clone(), attr_tokens(&attr.value), attr.span)),
            AttrKind::Ident(id) => unknown.push(id.clone()),
        }
    }
    if !unknown.is_empty() {
        return Err(syn::Error::new(
            unknown[0].span(),
            format!("unknown attribute `{}` on <{name}>", unknown[0]),
        ));
    }
    let mut ctor = match name.as_str() {
        "div" | "span" | "p" => quote! { ::gpui::div() },
        "button" => quote! { ::gpui::div().cursor_pointer() },
        "img" => {
            let src = src.ok_or_else(|| syn::Error::new(el.tag.span(), "<img> requires src"))?;
            let v = attr_tokens(&src.value);
            quote! { ::gpui::img(#v) }
        }
        other => {
            return Err(syn::Error::new(
                el.tag.span(),
                format!("unknown element <{other}>"),
            ))
        }
    };
    if name != "img" {
        if let Some(src) = src {
            return Err(syn::Error::new(src.span, "src is only valid on <img>"));
        }
    }

    // Check if class string contains focus: or active: variants (requires id)
    let class_needs_id = class
        .as_ref()
        .and_then(|c| {
            if let Some(lit) = string_lit_static(&c.value) {
                let s = lit.to_string();
                Some(s.contains("focus:") || s.contains("active:"))
            } else {
                None
            }
        })
        .unwrap_or(false);

    let needs_stateful = !events.is_empty()
        || hover.is_some()
        || active.is_some()
        || focus.is_some()
        || class.is_some()
        || events.iter().any(|(ev, _, _)| {
            matches!(
                ev.to_string().as_str(),
                "click"
                    | "hover"
                    | "mouse_down"
                    | "mouse_up"
                    | "mouse_move"
                    | "scroll"
                    | "key_down"
                    | "key_up"
            )
        });
    // hover is on InteractiveElement, not Stateful. active/on_click/on_hover need Stateful.
    let needs_id = id.is_none()
        && (active.is_some()
            || focus.is_some()
            || class.is_some()
            || class_needs_id
            || events
                .iter()
                .any(|(ev, _, _)| matches!(ev.to_string().as_str(), "click" | "hover")));

    if let Some(id_attr) = id {
        let v = if let Some(lit) = string_lit_static(&id_attr.value) {
            quote! { #lit }
        } else {
            attr_tokens(&id_attr.value)
        };
        ctor = quote! { #ctor.id(#v) };
    } else if needs_id {
        // Generate the id at runtime so that elements produced by a closure
        // invoked multiple times (e.g. `<For>` items) each get a distinct id,
        // while remaining stable across re-renders (the counter is reset on
        // every `VguiRoot` render). Using a `(&'static str, u64)` produces an
        // `ElementId::NamedInteger`, which is readable and collision-resistant.
        let name_lit = syn::LitStr::new(&name, el.tag.span());
        ctor = quote! { #ctor.id((#name_lit, ::vgui::next_auto_id())) };
    }
    let _ = needs_stateful;

    if let Some(style) = style {
        let v = attr_tokens(&style.value);
        ctor = quote! { ::vgui::ApplyStyle::apply_to(#v, #ctor) };
    }
    if let Some(class) = class {
        let v = attr_tokens(&class.value);
        ctor = quote! {{
            let __tw = ::vgui::tw!(#v);
            let mut __el = #ctor;
            (__tw.base)(__el.style());
            if let ::std::option::Option::Some(__h) = __tw.hover {
                __el = __el.hover(move |mut s| { __h(&mut s); s });
            }
            if let ::std::option::Option::Some(__f) = __tw.focus {
                __el = __el.focus(move |mut s| { __f(&mut s); s });
            }
            if let ::std::option::Option::Some(__a) = __tw.active {
                __el = __el.active(move |mut s| { __a(&mut s); s });
            }
            __el
        }};
    }
    if let Some(hover) = hover {
        let v = attr_tokens(&hover.value);
        ctor = quote! { #ctor.hover(move |s| #v.refine(s)) };
    }
    if let Some(active) = active {
        let v = attr_tokens(&active.value);
        ctor = quote! { #ctor.active(|s| #v.refine(s)) };
    }
    if let Some(focus) = focus {
        let v = attr_tokens(&focus.value);
        ctor = quote! { #ctor.focus(|s| #v.refine(s)) };
    }
    for (ev, handler, span) in events {
        ctor = emit_event(ctor, &ev, handler, span)?;
    }
    let kids = emit_children(&el.children)?;
    Ok(quote! {{
        let mut el = #ctor;
        #(el = el.child(#kids);)*
        el
    }})
}

fn emit_event(
    ctor: TokenStream2,
    ev: &Ident,
    handler: TokenStream2,
    span: Span,
) -> syn::Result<TokenStream2> {
    match ev.to_string().as_str() {
        "click" => Ok(quote! { #ctor.on_click(#handler) }),
        "mouse_down" => Ok(quote! { #ctor.on_mouse_down(::gpui::MouseButton::Left, #handler) }),
        "mouse_up" => Ok(quote! { #ctor.on_mouse_up(::gpui::MouseButton::Left, #handler) }),
        "mouse_move" => Ok(quote! { #ctor.on_mouse_move(#handler) }),
        "scroll" => Ok(quote! { #ctor.on_scroll_wheel(#handler) }),
        "key_down" => Ok(quote! { #ctor.on_key_down(#handler) }),
        "key_up" => Ok(quote! { #ctor.on_key_up(#handler) }),
        "hover" => Ok(quote! { #ctor.on_hover(#handler) }),
        other => Err(syn::Error::new(
            span,
            format!(
                "unsupported event `on:{other}`; supported: click, mouse_down, mouse_up, mouse_move, scroll, key_down, key_up, hover"
            ),
        )),
    }
}
