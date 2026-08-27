use gpui::{px, relative, DefiniteLength, Display, FlexDirection, Length, StyleRefinement, Styled};

use crate::{css, tw, ApplyStyle};

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
    assert_eq!(probe.0.size.width, Some(Length::from(relative(0.5))));
}

#[test]
fn css_margin_auto() {
    let probe = css! { margin: auto; }.apply(Probe(Default::default()));
    assert_eq!(probe.0.margin.top, Some(Length::Auto));
    assert_eq!(probe.0.margin.right, Some(Length::Auto));
    assert_eq!(probe.0.margin.bottom, Some(Length::Auto));
    assert_eq!(probe.0.margin.left, Some(Length::Auto));
}

#[test]
fn tw_flex_col_padding() {
    let style = tw!("flex flex-col p-4");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.flex_direction, Some(FlexDirection::Column));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
    assert_eq!(probe.0.padding.right, Some(DefiniteLength::from(px(16.))));
    assert_eq!(probe.0.padding.bottom, Some(DefiniteLength::from(px(16.))));
    assert_eq!(probe.0.padding.left, Some(DefiniteLength::from(px(16.))));
}

#[test]
fn tw_gap_and_justify() {
    let style = tw!("gap-3 justify-center items-center");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.gap.width, Some(DefiniteLength::from(px(12.))));
    assert_eq!(probe.0.gap.height, Some(DefiniteLength::from(px(12.))));
    assert_eq!(probe.0.justify_content, Some(gpui::JustifyContent::Center));
    assert_eq!(probe.0.align_items, Some(gpui::AlignItems::Center));
}

#[test]
fn tw_bg_color() {
    let style = tw!("bg-blue-500");
    let probe = style.apply_to(Probe(Default::default()));
    assert!(probe.0.background.is_some());
}

#[test]
fn tw_text_color_and_size() {
    let style = tw!("text-white text-xl");
    let probe = style.apply_to(Probe(Default::default()));
    assert!(probe.0.text.is_some());
    let text = probe.0.text.unwrap();
    assert!(text.color.is_some());
    assert_eq!(text.font_size, Some(gpui::AbsoluteLength::from(px(20.))));
}

#[test]
fn tw_arbitrary_values() {
    let style = tw!("w-[500px] h-[250px] bg-[#505050]");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.size.width, Some(Length::from(px(500.))));
    assert_eq!(probe.0.size.height, Some(Length::from(px(250.))));
    assert!(probe.0.background.is_some());
}

#[test]
fn tw_hover_variant() {
    let style = tw!("bg-blue-500 hover:bg-blue-600");
    assert!(style.hover.is_some());
    assert!(style.focus.is_none());
    assert!(style.active.is_none());
}

#[test]
fn tw_rounded() {
    let style = tw!("rounded-lg");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(
        probe.0.corner_radii.top_left,
        Some(gpui::AbsoluteLength::from(px(8.)))
    );
}

#[test]
fn tw_margin_auto() {
    let style = tw!("m-auto");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.margin.top, Some(Length::Auto));
    assert_eq!(probe.0.margin.right, Some(Length::Auto));
    assert_eq!(probe.0.margin.bottom, Some(Length::Auto));
    assert_eq!(probe.0.margin.left, Some(Length::Auto));
}

#[test]
fn tw_width_height() {
    let style = tw!("w-full h-full");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.size.width, Some(Length::from(relative(1.))));
    assert_eq!(probe.0.size.height, Some(Length::from(relative(1.))));
}

#[test]
fn tw_opacity_modifier() {
    let style = tw!("bg-red-500/50");
    let probe = style.apply_to(Probe(Default::default()));
    assert!(probe.0.background.is_some());
}

#[test]
fn tw_border() {
    let style = tw!("border border-2 border-red-500");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(
        probe.0.border_widths.top,
        Some(gpui::AbsoluteLength::from(px(2.)))
    );
    assert!(probe.0.border_color.is_some());
}

#[test]
fn tw_empty() {
    let style = tw!("");
    assert!(style.hover.is_none());
    assert!(style.focus.is_none());
    assert!(style.active.is_none());
}
