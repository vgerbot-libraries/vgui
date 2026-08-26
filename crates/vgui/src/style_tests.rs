use gpui::{px, relative, DefiniteLength, Display, Length, StyleRefinement, Styled};

use crate::css;

struct Probe(StyleRefinement);

impl Styled for Probe {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.0
    }
}

#[test]
fn css_flex_padding_background_width() {
    let probe = css! {
        display: flex;
        padding: 8px;
        background: #505050;
        width: 50%;
    }
    .apply(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(8.))));
    assert!(probe.0.background.is_some());
    assert_eq!(
        probe.0.size.width,
        Some(Length::from(relative(0.5)))
    );
}

#[test]
fn css_margin_auto() {
    let probe = css! { margin: auto; }.apply(Probe(Default::default()));
    assert_eq!(probe.0.margin.top, Some(Length::Auto));
    assert_eq!(probe.0.margin.right, Some(Length::Auto));
    assert_eq!(probe.0.margin.bottom, Some(Length::Auto));
    assert_eq!(probe.0.margin.left, Some(Length::Auto));
}
