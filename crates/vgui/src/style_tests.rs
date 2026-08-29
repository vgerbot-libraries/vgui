use gpui::{px, relative, DefiniteLength, Display, FlexDirection, Hsla, Length, StyleRefinement, Styled};

use crate::{css, set_theme, theme, tw, twc, ApplyStyle, IntoTwStyle, Theme, TwClass, tw_dynamic};

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
    assert!(probe.0.text.color.is_some());
    assert_eq!(probe.0.text.font_size, Some(gpui::AbsoluteLength::from(px(20.))));
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

// ---------------------------------------------------------------------------
// CSS variable (custom property) tests
// ---------------------------------------------------------------------------

#[test]
fn css_var_local_color() {
    let probe = css! {
        --primary: #ff0000;
        color: var(--primary);
    }
    .apply(Probe(Default::default()));
    assert_eq!(
        probe.0.text.color,
        Some(Hsla::from(gpui::rgb(0xff0000)))
    );
}

#[test]
fn css_var_local_length() {
    let probe = css! {
        --gap: 12px;
        gap: var(--gap);
    }
    .apply(Probe(Default::default()));
    assert_eq!(probe.0.gap.width, Some(DefiniteLength::from(px(12.))));
    assert_eq!(probe.0.gap.height, Some(DefiniteLength::from(px(12.))));
}

#[test]
fn css_var_fallback() {
    let probe = css! {
        color: var(--missing, #00ff00);
    }
    .apply(Probe(Default::default()));
    assert_eq!(
        probe.0.text.color,
        Some(Hsla::from(gpui::rgb(0x00ff00)))
    );
}

#[test]
fn css_var_theme_override() {
    set_theme(theme! {
        --primary: #0000ff;
    });
    let probe = css! {
        --primary: #ff0000;
        color: var(--primary);
    }
    .apply(Probe(Default::default()));
    // Theme wins over local default.
    assert_eq!(
        probe.0.text.color,
        Some(Hsla::from(gpui::rgb(0x0000ff)))
    );
    // Reset so other tests aren't affected.
    set_theme(Theme::default());
}

#[test]
fn css_var_keyword() {
    let probe = css! {
        --dir: column;
        flex-direction: var(--dir);
    }
    .apply(Probe(Default::default()));
    assert_eq!(probe.0.flex_direction, Some(FlexDirection::Column));
}

#[test]
fn css_var_number() {
    let probe = css! {
        --o: 0.5;
        opacity: var(--o);
    }
    .apply(Probe(Default::default()));
    assert_eq!(probe.0.opacity, Some(0.5));
}

#[test]
fn css_var_padding_shorthand() {
    let probe = css! {
        --p: 8px;
        padding: var(--p);
    }
    .apply(Probe(Default::default()));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(8.))));
    assert_eq!(probe.0.padding.right, Some(DefiniteLength::from(px(8.))));
    assert_eq!(probe.0.padding.bottom, Some(DefiniteLength::from(px(8.))));
    assert_eq!(probe.0.padding.left, Some(DefiniteLength::from(px(8.))));
}

#[test]
fn css_var_gradient() {
    let probe = css! {
        --a: #ff0000;
        --b: #0000ff;
        background: linear-gradient(90deg, var(--a), var(--b));
    }
    .apply(Probe(Default::default()));
    assert!(probe.0.background.is_some());
}

#[test]
#[should_panic(expected = "css var 'totally_unset' is not set")]
fn css_var_missing_panics() {
    let probe = css! {
        color: var(--totally_unset);
    }
    .apply(Probe(Default::default()));
    let _ = probe.0.text.color;
}

// ---------------------------------------------------------------------------
// Dynamic class composition tests (runtime interpreter)
// ---------------------------------------------------------------------------

#[test]
fn tw_dynamic_matches_tw_literal() {
    let classes = "flex flex-col gap-3 p-4";
    // Compile-time path
    let lit = tw!(classes);
    let lit_probe = lit.apply_to(Probe(Default::default()));
    // Runtime path
    let dyn_style = tw_dynamic(classes);
    let dyn_probe = dyn_style.apply_to(Probe(Default::default()));

    assert_eq!(lit_probe.0.display, dyn_probe.0.display);
    assert_eq!(lit_probe.0.flex_direction, dyn_probe.0.flex_direction);
    assert_eq!(lit_probe.0.gap.width, dyn_probe.0.gap.width);
    assert_eq!(lit_probe.0.gap.height, dyn_probe.0.gap.height);
    assert_eq!(lit_probe.0.padding.top, dyn_probe.0.padding.top);
    assert_eq!(lit_probe.0.padding.right, dyn_probe.0.padding.right);
    assert_eq!(lit_probe.0.padding.bottom, dyn_probe.0.padding.bottom);
    assert_eq!(lit_probe.0.padding.left, dyn_probe.0.padding.left);
}

#[test]
fn tw_dynamic_conflict_resolution() {
    // "p-4 p-2" → last wins → padding = 8px (p-2)
    let style = tw_dynamic("p-4 p-2");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(8.))));

    // "bg-red-500 bg-blue-500" → last wins → background equals bg-blue-500 alone
    let style = tw_dynamic("bg-red-500 bg-blue-500");
    let probe = style.apply_to(Probe(Default::default()));
    assert!(probe.0.background.is_some());
    // Verify last-wins: "bg-red-500 bg-blue-500" should equal "bg-blue-500", not "bg-red-500"
    let blue_only = tw_dynamic("bg-blue-500").apply_to(Probe(Default::default()));
    let red_only = tw_dynamic("bg-red-500").apply_to(Probe(Default::default()));
    assert_eq!(probe.0.background, blue_only.0.background);
    assert_ne!(probe.0.background, red_only.0.background);
}

#[test]
fn tw_class_conditional() {
    // add_if(true, "p-2") applies after "p-4" → padding = 8px
    let style = TwClass::new()
        .add("p-4")
        .add_if(false, "p-8")
        .add_if(true, "p-2")
        .build();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(8.))));

    // add_if(false, "p-8") skipped → padding stays at 16px (p-4)
    let style = TwClass::new()
        .add("p-4")
        .add_if(false, "p-8")
        .build();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
}

#[test]
fn tw_class_composition() {
    let style = TwClass::new()
        .add("flex")
        .add("p-4")
        .add("text-white")
        .build();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
    assert!(probe.0.text.color.is_some());
}

#[test]
fn tw_class_option_source() {
    let style = TwClass::new()
        .add("flex")
        .add(Some("p-4"))
        .add(None::<&str>)
        .build();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
}

#[test]
fn twc_macro() {
    let class = twc!("flex", true.then_some("p-4"), false.then_some("p-8"));
    let style = class.build();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
}

#[test]
fn tw_dynamic_hover_variant() {
    let style = tw_dynamic("flex hover:bg-blue-500");
    assert!(style.hover.is_some());
    // Base has display = Flex
    let base_probe = style.apply_to(Probe(Default::default()));
    assert_eq!(base_probe.0.display, Some(Display::Flex));
    // Apply hover closure to a fresh StyleRefinement
    let style2 = tw_dynamic("flex hover:bg-blue-500");
    if let Some(hover) = style2.hover {
        let mut hover_style = StyleRefinement::default();
        hover(&mut hover_style);
        assert!(hover_style.background.is_some());
    } else {
        panic!("hover closure should be Some");
    }
}

#[test]
fn tw_dynamic_arbitrary() {
    let style = tw_dynamic("w-[500px] bg-[#0000ff]");
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.size.width, Some(Length::from(px(500.))));
    assert!(probe.0.background.is_some());
}

#[test]
fn into_tw_style_str() {
    let style = "flex p-4".into_tw_style();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
    assert_eq!(probe.0.padding.top, Some(DefiniteLength::from(px(16.))));
}

#[test]
fn into_tw_style_twclass() {
    let style = TwClass::new().add("flex").build().into_tw_style();
    let probe = style.apply_to(Probe(Default::default()));
    assert_eq!(probe.0.display, Some(Display::Flex));
}
