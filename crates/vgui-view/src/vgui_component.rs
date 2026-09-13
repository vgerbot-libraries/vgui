use proc_macro2::{Ident, TokenStream as TokenStream2};
use quote::{format_ident, quote, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    FnArg,
    ItemFn,
    Pat,
    Type,
    ReturnType,
};

/// Parsed attribute arguments (currently empty, reserved for future options).
pub struct ComponentAttrArgs;

impl Parse for ComponentAttrArgs {
    fn parse(_input: ParseStream) -> syn::Result<Self> {
        Ok(ComponentAttrArgs)
    }
}

/// Attribute macro entry point.
pub fn vgui_component_impl(
    _args: ComponentAttrArgs,
    item: ItemFn,
) -> syn::Result<TokenStream2> {
    let fn_name = &item.sig.ident;
    let struct_name = snake_to_pascal(&fn_name.to_string());
    let struct_ident = Ident::new(&struct_name, fn_name.span());
    let render_fn_ident = Ident::new(&format!("__{}_render", fn_name), fn_name.span());

    // Collect param info: (field_name, field_type, conversion_kind)
    let mut params: Vec<(Ident, Type, ConvKind)> = Vec::new();
    for arg in &item.sig.inputs {
        match arg {
            FnArg::Typed(pat_type) => {
                let field_name = match &*pat_type.pat {
                    Pat::Ident(pi) => pi.ident.clone(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &pat_type.pat,
                            "only simple identifier patterns are supported",
                        ));
                    }
                };
                let ty = &*pat_type.ty;
                let kind = classify_type(ty);
                params.push((field_name, ty.clone(), kind));
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "self receivers are not supported in vgui_component",
                ));
            }
        }
    }

    // Check return type is impl IntoElement
    match &item.sig.output {
        ReturnType::Type(_, ty) => {
            // We don't strictly validate; just ensure it's not the default unit.
            if let Type::Tuple(t) = &**ty {
                if t.elems.is_empty() {
                    return Err(syn::Error::new_spanned(
                        ty,
                        "vgui_component functions must return impl IntoElement",
                    ));
                }
            }
        }
        ReturnType::Default => {
            return Err(syn::Error::new_spanned(
                &item.sig,
                "vgui_component functions must return impl IntoElement",
            ));
        }
    }

    // Build generic params and where clause
    let mut generic_params: Vec<Ident> = Vec::new();
    let mut generic_args: Vec<TokenStream2> = Vec::new();
    let mut where_preds: Vec<TokenStream2> = Vec::new();
    let mut field_defs: Vec<TokenStream2> = Vec::new();
    let mut call_args: Vec<TokenStream2> = Vec::new();

    for (field_name, field_type, kind) in &params {
        match kind {
            ConvKind::None => {
                field_defs.push(quote! { pub #field_name: #field_type });
                call_args.push(quote! { self.#field_name });
            }
            ConvKind::IntoString => {
                let g = format_ident!("T{}", snake_to_pascal(&field_name.to_string()));
                generic_params.push(g.clone());
                generic_args.push(quote! { #g });
                field_defs.push(quote! { pub #field_name: #g });
                where_preds.push(quote! { #g: ::std::convert::Into<::std::string::String> });
                call_args.push(quote! { ::std::convert::Into::into(self.#field_name) });
            }
            ConvKind::BoxClosure => {
                field_defs.push(quote! { pub #field_name: #field_type });
                call_args.push(quote! { self.#field_name });
            }
            ConvKind::OptionString => {
                let g = format_ident!("T{}", snake_to_pascal(&field_name.to_string()));
                generic_params.push(g.clone());
                generic_args.push(quote! { #g });
                field_defs.push(quote! { pub #field_name: ::std::option::Option<#g> });
                where_preds.push(quote! { #g: ::std::convert::Into<::std::string::String> });
                call_args.push(quote! { self.#field_name.map(|v| ::std::convert::Into::into(v)) });
            }
        }
    }

    // Build the render function (renamed original)
    let render_fn = {
        let block = &item.block;
        let mut new_sig = item.sig.clone();
        new_sig.ident = render_fn_ident.clone();
        quote! {
            #new_sig
            #block
        }
    };

    // Build struct definition
    let struct_def = if generic_params.is_empty() {
        quote! {
            pub struct #struct_ident {
                #(#field_defs),*
            }
        }
    } else {
        let generics = quote! { <#(#generic_params),*> };
        quote! {
            pub struct #struct_ident #generics {
                #(#field_defs),*
            }
        }
    };

    // Build IntoElement impl
    let into_element_impl = if generic_params.is_empty() {
        quote! {
            impl ::gpui::IntoElement for #struct_ident {
                type Element = ::gpui::AnyElement;
                fn into_element(self) -> Self::Element {
                    let __el = #render_fn_ident(#(#call_args),*);
                    ::gpui::IntoElement::into_any_element(__el)
                }
            }
        }
    } else {
        let generics = quote! { <#(#generic_params),*> };
        let where_clause = if where_preds.is_empty() {
            quote! {}
        } else {
            quote! { where #(#where_preds),* }
        };
        quote! {
            impl #generics ::gpui::IntoElement for #struct_ident #generics
            #where_clause
            {
                type Element = ::gpui::AnyElement;
                fn into_element(self) -> Self::Element {
                    let __el = #render_fn_ident(#(#call_args),*);
                    ::gpui::IntoElement::into_any_element(__el)
                }
            }
        }
    };

    Ok(quote! {
        #render_fn
        #struct_def
        #into_element_impl
    })
}

#[derive(Clone, Copy)]
enum ConvKind {
    None,
    IntoString,
    BoxClosure,
    OptionString,
}

fn classify_type(ty: &Type) -> ConvKind {
    let s = ty.to_token_stream().to_string().replace(' ', "");
    // Box<dyn Fn(...) + 'static>
    if s.starts_with("Box<dynFn") || s.starts_with("Box<dynFnMut") || s.starts_with("Box<dynFnOnce")
    {
        return ConvKind::BoxClosure;
    }
    // Option<String>
    if s.starts_with("Option<String>") || s.starts_with("std::option::Option<String>") {
        return ConvKind::OptionString;
    }
    if s == "String" {
        return ConvKind::IntoString;
    }
    ConvKind::None
}

fn snake_to_pascal(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}
