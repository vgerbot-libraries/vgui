use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, emit_children, string_lit_static};
use crate::{Attr, AttrKind, AttrValue, Element};

pub(crate) fn emit_builtin(el: &Element) -> syn::Result<TokenStream2> {
    let name = el.tag.to_string();
    if name == "input" {
        return emit_input(el);
    }
    let mut src = None;
    let mut id = None;
    let mut style = None;
    let mut hover = None;
    let mut active = None;
    let mut focus = None;
    let mut class = None;
    let mut tabindex = None;
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
            AttrKind::Tabindex => tabindex = Some(attr),
            AttrKind::On(ev) => events.push((ev.clone(), attr_tokens(&attr.value), attr.span)),
            AttrKind::Ident(id) => unknown.push(id.clone()),
            AttrKind::Type => {
                return Err(syn::Error::new(attr.span, "`type` attribute is only valid on <input>"))
            }
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
            || tabindex.is_some()
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
        let name_lit = syn::LitStr::new(&name, el.tag.span());
        ctor = quote! { #ctor.id((#name_lit, ::vgui::next_auto_id())) };
    }
    if let Some(tabindex_attr) = tabindex {
        let idx = tabindex_expr(&tabindex_attr.value);
        ctor = quote! {{
            let __tabindex = #idx;
            let mut __el = #ctor;
            if __tabindex >= 0 {
                __el = __el.tab_index(__tabindex);
            } else {
                __el = __el.focusable();
            }
            __el
        }};
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

// ── <input> dispatch ─────────────────────────────────────────────────


/// Return the `AttrKind::Type` attribute's value.
fn type_attr_value<'a>(el: &'a Element) -> Option<&'a AttrValue> {
    for attr in &el.attrs {
        if matches!(attr.kind, AttrKind::Type) {
            return Some(&attr.value);
        }
    }
    None
}

/// Extract a bool expression from an attribute value.  `true`/`false` literals
/// and `{expr}` groups both work.
fn bool_expr(v: &AttrValue) -> TokenStream2 {
    attr_tokens(v)
}

/// Generate the `Option<f64>` expression for a numeric attribute.
fn f64_opt_expr(v: &AttrValue) -> TokenStream2 {
    let e = attr_tokens(v);
    quote! { ::std::option::Option::Some((#e) as f64) }
}

/// Convert an `AttrValue` into an `isize` token stream for `tabindex`.
///
/// Accepted forms: `tabindex="0"` (string literal, parsed at compile time),
/// `tabindex=0` (integer literal), `tabindex={expr}` (any expression).
fn tabindex_expr(v: &AttrValue) -> TokenStream2 {
    // String literal: tabindex="0" → parse at compile time
    if let Some(lit) = string_lit_static(v) {
        if let Ok(lit_str) = syn::parse2::<syn::LitStr>(lit.clone()) {
            if let Ok(n) = lit_str.value().parse::<isize>() {
                return quote! { #n };
            }
        }
    }
    // Integer literal or {expr} → cast to isize
    let e = attr_tokens(v);
    quote! { ((#e) as isize) }
}

/// Apply style/class/hover/active/focus/id/events chaining on a div-returning
/// element, reusing the same logic as `emit_builtin`.
fn chain_div_extras(
    mut ctor: TokenStream2,
    el: &Element,
    style: Option<&Attr>,
    class: Option<&Attr>,
    hover: Option<&Attr>,
    active: Option<&Attr>,
    focus: Option<&Attr>,
    id: Option<&Attr>,
    tabindex: Option<&Attr>,
    events: &[(Ident, TokenStream2, Span)],
) -> TokenStream2 {
    let name = "input";

    let class_needs_id = class
        .and_then(|c| {
            if let Some(lit) = string_lit_static(&c.value) {
                let s = lit.to_string();
                Some(s.contains("focus:") || s.contains("active:"))
            } else {
                None
            }
        })
        .unwrap_or(false);

    let needs_id = id.is_none()
        && (active.is_some()
            || focus.is_some()
            || tabindex.is_some()
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
        let name_lit = syn::LitStr::new(name, el.tag.span());
        ctor = quote! { #ctor.id((#name_lit, ::vgui::next_auto_id())) };
    }
    if let Some(tabindex_attr) = tabindex {
        let idx = tabindex_expr(&tabindex_attr.value);
        ctor = quote! {{
            let __tabindex = #idx;
            let mut __el = #ctor;
            if __tabindex >= 0 {
                __el = __el.tab_index(__tabindex);
            } else {
                __el = __el.focusable();
            }
            __el
        }};
    }

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
        if let Ok(chained) = emit_event(ctor.clone(), ev, handler.clone(), *span) {
            ctor = chained;
        }
    }
    ctor
}

fn emit_input(el: &Element) -> syn::Result<TokenStream2> {
    // Reject children — <input> is a void element.
    if !el.children.is_empty() {
        return Err(syn::Error::new(
            el.tag.span(),
            "<input> is a void element and cannot have children",
        ));
    }

    // ── Determine type ────────────────────────────────────────────────
    let ty = type_attr_value(el);
    let ty_str: String = match ty {
        Some(v) => {
            match string_lit_static(v) {
                Some(ts) => {
                    // ts is a TokenStream containing a LitStr; parse it.
                    let lit: syn::LitStr = syn::parse2(ts.clone())
                        .map_err(|_| syn::Error::new(el.tag.span(), "`type` must be a string literal"))?;
                    lit.value()
                }
                None => {
                    return Err(syn::Error::new(
                        el.tag.span(),
                        "`type` attribute on <input> must be a string literal",
                    ));
                }
            }
        }
        None => "text".to_string(),
    };

    // ── Classify attributes ───────────────────────────────────────────
    let mut style = None;
    let mut hover = None;
    let mut active = None;
    let mut focus = None;
    let mut class = None;
    let mut id = None;
    let mut tabindex = None;
    let mut events: Vec<(Ident, TokenStream2, Span)> = Vec::new();
    // input-specific
    let mut on_input = None;
    let mut on_change = None;
    let mut value = None;
    let mut placeholder = None;
    let mut checked = None;
    let mut disabled = None;
    let mut readonly = None;
    let mut min = None;
    let mut max = None;
    let mut step = None;
    let mut multiple = None;

    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Type => {} // already handled above
            AttrKind::Style => style = Some(attr),
            AttrKind::Hover => hover = Some(attr),
            AttrKind::Active => active = Some(attr),
            AttrKind::Focus => focus = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::Id => id = Some(attr),
            AttrKind::Tabindex => tabindex = Some(attr),
            AttrKind::On(ev) => {
                let ev_name = ev.to_string();
                let handler = attr_tokens(&attr.value);
                match ev_name.as_str() {
                    "input" => on_input = Some(handler),
                    "change" => on_change = Some(handler),
                    _ => events.push((ev.clone(), handler, attr.span)),
                }
            }
            AttrKind::Ident(id2) => {
                let name = id2.to_string();
                match name.as_str() {
                    "value" => value = Some(&attr.value),
                    "placeholder" => placeholder = Some(&attr.value),
                    "checked" => checked = Some(&attr.value),
                    "disabled" => disabled = Some(&attr.value),
                    "readonly" => readonly = Some(&attr.value),
                    "min" => min = Some(&attr.value),
                    "max" => max = Some(&attr.value),
                    "step" => step = Some(&attr.value),
                    "multiple" => multiple = Some(&attr.value),
                    "accept" | "name" => {} // accepted but unused in v1
                    other => {
                        return Err(syn::Error::new(
                            id2.span(),
                            format!("unknown attribute `{other}` on <input>"),
                        ));
                    }
                }
            }
            AttrKind::Src => {
                return Err(syn::Error::new(attr.span, "src is not valid on <input>"));
            }
        }
    }

    // ── Dispatch on type ──────────────────────────────────────────────
    match ty_str.as_str() {
        // ── Text-based types ───────────────────────────────────────────
        "text" | "password" | "search" | "email" | "url" | "tel"
        | "number" | "date" | "datetime-local" | "time" | "month" | "week" | "color" => {
            let kind_variant = match ty_str.as_str() {
                "text" => quote! { ::vgui::TextKind::Text },
                "password" => quote! { ::vgui::TextKind::Password },
                "search" => quote! { ::vgui::TextKind::Search },
                "email" => quote! { ::vgui::TextKind::Email },
                "url" => quote! { ::vgui::TextKind::Url },
                "tel" => quote! { ::vgui::TextKind::Tel },
                "number" => quote! { ::vgui::TextKind::Number },
                "date" => quote! { ::vgui::TextKind::Date },
                "datetime-local" => quote! { ::vgui::TextKind::DateTime },
                "time" => quote! { ::vgui::TextKind::Time },
                "month" => quote! { ::vgui::TextKind::Month },
                "week" => quote! { ::vgui::TextKind::Week },
                "color" => quote! { ::vgui::TextKind::Color },
                _ => unreachable!(),
            };

            let value_expr = match value {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { ::std::string::ToString::to_string(&(#e)) }
                }
                None => quote! { ::std::string::String::new() },
            };

            let placeholder_expr = match placeholder {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { ::std::option::Option::Some(::std::convert::Into::into(#e)) }
                }
                None => quote! { ::std::option::Option::None },
            };

            let disabled_expr = match disabled {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };

            let readonly_expr = match readonly {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };

            let min_expr = match min {
                Some(v) => f64_opt_expr(v),
                None => quote! { ::std::option::Option::None },
            };

            let max_expr = match max {
                Some(v) => f64_opt_expr(v),
                None => quote! { ::std::option::Option::None },
            };

            let step_expr = match step {
                Some(v) => f64_opt_expr(v),
                None => quote! { ::std::option::Option::None },
            };

            let on_input_expr = match on_input {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::input_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };

            let on_change_expr = match on_change {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::str_change_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };

            let style_expr = match style {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(#v) }
                }
                None => quote! { ::std::option::Option::None },
            };

            let class_expr = match class {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(::vgui::tw!(#v)) }
                }
                None => quote! { ::std::option::Option::None },
            };

            let tabindex_expr_val = match tabindex {
                Some(v) => {
                    let e = tabindex_expr(&v.value);
                    quote! { ::std::option::Option::Some(#e) }
                }
                None => quote! { ::std::option::Option::None },
            };

            Ok(quote! {
                ::vgui::text_input(::vgui::TextInputProps {
                    kind: #kind_variant,
                    value: #value_expr,
                    placeholder: #placeholder_expr,
                    disabled: #disabled_expr,
                    readonly: #readonly_expr,
                    min: #min_expr,
                    max: #max_expr,
                    step: #step_expr,
                    on_input: #on_input_expr,
                    on_change: #on_change_expr,
                    style: #style_expr,
                    class: #class_expr,
                    tabindex: #tabindex_expr_val,
                })
            })
        }

        // ── Checkbox ───────────────────────────────────────────────────
        "checkbox" => {
            let checked_expr = match checked {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let disabled_expr = match disabled {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let on_change_expr = match on_change {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::bool_change_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };

            let ctor = quote! {
                ::vgui::checkbox(::vgui::CheckboxProps {
                    checked: #checked_expr,
                    disabled: #disabled_expr,
                    on_change: #on_change_expr,
                })
            };
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, id, tabindex, &events);
            Ok(quote! {{ #ctor }})
        }

        // ── Radio ──────────────────────────────────────────────────────
        "radio" => {
            let checked_expr = match checked {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let disabled_expr = match disabled {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let on_change_expr = match on_change {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::bool_change_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };

            let ctor = quote! {
                ::vgui::radio(::vgui::RadioProps {
                    checked: #checked_expr,
                    disabled: #disabled_expr,
                    on_change: #on_change_expr,
                })
            };
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, id, tabindex, &events);
            Ok(quote! {{ #ctor }})
        }

        // ── Range ──────────────────────────────────────────────────────
        "range" => {
            let value_expr = match value {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { (#e) as f64 }
                }
                None => quote! { 0.0f64 },
            };
            let min_expr = match min {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { (#e) as f64 }
                }
                None => quote! { 0.0f64 },
            };
            let max_expr = match max {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { (#e) as f64 }
                }
                None => quote! { 100.0f64 },
            };
            let step_expr = match step {
                Some(v) => {
                    let e = attr_tokens(v);
                    quote! { (#e) as f64 }
                }
                None => quote! { 1.0f64 },
            };
            let disabled_expr = match disabled {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let on_change_expr = match on_change {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::f64_change_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };
            let style_expr = match style {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(#v) }
                }
                None => quote! { ::std::option::Option::None },
            };
            let class_expr = match class {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(::vgui::tw!(#v)) }
                }
                None => quote! { ::std::option::Option::None },
            };

            let tabindex_expr_val = match tabindex {
                Some(v) => {
                    let e = tabindex_expr(&v.value);
                    quote! { ::std::option::Option::Some(#e) }
                }
                None => quote! { ::std::option::Option::None },
            };

            Ok(quote! {
                ::vgui::range_input(::vgui::RangeProps {
                    value: #value_expr,
                    min: #min_expr,
                    max: #max_expr,
                    step: #step_expr,
                    disabled: #disabled_expr,
                    on_change: #on_change_expr,
                    style: #style_expr,
                    class: #class_expr,
                    tabindex: #tabindex_expr_val,
                })
            })
        }

        // ── File ───────────────────────────────────────────────────────
        "file" => {
            let multiple_expr = match multiple {
                Some(v) => bool_expr(v),
                None => quote! { false },
            };
            let on_change_expr = match on_change {
                Some(h) => quote! { ::std::option::Option::Some(::vgui::files_cb(#h)) },
                None => quote! { ::std::option::Option::None },
            };

            let ctor = quote! {
                ::vgui::file_input(::vgui::FileProps {
                    multiple: #multiple_expr,
                    on_change: #on_change_expr,
                })
            };
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, id, tabindex, &events);
            // Use `value` attr as the button label text.
            if let Some(v) = value {
                let label = attr_tokens(v);
                Ok(quote! {{
                    let mut el = #ctor;
                    el = el.child(::vgui::into_child(#label));
                    el
                }})
            } else {
                Ok(quote! {{ #ctor }})
            }
        }

        // ── Submit / Button / Reset ────────────────────────────────────
        "submit" | "button" | "reset" => {
            // Build a button element: a div().cursor_pointer() with the value
            // attr as text content if no children, and on:click wired.
            let mut ctor = quote! { ::gpui::div().cursor_pointer() };

            // Use `value` attr as button label text.
            let label = value.map(|v| attr_tokens(v));

            ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, id, tabindex, &events);

            if let Some(label) = label {
                Ok(quote! {{
                    let mut el = #ctor;
                    el = el.child(::vgui::into_child(#label));
                    el
                }})
            } else {
                Ok(quote! {{ #ctor }})
            }
        }

        // ── Hidden ─────────────────────────────────────────────────────
        "hidden" => Ok(quote! { ::gpui::Empty }),

        // ── Unknown type ───────────────────────────────────────────────
        other => Err(syn::Error::new(
            el.tag.span(),
            format!(
                "unsupported `<input type=\"{other}\">`; supported types: text, password, search, email, url, tel, number, date, datetime-local, time, month, week, color, checkbox, radio, range, file, submit, button, reset, hidden"
            ),
        )),
    }
}
