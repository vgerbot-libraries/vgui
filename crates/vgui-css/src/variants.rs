//! `variants!` proc-macro expansion.
//!
//! Defines declarative component variant dimensions (e.g. a `Button` with
//! `color` and `size` dimensions), generating one `enum` per dimension, a
//! combined `Copy` struct, a `Default` impl (first option per dimension),
//! builder methods, and an `ApplyStyle` impl that applies the base style then
//! each selected dimension style sequentially.
//!
//! See `book/src/styling/variants.md` for the user-facing docs.

use proc_macro2::{Delimiter, Ident, Span, TokenStream as TokenStream2, TokenTree};
use quote::{format_ident, quote, ToTokens};

/// A single dimension (e.g. `size { sm => ..., md => ..., lg => ... }`).
struct Dimension {
    name: String,
    name_span: Span,
    /// `(option_name, span, expr_tokens)` per option.
    options: Vec<(String, Span, TokenStream2)>,
}

struct VariantDef {
    component: Ident,
    base: Option<TokenStream2>,
    dimensions: Vec<Dimension>,
}

pub(crate) fn expand_variants(input: TokenStream2) -> syn::Result<TokenStream2> {
    let def = parse_variants(input)?;
    codegen(&def)
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_variants(input: TokenStream2) -> syn::Result<VariantDef> {
    let mut tokens: Vec<TokenTree> = input.into_iter().collect();
    // Unwrap an outer `{ ... }` brace group if the input is a single group,
    // mirroring `expand_view` / `parse_decls`.
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == Delimiter::Brace {
                tokens = g.stream().into_iter().collect();
            }
        }
    }

    let mut i = 0;
    // Component name ident.
    let component = match tokens.get(i) {
        Some(TokenTree::Ident(id)) => {
            i += 1;
            id.clone()
        }
        other => {
            let span = other.map(|t| t.span()).unwrap_or_else(Span::call_site);
            return Err(syn::Error::new(span, "expected component name ident"));
        }
    };

    // Body group `{ ... }`.
    let body: Vec<TokenTree> = match tokens.get(i) {
        Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
            i += 1;
            g.stream().into_iter().collect()
        }
        other => {
            let span = other.map(|t| t.span()).unwrap_or_else(Span::call_site);
            return Err(syn::Error::new(span, "expected `{` body after component name"));
        }
    };
    skip_trailing(&tokens, &mut i, &component)?;

    let mut base: Option<TokenStream2> = None;
    let mut dimensions: Vec<Dimension> = Vec::new();

    let mut j = 0;
    while j < body.len() {
        skip_commas(&body, &mut j);
        if j >= body.len() {
            break;
        }
        let name_span = body[j].span();
        let name = match &body[j] {
            TokenTree::Ident(id) => id.to_string(),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected `base` or dimension name",
                ))
            }
        };
        j += 1;

        if name == "base" {
            expect_fat_arrow(&body, &mut j, name_span)?;
            let expr = take_until_comma_or_end(&body, &mut j);
            skip_optional_comma(&body, &mut j);
            if base.is_some() {
                return Err(syn::Error::new(name_span, "duplicate `base` clause"));
            }
            base = Some(TokenStream2::from_iter(expr));
        } else {
            // Dimension: expect a `{ ... }` group.
            let dim_body: Vec<TokenTree> = match body.get(j) {
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    j += 1;
                    g.stream().into_iter().collect()
                }
                other => {
                    let span = other.map(|t| t.span()).unwrap_or(name_span);
                    return Err(syn::Error::new(
                        span,
                        "expected `{` dimension body after dimension name",
                    ));
                }
            };
            let options = parse_options(&dim_body, &name, name_span)?;
            if dimensions.iter().any(|d| d.name == name) {
                return Err(syn::Error::new(name_span, "duplicate dimension name"));
            }
            dimensions.push(Dimension {
                name,
                name_span,
                options,
            });
            skip_optional_comma(&body, &mut j);
        }
    }

    if base.is_none() && dimensions.is_empty() {
        return Err(syn::Error::new(
            component.span(),
            "variants! requires at least a `base` style or one dimension",
        ));
    }
    for d in &dimensions {
        if d.options.is_empty() {
            return Err(syn::Error::new(
                d.name_span,
                "dimension must have at least one option",
            ));
        }
    }

    Ok(VariantDef {
        component,
        base,
        dimensions,
    })
}

fn parse_options(
    body: &[TokenTree],
    dim_name: &str,
    dim_span: Span,
) -> syn::Result<Vec<(String, Span, TokenStream2)>> {
    let mut options = Vec::new();
    let mut k = 0;
    while k < body.len() {
        skip_commas(body, &mut k);
        if k >= body.len() {
            break;
        }
        let span = body[k].span();
        let opt_name = match &body[k] {
            TokenTree::Ident(id) => id.to_string(),
            other => {
                return Err(syn::Error::new(
                    other.span(),
                    "expected option name ident in dimension",
                ))
            }
        };
        k += 1;
        expect_fat_arrow(body, &mut k, span)?;
        let expr = take_until_comma_or_end(body, &mut k);
        skip_optional_comma(body, &mut k);
        if options.iter().any(|(n, _, _)| *n == opt_name) {
            return Err(syn::Error::new(span, "duplicate option name in dimension"));
        }
        options.push((opt_name, span, TokenStream2::from_iter(expr)));
    }
    if options.is_empty() {
        return Err(syn::Error::new(
            dim_span,
            format!("dimension `{dim_name}` must have at least one option"),
        ));
    }
    Ok(options)
}

fn skip_trailing(tokens: &[TokenTree], i: &mut usize, component: &Ident) -> syn::Result<()> {
    skip_commas(tokens, i);
    if *i < tokens.len() {
        return Err(syn::Error::new(
            tokens[*i].span(),
            format!("unexpected tokens after `{}` body", component),
        ));
    }
    Ok(())
}

fn skip_commas(tokens: &[TokenTree], i: &mut usize) {
    while *i < tokens.len() {
        if is_punct(tokens[*i].clone(), ',') {
            *i += 1;
        } else {
            break;
        }
    }
}

fn skip_optional_comma(tokens: &[TokenTree], i: &mut usize) {
    if *i < tokens.len() && is_punct(tokens[*i].clone(), ',') {
        *i += 1;
    }
}

fn expect_fat_arrow(tokens: &[TokenTree], i: &mut usize, span: Span) -> syn::Result<()> {
    if *i >= tokens.len() || !is_punct(tokens[*i].clone(), '=') {
        return Err(syn::Error::new(span, "expected `=>` after name"));
    }
    *i += 1;
    if *i >= tokens.len() || !is_punct(tokens[*i].clone(), '>') {
        return Err(syn::Error::new(span, "expected `=>` after name"));
    }
    *i += 1;
    Ok(())
}

fn take_until_comma_or_end(tokens: &[TokenTree], i: &mut usize) -> Vec<TokenTree> {
    let mut out = Vec::new();
    while *i < tokens.len() {
        // Groups are single `TokenTree`s, so a `,` inside `css!{...}` or
        // `foo(a, b)` is naturally hidden from this top-level scan.
        if is_punct(tokens[*i].clone(), ',') {
            break;
        }
        out.push(tokens[*i].clone());
        *i += 1;
    }
    out
}

fn is_punct(tt: TokenTree, ch: char) -> bool {
    matches!(tt, TokenTree::Punct(p) if p.as_char() == ch)
}

// ---------------------------------------------------------------------------
// Naming helpers
// ---------------------------------------------------------------------------

/// `primary` -> `Primary`, `sm` -> `Sm` (first char uppercased, rest unchanged).
fn pascal(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    if let Some(c) = chars.next() {
        for u in c.to_uppercase() {
            out.push(u);
        }
    }
    out.push_str(chars.as_str());
    out
}

fn is_keyword(s: &str) -> bool {
    matches!(
        s,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

/// Field/method ident — raw (`r#name`) when the name is a Rust keyword.
fn field_ident(name: &str, span: Span) -> Ident {
    if is_keyword(name) {
        Ident::new_raw(name, span)
    } else {
        Ident::new(name, span)
    }
}

// ---------------------------------------------------------------------------
// Code generation
// ---------------------------------------------------------------------------

fn codegen(def: &VariantDef) -> syn::Result<TokenStream2> {
    let component = &def.component;
    let variants_struct = format_ident!("{}Variants", component);

    // One enum per dimension.
    let mut enum_defs: Vec<TokenStream2> = Vec::new();
    let mut dim_info: Vec<(Ident, Ident, Ident)> = Vec::new(); // (enum, field, dim_name_ident)
    for d in &def.dimensions {
        let enum_name = format_ident!("{}{}", component, pascal(&d.name));
        let field = field_ident(&d.name, d.name_span);
        let variants: Vec<TokenStream2> = d
            .options
            .iter()
            .map(|(opt, span, _)| Ident::new(&pascal(opt), *span))
            .map(|v| quote! { #v })
            .collect();
        enum_defs.push(quote! {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub enum #enum_name { #(#variants),* }
        });
        dim_info.push((enum_name.clone(), field, Ident::new(&d.name, d.name_span)));
    }

    // Struct + Default + builders.
    let has_dims = !def.dimensions.is_empty();
    let struct_def = if has_dims {
        let fields: Vec<TokenStream2> = dim_info
            .iter()
            .map(|(enum_name, field, _)| quote! { pub #field: #enum_name })
            .collect();
        quote! {
            #[derive(Clone, Copy, PartialEq, Eq, Debug)]
            pub struct #variants_struct { #(#fields),* }
        }
    } else {
        quote! {
            #[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
            pub struct #variants_struct;
        }
    };

    let default_impl = if has_dims {
        let default_fields: Vec<TokenStream2> = dim_info
            .iter()
            .zip(def.dimensions.iter())
            .map(|((enum_name, field, _), d)| {
                let first = Ident::new(&pascal(&d.options[0].0), d.options[0].1);
                quote! { #field: #enum_name::#first }
            })
            .collect();
        quote! {
            impl ::core::default::Default for #variants_struct {
                fn default() -> Self {
                    Self { #(#default_fields),* }
                }
            }
        }
    } else {
        quote! {}
    };

    let builder_impl = if has_dims {
        let builders: Vec<TokenStream2> = dim_info
            .iter()
            .map(|(enum_name, field, _)| {
                quote! {
                    pub fn #field(mut self, v: #enum_name) -> Self {
                        self.#field = v;
                        self
                    }
                }
            })
            .collect();
        quote! {
            impl #variants_struct {
                #(#builders)*
            }
        }
    } else {
        quote! {}
    };

    // ApplyStyle: base then each dimension's selected style sequentially.
    let mut apply_steps: Vec<TokenStream2> = Vec::new();
    if let Some(base) = &def.base {
        apply_steps.push(quote! {
            let el = ::vgui::ApplyStyle::apply_to(#base, el);
        });
    }
    for d in &def.dimensions {
        let field = field_ident(&d.name, d.name_span);
        let enum_name = format_ident!("{}{}", component, pascal(&d.name));
        let arms: Vec<TokenStream2> = d
            .options
            .iter()
            .map(|(opt, span, expr)| {
                let v = Ident::new(&pascal(opt), *span);
                quote! { #enum_name::#v => ::vgui::ApplyStyle::apply_to(#expr, el), }
            })
            .collect();
        apply_steps.push(quote! {
            let el = match self.#field {
                #(#arms)*
            };
        });
    }

    let apply_impl = quote! {
        impl<E: ::gpui::Styled> ::vgui::ApplyStyle<E> for #variants_struct {
            fn apply_to(self, el: E) -> E {
                #(#apply_steps)*
                el
            }
        }
    };

    let mut out = TokenStream2::new();
    for e in &enum_defs {
        e.to_tokens(&mut out);
    }
    struct_def.to_tokens(&mut out);
    default_impl.to_tokens(&mut out);
    builder_impl.to_tokens(&mut out);
    apply_impl.to_tokens(&mut out);
    Ok(out)
}
