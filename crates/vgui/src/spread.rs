//! Spread attributes / rest-props forwarding support.
//!
//! See the `view!` macro documentation for the `{..props}` syntax. This trait
//! is the runtime hook for built-in HTML elements; component spread uses Rust's
//! native struct update syntax and needs no trait.

/// Spread a props value onto a built-in element.
///
/// Enables `{..props}` syntax in `view!` for built-in HTML elements
/// (`<div>`, `<button>`, …). Implement for your props struct; the element
/// type `E` is the concrete gpui type after other attributes are applied:
/// - Bare `<div {..p} />` → `gpui::Div`
/// - With `class`/`on:click`/`ref`/`tabindex`/`id`/`active`/`focus` →
///   `gpui::Stateful<gpui::Div>`
///
/// Explicit attributes are applied before `spread`, so they take precedence.
///
/// # Example
///
/// ```ignore
/// struct DivExtras { bg: gpui::Hsla }
///
/// impl ::vgui::Spread<gpui::Div> for DivExtras {
///     fn spread(self, el: gpui::Div) -> gpui::Div {
///         el.bg(self.bg)
///     }
/// }
///
/// // view! { <div {..DivExtras { bg: gpui::red() }}>{"x"}</div> }
/// ```
pub trait Spread<E> {
    fn spread(self, el: E) -> E;
}
