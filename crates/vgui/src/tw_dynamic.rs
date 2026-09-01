//! Runtime Tailwind class interpreter — mirrors the `vgui-tailwind` proc-macro's
//! emit logic but mutates `gpui::StyleRefinement` directly instead of producing
//! `quote!` tokens.
//!
//! Public entry point: [`tw_dynamic`]. Used by `TwClass::build` and the
//! `IntoTwStyle` impls for `&str`/`String` to support runtime/composed classes.

use gpui::{
    point, px, relative, rems, AbsoluteLength, Background, BoxShadow, Corners, CursorStyle,
    DefiniteLength, Display, Fill, FlexDirection, FlexWrap, FontStyle, FontWeight, Hsla, Length,
    Overflow, Pixels, Position, Rgba, SharedString, StyleRefinement, TextAlign, TextOverflow,
    WhiteSpace, black, hsla, rgb, rgba, white,
};
use vgui_tailwind_core::{
    border_radius_value, border_width_value, color_rgb, font_size_value, parse_class, spacing_value,
    ParsedClass, Variant,
};

use crate::TwStyle;

/// Parse a class string at runtime and build a `TwStyle` whose closures apply
/// the resolved styles to a `StyleRefinement` on each render.
///
/// Classes are applied sequentially per variant; last-write-wins at the field
/// level, matching Tailwind's last-class-wins semantics.
pub fn tw_dynamic(classes: &str) -> TwStyle {
    let mut base: Vec<ParsedClass> = Vec::new();
    let mut hover: Vec<ParsedClass> = Vec::new();
    let mut focus: Vec<ParsedClass> = Vec::new();
    let mut active: Vec<ParsedClass> = Vec::new();
    let mut sm: Vec<ParsedClass> = Vec::new();
    let mut md: Vec<ParsedClass> = Vec::new();
    let mut lg: Vec<ParsedClass> = Vec::new();
    let mut xl: Vec<ParsedClass> = Vec::new();

    let mut animation_name: Option<String> = None;
    let mut transition_props: Option<crate::animation::TransitionProperties> = None;
    let mut duration_ms: Option<u64> = None;
    let mut easing_kind: Option<crate::animation::Easing> = None;
    let mut delay_ms: Option<u64> = None;

    for class in classes.split_whitespace() {
        if let Some(parsed) = parse_class(class) {
            let util = parsed.utility.as_str();
            if parsed.variant == Variant::Base {
                if let Some(name) = parse_animate_name(util) {
                    animation_name = Some(name.to_string());
                    continue;
                }
                if let Some(props) = parse_transition_props(util) {
                    transition_props = Some(props);
                    continue;
                }
                if let Some(ms) = parse_duration(util) {
                    duration_ms = Some(ms);
                    continue;
                }
                if let Some(ms) = parse_delay(util) {
                    delay_ms = Some(ms);
                    continue;
                }
                if let Some(e) = parse_easing(util) {
                    easing_kind = Some(e);
                    continue;
                }
            }
            match parsed.variant {
                Variant::Base => base.push(parsed),
                Variant::Hover => hover.push(parsed),
                Variant::Focus => focus.push(parsed),
                Variant::Active => active.push(parsed),
                Variant::Sm => sm.push(parsed),
                Variant::Md => md.push(parsed),
                Variant::Lg => lg.push(parsed),
                Variant::Xl => xl.push(parsed),
            }
        }
    }

    let base_fn: Box<dyn FnOnce(&mut StyleRefinement) + 'static> = if base.is_empty() {
        Box::new(|_s: &mut StyleRefinement| {})
    } else {
        Box::new(move |s: &mut StyleRefinement| {
            for cls in &base {
                apply_class(s, cls);
            }
        })
    };

    let hover_fn = if hover.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &hover {
                apply_class(s, cls);
            }
        }) as Box<dyn Fn(&mut StyleRefinement) + 'static>)
    };

    let focus_fn = if focus.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &focus {
                apply_class(s, cls);
            }
        }) as Box<dyn Fn(&mut StyleRefinement) + 'static>)
    };

    let active_fn = if active.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &active {
                apply_class(s, cls);
            }
        }) as Box<dyn Fn(&mut StyleRefinement) + 'static>)
    };

    let sm_fn = if sm.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &sm {
                apply_class(s, cls);
            }
        }) as Box<dyn FnOnce(&mut StyleRefinement) + 'static>)
    };

    let md_fn = if md.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &md {
                apply_class(s, cls);
            }
        }) as Box<dyn FnOnce(&mut StyleRefinement) + 'static>)
    };

    let lg_fn = if lg.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &lg {
                apply_class(s, cls);
            }
        }) as Box<dyn FnOnce(&mut StyleRefinement) + 'static>)
    };

    let xl_fn = if xl.is_empty() {
        None
    } else {
        Some(Box::new(move |s: &mut StyleRefinement| {
            for cls in &xl {
                apply_class(s, cls);
            }
        }) as Box<dyn FnOnce(&mut StyleRefinement) + 'static>)
    };

    let animation = animation_name.map(|name| {
        let dur = duration_ms.unwrap_or_else(|| default_duration(&name));
        let easing = easing_kind.unwrap_or(crate::animation::Easing::EaseInOut);
        let delay = delay_ms.unwrap_or(0);
        crate::animation::TwAnimation {
            name,
            duration: std::time::Duration::from_millis(dur),
            easing,
            delay: std::time::Duration::from_millis(delay),
            repeat: true,
        }
    });

    let transition = transition_props.map(|props| {
        let dur = duration_ms.unwrap_or(150);
        let easing = easing_kind.unwrap_or(crate::animation::Easing::EaseInOut);
        let delay = delay_ms.unwrap_or(0);
        crate::animation::TwTransition {
            properties: props,
            duration: std::time::Duration::from_millis(dur),
            easing,
            delay: std::time::Duration::from_millis(delay),
        }
    });

    TwStyle {
        base: base_fn,
        hover: hover_fn,
        focus: focus_fn,
        active: active_fn,
        sm: sm_fn,
        md: md_fn,
        lg: lg_fn,
        xl: xl_fn,
        animation,
        transition,
    }
}

// ---------------------------------------------------------------------------
// Animation / transition / timing class parsing (runtime mirror)
// ---------------------------------------------------------------------------

fn parse_animate_name(util: &str) -> Option<&'static str> {
    match util {
        "animate-pulse" => Some("pulse"),
        "animate-bounce" => Some("bounce"),
        "animate-ping" => Some("ping"),
        "animate-spin" => Some("spin"),
        _ => None,
    }
}

fn parse_transition_props(util: &str) -> Option<crate::animation::TransitionProperties> {
    use crate::animation::TransitionProperties;
    match util {
        "transition" | "transition-all" => Some(TransitionProperties::ALL),
        "transition-opacity" => Some(TransitionProperties::OPACITY),
        "transition-colors" => Some(TransitionProperties::COLORS),
        "transition-shadow" | "transition-transform" => None,
        _ => None,
    }
}

fn parse_duration(util: &str) -> Option<u64> {
    util.strip_prefix("duration-")?.parse::<u64>().ok()
}

fn parse_delay(util: &str) -> Option<u64> {
    util.strip_prefix("delay-")?.parse::<u64>().ok()
}

fn parse_easing(util: &str) -> Option<crate::animation::Easing> {
    use crate::animation::Easing;
    match util {
        "ease-linear" => Some(Easing::Linear),
        "ease-in" => Some(Easing::EaseIn),
        "ease-out" => Some(Easing::EaseOut),
        "ease-in-out" => Some(Easing::EaseInOut),
        _ => None,
    }
}

fn default_duration(name: &str) -> u64 {
    match name {
        "pulse" => 2000,
        "bounce" => 1000,
        "ping" => 1000,
        "spin" => 1000,
        _ => 1000,
    }
}

/// Mirror of `emit_class` — dispatch a single parsed class to the right apply path.
fn apply_class(s: &mut StyleRefinement, cls: &ParsedClass) {
    let util = cls.utility.as_str();
    let opacity = cls.opacity;

    if let Some(arb) = cls.arbitrary.as_deref() {
        apply_arbitrary(s, util, arb, opacity);
        return;
    }

    if apply_exact(s, util) {
        return;
    }

    if apply_prefixed(s, util, opacity) {
        return;
    }

    // Unknown class — skip silently (same as compile-time).
}

// ---------------------------------------------------------------------------
// apply_exact — mirror of emit_exact
// ---------------------------------------------------------------------------

fn apply_exact(s: &mut StyleRefinement, util: &str) -> bool {
    match util {
        // Display
        "flex" => s.display = Some(Display::Flex),
        "block" => s.display = Some(Display::Block),
        "hidden" => s.display = Some(Display::None),
        "grid" => s.display = Some(Display::Grid),
        "inline-flex" => s.display = Some(Display::Flex),

        // Flex direction
        "flex-row" => s.flex_direction = Some(FlexDirection::Row),
        "flex-col" => s.flex_direction = Some(FlexDirection::Column),
        "flex-row-reverse" => s.flex_direction = Some(FlexDirection::RowReverse),
        "flex-col-reverse" => s.flex_direction = Some(FlexDirection::ColumnReverse),

        // Flex wrap
        "flex-wrap" => s.flex_wrap = Some(FlexWrap::Wrap),
        "flex-nowrap" => s.flex_wrap = Some(FlexWrap::NoWrap),
        "flex-wrap-reverse" => s.flex_wrap = Some(FlexWrap::WrapReverse),

        // Flex
        "flex-1" => {
            s.flex_grow = Some(1f32);
            s.flex_shrink = Some(1f32);
            s.flex_basis = Some(Length::from(px(0.)));
        }
        "flex-auto" => {
            s.flex_grow = Some(1f32);
            s.flex_shrink = Some(1f32);
            s.flex_basis = Some(Length::Auto);
        }
        "flex-none" => {
            s.flex_grow = Some(0f32);
            s.flex_shrink = Some(0f32);
            s.flex_basis = Some(Length::Auto);
        }
        "flex-grow" => s.flex_grow = Some(1f32),
        "flex-grow-0" => s.flex_grow = Some(0f32),
        "flex-shrink" => s.flex_shrink = Some(1f32),
        "flex-shrink-0" => s.flex_shrink = Some(0f32),

        // Justify content
        "justify-center" => s.justify_content = Some(gpui::JustifyContent::Center),
        "justify-start" => s.justify_content = Some(gpui::JustifyContent::FlexStart),
        "justify-end" => s.justify_content = Some(gpui::JustifyContent::FlexEnd),
        "justify-between" => s.justify_content = Some(gpui::JustifyContent::SpaceBetween),
        "justify-around" => s.justify_content = Some(gpui::JustifyContent::SpaceAround),
        "justify-evenly" => s.justify_content = Some(gpui::JustifyContent::SpaceEvenly),

        // Align items
        "items-center" => s.align_items = Some(gpui::AlignItems::Center),
        "items-start" => s.align_items = Some(gpui::AlignItems::FlexStart),
        "items-end" => s.align_items = Some(gpui::AlignItems::FlexEnd),
        "items-baseline" => s.align_items = Some(gpui::AlignItems::Baseline),
        "items-stretch" => s.align_items = Some(gpui::AlignItems::Stretch),

        // Align self
        "self-center" => s.align_self = Some(gpui::AlignSelf::Center),
        "self-start" => s.align_self = Some(gpui::AlignSelf::FlexStart),
        "self-end" => s.align_self = Some(gpui::AlignSelf::FlexEnd),
        "self-stretch" => s.align_self = Some(gpui::AlignSelf::Stretch),
        "self-baseline" => s.align_self = Some(gpui::AlignSelf::Baseline),

        // Align content
        "content-center" => s.align_content = Some(gpui::AlignContent::Center),
        "content-start" => s.align_content = Some(gpui::AlignContent::FlexStart),
        "content-end" => s.align_content = Some(gpui::AlignContent::FlexEnd),
        "content-between" => s.align_content = Some(gpui::AlignContent::SpaceBetween),
        "content-around" => s.align_content = Some(gpui::AlignContent::SpaceAround),
        "content-stretch" => s.align_content = Some(gpui::AlignContent::Stretch),
        "content-evenly" => s.align_content = Some(gpui::AlignContent::SpaceEvenly),

        // Position
        "relative" => s.position = Some(Position::Relative),
        "absolute" => s.position = Some(Position::Absolute),
        "static" => s.position = Some(Position::Relative),

        // Overflow
        "overflow-hidden" => {
            s.overflow.x = Some(Overflow::Hidden);
            s.overflow.y = Some(Overflow::Hidden);
        }
        "overflow-scroll" => {
            s.overflow.x = Some(Overflow::Scroll);
            s.overflow.y = Some(Overflow::Scroll);
        }
        "overflow-auto" => {
            s.overflow.x = Some(Overflow::Scroll);
            s.overflow.y = Some(Overflow::Scroll);
        }
        "overflow-visible" => {
            s.overflow.x = Some(Overflow::Visible);
            s.overflow.y = Some(Overflow::Visible);
        }
        "overflow-x-hidden" => s.overflow.x = Some(Overflow::Hidden),
        "overflow-x-scroll" => s.overflow.x = Some(Overflow::Scroll),
        "overflow-x-auto" => s.overflow.x = Some(Overflow::Scroll),
        "overflow-x-visible" => s.overflow.x = Some(Overflow::Visible),
        "overflow-y-hidden" => s.overflow.y = Some(Overflow::Hidden),
        "overflow-y-scroll" => s.overflow.y = Some(Overflow::Scroll),
        "overflow-y-auto" => s.overflow.y = Some(Overflow::Scroll),
        "overflow-y-visible" => s.overflow.y = Some(Overflow::Visible),

        // Visibility
        "invisible" => s.visibility = Some(gpui::Visibility::Hidden),
        "visible" => s.visibility = Some(gpui::Visibility::Visible),

        // Font style
        "italic" => s.text.font_style = Some(FontStyle::Italic),
        "not-italic" => s.text.font_style = Some(FontStyle::Normal),

        // Text decoration
        "underline" => {
            s.text.underline = Some(gpui::UnderlineStyle {
                thickness: px(1.),
                ..Default::default()
            });
        }
        "line-through" => {
            s.text.strikethrough = Some(gpui::StrikethroughStyle {
                thickness: px(1.),
                ..Default::default()
            });
        }
        "no-underline" => {
            s.text.underline = None;
            s.text.strikethrough = None;
        }

        // White space
        "whitespace-normal" => s.text.white_space = Some(WhiteSpace::Normal),
        "whitespace-nowrap" => s.text.white_space = Some(WhiteSpace::Nowrap),

        // Border style
        "border-solid" => s.border_style = Some(gpui::BorderStyle::Solid),
        "border-dashed" => s.border_style = Some(gpui::BorderStyle::Dashed),

        // Border width (no value = 1px)
        "border" => {
            let w: AbsoluteLength = px(1.).into();
            s.border_widths.top = Some(w);
            s.border_widths.right = Some(w);
            s.border_widths.bottom = Some(w);
            s.border_widths.left = Some(w);
        }
        "border-t" => s.border_widths.top = Some(px(1.).into()),
        "border-r" => s.border_widths.right = Some(px(1.).into()),
        "border-b" => s.border_widths.bottom = Some(px(1.).into()),
        "border-l" => s.border_widths.left = Some(px(1.).into()),

        // Shadow
        "shadow-sm" => s.box_shadow = Some(shadow_data("sm").unwrap_or_default()),
        "shadow" => s.box_shadow = Some(shadow_data("").unwrap_or_default()),
        "shadow-md" => s.box_shadow = Some(shadow_data("md").unwrap_or_default()),
        "shadow-lg" => s.box_shadow = Some(shadow_data("lg").unwrap_or_default()),
        "shadow-xl" => s.box_shadow = Some(shadow_data("xl").unwrap_or_default()),
        "shadow-2xl" => s.box_shadow = Some(shadow_data("2xl").unwrap_or_default()),
        "shadow-none" => s.box_shadow = Some(shadow_data("none").unwrap_or_default()),

        // Rounded (no value = 4px)
        "rounded" => apply_rounded_all(s, 4.0),
        "rounded-sm" => apply_rounded_all(s, 2.0),
        "rounded-md" => apply_rounded_all(s, 6.0),
        "rounded-lg" => apply_rounded_all(s, 8.0),
        "rounded-xl" => apply_rounded_all(s, 12.0),
        "rounded-2xl" => apply_rounded_all(s, 16.0),
        "rounded-3xl" => apply_rounded_all(s, 24.0),
        "rounded-full" => apply_rounded_all(s, 9999.0),
        "rounded-none" => apply_rounded_all(s, 0.0),

        // Cursor
        "cursor-pointer" => s.mouse_cursor = Some(CursorStyle::PointingHand),
        "cursor-default" => s.mouse_cursor = Some(CursorStyle::Arrow),
        "cursor-text" => s.mouse_cursor = Some(CursorStyle::IBeam),
        "cursor-not-allowed" => s.mouse_cursor = Some(CursorStyle::OperationNotAllowed),
        "cursor-grab" => s.mouse_cursor = Some(CursorStyle::OpenHand),
        "cursor-grabbing" => s.mouse_cursor = Some(CursorStyle::ClosedHand),
        "cursor-crosshair" => s.mouse_cursor = Some(CursorStyle::Crosshair),

        // Font weight
        "font-thin" => s.text.font_weight = Some(FontWeight::THIN),
        "font-light" => s.text.font_weight = Some(FontWeight::LIGHT),
        "font-normal" => s.text.font_weight = Some(FontWeight::NORMAL),
        "font-medium" => s.text.font_weight = Some(FontWeight::MEDIUM),
        "font-semibold" => s.text.font_weight = Some(FontWeight::SEMIBOLD),
        "font-bold" => s.text.font_weight = Some(FontWeight::BOLD),
        "font-extrabold" => s.text.font_weight = Some(FontWeight::EXTRA_BOLD),
        "font-black" => s.text.font_weight = Some(FontWeight::BLACK),

        // Text align
        "text-left" => s.text.text_align = Some(TextAlign::Left),
        "text-center" => s.text.text_align = Some(TextAlign::Center),
        "text-right" => s.text.text_align = Some(TextAlign::Right),

        // Text size
        "text-xs" => apply_font_size(s, 12.0),
        "text-sm" => apply_font_size(s, 14.0),
        "text-base" => apply_font_size(s, 16.0),
        "text-lg" => apply_font_size(s, 18.0),
        "text-xl" => apply_font_size(s, 20.0),
        "text-2xl" => apply_font_size(s, 24.0),
        "text-3xl" => apply_font_size(s, 30.0),
        "text-4xl" => apply_font_size(s, 36.0),
        "text-5xl" => apply_font_size(s, 48.0),
        "text-6xl" => apply_font_size(s, 60.0),
        "text-7xl" => apply_font_size(s, 72.0),
        "text-8xl" => apply_font_size(s, 96.0),
        "text-9xl" => apply_font_size(s, 128.0),

        // Width
        "w-full" => s.size.width = Some(Length::from(relative(1.))),
        "w-auto" => s.size.width = Some(Length::Auto),
        "w-fit" => s.size.width = Some(Length::Auto),
        "w-screen" => s.size.width = Some(Length::from(relative(1.))),

        // Height
        "h-full" => s.size.height = Some(Length::from(relative(1.))),
        "h-auto" => s.size.height = Some(Length::Auto),
        "h-fit" => s.size.height = Some(Length::Auto),
        "h-screen" => s.size.height = Some(Length::from(relative(1.))),

        // Min width
        "min-w-full" => s.min_size.width = Some(Length::from(relative(1.))),
        "min-w-auto" => s.min_size.width = Some(Length::Auto),

        // Min height
        "min-h-full" => s.min_size.height = Some(Length::from(relative(1.))),
        "min-h-auto" => s.min_size.height = Some(Length::Auto),

        // Max width
        "max-w-full" => s.max_size.width = Some(Length::from(relative(1.))),
        "max-w-none" => s.max_size.width = Some(Length::Auto),

        // Max height
        "max-h-full" => s.max_size.height = Some(Length::from(relative(1.))),
        "max-h-none" => s.max_size.height = Some(Length::Auto),

        // Margin auto
        "m-auto" => {
            s.margin.top = Some(Length::Auto);
            s.margin.right = Some(Length::Auto);
            s.margin.bottom = Some(Length::Auto);
            s.margin.left = Some(Length::Auto);
        }
        "mx-auto" => {
            s.margin.left = Some(Length::Auto);
            s.margin.right = Some(Length::Auto);
        }
        "my-auto" => {
            s.margin.top = Some(Length::Auto);
            s.margin.bottom = Some(Length::Auto);
        }
        "mt-auto" => s.margin.top = Some(Length::Auto),
        "mr-auto" => s.margin.right = Some(Length::Auto),
        "mb-auto" => s.margin.bottom = Some(Length::Auto),
        "ml-auto" => s.margin.left = Some(Length::Auto),

        // Inset
        "inset-0" => {
            s.inset.top = Some(Length::from(px(0.)));
            s.inset.right = Some(Length::from(px(0.)));
            s.inset.bottom = Some(Length::from(px(0.)));
            s.inset.left = Some(Length::from(px(0.)));
        }
        "inset-auto" => {
            s.inset.top = Some(Length::Auto);
            s.inset.right = Some(Length::Auto);
            s.inset.bottom = Some(Length::Auto);
            s.inset.left = Some(Length::Auto);
        }

        // Colors - black/white/transparent
        "bg-black" => s.background = Some(Fill::from(black())),
        "bg-white" => s.background = Some(Fill::from(white())),
        "bg-transparent" => {}
        "text-black" => s.text.color = Some(black()),
        "text-white" => s.text.color = Some(white()),
        "text-transparent" => {}
        "border-black" => s.border_color = Some(black()),
        "border-white" => s.border_color = Some(white()),
        "border-transparent" => {}

        // Text overflow
        "truncate" => {
            s.overflow.x = Some(Overflow::Hidden);
            s.overflow.y = Some(Overflow::Hidden);
            s.text.white_space = Some(WhiteSpace::Nowrap);
            s.text.text_overflow =
                Some(TextOverflow::Truncate(SharedString::new_static("…")));
        }
        "text-ellipsis" => {
            s.text.text_overflow =
                Some(TextOverflow::Truncate(SharedString::new_static("…")));
        }
        "text-clip" => {
            s.text.text_overflow = Some(TextOverflow::Truncate(SharedString::new_static("")));
        }

        // Font family
        "font-mono" => s.text.font_family = Some(SharedString::from("monospace")),
        "font-sans" => s.text.font_family = Some(SharedString::from("sans-serif")),
        "font-serif" => s.text.font_family = Some(SharedString::from("serif")),

        // Line height
        "leading-none" => s.text.line_height = Some(DefiniteLength::from(px(1.))),
        "leading-tight" => s.text.line_height = Some(DefiniteLength::from(relative(1.25))),
        "leading-normal" => s.text.line_height = Some(DefiniteLength::from(relative(1.5))),
        "leading-loose" => s.text.line_height = Some(DefiniteLength::from(relative(2.))),

        // Text decoration style
        "decoration-solid" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.wavy = false;
            }
        }
        "decoration-wavy" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.wavy = true;
            }
        }
        "decoration-none" => {
            s.text.underline = None;
            s.text.strikethrough = None;
        }

        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------------
// apply_prefixed — mirror of emit_prefixed
// ---------------------------------------------------------------------------

fn apply_prefixed(s: &mut StyleRefinement, util: &str, opacity: Option<u8>) -> bool {
    if apply_two_part_prefix(s, util, opacity) {
        return true;
    }

    let (prefix, rest) = match util.split_once('-') {
        Some(v) => v,
        None => return false,
    };

    let matched = match prefix {
        "p" => apply_padding_box(s, rest),
        "px" => apply_padding_inline(s, rest),
        "py" => apply_padding_block(s, rest),
        "pt" => apply_padding_edge(s, "top", rest),
        "pr" => apply_padding_edge(s, "right", rest),
        "pb" => apply_padding_edge(s, "bottom", rest),
        "pl" => apply_padding_edge(s, "left", rest),
        "ps" => apply_padding_edge(s, "left", rest),
        "pe" => apply_padding_edge(s, "right", rest),

        "m" => apply_margin_box(s, rest),
        "mx" => apply_margin_inline(s, rest),
        "my" => apply_margin_block(s, rest),
        "mt" => apply_margin_edge(s, "top", rest),
        "mr" => apply_margin_edge(s, "right", rest),
        "mb" => apply_margin_edge(s, "bottom", rest),
        "ml" => apply_margin_edge(s, "left", rest),
        "ms" => apply_margin_edge(s, "left", rest),
        "me" => apply_margin_edge(s, "right", rest),

        "gap" => apply_gap(s, rest),
        "w" => apply_width(s, rest),
        "h" => apply_height(s, rest),
        "bg" => apply_background(s, rest, opacity),
        "text" => apply_text(s, rest, opacity),
        "border" => apply_border_prefixed(s, rest, opacity),
        "rounded" => apply_rounded_prefixed(s, rest),
        "opacity" => apply_opacity(s, rest),
        "shadow" => {
            if let Some(shadows) = shadow_data(rest) {
                s.box_shadow = Some(shadows);
                true
            } else {
                false
            }
        }
        "cursor" => apply_cursor(s, rest),
        "font" => apply_font_weight_prefixed(s, rest),
        "inset" => apply_inset(s, rest),
        "top" => apply_inset_edge(s, "top", rest),
        "right" => apply_inset_edge(s, "right", rest),
        "bottom" => apply_inset_edge(s, "bottom", rest),
        "left" => apply_inset_edge(s, "left", rest),
        "grid" => apply_grid(s, rest),
        "col" => apply_col(s, rest),
        "row" => apply_row(s, rest),
        "min" => apply_min_max(s, "min", rest),
        "max" => apply_min_max(s, "max", rest),
        "aspect" => apply_aspect(s, rest),
        "line" => apply_line_clamp(s, rest),
        "leading" => apply_leading(s, rest),
        "decoration" => apply_decoration(s, rest),
        "z" => apply_z_index(s, rest),

        _ => return false,
    };
    matched
}

// ---------------------------------------------------------------------------
// apply_two_part_prefix — mirror of emit_two_part_prefix
// ---------------------------------------------------------------------------

fn apply_two_part_prefix(s: &mut StyleRefinement, util: &str, _opacity: Option<u8>) -> bool {
    // border-t, border-r, border-b, border-l (+ optional width)
    for (prefix, field) in [
        ("border-t", "top"),
        ("border-r", "right"),
        ("border-b", "bottom"),
        ("border-l", "left"),
    ] {
        if util == prefix {
            set_border_width_edge(s, field, 1.0);
            return true;
        }
        if let Some(rest) = util.strip_prefix(&format!("{}-", prefix)) {
            if let Some(w) = border_width_value(rest) {
                set_border_width_edge(s, field, w);
                return true;
            }
        }
    }

    // rounded-tl, rounded-tr, rounded-bl, rounded-br (single corner)
    for (prefix, field) in [
        ("rounded-tl", "top_left"),
        ("rounded-tr", "top_right"),
        ("rounded-br", "bottom_right"),
        ("rounded-bl", "bottom_left"),
    ] {
        if util == prefix {
            set_corner(s, field, 4.0);
            return true;
        }
        if let Some(rest) = util.strip_prefix(&format!("{}-", prefix)) {
            if let Some(r) = border_radius_value(rest) {
                set_corner(s, field, r);
                return true;
            }
        }
    }

    // rounded-t, rounded-r, rounded-b, rounded-l (two corners each)
    if let Some(rest) = util.strip_prefix("rounded-t-") {
        if let Some(r) = border_radius_value(rest) {
            apply_rounded_top(s, r);
            return true;
        }
    }
    if util == "rounded-t" {
        apply_rounded_top(s, 4.0);
        return true;
    }
    if let Some(rest) = util.strip_prefix("rounded-r-") {
        if let Some(r) = border_radius_value(rest) {
            apply_rounded_right(s, r);
            return true;
        }
    }
    if util == "rounded-r" {
        apply_rounded_right(s, 4.0);
        return true;
    }
    if let Some(rest) = util.strip_prefix("rounded-b-") {
        if let Some(r) = border_radius_value(rest) {
            apply_rounded_bottom(s, r);
            return true;
        }
    }
    if util == "rounded-b" {
        apply_rounded_bottom(s, 4.0);
        return true;
    }
    if let Some(rest) = util.strip_prefix("rounded-l-") {
        if let Some(r) = border_radius_value(rest) {
            apply_rounded_left(s, r);
            return true;
        }
    }
    if util == "rounded-l" {
        apply_rounded_left(s, 4.0);
        return true;
    }

    // min-w-*, min-h-*, max-w-*, max-h-*
    if let Some(rest) = util.strip_prefix("min-w-") {
        return apply_min_size(s, "width", rest);
    }
    if let Some(rest) = util.strip_prefix("min-h-") {
        return apply_min_size(s, "height", rest);
    }
    if let Some(rest) = util.strip_prefix("max-w-") {
        return apply_max_size(s, "width", rest);
    }
    if let Some(rest) = util.strip_prefix("max-h-") {
        return apply_max_size(s, "height", rest);
    }

    // gap-x-*, gap-y-*
    if let Some(rest) = util.strip_prefix("gap-x-") {
        return apply_gap_axis(s, "width", rest);
    }
    if let Some(rest) = util.strip_prefix("gap-y-") {
        return apply_gap_axis(s, "height", rest);
    }

    // flex-grow-*, flex-shrink-*
    if let Some(rest) = util.strip_prefix("flex-grow-") {
        if let Ok(n) = rest.parse::<f32>() {
            s.flex_grow = Some(n);
            return true;
        }
    }
    if let Some(rest) = util.strip_prefix("flex-shrink-") {
        if let Ok(n) = rest.parse::<f32>() {
            s.flex_shrink = Some(n);
            return true;
        }
    }

    false
}

// ---------------------------------------------------------------------------
// Helper apply functions — direct-field-assignment mirrors of emit_* helpers
// ---------------------------------------------------------------------------

fn def_px(n: f32) -> DefiniteLength {
    DefiniteLength::from(px(n))
}

fn len_px(n: f32) -> Length {
    Length::from(px(n))
}

fn apply_padding_box(s: &mut StyleRefinement, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    s.padding.top = Some(v);
    s.padding.right = Some(v);
    s.padding.bottom = Some(v);
    s.padding.left = Some(v);
    true
}

fn apply_padding_inline(s: &mut StyleRefinement, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    s.padding.left = Some(v);
    s.padding.right = Some(v);
    true
}

fn apply_padding_block(s: &mut StyleRefinement, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    s.padding.top = Some(v);
    s.padding.bottom = Some(v);
    true
}

fn apply_padding_edge(s: &mut StyleRefinement, edge: &str, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    match edge {
        "top" => s.padding.top = Some(v),
        "right" => s.padding.right = Some(v),
        "bottom" => s.padding.bottom = Some(v),
        "left" => s.padding.left = Some(v),
        _ => return false,
    }
    true
}

fn apply_margin_box(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "auto" {
        s.margin.top = Some(Length::Auto);
        s.margin.right = Some(Length::Auto);
        s.margin.bottom = Some(Length::Auto);
        s.margin.left = Some(Length::Auto);
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    s.margin.top = Some(v);
    s.margin.right = Some(v);
    s.margin.bottom = Some(v);
    s.margin.left = Some(v);
    true
}

fn apply_margin_inline(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "auto" {
        s.margin.left = Some(Length::Auto);
        s.margin.right = Some(Length::Auto);
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    s.margin.left = Some(v);
    s.margin.right = Some(v);
    true
}

fn apply_margin_block(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "auto" {
        s.margin.top = Some(Length::Auto);
        s.margin.bottom = Some(Length::Auto);
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    s.margin.top = Some(v);
    s.margin.bottom = Some(v);
    true
}

fn apply_margin_edge(s: &mut StyleRefinement, edge: &str, rest: &str) -> bool {
    if rest == "auto" {
        match edge {
            "top" => s.margin.top = Some(Length::Auto),
            "right" => s.margin.right = Some(Length::Auto),
            "bottom" => s.margin.bottom = Some(Length::Auto),
            "left" => s.margin.left = Some(Length::Auto),
            _ => return false,
        }
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    match edge {
        "top" => s.margin.top = Some(v),
        "right" => s.margin.right = Some(v),
        "bottom" => s.margin.bottom = Some(v),
        "left" => s.margin.left = Some(v),
        _ => return false,
    }
    true
}

fn apply_gap(s: &mut StyleRefinement, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    s.gap.width = Some(v);
    s.gap.height = Some(v);
    true
}

fn apply_gap_axis(s: &mut StyleRefinement, axis: &str, rest: &str) -> bool {
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = def_px(n);
    match axis {
        "width" => s.gap.width = Some(v),
        "height" => s.gap.height = Some(v),
        _ => return false,
    }
    true
}

fn apply_width(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "full" {
        s.size.width = Some(Length::from(relative(1.)));
        return true;
    }
    if rest == "auto" || rest == "fit" {
        s.size.width = Some(Length::Auto);
        return true;
    }
    if rest == "screen" {
        s.size.width = Some(Length::from(relative(1.)));
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    s.size.width = Some(len_px(n));
    true
}

fn apply_height(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "full" {
        s.size.height = Some(Length::from(relative(1.)));
        return true;
    }
    if rest == "auto" || rest == "fit" {
        s.size.height = Some(Length::Auto);
        return true;
    }
    if rest == "screen" {
        s.size.height = Some(Length::from(relative(1.)));
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    s.size.height = Some(len_px(n));
    true
}

fn apply_background(s: &mut StyleRefinement, rest: &str, opacity: Option<u8>) -> bool {
    let color = match apply_color_value(rest, opacity) {
        Some(c) => c,
        None => return false,
    };
    s.background = Some(Fill::from(color));
    true
}

fn apply_text(s: &mut StyleRefinement, rest: &str, opacity: Option<u8>) -> bool {
    // Text align
    match rest {
        "left" => {
            s.text.text_align = Some(TextAlign::Left);
            return true;
        }
        "center" => {
            s.text.text_align = Some(TextAlign::Center);
            return true;
        }
        "right" => {
            s.text.text_align = Some(TextAlign::Right);
            return true;
        }
        "justify" => {
            s.text.text_align = Some(TextAlign::Left);
            return true;
        }
        _ => {}
    }

    // Font size
    if let Some((size, _)) = font_size_value(rest) {
        apply_font_size(s, size);
        return true;
    }

    // Text color
    if let Some(color) = apply_color_value(rest, opacity) {
        s.text.color = Some(color);
        return true;
    }

    false
}

fn apply_border_prefixed(s: &mut StyleRefinement, rest: &str, opacity: Option<u8>) -> bool {
    // Border width
    if let Some(w) = border_width_value(rest) {
        let a: AbsoluteLength = px(w).into();
        s.border_widths.top = Some(a);
        s.border_widths.right = Some(a);
        s.border_widths.bottom = Some(a);
        s.border_widths.left = Some(a);
        return true;
    }

    // Border color
    if let Some(color) = apply_color_value(rest, opacity) {
        s.border_color = Some(color);
        return true;
    }

    false
}

fn apply_rounded_prefixed(s: &mut StyleRefinement, rest: &str) -> bool {
    let r = match border_radius_value(rest) {
        Some(v) => v,
        None => return false,
    };
    apply_rounded_all(s, r);
    true
}

fn apply_rounded_all(s: &mut StyleRefinement, r: f32) {
    let v: AbsoluteLength = px(r).into();
    s.corner_radii.top_left = Some(v);
    s.corner_radii.top_right = Some(v);
    s.corner_radii.bottom_right = Some(v);
    s.corner_radii.bottom_left = Some(v);
}

fn set_corner(s: &mut StyleRefinement, field: &str, r: f32) {
    let v: AbsoluteLength = px(r).into();
    match field {
        "top_left" => s.corner_radii.top_left = Some(v),
        "top_right" => s.corner_radii.top_right = Some(v),
        "bottom_right" => s.corner_radii.bottom_right = Some(v),
        "bottom_left" => s.corner_radii.bottom_left = Some(v),
        _ => {}
    }
}

fn apply_rounded_top(s: &mut StyleRefinement, r: f32) {
    let v: AbsoluteLength = px(r).into();
    s.corner_radii.top_left = Some(v);
    s.corner_radii.top_right = Some(v);
}

fn apply_rounded_right(s: &mut StyleRefinement, r: f32) {
    let v: AbsoluteLength = px(r).into();
    s.corner_radii.top_right = Some(v);
    s.corner_radii.bottom_right = Some(v);
}

fn apply_rounded_bottom(s: &mut StyleRefinement, r: f32) {
    let v: AbsoluteLength = px(r).into();
    s.corner_radii.bottom_right = Some(v);
    s.corner_radii.bottom_left = Some(v);
}

fn apply_rounded_left(s: &mut StyleRefinement, r: f32) {
    let v: AbsoluteLength = px(r).into();
    s.corner_radii.top_left = Some(v);
    s.corner_radii.bottom_left = Some(v);
}

fn set_border_width_edge(s: &mut StyleRefinement, field: &str, w: f32) {
    let a: AbsoluteLength = px(w).into();
    match field {
        "top" => s.border_widths.top = Some(a),
        "right" => s.border_widths.right = Some(a),
        "bottom" => s.border_widths.bottom = Some(a),
        "left" => s.border_widths.left = Some(a),
        _ => {}
    }
}

fn apply_opacity(s: &mut StyleRefinement, rest: &str) -> bool {
    let n: f32 = match rest.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    s.opacity = Some(n / 100.0);
    true
}

fn apply_cursor(s: &mut StyleRefinement, rest: &str) -> bool {
    match rest {
        "pointer" => s.mouse_cursor = Some(CursorStyle::PointingHand),
        "default" => s.mouse_cursor = Some(CursorStyle::Arrow),
        "text" => s.mouse_cursor = Some(CursorStyle::IBeam),
        "not-allowed" => s.mouse_cursor = Some(CursorStyle::OperationNotAllowed),
        "grab" => s.mouse_cursor = Some(CursorStyle::OpenHand),
        "grabbing" => s.mouse_cursor = Some(CursorStyle::ClosedHand),
        "crosshair" => s.mouse_cursor = Some(CursorStyle::Crosshair),
        _ => return false,
    }
    true
}

fn apply_font_weight_prefixed(s: &mut StyleRefinement, rest: &str) -> bool {
    match rest {
        "thin" => s.text.font_weight = Some(FontWeight::THIN),
        "light" => s.text.font_weight = Some(FontWeight::LIGHT),
        "normal" => s.text.font_weight = Some(FontWeight::NORMAL),
        "medium" => s.text.font_weight = Some(FontWeight::MEDIUM),
        "semibold" => s.text.font_weight = Some(FontWeight::SEMIBOLD),
        "bold" => s.text.font_weight = Some(FontWeight::BOLD),
        "extrabold" => s.text.font_weight = Some(FontWeight::EXTRA_BOLD),
        "black" => s.text.font_weight = Some(FontWeight::BLACK),
        _ => return false,
    }
    true
}

fn apply_font_size(s: &mut StyleRefinement, size: f32) {
    s.text.font_size = Some(AbsoluteLength::from(px(size)));
}

fn apply_inset(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "auto" {
        s.inset.top = Some(Length::Auto);
        s.inset.right = Some(Length::Auto);
        s.inset.bottom = Some(Length::Auto);
        s.inset.left = Some(Length::Auto);
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    s.inset.top = Some(v);
    s.inset.right = Some(v);
    s.inset.bottom = Some(v);
    s.inset.left = Some(v);
    true
}

fn apply_inset_edge(s: &mut StyleRefinement, edge: &str, rest: &str) -> bool {
    if rest == "auto" {
        match edge {
            "top" => s.inset.top = Some(Length::Auto),
            "right" => s.inset.right = Some(Length::Auto),
            "bottom" => s.inset.bottom = Some(Length::Auto),
            "left" => s.inset.left = Some(Length::Auto),
            _ => return false,
        }
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    match edge {
        "top" => s.inset.top = Some(v),
        "right" => s.inset.right = Some(v),
        "bottom" => s.inset.bottom = Some(v),
        "left" => s.inset.left = Some(v),
        _ => return false,
    }
    true
}

fn apply_grid(s: &mut StyleRefinement, rest: &str) -> bool {
    if let Some(rest) = rest.strip_prefix("cols-") {
        if let Ok(n) = rest.parse::<u16>() {
            s.grid_cols = Some(gpui::GridTemplate {
                repeat: n,
                ..Default::default()
            });
            return true;
        }
    }
    if let Some(rest) = rest.strip_prefix("rows-") {
        if let Ok(n) = rest.parse::<u16>() {
            s.grid_rows = Some(gpui::GridTemplate {
                repeat: n,
                ..Default::default()
            });
            return true;
        }
    }
    false
}

fn apply_col(_s: &mut StyleRefinement, rest: &str) -> bool {
    if let Some(rest) = rest.strip_prefix("span-") {
        if rest.parse::<u16>().is_ok() {
            return true;
        }
    }
    false
}
fn apply_row(_s: &mut StyleRefinement, rest: &str) -> bool {
    if let Some(rest) = rest.strip_prefix("span-") {
        if rest.parse::<u16>().is_ok() {
            return true;
        }
    }
    false
}

fn apply_min_max(s: &mut StyleRefinement, kind: &str, rest: &str) -> bool {
    if let Some(rest) = rest.strip_prefix("w-") {
        return if kind == "min" {
            apply_min_size(s, "width", rest)
        } else {
            apply_max_size(s, "width", rest)
        };
    }
    if let Some(rest) = rest.strip_prefix("h-") {
        return if kind == "min" {
            apply_min_size(s, "height", rest)
        } else {
            apply_max_size(s, "height", rest)
        };
    }
    false
}

fn apply_min_size(s: &mut StyleRefinement, axis: &str, rest: &str) -> bool {
    if rest == "auto" {
        match axis {
            "width" => s.min_size.width = Some(Length::Auto),
            "height" => s.min_size.height = Some(Length::Auto),
            _ => return false,
        }
        return true;
    }
    if rest == "full" {
        match axis {
            "width" => s.min_size.width = Some(Length::from(relative(1.))),
            "height" => s.min_size.height = Some(Length::from(relative(1.))),
            _ => return false,
        }
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    match axis {
        "width" => s.min_size.width = Some(v),
        "height" => s.min_size.height = Some(v),
        _ => return false,
    }
    true
}

fn apply_max_size(s: &mut StyleRefinement, axis: &str, rest: &str) -> bool {
    if rest == "none" {
        match axis {
            "width" => s.max_size.width = Some(Length::Auto),
            "height" => s.max_size.height = Some(Length::Auto),
            _ => return false,
        }
        return true;
    }
    if rest == "full" {
        match axis {
            "width" => s.max_size.width = Some(Length::from(relative(1.))),
            "height" => s.max_size.height = Some(Length::from(relative(1.))),
            _ => return false,
        }
        return true;
    }
    let n = match spacing_value(rest) {
        Some(v) => v,
        None => return false,
    };
    let v = len_px(n);
    match axis {
        "width" => s.max_size.width = Some(v),
        "height" => s.max_size.height = Some(v),
        _ => return false,
    }
    true
}

fn apply_aspect(s: &mut StyleRefinement, rest: &str) -> bool {
    if rest == "square" {
        s.aspect_ratio = Some(1f32);
        return true;
    }
    if rest == "video" {
        s.aspect_ratio = Some(1.7777777f32);
        return true;
    }
    if let Some((w, h)) = rest.split_once('/') {
        if let (Ok(w), Ok(h)) = (w.parse::<f32>(), h.parse::<f32>()) {
            s.aspect_ratio = Some(w / h);
            return true;
        }
    }
    false
}

fn apply_line_clamp(s: &mut StyleRefinement, rest: &str) -> bool {
    if let Some(rest) = rest.strip_prefix("clamp-") {
        if let Ok(n) = rest.parse::<usize>() {
            s.text.line_clamp = Some(n);
            return true;
        }
    }
    false
}

fn apply_z_index(_s: &mut StyleRefinement, rest: &str) -> bool {
    // gpui doesn't have z-index in StyleRefinement — no-op (mirrors compile-time).
    let _n: i32 = rest.parse().unwrap_or(0);
    false
}

fn apply_leading(s: &mut StyleRefinement, rest: &str) -> bool {
    match rest {
        "3" => s.text.line_height = Some(DefiniteLength::from(relative(0.75))),
        "4" => s.text.line_height = Some(DefiniteLength::from(relative(1.0))),
        "5" => s.text.line_height = Some(DefiniteLength::from(relative(1.25))),
        "6" => s.text.line_height = Some(DefiniteLength::from(relative(1.5))),
        "7" => s.text.line_height = Some(DefiniteLength::from(relative(1.75))),
        "8" => s.text.line_height = Some(DefiniteLength::from(relative(2.0))),
        "9" => s.text.line_height = Some(DefiniteLength::from(relative(2.25))),
        "10" => s.text.line_height = Some(DefiniteLength::from(relative(2.5))),
        _ => {
            // Try arbitrary value: leading-[20px]
            if let Some(arb) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(len) = parse_arb_length(arb) {
                    // No "line-height" arm in emit_arbitrary_length — known no-op.
                    let _ = len;
                }
            }
            return false;
        }
    }
    true
}

fn apply_decoration(s: &mut StyleRefinement, rest: &str) -> bool {
    match rest {
        "0" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.thickness = px(0.);
            }
        }
        "1" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.thickness = px(1.);
            }
        }
        "2" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.thickness = px(2.);
            }
        }
        "4" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.thickness = px(4.);
            }
        }
        "8" => {
            if let Some(u) = s.text.underline.as_mut() {
                u.thickness = px(8.);
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------------
// apply_color_value — mirror of emit_color_value
// ---------------------------------------------------------------------------

fn apply_color_value(rest: &str, opacity: Option<u8>) -> Option<Hsla> {
    // black/white/transparent
    match rest {
        "black" => {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = 0x00000000 | alpha;
                return Some(rgba(packed).into());
            }
            return Some(black());
        }
        "white" => {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = 0xFFFFFF00 | alpha;
                return Some(rgba(packed).into());
            }
            return Some(white());
        }
        "transparent" => return Some(rgba(0).into()),
        _ => {}
    }

    // Try color-shade (e.g., "red-500", "blue-400")
    if let Some((color, shade)) = rest.split_once('-') {
        if let Some((r, g, b)) = color_rgb(color, Some(shade)) {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | alpha;
                return Some(rgba(packed).into());
            }
            let packed = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            return Some(rgb(packed).into());
        }
    }

    None
}

// ---------------------------------------------------------------------------
// Arbitrary value parsing (runtime)
// ---------------------------------------------------------------------------

/// Runtime mirror of the compile-time arbitrary length token. Represents a
/// parsed length from `[...]` that can be converted to the appropriate gpui
/// length type.
enum ArbLength {
    Px(f32),
    Rem(f32),
    Percent(f32),
}

impl ArbLength {
    fn into_length(self) -> Length {
        match self {
            ArbLength::Px(n) => Length::from(px(n)),
            ArbLength::Rem(n) => Length::from(rems(n)),
            ArbLength::Percent(f) => Length::from(relative(f)),
        }
    }

    fn into_definite(self) -> DefiniteLength {
        match self {
            ArbLength::Px(n) => DefiniteLength::from(px(n)),
            ArbLength::Rem(n) => DefiniteLength::from(rems(n)),
            ArbLength::Percent(f) => relative(f),
        }
    }

    fn into_absolute(self) -> AbsoluteLength {
        match self {
            ArbLength::Px(n) => AbsoluteLength::from(px(n)),
            ArbLength::Rem(n) => AbsoluteLength::from(rems(n)),
            // Percent is not absolute — fall back to px(0) (mirrors no-match skip).
            ArbLength::Percent(_) => AbsoluteLength::from(px(0.)),
        }
    }
}

fn parse_arb_length(s: &str) -> Option<ArbLength> {
    let s = s.trim();
    if let Some(rest) = s.strip_suffix("px") {
        let n: f32 = rest.parse().ok()?;
        return Some(ArbLength::Px(n));
    }
    if let Some(rest) = s.strip_suffix("rem") {
        let n: f32 = rest.parse().ok()?;
        return Some(ArbLength::Rem(n));
    }
    if let Some(rest) = s.strip_suffix('%') {
        let n: f32 = rest.parse().ok()?;
        return Some(ArbLength::Percent(n / 100.0));
    }
    // Try bare number (assume px)
    if let Ok(n) = s.parse::<f32>() {
        return Some(ArbLength::Px(n));
    }
    None
}

fn parse_arb_color(s: &str) -> Option<Hsla> {
    let s = s.trim();
    if s.starts_with('#') {
        return parse_hex_color(s);
    }
    if s.starts_with("rgb(") && s.ends_with(')') {
        let inner = &s[4..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() == 3 {
            let r: u32 = parts[0].parse().ok()?;
            let g: u32 = parts[1].parse().ok()?;
            let b: u32 = parts[2].parse().ok()?;
            let packed = (r << 16) | (g << 8) | b;
            return Some(rgb(packed).into());
        }
    }
    if s.starts_with("rgba(") && s.ends_with(')') {
        let inner = &s[5..s.len() - 1];
        let parts: Vec<&str> = inner.split(',').map(|p| p.trim()).collect();
        if parts.len() == 4 {
            let r: u32 = parts[0].parse().ok()?;
            let g: u32 = parts[1].parse().ok()?;
            let b: u32 = parts[2].parse().ok()?;
            let a: f32 = parts[3].parse().ok()?;
            let alpha = (a.clamp(0.0, 1.0) * 255.0).round() as u32;
            let packed = (r << 24) | (g << 16) | (b << 8) | alpha;
            return Some(rgba(packed).into());
        }
    }
    None
}

fn parse_hex_color(s: &str) -> Option<Hsla> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            let v = u32::from_str_radix(&out, 16).ok()?;
            Some(rgb(v).into())
        }
        6 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(rgb(v).into())
        }
        8 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(rgba(v).into())
        }
        _ => None,
    }
}

fn apply_arbitrary(s: &mut StyleRefinement, util: &str, arb: &str, _opacity: Option<u8>) {
    // Try color first (for bg-, text-, border-)
    if util == "bg" || util == "text" || util == "border" {
        if let Some(color) = parse_arb_color(arb) {
            if util == "bg" {
                s.background = Some(Fill::from(color));
            } else if util == "text" {
                s.text.color = Some(color);
            } else {
                s.border_color = Some(color);
            }
            return;
        }
    }

    // Try length (for w, h, p, m, gap, etc.)
    if let Some(len) = parse_arb_length(arb) {
        apply_arb_length(s, util, len);
    }

    // Unknown arbitrary value — skip
}

fn apply_arb_length(s: &mut StyleRefinement, util: &str, len: ArbLength) {
    match util {
        "w" => s.size.width = Some(len.into_length()),
        "h" => s.size.height = Some(len.into_length()),
        "p" => {
            let d = len.into_definite();
            s.padding.top = Some(d);
            s.padding.right = Some(d);
            s.padding.bottom = Some(d);
            s.padding.left = Some(d);
        }
        "px" => {
            let d = len.into_definite();
            s.padding.left = Some(d);
            s.padding.right = Some(d);
        }
        "py" => {
            let d = len.into_definite();
            s.padding.top = Some(d);
            s.padding.bottom = Some(d);
        }
        "pt" => s.padding.top = Some(len.into_definite()),
        "pr" => s.padding.right = Some(len.into_definite()),
        "pb" => s.padding.bottom = Some(len.into_definite()),
        "pl" => s.padding.left = Some(len.into_definite()),
        "m" => {
            let l = len.into_length();
            s.margin.top = Some(l);
            s.margin.right = Some(l);
            s.margin.bottom = Some(l);
            s.margin.left = Some(l);
        }
        "mx" => {
            let l = len.into_length();
            s.margin.left = Some(l);
            s.margin.right = Some(l);
        }
        "my" => {
            let l = len.into_length();
            s.margin.top = Some(l);
            s.margin.bottom = Some(l);
        }
        "mt" => s.margin.top = Some(len.into_length()),
        "mr" => s.margin.right = Some(len.into_length()),
        "mb" => s.margin.bottom = Some(len.into_length()),
        "ml" => s.margin.left = Some(len.into_length()),
        "gap" => {
            let d = len.into_definite();
            s.gap.width = Some(d);
            s.gap.height = Some(d);
        }
        "rounded" => {
            let a = len.into_absolute();
            s.corner_radii.top_left = Some(a);
            s.corner_radii.top_right = Some(a);
            s.corner_radii.bottom_right = Some(a);
            s.corner_radii.bottom_left = Some(a);
        }
        "text" => s.text.font_size = Some(len.into_absolute()),
        "border" => {
            let a = len.into_absolute();
            s.border_widths.top = Some(a);
            s.border_widths.right = Some(a);
            s.border_widths.bottom = Some(a);
            s.border_widths.left = Some(a);
        }
        "min-w" => s.min_size.width = Some(len.into_length()),
        "min-h" => s.min_size.height = Some(len.into_length()),
        "max-w" => s.max_size.width = Some(len.into_length()),
        "max-h" => s.max_size.height = Some(len.into_length()),
        "top" => s.inset.top = Some(len.into_length()),
        "right" => s.inset.right = Some(len.into_length()),
        "bottom" => s.inset.bottom = Some(len.into_length()),
        "left" => s.inset.left = Some(len.into_length()),
        "inset" => {
            let l = len.into_length();
            s.inset.top = Some(l);
            s.inset.right = Some(l);
            s.inset.bottom = Some(l);
            s.inset.left = Some(l);
        }
        // No "line-height" arm — known no-op (mirrors compile-time).
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// shadow_data — runtime mirror of shadow_value, returns Vec<BoxShadow>
// ---------------------------------------------------------------------------

fn shadow_data(s: &str) -> Option<Vec<BoxShadow>> {
    Some(match s {
        "none" => Vec::new(),
        "sm" => vec![BoxShadow {
            color: hsla(0., 0., 0., 0.05),
            offset: point(px(0.), px(1.)),
            blur_radius: px(2.),
            spread_radius: px(0.),
            inset: false,
        }],
        "" => vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(1.)),
                blur_radius: px(3.),
                spread_radius: px(0.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(1.)),
                blur_radius: px(2.),
                spread_radius: px(-1.),
                inset: false,
            },
        ],
        "md" => vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(4.)),
                blur_radius: px(6.),
                spread_radius: px(-1.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(2.)),
                blur_radius: px(4.),
                spread_radius: px(-2.),
                inset: false,
            },
        ],
        "lg" => vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(10.)),
                blur_radius: px(15.),
                spread_radius: px(-3.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(4.)),
                blur_radius: px(6.),
                spread_radius: px(-4.),
                inset: false,
            },
        ],
        "xl" => vec![
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(20.)),
                blur_radius: px(25.),
                spread_radius: px(-5.),
                inset: false,
            },
            BoxShadow {
                color: hsla(0., 0., 0., 0.1),
                offset: point(px(0.), px(8.)),
                blur_radius: px(10.),
                spread_radius: px(-6.),
                inset: false,
            },
        ],
        "2xl" => vec![BoxShadow {
            color: hsla(0., 0., 0., 0.25),
            offset: point(px(0.), px(25.)),
            blur_radius: px(50.),
            spread_radius: px(-12.),
            inset: false,
        }],
        _ => return None,
    })
}

// Suppress unused-import warnings for items used only in specific match arms.
#[allow(unused_imports)]
use gpui::{Background as _, Rgba as _};
