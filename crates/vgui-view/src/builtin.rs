use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;

use crate::emit::{attr_tokens, emit_children, string_lit_static};
use crate::{Attr, AttrKind, AttrValue, Element};

pub(crate) fn emit_builtin(el: &Element) -> syn::Result<TokenStream2> {
    let name = el.tag.to_string();
    if name == "input" {
        return emit_input(el);
    }
    if name == "label" {
        return emit_label(el);
    }
    if name == "textarea" {
        return emit_textarea(el);
    }
    if name == "select" {
        return emit_select(el);
    }
    if name == "dialog" {
        return emit_dialog(el);
    }
    if name == "portal" {
        return emit_portal(el);
    }
    if name == "floating" {
        return emit_floating(el);
    }
    if name == "radiogroup" {
        return emit_radiogroup(el);
    }
    if name == "wbr" {
        return Ok(quote! { ::gpui::Empty });
    }
    // <colgroup>/<col> have no flex-box meaning; column widths are controlled
    // per-cell via class/style. Render nothing.
    if name == "colgroup" || name == "col" {
        return Ok(quote! { ::gpui::Empty });
    }
    let mut src = None;
    let mut id = None;
    let mut style = None;
    let mut hover = None;
    let mut active = None;
    let mut focus = None;
    let mut class = None;
    let mut tabindex = None;
    let mut ref_attr = None;
    let mut object_fit = None;
    let mut animate = None;
    let mut events = Vec::new();
    let mut unknown = Vec::new();
    let mut spreads: Vec<TokenStream2> = Vec::new();
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Src => src = Some(attr),
            AttrKind::Id => id = Some(attr),
            AttrKind::Style => style = Some(attr),
            AttrKind::Hover => hover = Some(attr),
            AttrKind::Active => active = Some(attr),
            AttrKind::Focus => focus = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::Ref => ref_attr = Some(attr),
            AttrKind::Animate => animate = Some(attr),
            AttrKind::Tabindex => tabindex = Some(attr),
            AttrKind::On(ev) => events.push((ev.clone(), attr_tokens(&attr.value), attr.span)),
            AttrKind::Ident(id) => {
                let id_name = id.to_string();
                // Allow href on <a>, value/max on <progress>/<meter>, open on <details>,
                // colspan/rowspan on <td>/<th>.
                if (name == "a" && id_name == "href")
                    || ((name == "progress" || name == "meter") && (id_name == "value" || id_name == "max"))
                    || (name == "details" && id_name == "open")
                    || ((name == "td" || name == "th") && (id_name == "colspan" || id_name == "rowspan"))
                {
                    // ignore — consumed in the tag match
                } else if name == "img" && id_name == "object_fit" {
                    object_fit = Some(attr);
                } else {
                    unknown.push(id.clone());
                }
            }
            AttrKind::Type => {
                return Err(syn::Error::new(attr.span, "`type` attribute is only valid on <input>"))
            }
            AttrKind::Spread => {
                if !spreads.is_empty() {
                    return Err(syn::Error::new(
                        attr.span,
                        "only one `{..props}` spread is allowed per element",
                    ));
                }
                spreads.push(attr_tokens(&attr.value));
            }
            AttrKind::For => {
                return Err(syn::Error::new(attr.span, "`for` is only valid on <label>"))
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
        // Pure div aliases (semantic containers)
        "div" | "span" | "p"
        | "header" | "footer" | "nav" | "main" | "section" | "article" | "aside"
        | "address" | "form" | "fieldset" | "legend" | "figure" | "figcaption"
        | "pre" | "blockquote" | "q" => quote! { ::gpui::div() },
        // Headings h1-h6 with default font-size (rem) and font-weight
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            let (size, weight) = match name.as_str() {
                "h1" => (2.0f32, 600.0f32),
                "h2" => (1.5, 600.0),
                "h3" => (1.25, 600.0),
                "h4" => (1.0, 600.0),
                "h5" => (0.875, 600.0),
                "h6" => (0.85, 500.0),
                _ => unreachable!(),
            };
            quote! { ::gpui::div().text_size(::gpui::rems(#size)).font_weight(::gpui::FontWeight(#weight)) }
        }
        // Text formatting tags
        "strong" | "b" => quote! { ::gpui::div().font_weight(::gpui::FontWeight::BOLD) },
        "em" | "i" => quote! { ::gpui::div().italic() },
        "u" => quote! { ::gpui::div().underline() },
        "s" | "del" | "strike" => quote! { ::gpui::div().line_through() },
        "mark" => quote! { ::gpui::div().text_bg(::gpui::hsla(60./360., 1., 0.5, 1.)) },
        "small" => quote! { ::gpui::div().text_size(::gpui::rems(0.875)) },
        "code" | "kbd" | "samp" | "var" => quote! { ::gpui::div().font_family("monospace") },
        "cite" | "abbr" | "dfn" | "bdi" | "bdo" | "time" => quote! { ::gpui::div() },
        // Void elements
        "br" => quote! { ::gpui::div().h(::gpui::px(1. * 16.)) },
        "hr" => quote! { ::gpui::div().w_full().h(::gpui::px(1.)).bg(::gpui::hsla(0., 0., 0.5, 1.)) },
        // Link
        "a" => quote! { ::gpui::div().cursor_pointer().text_color(::gpui::hsla(220./360., 1., 0.5, 1.)) },
        // Lists
        "ul" | "ol" => quote! { ::gpui::div().flex_col() },
        "li" => quote! { ::gpui::div() },
        "dl" => quote! { ::gpui::div().flex_col() },
        "dt" => quote! { ::gpui::div().font_weight(::gpui::FontWeight::BOLD) },
        "dd" => quote! { ::gpui::div().pl(::gpui::px(16.)) },
        // Summary (clickable header for details)
        "summary" => quote! { ::gpui::div().cursor_pointer() },
        // Button (existing)
        "button" => quote! { ::gpui::div().cursor_pointer() },
        // Tables (flex-based layout; gpui has no native table layout)
        "table" | "thead" | "tbody" | "tfoot" => quote! { ::gpui::div().flex_col() },
        "caption" => quote! { ::gpui::div() },
        "tr" => quote! { ::gpui::div().flex().w_full() },
        "td" => emit_cell(el, false)?,
        "th" => emit_cell(el, true)?,
        // Image (existing)
        "img" => {
            let src = src.ok_or_else(|| syn::Error::new(el.tag.span(), "<img> requires src"))?;
            let v = attr_tokens(&src.value);
            quote! { ::gpui::img(#v) }
        }
        // SVG (uses gpui::svg() element, similar to img)
        "svg" => {
            let src = src.ok_or_else(|| syn::Error::new(el.tag.span(), "<svg> requires src"))?;
            let v = attr_tokens(&src.value);
            quote! { ::gpui::svg().path(#v) }
        }
        // Progress bar
        "progress" => {
            let value = el.attrs.iter().find_map(|a| {
                if let AttrKind::Ident(id) = &a.kind {
                    if id.to_string() == "value" { return Some(attr_tokens(&a.value)); }
                }
                None
            }).unwrap_or(quote! { 0f64 });
            let max = el.attrs.iter().find_map(|a| {
                if let AttrKind::Ident(id) = &a.kind {
                    if id.to_string() == "max" { return Some(attr_tokens(&a.value)); }
                }
                None
            }).unwrap_or(quote! { 1f64 });
            quote! { ::vgui::progress(#value, #max) }
        }
        // Meter (similar to progress)
        "meter" => {
            let value = el.attrs.iter().find_map(|a| {
                if let AttrKind::Ident(id) = &a.kind {
                    if id.to_string() == "value" { return Some(attr_tokens(&a.value)); }
                }
                None
            }).unwrap_or(quote! { 0f64 });
            let max = el.attrs.iter().find_map(|a| {
                if let AttrKind::Ident(id) = &a.kind {
                    if id.to_string() == "max" { return Some(attr_tokens(&a.value)); }
                }
                None
            }).unwrap_or(quote! { 1f64 });
            quote! { ::vgui::progress(#value, #max) }
        }
        // Details/summary collapsible container
        "details" => {
            let open = el.attrs.iter().find_map(|a| {
                if let AttrKind::Ident(id) = &a.kind {
                    if id.to_string() == "open" { return Some(attr_tokens(&a.value)); }
                }
                None
            }).unwrap_or(quote! { false });
            let kids = emit_children(&el.children)?;
            quote! { ::vgui::details(#open, ::gpui::div(), {
                let mut __p = ::gpui::div().flex_col();
                #(let __c = #kids; __p = __p.child(__c);)*
                __p
            }) }
        }
        other => {
            return Err(syn::Error::new(
                el.tag.span(),
                format!("unknown element <{other}>"),
            ))
        }
    };
    if name != "img" && name != "svg" {
        if let Some(src) = src {
            return Err(syn::Error::new(src.span, "src is only valid on <img>"));
        }
    }
    // Apply object-fit to img elements
    if name == "img" {
        if let Some(of) = object_fit {
            let v = match string_lit_static(&of.value).map(|s| s.to_string()).as_deref() {
                Some("fill") => quote! { ::gpui::ObjectFit::Fill },
                Some("contain") => quote! { ::gpui::ObjectFit::Contain },
                Some("cover") => quote! { ::gpui::ObjectFit::Cover },
                Some("scale-down") => quote! { ::gpui::ObjectFit::ScaleDown },
                Some("none") => quote! { ::gpui::ObjectFit::None },
                _ => return Err(syn::Error::new(of.span, "invalid object-fit value; expected fill/contain/cover/scale-down/none")),
            };
            ctor = quote! { #ctor.object_fit(#v) };
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
                    | "keydown"
                    | "keyup"
                    | "pointerdown"
                    | "pointerup"
                    | "pointermove"
                    | "scroll"
            )
        });
    // hover is on InteractiveElement, not Stateful. active/on_click/on_hover need Stateful.
    let needs_id = id.is_none()
        && (ref_attr.is_some()
            || active.is_some()
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

    if let Some(ref_attr) = ref_attr {
        let v = attr_tokens(&ref_attr.value);
        ctor = quote! {{
            let __ref = #v;
            let __bound = ::vgui::__bind_ref(&__ref);
            let mut __el = #ctor;
            if __bound {
                __el = __el.track_focus(&__ref.focus_handle());
                __el = __el.track_scroll(&__ref.scroll_handle());
            }
            __el
        }};
    }
    let _ = needs_stateful;

    if let Some(style) = style {
        let v = attr_tokens(&style.value);
        ctor = quote! { ::vgui::ApplyStyle::apply_to(#v, #ctor) };
    }
    let mut __class_value: Option<TokenStream2> = None;
    if let Some(class) = class {
        let v = attr_tokens(&class.value);
        __class_value = Some(v.clone());
        ctor = quote! {{
            let __tw = ::vgui::tw!(#v);
            let ::vgui::TwStyle { base, hover, focus, active, animation: _, transition } = __tw;
            let mut __el = #ctor;
            (base)(__el.style());
            // When a transition is configured, hover is driven by the transition
            // wrapper (applied after children); skip the static .hover() here.
            if !(transition.is_some() && hover.is_some()) {
                if let ::std::option::Option::Some(__h) = hover {
                    __el = __el.hover(move |mut s| { __h(&mut s); s });
                }
            }
            if let ::std::option::Option::Some(__f) = focus {
                __el = __el.focus(move |mut s| { __f(&mut s); s });
            }
            if let ::std::option::Option::Some(__a) = active {
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
    let __animate_expr: Option<TokenStream2> = animate.map(|a| attr_tokens(&a.value));
    for (ev, handler, span) in events {
        ctor = emit_event(ctor, &ev, handler, span)?;
    }
    let is_void = matches!(name.as_str(), "br" | "hr");
    let handles_children = matches!(name.as_str(), "details");
    let kids = if is_void || handles_children {
        Vec::new()
    } else {
        emit_children(&el.children)?
    };
    let anim_wrap = if let Some(ae) = &__animate_expr {
        // Explicit animate={...} attribute — applied last, after children.
        quote! { ::gpui::IntoElement::into_any_element(::vgui::apply_animation_expr(el, #ae)) }
    } else if let Some(cv) = &__class_value {
        quote! {{
            let __tw = ::vgui::tw!(#cv);
            let ::vgui::TwStyle { base: __b, hover: __h, focus: _, active: _, animation: __anim, transition: __trans } = __tw;
            if let ::std::option::Option::Some(__a) = __anim {
                ::gpui::IntoElement::into_any_element(::vgui::apply_animation(el, &__a))
            } else if let ::std::option::Option::Some(__t) = __trans {
                if __h.is_some() {
                    let mut __bs = ::gpui::StyleRefinement::default();
                    (__b)(&mut __bs);
                    let mut __hs = __bs.clone();
                    if let ::std::option::Option::Some(__hh) = &__h {
                        __hh(&mut __hs);
                    }
                    let (__hovered, __set_hovered) = ::vgui::create_signal(false);
                    let __el = el.on_hover(move |__is_hovered, _, __cx| {
                        __set_hovered.update(__cx, |__hh| *__hh = *__is_hovered);
                    });
                    ::gpui::IntoElement::into_any_element(::vgui::apply_transition(__el, __t, __bs, __hs, __hovered))
                } else {
                    ::gpui::IntoElement::into_any_element(el)
                }
            } else {
                ::gpui::IntoElement::into_any_element(el)
            }
        }}
    } else {
        quote! { el }
    };
    Ok(quote! {{
        let mut el = #ctor;
        #(el = el.child(#kids);)*
        #(el = ::vgui::Spread::spread(#spreads, el);)*
        #anim_wrap
    }})
}

/// Emit a `<td>` or `<th>` cell. `is_header` selects `<th>` defaults
/// (bold + centered text). `colspan` is mapped to `flex_grow` so a spanning
/// cell grows N× relative to colspan=1 cells. `rowspan` is accepted by the
/// attribute whitelist but has no layout effect in flex layout.
fn emit_cell(el: &Element, is_header: bool) -> syn::Result<TokenStream2> {
    let colspan: Option<TokenStream2> = el.attrs.iter().find_map(|a| {
        if let AttrKind::Ident(id) = &a.kind {
            if id.to_string() == "colspan" {
                return Some(attr_tokens(&a.value));
            }
        }
        None
    });

    let mut ctor = if is_header {
        quote! { ::gpui::div().flex_1().font_weight(::gpui::FontWeight::BOLD).text_center() }
    } else {
        quote! { ::gpui::div().flex_1() }
    };

    if let Some(cs) = colspan {
        ctor = quote! {{
            let mut __el = #ctor;
            __el.style().flex_grow = Some((#cs) as f32);
            __el
        }};
    }

    Ok(ctor)
}

fn emit_event(
    ctor: TokenStream2,
    ev: &Ident,
    handler: TokenStream2,
    span: Span,
) -> syn::Result<TokenStream2> {
    match ev.to_string().as_str() {
        "click" => Ok(quote! { #ctor.on_click(#handler) }),
        "scroll" => Ok(quote! { #ctor.on_scroll_wheel(#handler) }),
        "modifiers_changed" => Ok(quote! { #ctor.on_modifiers_changed(#handler) }),
        "mouse_down_out" => Ok(quote! { #ctor.on_mouse_down_out(#handler) }),
        "mouse_up_out" => Ok(quote! { #ctor.on_mouse_up_out(::gpui::MouseButton::Left, #handler) }),
        "any_mouse_down" => Ok(quote! { #ctor.on_any_mouse_down(#handler) }),
        // Web-aligned DOM events (normalized vgui event structs).
        "keydown" => Ok(quote! { #ctor.on_key_down(::vgui::__dom_key_down(#handler)) }),
        "keyup" => Ok(quote! { #ctor.on_key_up(::vgui::__dom_key_up(#handler)) }),
        "pointerdown" => Ok(quote! { #ctor.on_any_mouse_down(::vgui::__dom_pointer_down(#handler)) }),
        "pointerup" => Ok(quote! { #ctor.capture_any_mouse_up(::vgui::__dom_pointer_up(#handler)) }),
        "pointermove" => Ok(quote! { #ctor.on_mouse_move(::vgui::__dom_pointer_move(#handler)) }),
        // Window-level resize: register the handler into the render scope and
        // return the element unchanged (not an element event).
        "resize" => Ok(quote! { { ::vgui::__register_resize_handler(#handler); #ctor } }),
        other => Err(syn::Error::new(
            span,
            format!(
                "unsupported event `on:{other}`; supported: click, keydown, keyup, pointerdown, pointerup, pointermove, resize, scroll, modifiers_changed, mouse_down_out, mouse_up_out, any_mouse_down"
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
    ref_attr: Option<&Attr>,
    id: Option<&Attr>,
    tabindex: Option<&Attr>,
    force_id: bool,
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
        && (force_id
            || ref_attr.is_some()
            || active.is_some()
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

    if let Some(ref_attr) = ref_attr {
        let v = attr_tokens(&ref_attr.value);
        ctor = quote! {{
            let __ref = #v;
            let __bound = ::vgui::__bind_ref(&__ref);
            let mut __el = #ctor;
            if __bound {
                __el = __el.track_focus(&__ref.focus_handle());
                __el = __el.track_scroll(&__ref.scroll_handle());
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
            let ::vgui::TwStyle { base, hover, focus, active, animation: _, transition: _ } = __tw;
            let mut __el = #ctor;
            (base)(__el.style());
            if let ::std::option::Option::Some(__h) = hover {
                __el = __el.hover(move |mut s| { __h(&mut s); s });
            }
            if let ::std::option::Option::Some(__f) = focus {
                __el = __el.focus(move |mut s| { __f(&mut s); s });
            }
            if let ::std::option::Option::Some(__a) = active {
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
    let mut ref_attr = None;
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
            AttrKind::Ref => ref_attr = Some(attr),
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
            AttrKind::For => {
                return Err(syn::Error::new(attr.span, "`for` is not valid on <input>"));
            }
            AttrKind::Animate => {
                return Err(syn::Error::new(attr.span, "`animate` is not supported on <input>"));
            }
            AttrKind::Spread => {
                return Err(syn::Error::new(attr.span, "spread attributes are not supported on <input>"));
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

            let id_expr = match id {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(::std::string::ToString::to_string(&(#v))) }
                }
                None => quote! { ::std::option::Option::None },
            };

            Ok(quote! {
                ::vgui::text_input(::vgui::TextInputProps {
                    kind: #kind_variant,
                    multiline: false,
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
                    id: #id_expr,
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
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, ref_attr, id, tabindex, false, &events);
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
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, ref_attr, id, tabindex, false, &events);
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

            let id_expr = match id {
                Some(a) => {
                    let v = attr_tokens(&a.value);
                    quote! { ::std::option::Option::Some(::std::string::ToString::to_string(&(#v))) }
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
                    id: #id_expr,
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
            let ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, ref_attr, id, tabindex, false, &events);
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

            ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, ref_attr, id, tabindex, false, &events);

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

// ── <label> ──────────────────────────────────────────────────────────

fn emit_select(el: &Element) -> syn::Result<TokenStream2> {
    // Reject children — <select> uses options attribute, not child <option> elements.
    if !el.children.is_empty() {
        return Err(syn::Error::new(
            el.tag.span(),
            "<select> cannot have children; use options={...} attribute instead",
        ));
    }

    let mut style = None;
    let mut class = None;

    let mut on_change = None;
    let mut options = None;
    let mut value = None;
    let mut disabled = None;

    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Style => style = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::Id => {}
            AttrKind::On(ev) => {
                let ev_name = ev.to_string();
                let handler = attr_tokens(&attr.value);
                match ev_name.as_str() {
                    "change" => on_change = Some(handler),
                    other => return Err(syn::Error::new(attr.span, format!("unsupported event `on:{other}` on <select>"))),
                }
            }
            AttrKind::Ident(id2) => {
                let name = id2.to_string();
                match name.as_str() {
                    "options" => options = Some(&attr.value),
                    "value" => value = Some(&attr.value),
                    "disabled" => disabled = Some(&attr.value),
                    "name" => {} // accepted but unused
                    other => {
                        return Err(syn::Error::new(
                            id2.span(),
                            format!("unknown attribute `{other}` on <select>"),
                        ));
                    }
                }
            }
            AttrKind::Src => {
                return Err(syn::Error::new(attr.span, "src is not valid on <select>"));
            }
            AttrKind::Type => {
                return Err(syn::Error::new(attr.span, "`type` is not valid on <select>"));
            }
            AttrKind::For => {
                return Err(syn::Error::new(attr.span, "`for` is not valid on <select>"));
            }
            AttrKind::Tabindex => {} // accepted but unused
            AttrKind::Hover | AttrKind::Active | AttrKind::Focus => {
                return Err(syn::Error::new(attr.span, "hover/active/focus are not supported on <select>"));
            }
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <select>; use a wrapping <div ref={...}> instead"));
            }
            AttrKind::Animate => {
                return Err(syn::Error::new(attr.span, "`animate` is not supported on <select>"));
            }
            AttrKind::Spread => {
                return Err(syn::Error::new(attr.span, "spread attributes are not supported on <select>"));
            }
        }
    }

    let options_expr = match options {
        Some(v) => attr_tokens(v),
        None => quote! { ::std::vec::Vec::new() },
    };
    let value_expr = match value {
        Some(v) => {
            let e = attr_tokens(v);
            quote! { ::std::string::ToString::to_string(&(#e)) }
        }
        None => quote! { ::std::string::String::new() },
    };
    let disabled_expr = match disabled {
        Some(v) => bool_expr(v),
        None => quote! { false },
    };
    let on_change_expr = match on_change {
        Some(h) => quote! { ::std::option::Option::Some(::vgui::str_select_change_cb(#h)) },
        None => quote! { ::std::option::Option::None },
    };
    let style_expr = match style {
        Some(a) => {
            let v = attr_tokens(&a.value);
            quote! { ::std::option::Option::Some(#v) }
        }
        None => quote! { ::std::option::Option::None::<::vgui::Css> },
    };
    let class_expr = match class {
        Some(a) => {
            let v = attr_tokens(&a.value);
            quote! { ::std::option::Option::Some(::vgui::tw!(#v)) }
        }
        None => quote! { ::std::option::Option::None::<::vgui::TwStyle> },
    };

    Ok(quote! {{
        let __props = ::vgui::SelectProps {
            options: #options_expr,
            value: #value_expr,
            disabled: #disabled_expr,
            on_change: #on_change_expr,
        };
        let mut __el = ::vgui::select(__props);
        if let ::std::option::Option::Some(__s) = #style_expr {
            __el = __s.apply(__el);
        }
        if let ::std::option::Option::Some(__tw) = #class_expr {
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
        }
        __el
    }})
}

fn emit_textarea(el: &Element) -> syn::Result<TokenStream2> {
    // Reject children — <textarea> is a void element.
    if !el.children.is_empty() {
        return Err(syn::Error::new(
            el.tag.span(),
            "<textarea> is a void element and cannot have children",
        ));
    }

    let mut style = None;
    let mut class = None;
    let mut id = None;
    let mut tabindex = None;
    let mut on_input = None;
    let mut on_change = None;
    let mut value = None;
    let mut placeholder = None;
    let mut disabled = None;
    let mut readonly = None;

    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Style => style = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::Id => id = Some(attr),
            AttrKind::Tabindex => tabindex = Some(attr),
            AttrKind::On(ev) => {
                let ev_name = ev.to_string();
                let handler = attr_tokens(&attr.value);
                match ev_name.as_str() {
                    "input" => on_input = Some(handler),
                    "change" => on_change = Some(handler),
                    _ => return Err(syn::Error::new(attr.span, format!("unsupported event `on:{ev_name}` on <textarea>"))),
                }
            }
            AttrKind::Ident(id2) => {
                let name = id2.to_string();
                match name.as_str() {
                    "value" => value = Some(&attr.value),
                    "placeholder" => placeholder = Some(&attr.value),
                    "disabled" => disabled = Some(&attr.value),
                    "readonly" => readonly = Some(&attr.value),
                    "rows" | "name" => {} // accepted but unused
                    other => {
                        return Err(syn::Error::new(
                            id2.span(),
                            format!("unknown attribute `{other}` on <textarea>"),
                        ));
                    }
                }
            }
            AttrKind::Src => {
                return Err(syn::Error::new(attr.span, "src is not valid on <textarea>"));
            }
            AttrKind::Type => {
                return Err(syn::Error::new(attr.span, "`type` is not valid on <textarea>"));
            }
            AttrKind::For => {
                return Err(syn::Error::new(attr.span, "`for` is not valid on <textarea>"));
            }
            AttrKind::Hover | AttrKind::Active | AttrKind::Focus => {
                return Err(syn::Error::new(attr.span, "hover/active/focus are not supported on <textarea>"));
            }
            AttrKind::Ref => {
                return Err(syn::Error::new(attr.span, "ref is not supported on <textarea>; use a wrapping <div ref={...}> instead"));
            }
            AttrKind::Animate => {
                return Err(syn::Error::new(attr.span, "`animate` is not supported on <textarea>"));
            }
            AttrKind::Spread => {
                return Err(syn::Error::new(attr.span, "spread attributes are not supported on <textarea>"));
            }
        }
    }

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
    let id_expr = match id {
        Some(a) => {
            let v = attr_tokens(&a.value);
            quote! { ::std::option::Option::Some(::std::string::ToString::to_string(&(#v))) }
        }
        None => quote! { ::std::option::Option::None },
    };

    Ok(quote! {
        ::vgui::text_area(::vgui::TextAreaProps {
            value: #value_expr,
            placeholder: #placeholder_expr,
            disabled: #disabled_expr,
            readonly: #readonly_expr,
            on_input: #on_input_expr,
            on_change: #on_change_expr,
            style: #style_expr,
            class: #class_expr,
            id: #id_expr,
            tabindex: #tabindex_expr_val,
        })
    })
}

fn emit_label(el: &Element) -> syn::Result<TokenStream2> {
    let mut for_attr = None;
    let mut id = None;
    let mut style = None;
    let mut hover = None;
    let mut active = None;
    let mut focus = None;
    let mut class = None;
    let mut ref_attr = None;
    let mut tabindex = None;
    let mut events: Vec<(Ident, TokenStream2, Span)> = Vec::new();
    let mut unknown = Vec::new();

    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::For => for_attr = Some(attr),
            AttrKind::Id => id = Some(attr),
            AttrKind::Style => style = Some(attr),
            AttrKind::Hover => hover = Some(attr),
            AttrKind::Active => active = Some(attr),
            AttrKind::Focus => focus = Some(attr),
            AttrKind::Class => class = Some(attr),
            AttrKind::Ref => ref_attr = Some(attr),
            AttrKind::Tabindex => tabindex = Some(attr),
            AttrKind::On(ev) => events.push((ev.clone(), attr_tokens(&attr.value), attr.span)),
            AttrKind::Ident(id2) => unknown.push(id2.clone()),
            AttrKind::Type => {
                return Err(syn::Error::new(attr.span, "`type` is not valid on <label>"))
            }
            AttrKind::Src => {
                return Err(syn::Error::new(attr.span, "src is not valid on <label>"))
            }
            AttrKind::Animate => {
                return Err(syn::Error::new(attr.span, "`animate` is not supported on <label>"))
            }
            AttrKind::Spread => {
                return Err(syn::Error::new(attr.span, "spread attributes are not supported on <label>"));
            }
        }
    }
    if !unknown.is_empty() {
        return Err(syn::Error::new(
            unknown[0].span(),
            format!("unknown attribute `{}` on <label>", unknown[0]),
        ));
    }

    // Label always needs an id because on_mouse_down requires Stateful.
    let mut ctor = quote! { ::gpui::div().cursor_pointer() };
    ctor = chain_div_extras(ctor, el, style, class, hover, active, focus, ref_attr, id, tabindex, true, &events);

    let kids = emit_children(&el.children)?;

    if let Some(for_a) = for_attr {
        // Explicit `for="id"` — look up registry at click time.
        let for_id_expr = match string_lit_static(&for_a.value) {
            Some(lit) => quote! { ::std::string::String::from(#lit) },
            None => {
                let e = attr_tokens(&for_a.value);
                quote! { ::std::string::ToString::to_string(&(#e)) }
            }
        };
        Ok(quote! {{
            let mut el = #ctor;
            #(el = el.child(#kids);)*
            let __for_id = #for_id_expr;
            el = el.on_mouse_down(::gpui::MouseButton::Left, move |_e, window, cx| {
                ::vgui::focus_label_target(&__for_id, window, cx);
            });
            el
        }})
    } else {
        // Wrapping case — collect first LabelTarget from children.
        Ok(quote! {{
            ::vgui::__label_scope_enter();
            let mut el = #ctor;
            #(el = el.child(#kids);)*
            let __target = ::vgui::label_scope_exit();
            if let ::std::option::Option::Some(__t) = __target {
                let __h = __t.focus_handle;
                let __a = __t.click_action;
                el = el.on_mouse_down(::gpui::MouseButton::Left, move |_e, window, cx| {
                    window.focus(&__h, cx);
                    if let ::std::option::Option::Some(__action) = &__a {
                        __action(window, cx);
                    }
                });
            }
            el
        }})
    }
}
// ── <dialog> / <portal> / <floating> ─────────────────────────────────

/// Emit a `<dialog>` modal overlay. Allowed attributes: `open` (bool, default
/// `false`), `on:close` (`Fn(&mut App)` closure, default no-op). The closure is
/// passed verbatim — no `click()` wrapper.
fn emit_dialog(el: &Element) -> syn::Result<TokenStream2> {
    let mut open = None;
    let mut on_close = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id.to_string() == "open" => {
                open = Some(attr_tokens(&attr.value));
            }
            AttrKind::On(ev) if ev.to_string() == "close" => {
                on_close = Some(attr_tokens(&attr.value));
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <dialog>; allowed: `open`, `on:close`",
                ));
            }
        }
    }
    let open = open.unwrap_or(quote! { false });
    let on_close = on_close.unwrap_or(quote! { move |_cx: &mut ::gpui::App| {} });
    let kids = emit_children(&el.children)?;
    Ok(quote! { ::vgui::dialog(#open, #on_close, {
        let mut __p = ::gpui::div();
        #(let __c = #kids; __p = __p.child(__c);)*
        __p
    }) })
}

/// Emit a `<portal>` floating-layer element. Allowed attributes: `priority`
/// (usize, default `0`; higher paints on top).
fn emit_portal(el: &Element) -> syn::Result<TokenStream2> {
    let mut priority = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id.to_string() == "priority" => {
                priority = Some(attr_tokens(&attr.value));
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <portal>; allowed: `priority`",
                ));
            }
        }
    }
    let priority = priority.unwrap_or(quote! { 0usize });
    let kids = emit_children(&el.children)?;
    Ok(quote! { ::vgui::portal({
        let mut __p = ::gpui::div();
        #(let __c = #kids; __p = __p.child(__c);)*
        __p
    }, #priority) })
}

/// Emit a `<floating>` positioned element. Allowed attributes: `position`
/// (`Point<Pixels>`, **required**). Uses window-coordinate placement with
/// overflow avoidance.
fn emit_floating(el: &Element) -> syn::Result<TokenStream2> {
    let mut position = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id.to_string() == "position" => {
                position = Some(attr_tokens(&attr.value));
            }
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <floating>; allowed: `position`",
                ));
            }
        }
    }
    let position = position.ok_or_else(|| {
        syn::Error::new(el.tag.span(), "<floating> requires a `position` attribute")
    })?;
    let kids = emit_children(&el.children)?;
    Ok(quote! { ::vgui::floating(#position, {
        let mut __p = ::gpui::div();
        #(let __c = #kids; __p = __p.child(__c);)*
        __p
    }) })
}

/// Emit a `<radiogroup>` container with roving tabindex. Children are
/// rendered within a radio scope so their `FocusHandle`s are collected for
/// arrow-key navigation. No special attributes.
fn emit_radiogroup(el: &Element) -> syn::Result<TokenStream2> {
    // Reject attributes — <radiogroup> takes no attributes in v1.
    if let Some(attr) = el.attrs.first() {
        return Err(syn::Error::new(
            attr.span,
            "unsupported attribute on <radiogroup>; it takes no attributes",
        ));
    }
    let kids = emit_children(&el.children)?;
    Ok(quote! { {
        let __handles = ::vgui::__radiogroup_scope_enter();
        let mut __content = ::gpui::div();
        #(let __c = #kids; __content = __content.child(__c);)*
        ::vgui::__radiogroup_scope_exit();
        ::vgui::radiogroup(__handles, __content)
    } })
}
