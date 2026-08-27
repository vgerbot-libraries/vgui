use proc_macro2::{Delimiter, Span, TokenStream as TokenStream2, TokenTree};

use crate::Decl;

pub(crate) fn parse_decls(input: TokenStream2) -> syn::Result<Vec<Decl>> {
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

pub(crate) fn skip_separators(tokens: &[TokenTree], i: &mut usize) {
    while *i < tokens.len() {
        match &tokens[*i] {
            TokenTree::Punct(p) if p.as_char() == ';' => *i += 1,
            _ => break,
        }
    }
}

pub(crate) fn skip_optional_semi(tokens: &[TokenTree], i: &mut usize) {
    if *i < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[*i] {
            if p.as_char() == ';' {
                *i += 1;
            }
        }
    }
}

pub(crate) fn is_colon(tt: &TokenTree) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ':')
}

pub(crate) fn is_pseudo(tokens: &[TokenTree], i: usize) -> bool {
    matches!(&tokens[i], TokenTree::Punct(p) if p.as_char() == '&')
}

pub(crate) fn parse_pseudo_name(tokens: &[TokenTree], i: &mut usize) -> syn::Result<String> {
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

pub(crate) fn parse_property(tokens: &[TokenTree], i: &mut usize) -> syn::Result<(String, Span)> {
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

pub(crate) fn take_until_semi_or_end(tokens: &[TokenTree], i: &mut usize) -> Vec<TokenTree> {
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

pub(crate) fn tokens_display(tokens: &[TokenTree]) -> String {
    TokenStream2::from_iter(tokens.iter().cloned())
        .to_string()
        .replace('\n', " ")
}

pub(crate) fn unsupported(prop: &str, tokens: &[TokenTree], span: Span) -> syn::Error {
    syn::Error::new(
        span,
        format!(
            "unsupported CSS value for '{}': {}",
            prop,
            tokens_display(tokens)
        ),
    )
}

pub(crate) fn unknown_prop(name: &str, span: Span) -> syn::Error {
    syn::Error::new(span, format!("unknown CSS property '{name}'"))
}

pub(crate) fn keyword(tokens: &[TokenTree]) -> Option<String> {
    if tokens.len() == 1 {
        if let TokenTree::Ident(id) = &tokens[0] {
            return Some(id.to_string());
        }
    }
    None
}

pub(crate) fn is_interp(tokens: &[TokenTree]) -> Option<TokenStream2> {
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == Delimiter::Brace {
                return Some(g.stream());
            }
        }
    }
    None
}

pub(crate) fn hyphen_keyword(tokens: &[TokenTree]) -> Option<String> {
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
