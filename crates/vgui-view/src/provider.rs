use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

use crate::emit::{attr_tokens, emit_children};
use crate::{AttrKind, Element};

/// Emit a `<Provider context={...} value={...}>` element.
///
/// Pushes a context value onto the thread-local provider stack *before*
/// evaluating children and pops it *after*, so descendants constructed
/// between them observe the value via `use_context`. Mirrors `emit_radiogroup`
/// (`builtin.rs`). Only `context` and `value` attributes are allowed; any
/// other attribute (including `ref`, `style`, `class`, `on:*`) is rejected.
///
/// `__provider_scope_enter(&__ctx, __val)` unifies `T` from both args, so a
/// mismatch between the `Context<T>` and the value type is a compile error.
/// `Context<T>: Copy`, so `let __ctx = #context;` copies the zero-sized
/// marker from a static. The wrapper is a plain `gpui::div()` — flex is
/// opt-in in gpui, so it is layout-neutral (same choice as `<radiogroup>`).
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
    let kids = emit_children(&el.children)?;
    Ok(quote! { {
        let __ctx = #context;
        let __val = #value;
        ::vgui::__provider_scope_enter(&__ctx, __val);
        let mut __content = ::gpui::div();
        #(let __c = #kids; __content = __content.child(__c);)*
        ::vgui::__provider_scope_exit();
        __content
    } })
}
