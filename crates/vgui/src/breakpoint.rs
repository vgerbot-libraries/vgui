//! Responsive breakpoints — `sm:`/`md:`/`lg:`/`xl:` support.
//!
//! Provides a `Breakpoint` enum and `use_breakpoint()` hook that tracks
//! the current viewport width and returns the active breakpoint.
//!
//! Breakpoint thresholds (matching Tailwind CSS defaults):
//!
//! | Breakpoint | Min width |
//! |------------|-----------|
//! | `Base`     | < 640px   |
//! | `Sm`       | >= 640px  |
//! | `Md`       | >= 768px  |
//! | `Lg`       | >= 1024px |
//! | `Xl`       | >= 1280px |

use gpui::{Pixels, StyleRefinement};

/// Apply responsive breakpoint styles to a `StyleRefinement` based on the
/// current viewport width.
///
/// Called by the `view!` macro when `sm:`/`md:`/`lg:`/`xl:` variant classes
/// are present. Each breakpoint closure is applied if the current viewport
/// width is at or above that breakpoint's minimum width.
#[doc(hidden)]
pub fn __apply_breakpoint_styles(
    style: &mut StyleRefinement,
    sm: Option<Box<dyn FnOnce(&mut StyleRefinement) + 'static>>,
    md: Option<Box<dyn FnOnce(&mut StyleRefinement) + 'static>>,
    lg: Option<Box<dyn FnOnce(&mut StyleRefinement) + 'static>>,
    xl: Option<Box<dyn FnOnce(&mut StyleRefinement) + 'static>>,
) {
    let width = crate::reactive::get_viewport_width();
    let bp = match width {
        Some(w) => Breakpoint::from_width(w),
        None => Breakpoint::Base,
    };

    if bp >= Breakpoint::Sm {
        if let Some(f) = sm {
            f(style);
        }
    }
    if bp >= Breakpoint::Md {
        if let Some(f) = md {
            f(style);
        }
    }
    if bp >= Breakpoint::Lg {
        if let Some(f) = lg {
            f(style);
        }
    }
    if bp >= Breakpoint::Xl {
        if let Some(f) = xl {
            f(style);
        }
    }
}

/// Responsive breakpoint, ordered from smallest to largest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breakpoint {
    /// Default: < 640px
    Base,
    /// >= 640px
    Sm,
    /// >= 768px
    Md,
    /// >= 1024px
    Lg,
    /// >= 1280px
    Xl,
}

impl Breakpoint {
    /// Minimum width in pixels for this breakpoint.
    pub fn min_width(self) -> f32 {
        match self {
            Breakpoint::Base => 0.0,
            Breakpoint::Sm => 640.0,
            Breakpoint::Md => 768.0,
            Breakpoint::Lg => 1024.0,
            Breakpoint::Xl => 1280.0,
        }
    }

    /// Resolve which breakpoint applies for a given viewport width.
    pub fn from_width(width: f32) -> Breakpoint {
        if width >= 1280.0 {
            Breakpoint::Xl
        } else if width >= 1024.0 {
            Breakpoint::Lg
        } else if width >= 768.0 {
            Breakpoint::Md
        } else if width >= 640.0 {
            Breakpoint::Sm
        } else {
            Breakpoint::Base
        }
    }

    /// Resolve which breakpoint applies for a given viewport width (Pixels).
    pub fn from_pixels(width: Pixels) -> Breakpoint {
        Self::from_width(f32::from(width))
    }

    /// Returns `true` if this breakpoint is at or above the given level.
    pub fn at_or_above(self, other: Breakpoint) -> bool {
        self >= other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_width_thresholds() {
        assert_eq!(Breakpoint::from_width(0.0), Breakpoint::Base);
        assert_eq!(Breakpoint::from_width(639.0), Breakpoint::Base);
        assert_eq!(Breakpoint::from_width(640.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(767.0), Breakpoint::Sm);
        assert_eq!(Breakpoint::from_width(768.0), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(1023.0), Breakpoint::Md);
        assert_eq!(Breakpoint::from_width(1024.0), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1279.0), Breakpoint::Lg);
        assert_eq!(Breakpoint::from_width(1280.0), Breakpoint::Xl);
        assert_eq!(Breakpoint::from_width(1920.0), Breakpoint::Xl);
    }

    #[test]
    fn min_widths() {
        assert_eq!(Breakpoint::Base.min_width(), 0.0);
        assert_eq!(Breakpoint::Sm.min_width(), 640.0);
        assert_eq!(Breakpoint::Md.min_width(), 768.0);
        assert_eq!(Breakpoint::Lg.min_width(), 1024.0);
        assert_eq!(Breakpoint::Xl.min_width(), 1280.0);
    }

    #[test]
    fn at_or_above() {
        assert!(Breakpoint::Md.at_or_above(Breakpoint::Sm));
        assert!(Breakpoint::Md.at_or_above(Breakpoint::Md));
        assert!(!Breakpoint::Md.at_or_above(Breakpoint::Lg));
    }
}
