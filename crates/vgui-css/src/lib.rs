extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    match expand_css(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

struct Decl {
    name: String,
    span: Span,
    value: Vec<TokenTree>,
}

fn expand_css(input: TokenStream2) -> syn::Result<TokenStream2> {
    let decls = parse_decls(input)?;
    if decls.is_empty() {
        return Ok(quote! { ::vgui::Css::new(|_| {}) });
    }
    let mut stmts = Vec::new();
    for decl in decls {
        if decl.name.starts_with('&') {
            return Err(syn::Error::new(
                decl.span,
                "pseudo selectors belong on the element: hover={css! { ... }}",
            ));
        }
        stmts.push(emit_decl(&decl)?);
    }
    Ok(quote! {
        {
            #[allow(unused_imports)]
            use ::vgui as _;
            ::vgui::Css::new(|s| {
                #(#stmts)*
            })
        }
    })
}

fn parse_decls(input: TokenStream2) -> syn::Result<Vec<Decl>> {
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();
    if tokens.len() == 1 {
        if let TokenTree::Group(group) = &tokens[0] {
            if group.delimiter() == Delimiter::Brace {
                tokens = group.stream().into_iter().collect();
            }
        }
    }
    let mut decls = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        skip_separators(&tokens, &mut i);
        if i >= tokens.len() {
            break;
        }
        if is_pseudo(&tokens, i) {
            let span = tokens[i].span();
            let name = parse_pseudo_name(&tokens, &mut i)?;
            skip_separators(&tokens, &mut i);
            let value = if i < tokens.len() {
                if let TokenTree::Group(g) = &tokens[i] {
                    if g.delimiter() == Delimiter::Brace {
                        let v = vec![tokens[i].clone()];
                        i += 1;
                        v
                    } else {
                        return Err(syn::Error::new(span, "expected `{` after pseudo selector"));
                    }
                } else {
                    return Err(syn::Error::new(span, "expected `{` after pseudo selector"));
                }
            } else {
                return Err(syn::Error::new(span, "expected `{` after pseudo selector"));
            };
            skip_optional_semi(&tokens, &mut i);
            decls.push(Decl { name, span, value });
            continue;
        }
        let (name, span) = parse_property(&tokens, &mut i)?;
        skip_separators(&tokens, &mut i);
        if i >= tokens.len() || !is_colon(&tokens[i]) {
            return Err(syn::Error::new(span, "expected `:` after CSS property"));
        }
        i += 1;
        let value = take_until_semi_or_end(&tokens, &mut i);
        skip_optional_semi(&tokens, &mut i);
        decls.push(Decl { name, span, value });
    }
    Ok(decls)
}

fn skip_separators(tokens: &[TokenTree], i: &mut usize) {
    while *i < tokens.len() {
        match &tokens[*i] {
            TokenTree::Punct(p) if p.as_char() == ';' => *i += 1,
            _ => break,
        }
    }
}

fn skip_optional_semi(tokens: &[TokenTree], i: &mut usize) {
    if *i < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[*i] {
            if p.as_char() == ';' {
                *i += 1;
            }
        }
    }
}

fn is_colon(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ':')
}

fn is_pseudo(tokens: &[TokenTree], i: usize) -> bool {
    matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == '&')
}

fn parse_pseudo_name(tokens: &[TokenTree], i: &mut usize) -> syn::Result<String> {
    let span = tokens[*i].span();
    *i += 1; // &
    if *i < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[*i] {
            if p.as_char() == ':' {
                *i += 1;
            }
        }
    }
    if *i >= tokens.len() {
        return Err(syn::Error::new(span, "expected pseudo selector name"));
    }
    match &tokens[*i] {
        TokenTree::Ident(ident) => {
            let name = format!("&:{}", ident);
            *i += 1;
            Ok(name)
        }
        _ => Err(syn::Error::new(span, "expected pseudo selector name")),
    }
}

fn parse_property(tokens: &[TokenTree], i: &mut usize) -> syn::Result<(String, Span)> {
    let mut parts = Vec::new();
    let span = tokens[*i].span();
    loop {
        if *i >= tokens.len() {
            break;
        }
        match &tokens[*i] {
            TokenTree::Ident(ident) => {
                parts.push(ident.to_string());
                *i += 1;
            }
            TokenTree::Punct(p) if p.as_char() == '-' => {
                *i += 1;
            }
            _ => break,
        }
    }
    if parts.is_empty() {
        return Err(syn::Error::new(span, "expected CSS property name"));
    }
    Ok((parts.join("-"), span))
}

fn take_until_semi_or_end(tokens: &[TokenTree], i: &mut usize) -> Vec<TokenTree> {
    let mut value = Vec::new();
    while *i < tokens.len() {
        match &tokens[*i] {
            TokenTree::Punct(p) if p.as_char() == ';' => break,
            tt => {
                value.push(tt.clone());
                *i += 1;
            }
        }
    }
    value
}

fn tokens_display(tokens: &[TokenTree]) -> String {
    TokenStream2::from_iter(tokens.iter().cloned())
        .to_string()
        .replace('\n', " ")
}

fn unsupported(prop: &str, tokens: &[TokenTree], span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "unsupported CSS value for '{}': {}",
            prop,
            tokens_display(tokens)
        ),
    )
}

fn unknown_prop(name: &str, span: Span) -> syn::Error {
    syn::Error::new(span, format!("unknown CSS property '{name}'"))
}

fn keyword(tokens: &[TokenTree]) -> Option<String> {
    if tokens.len() == 1 {
        if let TokenTree::Ident(id) = &tokens[0] {
            return Some(id.to_string());
        }
    }
    None
}

fn is_interp(tokens: &[TokenTree]) -> Option<TokenStream2> {
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == Delimiter::Brace {
                return Some(g.stream());
            }
        }
    }
    None
}

fn parse_number(tt: &TokenTree) -> Option<f32> {
    if let TokenTree::Literal(lit) = tt {
        let s = lit.to_string().replace('_', "");
        if s.ends_with("f32") || s.ends_with("f64") {
            return s
                .trim_end_matches("f32")
                .trim_end_matches("f64")
                .parse()
                .ok();
        }
        if s.contains('.') {
            return s.parse().ok();
        }
        if let Ok(n) = s.parse::<i64>() {
            return Some(n as f32);
        }
        s.parse().ok()
    } else {
        None
    }
}

fn parse_suffixed_length(tt: &TokenTree) -> Option<LengthVal> {
    let TokenTree::Literal(lit) = tt else {
        return None;
    };
    let s = lit.to_string().replace('_', "");
    let span = tt.span();
    if let Some(num) = s.strip_suffix("px") {
        let n: f32 = num.parse().ok()?;
        return Some(LengthVal {
            kind: LengthKind::Px,
            n,
            span,
        });
    }
    if let Some(num) = s.strip_suffix("rem") {
        let n: f32 = num.parse().ok()?;
        return Some(LengthVal {
            kind: LengthKind::Rem,
            n,
            span,
        });
    }
    None
}

fn parse_int(tt: &TokenTree) -> Option<i64> {
    if let TokenTree::Literal(lit) = tt {
        let s = lit.to_string().replace('_', "");
        s.parse().ok()
    } else {
        None
    }
}

enum LengthKind {
    Px,
    Rem,
    Percent,
    Auto,
    Interp(TokenStream2),
}

struct LengthVal {
    kind: LengthKind,
    n: f32,
    span: Span,
}

fn split_values(tokens: &[TokenTree]) -> Vec<Vec<TokenTree>> {
    let mut out = Vec::new();
    let mut cur = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                i += 1;
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Literal(_) => {
                if parse_suffixed_length(&tokens[i]).is_some() {
                    if !cur.is_empty() {
                        out.push(std::mem::take(&mut cur));
                    }
                    cur.push(tokens[i].clone());
                    out.push(std::mem::take(&mut cur));
                    i += 1;
                    continue;
                }
                if i + 1 < tokens.len() {
                    if let TokenTree::Ident(id) = &tokens[i + 1] {
                        let name = id.to_string();
                        if name == "px" || name == "rem" || name == "rems" {
                            if !cur.is_empty() {
                                out.push(std::mem::take(&mut cur));
                            }
                            cur.push(tokens[i].clone());
                            cur.push(tokens[i + 1].clone());
                            out.push(std::mem::take(&mut cur));
                            i += 2;
                            continue;
                        }
                    }
                    if let TokenTree::Punct(p) = &tokens[i + 1] {
                        if p.as_char() == '%' {
                            if !cur.is_empty() {
                                out.push(std::mem::take(&mut cur));
                            }
                            cur.push(tokens[i].clone());
                            cur.push(tokens[i + 1].clone());
                            out.push(std::mem::take(&mut cur));
                            i += 2;
                            continue;
                        }
                    }
                }
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Ident(_) => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                out.push(std::mem::take(&mut cur));
                i += 1;
            }
            TokenTree::Punct(p) if p.as_char() == '#' => {
                if !cur.is_empty() {
                    out.push(std::mem::take(&mut cur));
                }
                cur.push(tokens[i].clone());
                if i + 1 < tokens.len() {
                    cur.push(tokens[i + 1].clone());
                    i += 2;
                } else {
                    i += 1;
                }
                out.push(std::mem::take(&mut cur));
            }
            _ => {
                cur.push(tokens[i].clone());
                i += 1;
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

fn parse_length(tokens: &[TokenTree]) -> Option<LengthVal> {
    if tokens.is_empty() {
        return None;
    }
    if let Some(expr) = is_interp(tokens) {
        return Some(LengthVal {
            kind: LengthKind::Interp(expr),
            n: 0.0,
            span: tokens[0].span(),
        });
    }
    if tokens.len() == 1 {
        if let TokenTree::Ident(id) = &tokens[0] {
            if id.to_string() == "auto" {
                return Some(LengthVal {
                    kind: LengthKind::Auto,
                    n: 0.0,
                    span: id.span(),
                });
            }
        }
        if let Some(len) = parse_suffixed_length(&tokens[0]) {
            return Some(len);
        }
        if let Some(n) = parse_number(&tokens[0]) {
            return Some(LengthVal {
                kind: LengthKind::Px,
                n,
                span: tokens[0].span(),
            });
        }
    }
    if tokens.len() == 2 {
        if let Some(n) = parse_number(&tokens[0]) {
            match &tokens[1] {
                TokenTree::Ident(id) => {
                    let name = id.to_string();
                    if name == "px" {
                        return Some(LengthVal {
                            kind: LengthKind::Px,
                            n,
                            span: tokens[0].span(),
                        });
                    }
                    if name == "rem" || name == "rems" {
                        return Some(LengthVal {
                            kind: LengthKind::Rem,
                            n,
                            span: tokens[0].span(),
                        });
                    }
                }
                TokenTree::Punct(p) if p.as_char() == '%' => {
                    return Some(LengthVal {
                        kind: LengthKind::Percent,
                        n,
                        span: tokens[0].span(),
                    });
                }
                _ => {}
            }
        }
    }
    None
}

fn emit_length(len: &LengthVal) -> TokenStream2 {
    match &len.kind {
        LengthKind::Px => {
            let n = len.n;
            quote! { ::gpui::px(#n as f32) }
        }
        LengthKind::Rem => {
            let n = len.n;
            quote! { ::gpui::rems(#n as f32) }
        }
        LengthKind::Percent => {
            let frac = len.n / 100.0;
            quote! { ::gpui::relative(#frac as f32) }
        }
        LengthKind::Auto => quote! { ::gpui::Length::Auto },
        LengthKind::Interp(expr) => expr.clone(),
    }
}

fn emit_as_length(len: &LengthVal, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Ok(quote! { ::gpui::Length::Auto }),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::Length>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote_spanned! {span=> ::core::convert::Into::<::gpui::Length>::into(#inner) })
        }
    }
}

fn emit_as_definite(len: &LengthVal, prop: &str, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Err(syn::Error::new(span, format!("{prop} cannot be auto"))),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#inner) })
        }
    }
}

fn emit_as_absolute(len: &LengthVal, prop: &str, span: Span) -> syn::Result<TokenStream2> {
    match &len.kind {
        LengthKind::Auto => Err(syn::Error::new(span, format!("{prop} cannot be auto"))),
        LengthKind::Percent => Err(syn::Error::new(
            span,
            format!("{prop} cannot be a percentage"),
        )),
        LengthKind::Interp(expr) => {
            Ok(quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#expr) })
        }
        _ => {
            let inner = emit_length(len);
            Ok(quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#inner) })
        }
    }
}

fn parse_hex_digits(raw: &str) -> Result<(u32, bool), ()> {
    let hex: String = raw.chars().filter(|c| *c != '_').collect();
    let hex = hex.strip_prefix('#').unwrap_or(&hex);
    match hex.len() {
        3 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            u32::from_str_radix(&out, 16)
                .map(|v| (v, false))
                .map_err(|_| ())
        }
        4 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            u32::from_str_radix(&out, 16)
                .map(|v| (v, true))
                .map_err(|_| ())
        }
        6 => u32::from_str_radix(hex, 16)
            .map(|v| (v, false))
            .map_err(|_| ()),
        8 => u32::from_str_radix(hex, 16)
            .map(|v| (v, true))
            .map_err(|_| ()),
        _ => Err(()),
    }
}

fn emit_hex(raw: &str, span: Span) -> syn::Result<TokenStream2> {
    match parse_hex_digits(raw) {
        Ok((v, true)) => Ok(quote_spanned! {span=> ::gpui::rgba(#v) }),
        Ok((v, false)) => Ok(quote_spanned! {span=> ::gpui::rgb(#v) }),
        Err(()) => Err(syn::Error::new(span, "invalid hex color")),
    }
}

fn named_color(name: &str) -> Option<TokenStream2> {
    match name {
        "black" => Some(quote! { ::gpui::black() }),
        "white" => Some(quote! { ::gpui::white() }),
        "red" => Some(quote! { ::gpui::red() }),
        "green" => Some(quote! { ::gpui::green() }),
        "blue" => Some(quote! { ::gpui::blue() }),
        "yellow" => Some(quote! { ::gpui::yellow() }),
        "cyan" => Some(quote! { ::gpui::rgb(0x00ffff) }),
        "magenta" => Some(quote! { ::gpui::rgb(0xff00ff) }),
        "orange" => Some(quote! { ::gpui::rgb(0xffa500) }),
        "purple" => Some(quote! { ::gpui::rgb(0x800080) }),
        "gray" | "grey" => Some(quote! { ::gpui::rgb(0x808080) }),
        _ => None,
    }
}

fn emit_color(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(expr) = is_interp(tokens) {
        return Ok(expr);
    }
    if tokens.len() == 1 {
        match &tokens[0] {
            TokenTree::Ident(id) => {
                let name = id.to_string();
                if let Some(c) = named_color(&name) {
                    return Ok(c);
                }
            }
            TokenTree::Literal(lit) => {
                let s = lit.to_string();
                if s.starts_with('"') && s.ends_with('"') {
                    let inner = &s[1..s.len() - 1];
                    if inner.starts_with('#') {
                        return emit_hex(inner, span);
                    }
                }
            }
            _ => {}
        }
    }
    if !tokens.is_empty() {
        if let TokenTree::Punct(p) = &tokens[0] {
            if p.as_char() == '#' && tokens.len() >= 2 {
                let raw = match &tokens[1] {
                    TokenTree::Ident(id) => id.to_string(),
                    TokenTree::Literal(lit) => lit.to_string().replace('_', ""),
                    _ => return Err(syn::Error::new(span, "invalid hex color")),
                };
                return emit_hex(&raw, span);
            }
        }
    }
    if let TokenTree::Ident(id) = &tokens[0] {
        let name = id.to_string();
        if name == "rgb" || name == "rgba" {
            return emit_rgb_fn(tokens, span);
        }
    }
    Err(syn::Error::new(span, "invalid hex color").combine_or(unsupported("color", tokens, span)))
}

trait CombineErr {
    fn combine_or(self, other: syn::Error) -> syn::Error;
}

impl CombineErr for syn::Error {
    fn combine_or(self, other: syn::Error) -> syn::Error {
        let _ = other;
        self
    }
}

fn emit_rgb_fn(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if tokens.len() != 2 {
        return Err(unsupported("color", tokens, span));
    }
    let TokenTree::Ident(id) = &tokens[0] else {
        return Err(unsupported("color", tokens, span));
    };
    let TokenTree::Group(g) = &tokens[1] else {
        return Err(unsupported("color", tokens, span));
    };
    if g.delimiter() != Delimiter::Parenthesis {
        return Err(unsupported("color", tokens, span));
    }
    let args: Vec<TokenTree> = g.stream().into_iter().collect();
    let mut nums = Vec::new();
    let mut cur = Vec::new();
    for tt in args {
        match &tt {
            TokenTree::Punct(p) if p.as_char() == ',' => {
                if !cur.is_empty() {
                    nums.push(std::mem::take(&mut cur));
                }
            }
            _ => cur.push(tt),
        }
    }
    if !cur.is_empty() {
        nums.push(cur);
    }
    let name = id.to_string();
    if name == "rgb" {
        if nums.len() != 3 {
            return Err(unsupported("color", tokens, span));
        }
        let r = parse_int(&nums[0][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let g = parse_int(&nums[1][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let b = parse_int(&nums[2][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let packed = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
        Ok(quote! { ::gpui::rgb(#packed) })
    } else {
        if nums.len() != 4 {
            return Err(unsupported("color", tokens, span));
        }
        let r = parse_int(&nums[0][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let g = parse_int(&nums[1][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let b = parse_int(&nums[2][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let a = parse_number(&nums[3][0]).ok_or_else(|| unsupported("color", tokens, span))?;
        let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
        let packed = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | alpha;
        Ok(quote! { ::gpui::rgba(#packed) })
    }
}

fn expand_box_edges(values: &[LengthVal]) -> syn::Result<[LengthVal; 4]> {
    match values.len() {
        1 => Ok([
            clone_len(&values[0]),
            clone_len(&values[0]),
            clone_len(&values[0]),
            clone_len(&values[0]),
        ]),
        2 => Ok([
            clone_len(&values[0]),
            clone_len(&values[1]),
            clone_len(&values[0]),
            clone_len(&values[1]),
        ]),
        4 => Ok([
            clone_len(&values[0]),
            clone_len(&values[1]),
            clone_len(&values[2]),
            clone_len(&values[3]),
        ]),
        n => Err(syn::Error::new(
            values
                .get(0)
                .map(|v| v.span)
                .unwrap_or_else(Span::call_site),
            format!("expected 1, 2, or 4 values, got {n}"),
        )),
    }
}

fn clone_len(len: &LengthVal) -> LengthVal {
    LengthVal {
        kind: match &len.kind {
            LengthKind::Px => LengthKind::Px,
            LengthKind::Rem => LengthKind::Rem,
            LengthKind::Percent => LengthKind::Percent,
            LengthKind::Auto => LengthKind::Auto,
            LengthKind::Interp(e) => LengthKind::Interp(e.clone()),
        },
        n: len.n,
        span: len.span,
    }
}

fn parse_lengths(tokens: &[TokenTree], prop: &str, span: Span) -> syn::Result<Vec<LengthVal>> {
    if let Some(len) = parse_length(tokens) {
        return Ok(vec![len]);
    }
    let parts = split_values(tokens);
    let mut out = Vec::new();
    for part in parts {
        let len = parse_length(&part).ok_or_else(|| unsupported(prop, tokens, span))?;
        out.push(len);
    }
    Ok(out)
}

fn emit_overflow(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "hidden" => Ok(quote_spanned! {span=> ::gpui::Overflow::Hidden }),
        "scroll" => Ok(quote_spanned! {span=> ::gpui::Overflow::Scroll }),
        "visible" => Ok(quote_spanned! {span=> ::gpui::Overflow::Visible }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'overflow': {other}"),
        )),
    }
}

fn emit_flex_direction(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "row" => Ok(quote! { ::gpui::FlexDirection::Row }),
        "column" => Ok(quote! { ::gpui::FlexDirection::Column }),
        "row-reverse" => Ok(quote! { ::gpui::FlexDirection::RowReverse }),
        "column-reverse" => Ok(quote! { ::gpui::FlexDirection::ColumnReverse }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'flex-direction': {other}"),
        )),
    }
}

fn emit_justify(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::JustifyContent::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::JustifyContent::FlexEnd }),
        "center" => Ok(quote! { ::gpui::JustifyContent::Center }),
        "space-between" => Ok(quote! { ::gpui::JustifyContent::SpaceBetween }),
        "space-around" => Ok(quote! { ::gpui::JustifyContent::SpaceAround }),
        "space-evenly" => Ok(quote! { ::gpui::JustifyContent::SpaceEvenly }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'justify-content': {other}"),
        )),
    }
}

fn emit_align_items(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::AlignItems::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::AlignItems::FlexEnd }),
        "center" => Ok(quote! { ::gpui::AlignItems::Center }),
        "baseline" => Ok(quote! { ::gpui::AlignItems::Baseline }),
        "stretch" => Ok(quote! { ::gpui::AlignItems::Stretch }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'align-items': {other}"),
        )),
    }
}

fn emit_align_content(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "flex-start" | "start" => Ok(quote! { ::gpui::AlignContent::FlexStart }),
        "flex-end" | "end" => Ok(quote! { ::gpui::AlignContent::FlexEnd }),
        "center" => Ok(quote! { ::gpui::AlignContent::Center }),
        "space-between" => Ok(quote! { ::gpui::AlignContent::SpaceBetween }),
        "space-around" => Ok(quote! { ::gpui::AlignContent::SpaceAround }),
        "stretch" => Ok(quote! { ::gpui::AlignContent::Stretch }),
        "space-evenly" => Ok(quote! { ::gpui::AlignContent::SpaceEvenly }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'align-content': {other}"),
        )),
    }
}

fn emit_font_weight(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(expr) = is_interp(tokens) {
        return Ok(expr);
    }
    if let Some(kw) = keyword(tokens) {
        return match kw.as_str() {
            "thin" => Ok(quote! { ::gpui::FontWeight::THIN }),
            "extra-light" => Ok(quote! { ::gpui::FontWeight::EXTRA_LIGHT }),
            "light" => Ok(quote! { ::gpui::FontWeight::LIGHT }),
            "normal" => Ok(quote! { ::gpui::FontWeight::NORMAL }),
            "medium" => Ok(quote! { ::gpui::FontWeight::MEDIUM }),
            "semibold" => Ok(quote! { ::gpui::FontWeight::SEMIBOLD }),
            "bold" => Ok(quote! { ::gpui::FontWeight::BOLD }),
            "extrabold" | "extra-bold" => Ok(quote! { ::gpui::FontWeight::EXTRA_BOLD }),
            "black" => Ok(quote! { ::gpui::FontWeight::BLACK }),
            other => Err(syn::Error::new(
                span,
                format!("unsupported CSS value for 'font-weight': {other}"),
            )),
        };
    }
    if tokens.len() == 1 {
        if let Some(n) = parse_int(&tokens[0]) {
            return match n {
                100 => Ok(quote! { ::gpui::FontWeight::THIN }),
                200 => Ok(quote! { ::gpui::FontWeight::EXTRA_LIGHT }),
                300 => Ok(quote! { ::gpui::FontWeight::LIGHT }),
                400 => Ok(quote! { ::gpui::FontWeight::NORMAL }),
                500 => Ok(quote! { ::gpui::FontWeight::MEDIUM }),
                600 => Ok(quote! { ::gpui::FontWeight::SEMIBOLD }),
                700 => Ok(quote! { ::gpui::FontWeight::BOLD }),
                800 => Ok(quote! { ::gpui::FontWeight::EXTRA_BOLD }),
                900 => Ok(quote! { ::gpui::FontWeight::BLACK }),
                _ => Err(unsupported("font-weight", tokens, span)),
            };
        }
    }
    Err(unsupported("font-weight", tokens, span))
}

fn emit_cursor(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "pointer" => Ok(quote! { ::gpui::CursorStyle::PointingHand }),
        "default" => Ok(quote! { ::gpui::CursorStyle::Arrow }),
        "text" => Ok(quote! { ::gpui::CursorStyle::IBeam }),
        "crosshair" => Ok(quote! { ::gpui::CursorStyle::Crosshair }),
        "not-allowed" => Ok(quote! { ::gpui::CursorStyle::OperationNotAllowed }),
        "grab" => Ok(quote! { ::gpui::CursorStyle::OpenHand }),
        "grabbing" => Ok(quote! { ::gpui::CursorStyle::ClosedHand }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'cursor': {other}"),
        )),
    }
}

fn emit_shadow(kw: &str, span: Span) -> syn::Result<TokenStream2> {
    match kw {
        "none" => Ok(quote! { ::std::vec::Vec::new() }),
        "sm" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(3.),
                    spread_radius: ::gpui::px(0.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(2.),
                    spread_radius: ::gpui::px(-1.),
                }
            ]
        }),
        "md" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-1.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(2.)),
                    blur_radius: ::gpui::px(4.),
                    spread_radius: ::gpui::px(-2.),
                }
            ]
        }),
        "lg" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(10.)),
                    blur_radius: ::gpui::px(15.),
                    spread_radius: ::gpui::px(-3.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-4.),
                }
            ]
        }),
        "xl" => Ok(quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(20.)),
                    blur_radius: ::gpui::px(25.),
                    spread_radius: ::gpui::px(-5.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(8.)),
                    blur_radius: ::gpui::px(10.),
                    spread_radius: ::gpui::px(-6.),
                }
            ]
        }),
        other => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for 'box-shadow': {other}"),
        )),
    }
}

fn hyphen_keyword(tokens: &[TokenTree]) -> Option<String> {
    let mut parts = Vec::new();
    for tt in tokens {
        match tt {
            TokenTree::Ident(id) => parts.push(id.to_string()),
            TokenTree::Punct(p) if p.as_char() == '-' => {}
            _ => return None,
        }
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("-"))
    }
}

fn emit_border(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let parts = split_values(tokens);
    let mut width = None;
    let mut style = None;
    let mut color = None;
    for part in parts {
        if let Some(len) = parse_length(&part) {
            width = Some(len);
            continue;
        }
        if let Some(kw) = hyphen_keyword(&part) {
            match kw.as_str() {
                "solid" => {
                    style = Some(quote! { ::gpui::BorderStyle::Solid });
                    continue;
                }
                "dashed" => {
                    style = Some(quote! { ::gpui::BorderStyle::Dashed });
                    continue;
                }
                _ => {}
            }
        }
        if emit_color(&part, span).is_ok() {
            color = Some(emit_color(&part, span)?);
            continue;
        }
        return Err(unsupported("border", tokens, span));
    }
    let width = width.ok_or_else(|| unsupported("border", tokens, span))?;
    let w = emit_as_absolute(&width, "border", span)?;
    let style = style.unwrap_or(quote! { ::gpui::BorderStyle::Solid });
    let mut out = quote! {
        s.border_widths.top = Some(#w);
        s.border_widths.right = Some(#w);
        s.border_widths.bottom = Some(#w);
        s.border_widths.left = Some(#w);
        s.border_style = Some(#style);
    };
    if let Some(c) = color {
        out.extend(quote! { s.border_color = Some((#c).into()); });
    }
    Ok(out)
}

fn emit_decl(decl: &Decl) -> syn::Result<TokenStream2> {
    let prop = decl.name.as_str();
    let tokens = &decl.value;
    let span = decl.span;
    if let Some(expr) = is_interp(tokens) {
        return emit_interp_prop(prop, expr, span);
    }
    match prop {
        "display" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "flex" => quote! { ::gpui::Display::Flex },
                "block" => quote! { ::gpui::Display::Block },
                "none" => quote! { ::gpui::Display::None },
                "grid" => quote! { ::gpui::Display::Grid },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'display': {other}"),
                    ))
                }
            };
            Ok(quote! { s.display = Some(#v); })
        }
        "visibility" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "hidden" => quote! { ::gpui::Visibility::Hidden },
                "visible" => quote! { ::gpui::Visibility::Visible },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'visibility': {other}"),
                    ))
                }
            };
            Ok(quote! { s.visibility = Some(#v); })
        }
        "overflow" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(quote! { s.overflow.x = Some(#v); s.overflow.y = Some(#v); })
        }
        "overflow-x" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(quote! { s.overflow.x = Some(#v); })
        }
        "overflow-y" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_overflow(&kw, span)?;
            Ok(quote! { s.overflow.y = Some(#v); })
        }
        "position" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "relative" => quote! { ::gpui::Position::Relative },
                "absolute" => quote! { ::gpui::Position::Absolute },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'position': {other}"),
                    ))
                }
            };
            Ok(quote! { s.position = Some(#v); })
        }
        "flex-direction" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_flex_direction(&kw, span)?;
            Ok(quote! { s.flex_direction = Some(#v); })
        }
        "flex-wrap" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "nowrap" => quote! { ::gpui::FlexWrap::NoWrap },
                "wrap" => quote! { ::gpui::FlexWrap::Wrap },
                "wrap-reverse" => quote! { ::gpui::FlexWrap::WrapReverse },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'flex-wrap': {other}"),
                    ))
                }
            };
            Ok(quote! { s.flex_wrap = Some(#v); })
        }
        "flex" => emit_flex(tokens, span),
        "flex-grow" => {
            let n = number_value(tokens, prop, span)?;
            Ok(quote! { s.flex_grow = Some(#n as f32); })
        }
        "flex-shrink" => {
            let n = number_value(tokens, prop, span)?;
            Ok(quote! { s.flex_shrink = Some(#n as f32); })
        }
        "flex-basis" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(quote! { s.flex_basis = Some(#v); })
        }
        "justify-content" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_justify(&kw, span)?;
            Ok(quote! { s.justify_content = Some(#v); })
        }
        "align-items" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_items(&kw, span)?;
            Ok(quote! { s.align_items = Some(#v); })
        }
        "align-self" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_items(&kw, span)?;
            Ok(quote! { s.align_self = Some(#v); })
        }
        "align-content" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_align_content(&kw, span)?;
            Ok(quote! { s.align_content = Some(#v); })
        }
        "gap" => {
            let lens = parse_lengths(tokens, prop, span)?;
            match lens.len() {
                1 => {
                    let v = emit_as_definite(&lens[0], prop, span)?;
                    Ok(quote! { s.gap.width = Some(#v); s.gap.height = Some(#v); })
                }
                2 => {
                    let h = emit_as_definite(&lens[0], prop, span)?;
                    let w = emit_as_definite(&lens[1], prop, span)?;
                    Ok(quote! { s.gap.height = Some(#h); s.gap.width = Some(#w); })
                }
                _ => Err(unsupported(prop, tokens, span)),
            }
        }
        "row-gap" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(quote! { s.gap.height = Some(#v); })
        }
        "column-gap" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(quote! { s.gap.width = Some(#v); })
        }
        "grid-template-columns" => {
            let n = number_value(tokens, prop, span)?;
            let n = n as u16;
            Ok(quote! { s.grid_cols = Some(#n); })
        }
        "grid-template-rows" => {
            let n = number_value(tokens, prop, span)?;
            let n = n as u16;
            Ok(quote! { s.grid_rows = Some(#n); })
        }
        "width" => size_field("width", tokens, span),
        "height" => size_field("height", tokens, span),
        "min-width" => size_min_max("min_size", "width", tokens, span),
        "min-height" => size_min_max("min_size", "height", tokens, span),
        "max-width" => size_min_max("max_size", "width", tokens, span),
        "max-height" => size_min_max("max_size", "height", tokens, span),
        "padding" => box_edges("padding", tokens, span, true),
        "padding-top" => edge("padding", "top", tokens, span, true),
        "padding-right" => edge("padding", "right", tokens, span, true),
        "padding-bottom" => edge("padding", "bottom", tokens, span, true),
        "padding-left" => edge("padding", "left", tokens, span, true),
        "padding-inline" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(quote! { s.padding.left = Some(#v); s.padding.right = Some(#v); })
        }
        "padding-block" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(quote! { s.padding.top = Some(#v); s.padding.bottom = Some(#v); })
        }
        "margin" => box_edges("margin", tokens, span, false),
        "margin-top" => edge("margin", "top", tokens, span, false),
        "margin-right" => edge("margin", "right", tokens, span, false),
        "margin-bottom" => edge("margin", "bottom", tokens, span, false),
        "margin-left" => edge("margin", "left", tokens, span, false),
        "margin-inline" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(quote! { s.margin.left = Some(#v); s.margin.right = Some(#v); })
        }
        "margin-block" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_length(&len, span)?;
            Ok(quote! { s.margin.top = Some(#v); s.margin.bottom = Some(#v); })
        }
        "inset" => box_edges("inset", tokens, span, false),
        "top" => edge("inset", "top", tokens, span, false),
        "right" => edge("inset", "right", tokens, span, false),
        "bottom" => edge("inset", "bottom", tokens, span, false),
        "left" => edge("inset", "left", tokens, span, false),
        "aspect-ratio" => emit_aspect_ratio(tokens, span),
        "background" | "background-color" => {
            let c = emit_color(tokens, span)?;
            Ok(quote! { s.background = Some((#c).into()); })
        }
        "color" => {
            let c = emit_color(tokens, span)?;
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).color = Some((#c).into());
            })
        }
        "opacity" => {
            let n = number_value(tokens, prop, span)?;
            Ok(quote! { s.opacity = Some(#n as f32); })
        }
        "border-color" => {
            let c = emit_color(tokens, span)?;
            Ok(quote! { s.border_color = Some((#c).into()); })
        }
        "border-style" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "solid" => quote! { ::gpui::BorderStyle::Solid },
                "dashed" => quote! { ::gpui::BorderStyle::Dashed },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'border-style': {other}"),
                    ))
                }
            };
            Ok(quote! { s.border_style = Some(#v); })
        }
        "border-width" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(quote! {
                s.border_widths.top = Some(#v);
                s.border_widths.right = Some(#v);
                s.border_widths.bottom = Some(#v);
                s.border_widths.left = Some(#v);
            })
        }
        "border" => emit_border(tokens, span),
        "border-radius" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(quote! {
                s.corner_radii.top_left = Some(#v);
                s.corner_radii.top_right = Some(#v);
                s.corner_radii.bottom_right = Some(#v);
                s.corner_radii.bottom_left = Some(#v);
            })
        }
        "border-top-left-radius" => corner("top_left", tokens, span),
        "border-top-right-radius" => corner("top_right", tokens, span),
        "border-bottom-right-radius" => corner("bottom_right", tokens, span),
        "border-bottom-left-radius" => corner("bottom_left", tokens, span),
        "cursor" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_cursor(&kw, span)?;
            Ok(quote! { s.mouse_cursor = Some(#v); })
        }
        "box-shadow" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_shadow(&kw, span)?;
            Ok(quote! { s.box_shadow = Some(#v); })
        }
        "font-size" => {
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            if matches!(len.kind, LengthKind::Percent) {
                return Err(syn::Error::new(span, "font-size cannot be a percentage"));
            }
            let v = emit_as_absolute(&len, prop, span)?;
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_size = Some(#v);
            })
        }
        "font-weight" => {
            let v = emit_font_weight(tokens, span)?;
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(#v);
            })
        }
        "font-style" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "italic" => quote! { ::gpui::FontStyle::Italic },
                "normal" => quote! { ::gpui::FontStyle::Normal },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'font-style': {other}"),
                    ))
                }
            };
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).font_style = Some(#v);
            })
        }
        "text-align" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "left" => quote! { ::gpui::TextAlign::Left },
                "center" => quote! { ::gpui::TextAlign::Center },
                "right" => quote! { ::gpui::TextAlign::Right },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'text-align': {other}"),
                    ))
                }
            };
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(#v);
            })
        }
        "text-decoration" => {
            let kw = hyphen_keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            match kw.as_str() {
                "underline" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).underline = Some(::gpui::UnderlineStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "line-through" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).strikethrough = Some(::gpui::StrikethroughStyle {
                        thickness: ::gpui::px(1.),
                        ..::core::default::Default::default()
                    });
                }),
                "none" => Ok(quote! {
                    s.text.get_or_insert_with(::core::default::Default::default).underline = None;
                    s.text.get_or_insert_with(::core::default::Default::default).strikethrough = None;
                }),
                other => Err(syn::Error::new(
                    span,
                    format!("unsupported CSS value for 'text-decoration': {other}"),
                )),
            }
        }
        "white-space" => {
            let kw = keyword(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = match kw.as_str() {
                "nowrap" => quote! { ::gpui::WhiteSpace::Nowrap },
                "normal" => quote! { ::gpui::WhiteSpace::Normal },
                other => {
                    return Err(syn::Error::new(
                        span,
                        format!("unsupported CSS value for 'white-space': {other}"),
                    ))
                }
            };
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).white_space = Some(#v);
            })
        }
        "line-height" => {
            if tokens.len() == 1 {
                if let TokenTree::Literal(_) = &tokens[0] {
                    if parse_suffixed_length(&tokens[0]).is_none() {
                        if let Some(n) = parse_number(&tokens[0]) {
                            return Ok(quote! {
                                s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(::gpui::relative(#n as f32));
                            });
                        }
                    }
                }
            }
            let len = parse_length(tokens).ok_or_else(|| unsupported(prop, tokens, span))?;
            let v = emit_as_definite(&len, prop, span)?;
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(#v);
            })
        }
        "line-clamp" => {
            let n = number_value(tokens, prop, span)? as usize;
            Ok(quote! {
                s.text.get_or_insert_with(::core::default::Default::default).line_clamp = Some(#n);
            })
        }
        _ => Err(unknown_prop(prop, span)),
    }
}

fn emit_interp_prop(prop: &str, expr: TokenStream2, span: Span) -> syn::Result<TokenStream2> {
    match prop {
        "width" => Ok(
            quote! { s.size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "height" => Ok(
            quote! { s.size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "min-width" => Ok(
            quote! { s.min_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "min-height" => Ok(
            quote! { s.min_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "max-width" => Ok(
            quote! { s.max_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "max-height" => Ok(
            quote! { s.max_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "flex-grow" => Ok(quote! { s.flex_grow = Some(#expr as f32); }),
        "flex-shrink" => Ok(quote! { s.flex_shrink = Some(#expr as f32); }),
        "flex-basis" => Ok(
            quote! { s.flex_basis = Some(::core::convert::Into::<::gpui::Length>::into(#expr)); },
        ),
        "opacity" => Ok(quote! { s.opacity = Some(#expr as f32); }),
        "background" | "background-color" => Ok(quote! { s.background = Some((#expr).into()); }),
        "color" => Ok(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).color = Some((#expr).into());
        }),
        "gap" => Ok(quote! {
            let __gap = ::core::convert::Into::<::gpui::DefiniteLength>::into(#expr);
            s.gap.width = Some(__gap);
            s.gap.height = Some(__gap);
        }),
        "aspect-ratio" => Ok(quote! { s.aspect_ratio = Some(#expr as f32); }),
        "line-height" => Ok(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#expr));
        }),
        "line-clamp" => Ok(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_clamp = Some(#expr as usize);
        }),
        "grid-template-columns" => Ok(quote! { s.grid_cols = Some(#expr as u16); }),
        "grid-template-rows" => Ok(quote! { s.grid_rows = Some(#expr as u16); }),
        _ => Err(syn::Error::new(
            span,
            format!("unsupported CSS value for '{prop}': interpolation"),
        )),
    }
}

fn number_value(tokens: &[TokenTree], prop: &str, span: Span) -> syn::Result<f32> {
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(n);
        }
    }
    Err(unsupported(prop, tokens, span))
}

fn emit_flex(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if let Some(kw) = keyword(tokens) {
        return match kw.as_str() {
            "none" => Ok(quote! {
                s.flex_grow = Some(0f32);
                s.flex_shrink = Some(0f32);
                s.flex_basis = Some(::gpui::Length::Auto);
            }),
            "auto" => Ok(quote! {
                s.flex_grow = Some(1f32);
                s.flex_shrink = Some(1f32);
                s.flex_basis = Some(::gpui::Length::Auto);
            }),
            _ => Err(unsupported("flex", tokens, span)),
        };
    }
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(quote! {
                s.flex_grow = Some(#n as f32);
                s.flex_shrink = Some(1f32);
                s.flex_basis = Some(::core::convert::Into::<::gpui::Length>::into(::gpui::px(0.)));
            });
        }
    }
    Err(unsupported("flex", tokens, span))
}

fn size_field(field: &str, tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let v = emit_as_length(&len, span)?;
    let ident = Ident::new(field, span);
    Ok(quote! { s.size.#ident = Some(#v); })
}

fn size_min_max(
    which: &str,
    field: &str,
    tokens: &[TokenTree],
    span: Span,
) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let v = emit_as_length(&len, span)?;
    let which = Ident::new(which, span);
    let field = Ident::new(field, span);
    Ok(quote! { s.#which.#field = Some(#v); })
}

fn box_edges(
    field: &str,
    tokens: &[TokenTree],
    span: Span,
    definite: bool,
) -> syn::Result<TokenStream2> {
    let lens = parse_lengths(tokens, field, span)?;
    let edges = expand_box_edges(&lens)?;
    let ident = Ident::new(field, span);
    let mut stmts = Vec::new();
    let names = ["top", "right", "bottom", "left"];
    for (name, len) in names.iter().zip(edges.iter()) {
        let edge = Ident::new(name, span);
        let v = if definite {
            emit_as_definite(len, field, span)?
        } else {
            emit_as_length(len, span)?
        };
        stmts.push(quote! { s.#ident.#edge = Some(#v); });
    }
    Ok(quote! { #(#stmts)* })
}

fn edge(
    field: &str,
    edge: &str,
    tokens: &[TokenTree],
    span: Span,
    definite: bool,
) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported(field, tokens, span))?;
    let ident = Ident::new(field, span);
    let edge = Ident::new(edge, span);
    let v = if definite {
        emit_as_definite(&len, field, span)?
    } else {
        emit_as_length(&len, span)?
    };
    Ok(quote! { s.#ident.#edge = Some(#v); })
}

fn corner(name: &str, tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    let len = parse_length(tokens).ok_or_else(|| unsupported("border-radius", tokens, span))?;
    let v = emit_as_absolute(&len, "border-radius", span)?;
    let ident = Ident::new(name, span);
    Ok(quote! { s.corner_radii.#ident = Some(#v); })
}

fn emit_aspect_ratio(tokens: &[TokenTree], span: Span) -> syn::Result<TokenStream2> {
    if tokens.len() == 1 {
        if let Some(n) = parse_number(&tokens[0]) {
            return Ok(quote! { s.aspect_ratio = Some(#n as f32); });
        }
    }
    if tokens.len() == 3 {
        if let (Some(w), TokenTree::Punct(p), Some(h)) = (
            parse_number(&tokens[0]),
            &tokens[1],
            parse_number(&tokens[2]),
        ) {
            if p.as_char() == '/' && h != 0.0 {
                let n = w / h;
                return Ok(quote! { s.aspect_ratio = Some(#n as f32); });
            }
        }
    }
    Err(unsupported("aspect-ratio", tokens, span))
}
