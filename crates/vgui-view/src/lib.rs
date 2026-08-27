extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, ToTokens};

#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    match expand_view(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

enum Node {
    Element(Element),
    Fragment(Vec<Node>),
    Interp(TokenStream2),
    Text(syn::LitStr),
}

struct Element {
    tag: Ident,
    attrs: Vec<Attr>,
    children: Vec<Node>,
    self_closing: bool,
}

struct Attr {
    kind: AttrKind,
    value: AttrValue,
    span: Span,
}

enum AttrKind {
    Ident(Ident),
    On(Ident),
    Style,
    Hover,
    Active,
    Focus,
    Id,
    Src,
    Class,
}

enum AttrValue {
    Expr(TokenStream2),
    Lit(TokenStream2),
}

fn expand_view(input: TokenStream2) -> syn::Result<TokenStream2> {
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == Delimiter::Brace {
                tokens = g.stream().into_iter().collect();
            }
        }
    }
    let mut i = 0;
    let node = parse_node(&tokens, &mut i)?;
    skip_ws(&tokens, &mut i);
    if i < tokens.len() {
        return Err(syn::Error::new(
            tokens[i].span(),
            "unexpected tokens after view root",
        ));
    }
    let expr = emit_node(&node)?;
    Ok(quote! {{ let el = #expr; el }})
}

fn skip_ws(_tokens: &[TokenTree], _i: &mut usize) {}

fn parse_node(tokens: &[TokenTree], i: &mut usize) -> syn::Result<Node> {
    if *i >= tokens.len() {
        return Err(syn::Error::new(Span::call_site(), "expected view node"));
    }
    match &tokens[*i] {
        TokenTree::Punct(p) if p.as_char() == '<' => parse_element_or_fragment(tokens, i),
        TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
            let stream = g.stream();
            *i += 1;
            Ok(Node::Interp(stream))
        }
        TokenTree::Literal(lit) => {
            let lit_ts = TokenStream2::from(TokenTree::Literal(lit.clone()));
            *i += 1;
            if let Ok(s) = syn::parse2::<syn::LitStr>(lit_ts.clone()) {
                Ok(Node::Text(s))
            } else {
                Ok(Node::Interp(lit_ts))
            }
        }
        other => Err(syn::Error::new(
            other.span(),
            "expected element, fragment, or `{expr}`",
        )),
    }
}

fn parse_element_or_fragment(tokens: &[TokenTree], i: &mut usize) -> syn::Result<Node> {
    let span = tokens[*i].span();
    *i += 1; // <
    if *i >= tokens.len() {
        return Err(syn::Error::new(span, "expected tag after `<`"));
    }
    if is_punct(&tokens[*i], '>') {
        *i += 1;
        let mut children = Vec::new();
        loop {
            if *i >= tokens.len() {
                return Err(syn::Error::new(span, "unclosed fragment"));
            }
            if is_punct(&tokens[*i], '<') && *i + 1 < tokens.len() && is_punct(&tokens[*i + 1], '/')
            {
                *i += 2;
                if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                    return Err(syn::Error::new(span, "expected `</>`"));
                }
                *i += 1;
                break;
            }
            children.push(parse_node(tokens, i)?);
        }
        return Ok(Node::Fragment(children));
    }
    let tag = match &tokens[*i] {
        TokenTree::Ident(id) => id.clone(),
        other => return Err(syn::Error::new(other.span(), "expected tag name")),
    };
    *i += 1;
    let mut attrs = Vec::new();
    loop {
        if *i >= tokens.len() {
            return Err(syn::Error::new(tag.span(), "unclosed tag"));
        }
        if is_punct(&tokens[*i], '/') {
            *i += 1;
            if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                return Err(syn::Error::new(tag.span(), "expected `/>`"));
            }
            *i += 1;
            return Ok(Node::Element(Element {
                tag,
                attrs,
                children: Vec::new(),
                self_closing: true,
            }));
        }
        if is_punct(&tokens[*i], '>') {
            *i += 1;
            break;
        }
        attrs.push(parse_attr(tokens, i)?);
    }
    let mut children = Vec::new();
    loop {
        if *i >= tokens.len() {
            return Err(syn::Error::new(tag.span(), "unclosed tag"));
        }
        if is_punct(&tokens[*i], '<') && *i + 1 < tokens.len() && is_punct(&tokens[*i + 1], '/') {
            *i += 2;
            if *i >= tokens.len() {
                return Err(syn::Error::new(tag.span(), "expected closing tag"));
            }
            let close = match &tokens[*i] {
                TokenTree::Ident(id) => id.clone(),
                other => return Err(syn::Error::new(other.span(), "expected closing tag name")),
            };
            *i += 1;
            if *i >= tokens.len() || !is_punct(&tokens[*i], '>') {
                return Err(syn::Error::new(
                    close.span(),
                    "expected `>` after closing tag",
                ));
            }
            *i += 1;
            if close.to_string() != tag.to_string() {
                return Err(syn::Error::new(close.span(), "mismatched closing tag"));
            }
            break;
        }
        children.push(parse_node(tokens, i)?);
    }
    let _ = span;
    Ok(Node::Element(Element {
        tag,
        attrs,
        children,
        self_closing: false,
    }))
}

fn is_punct(tt: &TokenTree, ch: char) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ch)
}

fn parse_attr(tokens: &[TokenTree], i: &mut usize) -> syn::Result<Attr> {
    let span = tokens[*i].span();
    let kind = match &tokens[*i] {
        TokenTree::Ident(id) => {
            let name = id.to_string();
            *i += 1;
            match name.as_str() {
                "style" => AttrKind::Style,
                "hover" => AttrKind::Hover,
                "active" => AttrKind::Active,
                "focus" => AttrKind::Focus,
                "id" => AttrKind::Id,
                "src" => AttrKind::Src,
                "class" => AttrKind::Class,
                "on" => {
                    if *i >= tokens.len() || !is_punct(&tokens[*i], ':') {
                        return Err(syn::Error::new(span, "expected `on:event`"));
                    }
                    *i += 1;
                    match tokens.get(*i) {
                        Some(TokenTree::Ident(ev)) => {
                            let ev = ev.clone();
                            *i += 1;
                            AttrKind::On(ev)
                        }
                        _ => return Err(syn::Error::new(span, "expected event name after `on:`")),
                    }
                }
                _ => AttrKind::Ident(id.clone()),
            }
        }
        other => return Err(syn::Error::new(other.span(), "expected attribute name")),
    };
    if *i >= tokens.len() || !is_punct(&tokens[*i], '=') {
        return Err(syn::Error::new(span, "expected `=` after attribute"));
    }
    *i += 1;
    if *i >= tokens.len() {
        return Err(syn::Error::new(span, "expected attribute value"));
    }
    let value = match &tokens[*i] {
        TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
            let stream = g.stream();
            *i += 1;
            AttrValue::Expr(stream)
        }
        TokenTree::Literal(lit) => {
            let ts = TokenStream2::from(TokenTree::Literal(lit.clone()));
            *i += 1;
            AttrValue::Lit(ts)
        }
        TokenTree::Ident(id) => {
            let name = id.to_string();
            *i += 1;
            if name == "true" || name == "false" {
                AttrValue::Lit(Ident::new(&name, id.span()).to_token_stream())
            } else {
                AttrValue::Expr(id.to_token_stream())
            }
        }
        other => return Err(syn::Error::new(other.span(), "expected attribute value")),
    };
    Ok(Attr { kind, value, span })
}

fn emit_node(node: &Node) -> syn::Result<TokenStream2> {
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

fn emit_children(children: &[Node]) -> syn::Result<Vec<TokenStream2>> {
    children.iter().map(emit_child).collect()
}

fn emit_child(node: &Node) -> syn::Result<TokenStream2> {
    match node {
        Node::Interp(expr) => Ok(quote! { ::vgui::into_child(#expr) }),
        Node::Text(s) => Ok(quote! { ::vgui::into_child(#s) }),
        other => emit_node(other),
    }
}

fn wrap_children_element(children: &[Node]) -> syn::Result<TokenStream2> {
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

fn attr_tokens(value: &AttrValue) -> TokenStream2 {
    match value {
        AttrValue::Expr(e) | AttrValue::Lit(e) => e.clone(),
    }
}

fn string_lit_static(value: &AttrValue) -> Option<TokenStream2> {
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

fn emit_element(el: &Element) -> syn::Result<TokenStream2> {
    let name = el.tag.to_string();
    if name == "Show" {
        return emit_show(el);
    }
    if name == "For" {
        return emit_for(el);
    }
    if name
        .chars()
        .next()
        .map(|c| c.is_uppercase())
        .unwrap_or(false)
    {
        return emit_component(el);
    }
    emit_builtin(el)
}

fn emit_show(el: &Element) -> syn::Result<TokenStream2> {
    let mut when = None;
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "when" => when = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "fallback" => fallback = Some(attr_tokens(&attr.value)),
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

fn emit_for(el: &Element) -> syn::Result<TokenStream2> {
    let mut each = None;
    let mut fallback = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "each" => each = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "fallback" => fallback = Some(attr_tokens(&attr.value)),
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

fn looks_like_closure(expr: &TokenStream2) -> bool {
    let s = expr.to_string();
    s.contains('|') || s.contains("move")
}

fn emit_component(el: &Element) -> syn::Result<TokenStream2> {
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
    for attr in &el.attrs {
        let value = attr_tokens(&attr.value);
        match &attr.kind {
            AttrKind::Ident(id) => fields.push(quote! { #id: #value }),
            AttrKind::Id => fields.push(quote! { id: #value }),
            AttrKind::Src => fields.push(quote! { src: #value }),
            AttrKind::Style => fields.push(quote! { style: #value }),
            AttrKind::Hover => fields.push(quote! { hover: #value }),
            AttrKind::Active => fields.push(quote! { active: #value }),
            AttrKind::Focus => fields.push(quote! { focus: #value }),
            AttrKind::Class => fields.push(quote! { class: #value }),
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
    Ok(quote! { #tag { #(#fields),* } })
}

fn emit_builtin(el: &Element) -> syn::Result<TokenStream2> {
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
