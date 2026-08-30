//! Integration test for spread attributes / rest-props forwarding.
//!
//! Exercises:
//! 1. Component spread via struct update syntax (`<Greeting {..props} />`).
//! 2. Component spread with explicit override (`<Greeting {..props} name=... />`).
//! 3. Built-in element spread via the `Spread<E>` trait (`<div {..extras} />`).
//! 4. Rest-props forwarding (`Outer` forwards `self.inner` to `<Inner>`).

use gpui::IntoElement;
use vgui::prelude::*;
use vgui::{view, Spread};

/// A render-scope guard mirroring `enter_scope`/`exit_scope` id-counter
/// behavior, copied from `tests/events.rs`.
struct RenderScope;

impl RenderScope {
    fn new() -> Self {
        vgui::__test_enter_render_scope();
        RenderScope
    }
}

impl Drop for RenderScope {
    fn drop(&mut self) {
        vgui::__test_exit_render_scope();
    }
}

// ── Test component: Greeting ─────────────────────────────────────────

/// A simple component with a single `name` field.
pub struct Greeting {
    pub name: String,
}

impl gpui::IntoElement for Greeting {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let name = self.name;
        view! {
            <div>
                {"Hello, "}
                {name}
                {"!"}
            </div>
        }
        .into_any_element()
    }
}

// ── Test component: Inner (for rest-props forwarding) ────────────────

pub struct Inner {
    pub text: String,
}

impl gpui::IntoElement for Inner {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let text = self.text;
        view! {
            <span>{text}</span>
        }
        .into_any_element()
    }
}

pub struct Outer {
    pub label: String,
    pub inner: Inner,
}

impl gpui::IntoElement for Outer {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        // Forward `self.inner` onto `<Inner>` via struct update syntax.
        view! {
            <div>
                <Inner {..self.inner} />
            </div>
        }
        .into_any_element()
    }
}

// ── Test props struct for built-in spread ────────────────────────────

struct DivExtras {
    bg: gpui::Hsla,
}

impl Spread<gpui::Div> for DivExtras {
    fn spread(self, el: gpui::Div) -> gpui::Div {
        el.bg(self.bg)
    }
}

// ── Tests ────────────────────────────────────────────────────────────

#[test]
fn component_spread_builds() {
    let _scope = RenderScope;
    let props = Greeting { name: "world".to_string() };
    let el = view! {
        <Greeting {..props} />
    };
    let _any = el.into_any_element();
}

#[test]
fn component_spread_with_override_builds() {
    let _scope = RenderScope;
    let props = Greeting { name: "world".to_string() };
    // Explicit `name` field overrides the spread — Rust struct update
    // semantics: named fields win over `..base`.
    let el = view! {
        <Greeting {..props} name={"override".to_string()} />
    };
    let _any = el.into_any_element();
}

#[test]
fn builtin_spread_via_trait_builds() {
    let _scope = RenderScope;
    let el = view! {
        <div {..DivExtras { bg: gpui::hsla(0., 1., 0.5, 1.) }}>
            {"spread on div"}
        </div>
    };
    let _any = el.into_any_element();
}

#[test]
fn rest_props_forwarding_builds() {
    let _scope = RenderScope;
    let outer = Outer {
        label: "unused".to_string(),
        inner: Inner { text: "forwarded".to_string() },
    };
    let el = outer.into_element();
    let _any = el.into_any_element();
}
