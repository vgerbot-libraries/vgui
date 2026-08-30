//! Animation & transition infrastructure for vgui.
//!
//! Built on top of gpui's imperative animation API (`AnimationExt::with_animation`).
//! Provides:
//! - Built-in keyframe animations (`animate-pulse`, `animate-bounce`, `animate-ping`)
//!   via [`apply_animation`].
//! - CSS-style transitions between base and hover styles via [`apply_transition`].
//! - Easing functions ([`Easing`]) and property selectors ([`TransitionProperties`]).
//!
//! See `book/src/styling/animations.md` for user-facing docs.
use std::rc::Rc;
use std::time::Duration;

use gpui::{Animation, AnimationElement, AnimationExt, Fill, Interpolate, Length, Refineable, Styled, px};

use crate::reactive::ReadSignal;

// ---------------------------------------------------------------------------
// Easing
// ---------------------------------------------------------------------------

/// Tailwind-compatible easing curves.
///
/// Maps to the standard CSS cubic-bezier values used by Tailwind's `ease-*`
/// utilities. [`Easing::to_fn`] returns the gpui easing closure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Easing {
    /// `ease-linear` — constant speed.
    Linear,
    /// `ease-in` — cubic-bezier(0.4, 0, 1, 1).
    EaseIn,
    /// `ease-out` — cubic-bezier(0, 0, 0.2, 1).
    EaseOut,
    /// `ease-in-out` — cubic-bezier(0.4, 0, 0.2, 1).
    EaseInOut,
}

impl Easing {
    /// Convert to a gpui-compatible easing function `f32 -> f32` (0..1 -> 0..1).
    pub fn to_fn(self) -> Rc<dyn Fn(f32) -> f32> {
        match self {
            Easing::Linear => Rc::new(gpui::linear),
            Easing::EaseIn => Rc::new(cubic_bezier(0.4, 0.0, 1.0, 1.0)),
            Easing::EaseOut => Rc::new(cubic_bezier(0.0, 0.0, 0.2, 1.0)),
            Easing::EaseInOut => Rc::new(cubic_bezier(0.4, 0.0, 0.2, 1.0)),
        }
    }
}

/// Solve a cubic-bezier easing curve with control points P1=(x1,y1), P2=(x2,y2)
/// (P0=(0,0), P3=(1,1)). Uses Newton-Raphson (8 iterations) with a bisection
/// fallback, mirroring WebKit's `UnitBezier`.
fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32) -> impl Fn(f32) -> f32 {
    move |t| {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        // Solve bezier_x(s) = t for s, then return bezier_y(s).
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        let mut s = t;
        for _ in 0..8 {
            let x = bezier_coord(s, x1, x2) - t;
            if x.abs() < 1e-6 {
                break;
            }
            let dx = bezier_deriv(s, x1, x2);
            if dx.abs() > 1e-6 {
                let next = s - x / dx;
                if (0.0..=1.0).contains(&next) {
                    s = next;
                    continue;
                }
            }
            // Fallback to bisection when Newton steps outside [0,1].
            if x > 0.0 {
                hi = s;
            } else {
                lo = s;
            }
            s = (lo + hi) / 2.0;
        }
        bezier_coord(s, y1, y2)
    }
}

#[inline]
fn bezier_coord(t: f32, p1: f32, p2: f32) -> f32 {
    let c = 3.0 * p1;
    let b = 3.0 * (p2 - p1) - c;
    let a = 1.0 - c - b;
    ((a * t + b) * t + c) * t
}

#[inline]
fn bezier_deriv(t: f32, p1: f32, p2: f32) -> f32 {
    let c = 3.0 * p1;
    let b = 3.0 * (p2 - p1) - c;
    let a = 1.0 - c - b;
    (3.0 * a * t + 2.0 * b) * t + c
}

// ---------------------------------------------------------------------------
// Config structs
// ---------------------------------------------------------------------------

/// Configuration for a built-in `animate-*` keyframe animation.
#[derive(Clone, Debug)]
pub struct TwAnimation {
    /// Keyframe name: `"pulse"`, `"bounce"`, `"ping"`, or `"spin"` (no-op).
    pub name: String,
    /// Cycle duration.
    pub duration: Duration,
    pub easing: Easing,
    /// Start delay (reserved for future use; gpui has no native delay support).
    pub delay: Duration,
    /// Whether the animation loops forever (`true` for `animate-*`).
    pub repeat: bool,
}

/// Configuration for a CSS-style transition between state styles.
#[derive(Clone, Debug)]
pub struct TwTransition {
    /// Which properties animate.
    pub properties: TransitionProperties,
    /// Transition duration.
    pub duration: Duration,
    /// Easing curve.
    pub easing: Easing,
    /// Start delay (reserved for future use).
    pub delay: Duration,
}

/// Bitflags selecting which style properties a transition animates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TransitionProperties(u8);

impl TransitionProperties {
    pub const NONE: Self = Self(0);
    pub const OPACITY: Self = Self(1);
    pub const COLORS: Self = Self(2);
    pub const ALL: Self = Self(Self::OPACITY.0 | Self::COLORS.0);

    #[inline]
    pub fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }
}

// ---------------------------------------------------------------------------
// apply_animation — built-in keyframe classes
// ---------------------------------------------------------------------------

/// Wrap an element with a built-in `animate-*` keyframe animation.
///
/// `animate-spin` is a no-op (gpui has no rotation support for div elements).
pub fn apply_animation<E: gpui::IntoElement + Styled + 'static>(
    el: E,
    anim: &TwAnimation,
) -> AnimationElement<E> {
    let easing = anim.easing.to_fn();
    let mut animation = Animation::new(anim.duration).with_easing(move |t| easing(t));
    if anim.repeat {
        animation = animation.repeat();
    }
    let name = anim.name.clone();
    el.with_animation(
        format!("vgui-animate-{}", name),
        animation,
        move |mut el, delta| match name.as_str() {
            // opacity 1 -> 0.5 -> 1 over one cycle
            "pulse" => {
                let opacity = 1.0 - 0.5 * (delta * std::f32::consts::PI).sin();
                el.opacity(opacity)
            }
            // vertical bounce via margin_top: 0 -> -10px -> 0
            "bounce" => {
                let offset = if delta < 0.5 {
                    -10.0 * (delta * 2.0) * (delta * 2.0)
                } else {
                    let d = (delta - 0.5) * 2.0;
                    -10.0 * (1.0 - d * d)
                };
                el.style().margin.top = Some(Length::from(px(offset)));
                el
            }
            // opacity 1 -> 0 over the first 75% of the cycle, then 0
            "ping" => {
                let opacity = (1.0 - delta / 0.75).max(0.0);
                el.opacity(opacity)
            }
            // no rotation support in gpui — render statically
            "spin" => el,
            _ => el,
        },
    )
}

/// Apply a user-supplied animation expression (the `animate={...}` attribute).
///
/// `f` receives the element and returns an [`AnimationElement`], e.g.
/// `|el| el.with_animation("custom", Animation::new(Duration::from_secs(1)).repeat(), |el, d| el.opacity(d))`.
pub fn apply_animation_expr<E: gpui::IntoElement + 'static>(
    el: E,
    f: impl FnOnce(E) -> AnimationElement<E>,
) -> AnimationElement<E> {
    f(el)
}

// ---------------------------------------------------------------------------
// apply_transition — CSS-style transitions on hover
// ---------------------------------------------------------------------------

/// Wrap an element with a hover transition. The element must already have an
/// ID and `on_hover` registered by the caller (the `view!` macro does both).
///
/// `base_snap` is the resting style; `hover_snap` is the hovered style
/// (base + hover refinements). `hovered` tracks the current hover state.
pub fn apply_transition<E: gpui::IntoElement + Styled + 'static>(
    el: E,
    trans: TwTransition,
    base_snap: gpui::StyleRefinement,
    hover_snap: gpui::StyleRefinement,
    hovered: ReadSignal<bool>,
) -> AnimationElement<E> {
    let dur = trans.duration;
    let easing_fn = trans.easing.to_fn();
    let props = trans.properties;
    let hovered_read = hovered.clone();
    let base = base_snap.clone();
    let hover = hover_snap.clone();

    let anim_id = format!("vgui-transition-{}", hovered.get());
    el.with_animation(
        anim_id,
        Animation::new(dur).with_easing(move |t| easing_fn(t)),
        move |mut el, delta| {
            // When hovered, animate base -> hover; when unhovered, hover -> base.
            let (from, to) = if hovered_read.get() {
                (&base, &hover)
            } else {
                (&hover, &base)
            };
            let interpolated = interpolate_refinement(from, to, delta, props);
            el.style().refine(&interpolated);
            el
        },
    )
}

/// Interpolate between two style snapshots for the properties in `props`.
///
/// Non-selected properties are taken from `to` (instant change, matching CSS
/// behavior for non-transitioned properties). Selected properties are blended
/// via [`gpui::Interpolate`] when both sides are set; otherwise `to` wins.
pub fn interpolate_refinement(
    from: &gpui::StyleRefinement,
    to: &gpui::StyleRefinement,
    phase: f32,
    props: TransitionProperties,
) -> gpui::StyleRefinement {
    let mut out = to.clone();

    if props.contains(TransitionProperties::OPACITY) {
        if let (Some(a), Some(b)) = (from.opacity, to.opacity) {
            out.opacity = Some(Interpolate::interpolate(a, b, phase));
        }
    }

    if props.contains(TransitionProperties::COLORS) {
        // background (solid fills only)
        if let (Some(fa), Some(tb)) = (&from.background, &to.background) {
            if let (Some(fc), Some(tc)) = (fa.color().and_then(|c| c.as_solid()), tb.color().and_then(|c| c.as_solid())) {
                out.background = Some(Fill::from(Interpolate::interpolate(fc, tc, phase)));
            }
        }
        // border color
        if let (Some(fc), Some(tc)) = (from.border_color, to.border_color) {
            out.border_color = Some(Interpolate::interpolate(fc, tc, phase));
        }
        // text color
        if let (Some(fc), Some(tc)) = (from.text.color, to.text.color) {
            out.text.color = Some(Interpolate::interpolate(fc, tc, phase));
        }
    }

    out
}
