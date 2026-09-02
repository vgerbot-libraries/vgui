use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::emit::{attr_tokens, emit_child};
use crate::{AttrKind, Element};

/// Emit a `<Provider context={...} value={...}>` element.
///
/// `<Provider>` is a **logical (layout-transparent) node**: it pushes a
/// context value onto the thread-local provider stack before evaluating its
/// child and pops it afterwards, so descendants constructed in between
/// observe the value via `use_context`. It does *not* introduce any element
/// into the rendered tree — the child is returned verbatim, with no wrapping
/// `div`. This keeps percentage-height / flex resolution chains intact
/// (e.g. a child `height: 100%` resolves against Provider's parent, not
/// against an intermediate auto-sized wrapper).
///
/// Only `context` and `value` attributes are allowed; any other attribute
/// (including `ref`, `style`, `class`, `on:*`) is rejected.
///
/// `__provider_scope_enter(&__ctx, __val)` unifies `T` from both args, so a
/// mismatch between the `Context<T>` and the value type is a compile error.
/// `Context<T>: Copy`, so `let __ctx = #context;` copies the zero-sized
/// marker from a static.
///
/// # Child count
///
/// - **0 children** → emits `gpui::Empty` (renders nothing, no layout node).
/// - **1 child** → emits the child verbatim (no wrapper). This is the common
///   case and the reason Provider is layout-transparent.
/// - **>1 children** → compile error. Provider is a logical node and cannot
///   hold multiple rendered children without introducing a container; wrap
///   siblings in an explicit `<div>` (or any element) so the structure is
///   visible in your source.
pub(crate) fn emit_provider(el: &Element) -> syn::Result<TokenStream2> {
    let mut context = None;
    let mut value = None;
    for attr in &el.attrs {
        match &attr.kind {
            AttrKind::Ident(id) if id == "context" => context = Some(attr_tokens(&attr.value)),
            AttrKind::Ident(id) if id == "value" => value = Some(attr_tokens(&attr.value)),
            _ => {
                return Err(syn::Error::new(
                    attr.span,
                    "unsupported attribute on <Provider>; allowed: `context`, `value`",
                ));
            }
        }
    }
    let context = context.ok_or_else(|| {
        syn::Error::new(el.tag.span(), "<Provider> requires a `context` attribute")
    })?;
    let value = value.ok_or_else(|| {
        syn::Error::new(el.tag.span(), "<Provider> requires a `value` attribute")
    })?;

    // Provider is a logical node: it must not introduce a rendered element.
    // 0 children → Empty; 1 child → the child verbatim; >1 → compile error.
    let body = match el.children.len() {
        0 => quote! { ::gpui::Empty },
        1 => emit_child(&el.children[0])?,
        n => {
            return Err(syn::Error::new(
                el.tag.span(),
                format!(
                    "<Provider> is a logical (layout-transparent) node and accepts exactly \
                     one child, but {n} children were provided. Wrap multiple children in an \
                     explicit <div> (or any element) so the container is visible in your source."
                ),
            ));
        }
    };

    Ok(quote! { {
        let __ctx = #context;
        let __val = #value;
        ::vgui::__provider_scope_enter(&__ctx, __val);
        let __content = #body;
        ::vgui::__provider_scope_exit();
        __content
    } })
}
