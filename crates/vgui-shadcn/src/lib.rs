//! shadcn-style components for vgui.
//!
//! Tokens follow the shadcn-solid default preset. Import
//! [`prelude`] and call [`theme::apply_theme`] from the render closure.

extern crate self as vgui_shadcn;

pub mod components;
pub mod hoc;
pub mod icons;
pub mod mobile;
pub mod prelude;
pub mod theme;
