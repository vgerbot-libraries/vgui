//! Bundled vector icon set.
//!
//! Real SVG files live in `assets/icons/` (ISC-licensed Lucide icons — see
//! `assets/icons/LICENSE`). `build.rs` embeds each file's bytes into
//! [`icon_bytes`]; [`icon`] renders them through gpui's native SVG renderer,
//! so the drawn output is the actual vector path, not a text glyph or stub.
//!
//! ```ignore
//! use vgui_shadcn::prelude::*;
//! let search = icon("search").size_px(16.);
//! ```

use gpui::{px, AnyElement, Hsla, IntoElement, Styled};
use vgui::__var_color;

include!(concat!(env!("OUT_DIR"), "/icon_data.rs"));

/// Whether `name` is a bundled icon.
pub fn has_icon(name: &str) -> bool {
    icon_bytes(name).is_some()
}

/// Number of bundled icons.
pub fn icon_count() -> usize {
    ICON_NAMES.len()
}

/// A bundled SVG icon, rendered at 16x16 in the foreground color.
///
/// Returns an empty placeholder for an unknown name so a typo never panics
/// the whole tree; use [`has_icon`] to check membership.
pub fn icon(name: &str) -> Icon {
    let bytes = icon_bytes(name);
    Icon {
        name: name.to_string(),
        bytes,
        size: 16.0,
        color: None,
    }
}

/// Renderable, configurable icon returned by [`icon`].
pub struct Icon {
    name: String,
    bytes: Option<&'static [u8]>,
    size: f32,
    color: Option<Hsla>,
}

impl Icon {
    /// Set the edge length in px (icons are square).
    pub fn size_px(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// Override the paint color; defaults to the `--foreground` theme token.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// The icon's bundled name.
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl IntoElement for Icon {
    type Element = AnyElement;

    fn into_element(self) -> AnyElement {
        let color = self
            .color
            .unwrap_or_else(|| __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.))));
        match self.bytes {
            Some(bytes) => gpui::svg()
                .data(bytes)
                .w(px(self.size))
                .h(px(self.size))
                .text_color(color)
                .into_any_element(),
            // Unknown name: reserve the same box but paint nothing, so layout
            // is stable and the error is non-fatal.
            None => gpui::div()
                .w(px(self.size))
                .h(px(self.size))
                .into_any_element(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundles_a_large_real_set() {
        assert!(
            icon_count() >= 100,
            "expected >=100 bundled icons, got {}",
            icon_count()
        );
    }

    #[test]
    fn registry_returns_svg_bytes() {
        for name in ["bell", "check", "chevron_right", "close", "star"] {
            let bytes = icon_bytes(name).unwrap_or_else(|| panic!("missing {name}"));
            assert!(bytes.starts_with(b"<svg"), "{name} is not an SVG document");
            assert!(bytes.windows(6).any(|w| w == b"</svg>"), "{name} not closed");
            assert!(has_icon(name));
        }
        assert_eq!(icon_bytes("does-not-exist"), None);
        assert!(!has_icon("does-not-exist"));
    }

    #[test]
    fn names_are_sorted_and_unique() {
        let mut sorted = ICON_NAMES.to_vec();
        sorted.sort_unstable();
        assert_eq!(sorted.as_slice(), ICON_NAMES);
        let unique = sorted.iter().collect::<std::collections::HashSet<_>>().len();
        assert_eq!(unique, ICON_NAMES.len());
    }
}
