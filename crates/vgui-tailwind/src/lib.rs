//! vgui-tailwind — the `tw!` proc-macro and Tailwind class registry.
//!
//! Expands Tailwind-style utility classes (`flex p-4 bg-[#0000ff]`) into
//! gpui style refinements, with `hover:`/`focus:`/`active:` variant support.
extern crate proc_macro;

mod colors;
mod spacing;

use proc_macro::TokenStream;
use proc_macro2::{TokenStream as TokenStream2, TokenTree};
use quote::quote;
use syn::spanned::Spanned;

#[proc_macro]
pub fn tw(input: TokenStream) -> TokenStream {
    match expand_tw(input.into()) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_tw(input: TokenStream2) -> syn::Result<TokenStream2> {
    let s = extract_string_literal(&input)?;
    let classes: Vec<&str> = s.split_whitespace().collect();
    if classes.is_empty() {
        return Ok(quote! {
            ::vgui::TwStyle {
                base: ::std::boxed::Box::new(|_s: &mut ::gpui::StyleRefinement| {}),
                hover: ::std::option::Option::None,
                focus: ::std::option::Option::None,
                active: ::std::option::Option::None,
            }
        });
    }

    let mut base_stmts = Vec::new();
    let mut hover_stmts = Vec::new();
    let mut focus_stmts = Vec::new();
    let mut active_stmts = Vec::new();

    for class in &classes {
        let parsed = parse_class(class);
        let parsed = match parsed {
            Some(p) => p,
            None => continue, // skip unknown classes silently
        };
        let stmt = emit_class(&parsed)?;
        match parsed.variant {
            Variant::Base => base_stmts.push(stmt),
            Variant::Hover => hover_stmts.push(stmt),
            Variant::Focus => focus_stmts.push(stmt),
            Variant::Active => active_stmts.push(stmt),
        }
    }

    let base = if base_stmts.is_empty() {
        quote! { ::std::boxed::Box::new(|_s: &mut ::gpui::StyleRefinement| {}) }
    } else {
        quote! { ::std::boxed::Box::new(|s: &mut ::gpui::StyleRefinement| { #(#base_stmts)* }) }
    };
    let hover = if hover_stmts.is_empty() {
        quote! { ::std::option::Option::None }
    } else {
        quote! { ::std::option::Option::Some(::std::boxed::Box::new(|s: &mut ::gpui::StyleRefinement| { #(#hover_stmts)* })) }
    };
    let focus = if focus_stmts.is_empty() {
        quote! { ::std::option::Option::None }
    } else {
        quote! { ::std::option::Option::Some(::std::boxed::Box::new(|s: &mut ::gpui::StyleRefinement| { #(#focus_stmts)* })) }
    };
    let active = if active_stmts.is_empty() {
        quote! { ::std::option::Option::None }
    } else {
        quote! { ::std::option::Option::Some(::std::boxed::Box::new(|s: &mut ::gpui::StyleRefinement| { #(#active_stmts)* })) }
    };

    Ok(quote! {
        ::vgui::TwStyle {
            base: #base,
            hover: #hover,
            focus: #focus,
            active: #active,
        }
    })
}

fn extract_string_literal(input: &TokenStream2) -> syn::Result<String> {
    let tokens: Vec<TokenTree> = input.clone().into_iter().collect();
    if tokens.len() == 1 {
        if let TokenTree::Group(g) = &tokens[0] {
            if g.delimiter() == proc_macro2::Delimiter::Brace {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                if inner.len() == 1 {
                    if let TokenTree::Literal(lit) = &inner[0] {
                        let s = lit.to_string();
                        if s.starts_with('"') && s.ends_with('"') {
                            return Ok(parse_rust_string(&s));
                        }
                    }
                }
            }
        }
        if let TokenTree::Literal(lit) = &tokens[0] {
            let s = lit.to_string();
            if s.starts_with('"') && s.ends_with('"') {
                return Ok(parse_rust_string(&s));
            }
        }
    }
    Err(syn::Error::new(
        input.span(),
        "tw! expects a string literal: tw!(\"flex p-4\") or tw! { \"flex p-4\" }",
    ))
}

fn parse_rust_string(s: &str) -> String {
    let inner = &s[1..s.len() - 1];
    let mut out = String::new();
    let mut chars = inner.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some('"') => out.push('"'),
                Some('0') => out.push('\0'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq)]
enum Variant {
    Base,
    Hover,
    Focus,
    Active,
}

struct ParsedClass {
    variant: Variant,
    utility: String,
    arbitrary: Option<String>,
    opacity: Option<u8>,
}

fn parse_class(class: &str) -> Option<ParsedClass> {
    let mut variant = Variant::Base;
    let mut rest = class;

    loop {
        if let Some(pos) = rest.find(':') {
            let prefix = &rest[..pos];
            match prefix {
                "hover" => variant = Variant::Hover,
                "focus" => variant = Variant::Focus,
                "active" => variant = Variant::Active,
                _ => break,
            }
            rest = &rest[pos + 1..];
        } else {
            break;
        }
    }

    // Split on `/` for opacity modifier
    let (utility_part, opacity) = if let Some(pos) = rest.rfind('/') {
        let op_str = &rest[pos + 1..];
        if op_str.chars().all(|c| c.is_ascii_digit()) {
            let op: u8 = op_str.parse().ok()?;
            (rest[..pos].to_string(), Some(op))
        } else {
            (rest.to_string(), None)
        }
    } else {
        (rest.to_string(), None)
    };

    // Extract arbitrary value
    let (utility, arbitrary) = if let Some(start) = utility_part.find('[') {
        if utility_part.ends_with(']') {
            let arb = utility_part[start + 1..utility_part.len() - 1].to_string();
            let util = utility_part[..start].trim_end_matches('-').to_string();
            (util, Some(arb))
        } else {
            (utility_part, None)
        }
    } else {
        (utility_part, None)
    };

    Some(ParsedClass {
        variant,
        utility,
        arbitrary,
        opacity,
    })
}

fn emit_class(cls: &ParsedClass) -> syn::Result<TokenStream2> {
    let util = cls.utility.as_str();
    let arb = cls.arbitrary.as_deref();
    let opacity = cls.opacity;

    // Try arbitrary value first
    if let Some(arb) = arb {
        return emit_arbitrary(util, arb, opacity);
    }

    // Match exact utilities (no value part)
    if let Some(stmt) = emit_exact(util) {
        return Ok(stmt);
    }

    // Match prefixed utilities
    if let Some(stmt) = emit_prefixed(util, opacity) {
        return Ok(stmt);
    }

    // Unknown class - skip silently
    Ok(quote! {})
}

fn emit_exact(util: &str) -> Option<TokenStream2> {
    Some(match util {
        // Display
        "flex" => quote! { s.display = Some(::gpui::Display::Flex); },
        "block" => quote! { s.display = Some(::gpui::Display::Block); },
        "hidden" => quote! { s.display = Some(::gpui::Display::None); },
        "grid" => quote! { s.display = Some(::gpui::Display::Grid); },
        "inline-flex" => quote! { s.display = Some(::gpui::Display::Flex); },

        // Flex direction
        "flex-row" => quote! { s.flex_direction = Some(::gpui::FlexDirection::Row); },
        "flex-col" => quote! { s.flex_direction = Some(::gpui::FlexDirection::Column); },
        "flex-row-reverse" => {
            quote! { s.flex_direction = Some(::gpui::FlexDirection::RowReverse); }
        }
        "flex-col-reverse" => {
            quote! { s.flex_direction = Some(::gpui::FlexDirection::ColumnReverse); }
        }

        // Flex wrap
        "flex-wrap" => quote! { s.flex_wrap = Some(::gpui::FlexWrap::Wrap); },
        "flex-nowrap" => quote! { s.flex_wrap = Some(::gpui::FlexWrap::NoWrap); },
        "flex-wrap-reverse" => quote! { s.flex_wrap = Some(::gpui::FlexWrap::WrapReverse); },

        // Flex
        "flex-1" => quote! {
            s.flex_grow = Some(1f32);
            s.flex_shrink = Some(1f32);
            s.flex_basis = Some(::core::convert::Into::<::gpui::Length>::into(::gpui::px(0.)));
        },
        "flex-auto" => quote! {
            s.flex_grow = Some(1f32);
            s.flex_shrink = Some(1f32);
            s.flex_basis = Some(::gpui::Length::Auto);
        },
        "flex-none" => quote! {
            s.flex_grow = Some(0f32);
            s.flex_shrink = Some(0f32);
            s.flex_basis = Some(::gpui::Length::Auto);
        },
        "flex-grow" => quote! { s.flex_grow = Some(1f32); },
        "flex-grow-0" => quote! { s.flex_grow = Some(0f32); },
        "flex-shrink" => quote! { s.flex_shrink = Some(1f32); },
        "flex-shrink-0" => quote! { s.flex_shrink = Some(0f32); },

        // Justify content
        "justify-center" => quote! { s.justify_content = Some(::gpui::JustifyContent::Center); },
        "justify-start" => quote! { s.justify_content = Some(::gpui::JustifyContent::FlexStart); },
        "justify-end" => quote! { s.justify_content = Some(::gpui::JustifyContent::FlexEnd); },
        "justify-between" => {
            quote! { s.justify_content = Some(::gpui::JustifyContent::SpaceBetween); }
        }
        "justify-around" => {
            quote! { s.justify_content = Some(::gpui::JustifyContent::SpaceAround); }
        }
        "justify-evenly" => {
            quote! { s.justify_content = Some(::gpui::JustifyContent::SpaceEvenly); }
        }

        // Align items
        "items-center" => quote! { s.align_items = Some(::gpui::AlignItems::Center); },
        "items-start" => quote! { s.align_items = Some(::gpui::AlignItems::FlexStart); },
        "items-end" => quote! { s.align_items = Some(::gpui::AlignItems::FlexEnd); },
        "items-baseline" => quote! { s.align_items = Some(::gpui::AlignItems::Baseline); },
        "items-stretch" => quote! { s.align_items = Some(::gpui::AlignItems::Stretch); },

        // Align self
        "self-center" => quote! { s.align_self = Some(::gpui::AlignItems::Center); },
        "self-start" => quote! { s.align_self = Some(::gpui::AlignItems::FlexStart); },
        "self-end" => quote! { s.align_self = Some(::gpui::AlignItems::FlexEnd); },
        "self-stretch" => quote! { s.align_self = Some(::gpui::AlignItems::Stretch); },
        "self-baseline" => quote! { s.align_self = Some(::gpui::AlignItems::Baseline); },

        // Align content
        "content-center" => quote! { s.align_content = Some(::gpui::AlignContent::Center); },
        "content-start" => quote! { s.align_content = Some(::gpui::AlignContent::FlexStart); },
        "content-end" => quote! { s.align_content = Some(::gpui::AlignContent::FlexEnd); },
        "content-between" => quote! { s.align_content = Some(::gpui::AlignContent::SpaceBetween); },
        "content-around" => quote! { s.align_content = Some(::gpui::AlignContent::SpaceAround); },
        "content-stretch" => quote! { s.align_content = Some(::gpui::AlignContent::Stretch); },
        "content-evenly" => quote! { s.align_content = Some(::gpui::AlignContent::SpaceEvenly); },

        // Position
        "relative" => quote! { s.position = Some(::gpui::Position::Relative); },
        "absolute" => quote! { s.position = Some(::gpui::Position::Absolute); },
        "static" => quote! { s.position = Some(::gpui::Position::Relative); },

        // Overflow
        "overflow-hidden" => {
            quote! { s.overflow.x = Some(::gpui::Overflow::Hidden); s.overflow.y = Some(::gpui::Overflow::Hidden); }
        }
        "overflow-scroll" => {
            quote! { s.overflow.x = Some(::gpui::Overflow::Scroll); s.overflow.y = Some(::gpui::Overflow::Scroll); }
        }
        "overflow-auto" => {
            quote! { s.overflow.x = Some(::gpui::Overflow::Scroll); s.overflow.y = Some(::gpui::Overflow::Scroll); }
        }
        "overflow-visible" => {
            quote! { s.overflow.x = Some(::gpui::Overflow::Visible); s.overflow.y = Some(::gpui::Overflow::Visible); }
        }
        "overflow-x-hidden" => quote! { s.overflow.x = Some(::gpui::Overflow::Hidden); },
        "overflow-x-scroll" => quote! { s.overflow.x = Some(::gpui::Overflow::Scroll); },
        "overflow-x-auto" => quote! { s.overflow.x = Some(::gpui::Overflow::Scroll); },
        "overflow-x-visible" => quote! { s.overflow.x = Some(::gpui::Overflow::Visible); },
        "overflow-y-hidden" => quote! { s.overflow.y = Some(::gpui::Overflow::Hidden); },
        "overflow-y-scroll" => quote! { s.overflow.y = Some(::gpui::Overflow::Scroll); },
        "overflow-y-auto" => quote! { s.overflow.y = Some(::gpui::Overflow::Scroll); },
        "overflow-y-visible" => quote! { s.overflow.y = Some(::gpui::Overflow::Visible); },

        // Visibility
        "invisible" => quote! { s.visibility = Some(::gpui::Visibility::Hidden); },
        "visible" => quote! { s.visibility = Some(::gpui::Visibility::Visible); },

        // Font style
        "italic" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_style = Some(::gpui::FontStyle::Italic); }
        }
        "not-italic" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_style = Some(::gpui::FontStyle::Normal); }
        }

        // Text decoration
        "underline" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).underline = Some(::gpui::UnderlineStyle {
                thickness: ::gpui::px(1.),
                ..::core::default::Default::default()
            });
        },
        "line-through" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).strikethrough = Some(::gpui::StrikethroughStyle {
                thickness: ::gpui::px(1.),
                ..::core::default::Default::default()
            });
        },
        "no-underline" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).underline = None;
            s.text.get_or_insert_with(::core::default::Default::default).strikethrough = None;
        },

        // White space
        "whitespace-normal" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).white_space = Some(::gpui::WhiteSpace::Normal); }
        }
        "whitespace-nowrap" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).white_space = Some(::gpui::WhiteSpace::Nowrap); }
        }

        // Border style
        "border-solid" => quote! { s.border_style = Some(::gpui::BorderStyle::Solid); },
        "border-dashed" => quote! { s.border_style = Some(::gpui::BorderStyle::Dashed); },

        // Border width (no value = 1px)
        "border" => quote! {
            s.border_widths.top = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.)));
            s.border_widths.right = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.)));
            s.border_widths.bottom = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.)));
            s.border_widths.left = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.)));
        },
        "border-t" => {
            quote! { s.border_widths.top = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.))); }
        }
        "border-r" => {
            quote! { s.border_widths.right = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.))); }
        }
        "border-b" => {
            quote! { s.border_widths.bottom = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.))); }
        }
        "border-l" => {
            quote! { s.border_widths.left = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.))); }
        }

        // Shadow
        "shadow-sm" => spacing::shadow_value("sm").unwrap(),
        "shadow" => spacing::shadow_value("").unwrap(),
        "shadow-md" => spacing::shadow_value("md").unwrap(),
        "shadow-lg" => spacing::shadow_value("lg").unwrap(),
        "shadow-xl" => spacing::shadow_value("xl").unwrap(),
        "shadow-2xl" => spacing::shadow_value("2xl").unwrap(),
        "shadow-none" => spacing::shadow_value("none").unwrap(),

        // Rounded (no value = 4px)
        "rounded" => emit_rounded_all(4.0),
        "rounded-sm" => emit_rounded_all(2.0),
        "rounded-md" => emit_rounded_all(6.0),
        "rounded-lg" => emit_rounded_all(8.0),
        "rounded-xl" => emit_rounded_all(12.0),
        "rounded-2xl" => emit_rounded_all(16.0),
        "rounded-3xl" => emit_rounded_all(24.0),
        "rounded-full" => emit_rounded_all(9999.0),
        "rounded-none" => emit_rounded_all(0.0),

        // Cursor
        "cursor-pointer" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::PointingHand); },
        "cursor-default" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::Arrow); },
        "cursor-text" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::IBeam); },
        "cursor-not-allowed" => {
            quote! { s.mouse_cursor = Some(::gpui::CursorStyle::OperationNotAllowed); }
        }
        "cursor-grab" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::OpenHand); },
        "cursor-grabbing" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::ClosedHand); },
        "cursor-crosshair" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::Crosshair); },

        // Font weight
        "font-thin" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::THIN); }
        }
        "font-light" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::LIGHT); }
        }
        "font-normal" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::NORMAL); }
        }
        "font-medium" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::MEDIUM); }
        }
        "font-semibold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::SEMIBOLD); }
        }
        "font-bold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::BOLD); }
        }
        "font-extrabold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::EXTRA_BOLD); }
        }
        "font-black" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::BLACK); }
        }

        // Text align
        "text-left" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Left); }
        }
        "text-center" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Center); }
        }
        "text-right" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Right); }
        }

        // Text size
        "text-xs" => emit_font_size(12.0),
        "text-sm" => emit_font_size(14.0),
        "text-base" => emit_font_size(16.0),
        "text-lg" => emit_font_size(18.0),
        "text-xl" => emit_font_size(20.0),
        "text-2xl" => emit_font_size(24.0),
        "text-3xl" => emit_font_size(30.0),
        "text-4xl" => emit_font_size(36.0),
        "text-5xl" => emit_font_size(48.0),
        "text-6xl" => emit_font_size(60.0),
        "text-7xl" => emit_font_size(72.0),
        "text-8xl" => emit_font_size(96.0),
        "text-9xl" => emit_font_size(128.0),

        // Width
        "w-full" => quote! { s.size.width = Some(::gpui::Length::from(::gpui::relative(1.))); },
        "w-auto" => quote! { s.size.width = Some(::gpui::Length::Auto); },
        "w-fit" => quote! { s.size.width = Some(::gpui::Length::Auto); },
        "w-screen" => quote! { s.size.width = Some(::gpui::Length::from(::gpui::relative(1.))); },

        // Height
        "h-full" => quote! { s.size.height = Some(::gpui::Length::from(::gpui::relative(1.))); },
        "h-auto" => quote! { s.size.height = Some(::gpui::Length::Auto); },
        "h-fit" => quote! { s.size.height = Some(::gpui::Length::Auto); },
        "h-screen" => quote! { s.size.height = Some(::gpui::Length::from(::gpui::relative(1.))); },

        // Min width
        "min-w-full" => {
            quote! { s.min_size.width = Some(::gpui::Length::from(::gpui::relative(1.))); }
        }
        "min-w-auto" => quote! { s.min_size.width = Some(::gpui::Length::Auto); },

        // Min height
        "min-h-full" => {
            quote! { s.min_size.height = Some(::gpui::Length::from(::gpui::relative(1.))); }
        }
        "min-h-auto" => quote! { s.min_size.height = Some(::gpui::Length::Auto); },

        // Max width
        "max-w-full" => {
            quote! { s.max_size.width = Some(::gpui::Length::from(::gpui::relative(1.))); }
        }
        "max-w-none" => quote! { s.max_size.width = Some(::gpui::Length::Auto); },

        // Max height
        "max-h-full" => {
            quote! { s.max_size.height = Some(::gpui::Length::from(::gpui::relative(1.))); }
        }
        "max-h-none" => quote! { s.max_size.height = Some(::gpui::Length::Auto); },

        // Margin auto
        "m-auto" => quote! {
            s.margin.top = Some(::gpui::Length::Auto);
            s.margin.right = Some(::gpui::Length::Auto);
            s.margin.bottom = Some(::gpui::Length::Auto);
            s.margin.left = Some(::gpui::Length::Auto);
        },
        "mx-auto" => quote! {
            s.margin.left = Some(::gpui::Length::Auto);
            s.margin.right = Some(::gpui::Length::Auto);
        },
        "my-auto" => quote! {
            s.margin.top = Some(::gpui::Length::Auto);
            s.margin.bottom = Some(::gpui::Length::Auto);
        },
        "mt-auto" => quote! { s.margin.top = Some(::gpui::Length::Auto); },
        "mr-auto" => quote! { s.margin.right = Some(::gpui::Length::Auto); },
        "mb-auto" => quote! { s.margin.bottom = Some(::gpui::Length::Auto); },
        "ml-auto" => quote! { s.margin.left = Some(::gpui::Length::Auto); },

        // Inset
        "inset-0" => quote! {
            s.inset.top = Some(::gpui::Length::from(::gpui::px(0.)));
            s.inset.right = Some(::gpui::Length::from(::gpui::px(0.)));
            s.inset.bottom = Some(::gpui::Length::from(::gpui::px(0.)));
            s.inset.left = Some(::gpui::Length::from(::gpui::px(0.)));
        },
        "inset-auto" => quote! {
            s.inset.top = Some(::gpui::Length::Auto);
            s.inset.right = Some(::gpui::Length::Auto);
            s.inset.bottom = Some(::gpui::Length::Auto);
            s.inset.left = Some(::gpui::Length::Auto);
        },

        // Colors - black/white/transparent
        "bg-black" => quote! { s.background = Some((::gpui::black()).into()); },
        "bg-white" => quote! { s.background = Some((::gpui::white()).into()); },
        "bg-transparent" => quote! {},
        "text-black" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).color = Some((::gpui::black()).into()); }
        }
        "text-white" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).color = Some((::gpui::white()).into()); }
        }
        "text-transparent" => quote! {},
        "border-black" => quote! { s.border_color = Some((::gpui::black()).into()); },
        "border-white" => quote! { s.border_color = Some((::gpui::white()).into()); },
        "border-transparent" => quote! {},

        // Text overflow
        "truncate" => quote! {
            s.overflow.x = Some(::gpui::Overflow::Hidden);
            s.overflow.y = Some(::gpui::Overflow::Hidden);
            s.text.get_or_insert_with(::core::default::Default::default).white_space = Some(::gpui::WhiteSpace::Nowrap);
            s.text.get_or_insert_with(::core::default::Default::default).text_overflow = Some(::gpui::TextOverflow::Truncate(::gpui::SharedString::new_static("…")));
        },
        "text-ellipsis" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).text_overflow = Some(::gpui::TextOverflow::Truncate(::gpui::SharedString::new_static("…")));
        },
        "text-clip" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).text_overflow = Some(::gpui::TextOverflow::Truncate(::gpui::SharedString::new_static("")));
        },

        // Font family
        "font-mono" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).font_family = ::std::option::Option::Some(::gpui::SharedString::from("monospace"));
        },
        "font-sans" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).font_family = ::std::option::Option::Some(::gpui::SharedString::from("sans-serif"));
        },
        "font-serif" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).font_family = ::std::option::Option::Some(::gpui::SharedString::from("serif"));
        },

        // Line height
        "leading-none" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::px(1.)));
        },
        "leading-tight" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.25)));
        },
        "leading-normal" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.5)));
        },
        "leading-loose" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(2.)));
        },

        // Text decoration style
        "decoration-solid" => quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.wavy = false;
            }
        },
        "decoration-wavy" => quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.wavy = true;
            }
        },
        "decoration-none" => quote! {
            s.text.get_or_insert_with(::core::default::Default::default).underline = ::std::option::Option::None;
            s.text.get_or_insert_with(::core::default::Default::default).strikethrough = ::std::option::Option::None;
        },

        _ => return None,
    })
}

fn emit_prefixed(util: &str, opacity: Option<u8>) -> Option<TokenStream2> {
    // Try two-part prefix first (e.g., "border-t-2", "overflow-x-hidden")
    if let Some(stmt) = emit_two_part_prefix(util, opacity) {
        return Some(stmt);
    }

    // Split on first dash
    let (prefix, rest) = util.split_once('-')?;

    Some(match prefix {
        "p" => emit_padding_box(rest, true)?,
        "px" => emit_padding_inline(rest, true)?,
        "py" => emit_padding_block(rest, true)?,
        "pt" => emit_padding_edge("top", rest, true)?,
        "pr" => emit_padding_edge("right", rest, true)?,
        "pb" => emit_padding_edge("bottom", rest, true)?,
        "pl" => emit_padding_edge("left", rest, true)?,
        "ps" => emit_padding_edge("left", rest, true)?,
        "pe" => emit_padding_edge("right", rest, true)?,

        "m" => emit_margin_box(rest)?,
        "mx" => emit_margin_inline(rest)?,
        "my" => emit_margin_block(rest)?,
        "mt" => emit_margin_edge("top", rest)?,
        "mr" => emit_margin_edge("right", rest)?,
        "mb" => emit_margin_edge("bottom", rest)?,
        "ml" => emit_margin_edge("left", rest)?,
        "ms" => emit_margin_edge("left", rest)?,
        "me" => emit_margin_edge("right", rest)?,

        "gap" => emit_gap(rest)?,
        "w" => emit_width(rest)?,
        "h" => emit_height(rest)?,
        "bg" => emit_background(rest, opacity)?,
        "text" => emit_text(rest, opacity)?,
        "border" => emit_border_prefixed(rest, opacity)?,
        "rounded" => emit_rounded_prefixed(rest)?,
        "opacity" => emit_opacity(rest)?,
        "shadow" => spacing::shadow_value(rest)?,
        "cursor" => emit_cursor(rest)?,
        "font" => emit_font_weight_prefixed(rest)?,
        "inset" => emit_inset(rest)?,
        "top" => emit_inset_edge("top", rest)?,
        "right" => emit_inset_edge("right", rest)?,
        "bottom" => emit_inset_edge("bottom", rest)?,
        "left" => emit_inset_edge("left", rest)?,
        "grid" => emit_grid(rest)?,
        "col" => emit_col(rest)?,
        "row" => emit_row(rest)?,
        "min" => emit_min_max("min", rest)?,
        "max" => emit_min_max("max", rest)?,
        "aspect" => emit_aspect(rest)?,
        "line" => emit_line_clamp(rest)?,
        "leading" => emit_leading(rest)?,
        "decoration" => emit_decoration(rest)?,
        "z" => emit_z_index(rest)?,

        _ => return None,
    })
}

fn emit_two_part_prefix(util: &str, _opacity: Option<u8>) -> Option<TokenStream2> {
    // border-t-2, border-r-2, etc.
    for (prefix, field) in [
        ("border-t", "top"),
        ("border-r", "right"),
        ("border-b", "bottom"),
        ("border-l", "left"),
    ] {
        let field_ident = format_ident(field);
        if util == prefix {
            return Some(quote! {
                s.border_widths.#field_ident = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(1.)));
            });
        }
        if let Some(rest) = util.strip_prefix(&format!("{}-", prefix)) {
            if let Some(w) = spacing::border_width_value(rest) {
                return Some(quote! {
                    s.border_widths.#field_ident = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#w)));
                });
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
            return Some(emit_rounded_corner(field, 4.0));
        }
        if let Some(rest) = util.strip_prefix(&format!("{}-", prefix)) {
            if let Some(r) = spacing::border_radius_value(rest) {
                return Some(emit_rounded_corner(field, r));
            }
        }
    }

    // rounded-t, rounded-r, rounded-b, rounded-l (two corners each)
    if let Some(rest) = util.strip_prefix("rounded-t-") {
        if let Some(r) = spacing::border_radius_value(rest) {
            return Some(emit_rounded_top(r));
        }
    }
    if util == "rounded-t" {
        return Some(emit_rounded_top(4.0));
    }
    if let Some(rest) = util.strip_prefix("rounded-r-") {
        if let Some(r) = spacing::border_radius_value(rest) {
            return Some(emit_rounded_right(r));
        }
    }
    if util == "rounded-r" {
        return Some(emit_rounded_right(4.0));
    }
    if let Some(rest) = util.strip_prefix("rounded-b-") {
        if let Some(r) = spacing::border_radius_value(rest) {
            return Some(emit_rounded_bottom(r));
        }
    }
    if util == "rounded-b" {
        return Some(emit_rounded_bottom(4.0));
    }
    if let Some(rest) = util.strip_prefix("rounded-l-") {
        if let Some(r) = spacing::border_radius_value(rest) {
            return Some(emit_rounded_left(r));
        }
    }
    if util == "rounded-l" {
        return Some(emit_rounded_left(4.0));
    }

    // min-w-*, min-h-*, max-w-*, max-h-*
    if let Some(rest) = util.strip_prefix("min-w-") {
        return emit_min_size("width", rest);
    }
    if let Some(rest) = util.strip_prefix("min-h-") {
        return emit_min_size("height", rest);
    }
    if let Some(rest) = util.strip_prefix("max-w-") {
        return emit_max_size("width", rest);
    }
    if let Some(rest) = util.strip_prefix("max-h-") {
        return emit_max_size("height", rest);
    }

    // gap-x-*, gap-y-*
    if let Some(rest) = util.strip_prefix("gap-x-") {
        return emit_gap_axis("width", rest);
    }
    if let Some(rest) = util.strip_prefix("gap-y-") {
        return emit_gap_axis("height", rest);
    }

    // flex-grow-*, flex-shrink-*
    if let Some(rest) = util.strip_prefix("flex-grow-") {
        let n: f32 = rest.parse().ok()?;
        return Some(quote! { s.flex_grow = Some(#n); });
    }
    if let Some(rest) = util.strip_prefix("flex-shrink-") {
        let n: f32 = rest.parse().ok()?;
        return Some(quote! { s.flex_shrink = Some(#n); });
    }

    None
}

fn emit_rounded_top(r: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! {
        s.corner_radii.top_left = Some(#v);
        s.corner_radii.top_right = Some(#v);
    }
}

fn emit_rounded_right(r: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! {
        s.corner_radii.top_right = Some(#v);
        s.corner_radii.bottom_right = Some(#v);
    }
}

fn emit_rounded_bottom(r: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! {
        s.corner_radii.bottom_right = Some(#v);
        s.corner_radii.bottom_left = Some(#v);
    }
}

fn emit_rounded_left(r: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! {
        s.corner_radii.top_left = Some(#v);
        s.corner_radii.bottom_left = Some(#v);
    }
}

// Helper: emit px as DefiniteLength
fn def_px(n: f32) -> TokenStream2 {
    quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(::gpui::px(#n)) }
}

// Helper: emit px as Length
fn len_px(n: f32) -> TokenStream2 {
    quote! { ::core::convert::Into::<::gpui::Length>::into(::gpui::px(#n)) }
}

fn emit_padding_box(rest: &str, _is_def: bool) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    Some(quote! {
        s.padding.top = Some(#v);
        s.padding.right = Some(#v);
        s.padding.bottom = Some(#v);
        s.padding.left = Some(#v);
    })
}

fn emit_padding_inline(rest: &str, _is_def: bool) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    Some(quote! {
        s.padding.left = Some(#v);
        s.padding.right = Some(#v);
    })
}

fn emit_padding_block(rest: &str, _is_def: bool) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    Some(quote! {
        s.padding.top = Some(#v);
        s.padding.bottom = Some(#v);
    })
}

fn emit_padding_edge(edge: &str, rest: &str, _is_def: bool) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    let field = format_ident(edge);
    Some(quote! { s.padding.#field = Some(#v); })
}

fn emit_margin_box(rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        return Some(quote! {
            s.margin.top = Some(::gpui::Length::Auto);
            s.margin.right = Some(::gpui::Length::Auto);
            s.margin.bottom = Some(::gpui::Length::Auto);
            s.margin.left = Some(::gpui::Length::Auto);
        });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! {
        s.margin.top = Some(#v);
        s.margin.right = Some(#v);
        s.margin.bottom = Some(#v);
        s.margin.left = Some(#v);
    })
}

fn emit_margin_inline(rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        return Some(quote! {
            s.margin.left = Some(::gpui::Length::Auto);
            s.margin.right = Some(::gpui::Length::Auto);
        });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! {
        s.margin.left = Some(#v);
        s.margin.right = Some(#v);
    })
}

fn emit_margin_block(rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        return Some(quote! {
            s.margin.top = Some(::gpui::Length::Auto);
            s.margin.bottom = Some(::gpui::Length::Auto);
        });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! {
        s.margin.top = Some(#v);
        s.margin.bottom = Some(#v);
    })
}

fn emit_margin_edge(edge: &str, rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        let field = format_ident(edge);
        return Some(quote! { s.margin.#field = Some(::gpui::Length::Auto); });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    let field = format_ident(edge);
    Some(quote! { s.margin.#field = Some(#v); })
}

fn emit_gap(rest: &str) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    Some(quote! {
        s.gap.width = Some(#v);
        s.gap.height = Some(#v);
    })
}

fn emit_gap_axis(axis: &str, rest: &str) -> Option<TokenStream2> {
    let n = spacing::spacing_value(rest)?;
    let v = def_px(n);
    let field = format_ident(axis);
    Some(quote! { s.gap.#field = Some(#v); })
}

fn emit_width(rest: &str) -> Option<TokenStream2> {
    if rest == "full" {
        return Some(quote! { s.size.width = Some(::gpui::Length::from(::gpui::relative(1.))); });
    }
    if rest == "auto" {
        return Some(quote! { s.size.width = Some(::gpui::Length::Auto); });
    }
    if rest == "fit" {
        return Some(quote! { s.size.width = Some(::gpui::Length::Auto); });
    }
    if rest == "screen" {
        return Some(quote! { s.size.width = Some(::gpui::Length::from(::gpui::relative(1.))); });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! { s.size.width = Some(#v); })
}

fn emit_height(rest: &str) -> Option<TokenStream2> {
    if rest == "full" {
        return Some(quote! { s.size.height = Some(::gpui::Length::from(::gpui::relative(1.))); });
    }
    if rest == "auto" {
        return Some(quote! { s.size.height = Some(::gpui::Length::Auto); });
    }
    if rest == "fit" {
        return Some(quote! { s.size.height = Some(::gpui::Length::Auto); });
    }
    if rest == "screen" {
        return Some(quote! { s.size.height = Some(::gpui::Length::from(::gpui::relative(1.))); });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! { s.size.height = Some(#v); })
}

fn emit_background(rest: &str, opacity: Option<u8>) -> Option<TokenStream2> {
    let color = emit_color_value(rest, opacity)?;
    Some(quote! { s.background = Some((#color).into()); })
}

fn emit_text(rest: &str, opacity: Option<u8>) -> Option<TokenStream2> {
    // Text align
    match rest {
        "left" => {
            return Some(
                quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Left); },
            )
        }
        "center" => {
            return Some(
                quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Center); },
            )
        }
        "right" => {
            return Some(
                quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Right); },
            )
        }
        "justify" => {
            return Some(
                quote! { s.text.get_or_insert_with(::core::default::Default::default).text_align = Some(::gpui::TextAlign::Left); },
            )
        }
        _ => {}
    }

    // Font size
    if let Some((size, _)) = spacing::font_size_value(rest) {
        return Some(emit_font_size(size));
    }

    // Text color
    if let Some(color) = emit_color_value(rest, opacity) {
        return Some(
            quote! { s.text.get_or_insert_with(::core::default::Default::default).color = Some((#color).into()); },
        );
    }

    None
}

fn emit_border_prefixed(rest: &str, opacity: Option<u8>) -> Option<TokenStream2> {
    // Border width
    if let Some(w) = spacing::border_width_value(rest) {
        return Some(quote! {
            s.border_widths.top = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#w)));
            s.border_widths.right = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#w)));
            s.border_widths.bottom = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#w)));
            s.border_widths.left = Some(::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#w)));
        });
    }

    // Border color
    if let Some(color) = emit_color_value(rest, opacity) {
        return Some(quote! { s.border_color = Some((#color).into()); });
    }

    None
}

fn emit_rounded_prefixed(rest: &str) -> Option<TokenStream2> {
    let r = spacing::border_radius_value(rest)?;
    Some(emit_rounded_all(r))
}

fn emit_rounded_all(r: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! {
        s.corner_radii.top_left = Some(#v);
        s.corner_radii.top_right = Some(#v);
        s.corner_radii.bottom_right = Some(#v);
        s.corner_radii.bottom_left = Some(#v);
    }
}

fn emit_rounded_corner(field: &str, r: f32) -> TokenStream2 {
    let field = format_ident(field);
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#r)) };
    quote! { s.corner_radii.#field = Some(#v); }
}

fn emit_opacity(rest: &str) -> Option<TokenStream2> {
    let n: f32 = rest.parse().ok()?;
    let frac = n / 100.0;
    Some(quote! { s.opacity = Some(#frac); })
}

fn emit_cursor(rest: &str) -> Option<TokenStream2> {
    Some(match rest {
        "pointer" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::PointingHand); },
        "default" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::Arrow); },
        "text" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::IBeam); },
        "not-allowed" => {
            quote! { s.mouse_cursor = Some(::gpui::CursorStyle::OperationNotAllowed); }
        }
        "grab" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::OpenHand); },
        "grabbing" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::ClosedHand); },
        "crosshair" => quote! { s.mouse_cursor = Some(::gpui::CursorStyle::Crosshair); },
        _ => return None,
    })
}

fn emit_font_weight_prefixed(rest: &str) -> Option<TokenStream2> {
    Some(match rest {
        "thin" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::THIN); }
        }
        "light" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::LIGHT); }
        }
        "normal" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::NORMAL); }
        }
        "medium" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::MEDIUM); }
        }
        "semibold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::SEMIBOLD); }
        }
        "bold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::BOLD); }
        }
        "extrabold" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::EXTRA_BOLD); }
        }
        "black" => {
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_weight = Some(::gpui::FontWeight::BLACK); }
        }
        _ => return None,
    })
}

fn emit_font_size(size: f32) -> TokenStream2 {
    let v = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(::gpui::px(#size)) };
    quote! {
        s.text.get_or_insert_with(::core::default::Default::default).font_size = Some(#v);
    }
}

fn emit_inset(rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        return Some(quote! {
            s.inset.top = Some(::gpui::Length::Auto);
            s.inset.right = Some(::gpui::Length::Auto);
            s.inset.bottom = Some(::gpui::Length::Auto);
            s.inset.left = Some(::gpui::Length::Auto);
        });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    Some(quote! {
        s.inset.top = Some(#v);
        s.inset.right = Some(#v);
        s.inset.bottom = Some(#v);
        s.inset.left = Some(#v);
    })
}

fn emit_inset_edge(edge: &str, rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        let field = format_ident(edge);
        return Some(quote! { s.inset.#field = Some(::gpui::Length::Auto); });
    }
    let n = spacing::spacing_value(rest)?;
    let v = len_px(n);
    let field = format_ident(edge);
    Some(quote! { s.inset.#field = Some(#v); })
}

fn emit_grid(rest: &str) -> Option<TokenStream2> {
    if let Some(rest) = rest.strip_prefix("cols-") {
        let n: u16 = rest.parse().ok()?;
        return Some(quote! { s.grid_cols = Some(#n); });
    }
    if let Some(rest) = rest.strip_prefix("rows-") {
        let n: u16 = rest.parse().ok()?;
        return Some(quote! { s.grid_rows = Some(#n); });
    }
    None
}

fn emit_col(rest: &str) -> Option<TokenStream2> {
    if let Some(rest) = rest.strip_prefix("span-") {
        let n: u16 = rest.parse().ok()?;
        let _ = n;
        return Some(quote! {});
    }
    None
}

fn emit_row(rest: &str) -> Option<TokenStream2> {
    if let Some(rest) = rest.strip_prefix("span-") {
        let n: u16 = rest.parse().ok()?;
        let _ = n;
        return Some(quote! {});
    }
    None
}

fn emit_min_max(kind: &str, rest: &str) -> Option<TokenStream2> {
    if let Some(rest) = rest.strip_prefix("w-") {
        return if kind == "min" {
            emit_min_size("width", rest)
        } else {
            emit_max_size("width", rest)
        };
    }
    if let Some(rest) = rest.strip_prefix("h-") {
        return if kind == "min" {
            emit_min_size("height", rest)
        } else {
            emit_max_size("height", rest)
        };
    }
    None
}

fn emit_min_size(axis: &str, rest: &str) -> Option<TokenStream2> {
    if rest == "auto" {
        let field = format_ident(axis);
        return Some(quote! { s.min_size.#field = Some(::gpui::Length::Auto); });
    }
    if rest == "full" {
        let field = format_ident(axis);
        return Some(
            quote! { s.min_size.#field = Some(::gpui::Length::from(::gpui::relative(1.))); },
        );
    }
    let n = spacing::spacing_value(rest)?;
    let field = format_ident(axis);
    let v = len_px(n);
    Some(quote! { s.min_size.#field = Some(#v); })
}

fn emit_max_size(axis: &str, rest: &str) -> Option<TokenStream2> {
    if rest == "none" {
        let field = format_ident(axis);
        return Some(quote! { s.max_size.#field = Some(::gpui::Length::Auto); });
    }
    if rest == "full" {
        let field = format_ident(axis);
        return Some(
            quote! { s.max_size.#field = Some(::gpui::Length::from(::gpui::relative(1.))); },
        );
    }
    let n = spacing::spacing_value(rest)?;
    let field = format_ident(axis);
    let v = len_px(n);
    Some(quote! { s.max_size.#field = Some(#v); })
}

fn emit_aspect(rest: &str) -> Option<TokenStream2> {
    if rest == "square" {
        return Some(quote! { s.aspect_ratio = Some(1f32); });
    }
    if rest == "video" {
        return Some(quote! { s.aspect_ratio = Some(1.7777777f32); });
    }
    if let Some((w, h)) = rest.split_once('/') {
        let w: f32 = w.parse().ok()?;
        let h: f32 = h.parse().ok()?;
        let ratio = w / h;
        return Some(quote! { s.aspect_ratio = Some(#ratio); });
    }
    None
}

fn emit_line_clamp(rest: &str) -> Option<TokenStream2> {
    if let Some(rest) = rest.strip_prefix("clamp-") {
        let n: usize = rest.parse().ok()?;
        return Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_clamp = Some(#n);
        });
    }
    None
}

fn emit_z_index(rest: &str) -> Option<TokenStream2> {
    let n: i32 = rest.parse().ok()?;
    let _ = n;
    None // gpui doesn't have z-index in StyleRefinement
}

fn emit_leading(rest: &str) -> Option<TokenStream2> {
    match rest {
        "3" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(0.75)));
        }),
        "4" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.0)));
        }),
        "5" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.25)));
        }),
        "6" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.5)));
        }),
        "7" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(1.75)));
        }),
        "8" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(2.0)));
        }),
        "9" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(2.25)));
        }),
        "10" => Some(quote! {
            s.text.get_or_insert_with(::core::default::Default::default).line_height = ::std::option::Option::Some(::gpui::DefiniteLength::from(::gpui::relative(2.5)));
        }),
        _ => {
            // Try arbitrary value: leading-[20px]
            if let Some(arb) = rest.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
                if let Some(len) = parse_arbitrary_length(arb) {
                    return Some(emit_arbitrary_length("line-height", len));
                }
            }
            None
        }
    }
}

fn emit_decoration(rest: &str) -> Option<TokenStream2> {
    match rest {
        "0" => Some(quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.thickness = ::gpui::px(0.);
            }
        }),
        "1" => Some(quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.thickness = ::gpui::px(1.);
            }
        }),
        "2" => Some(quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.thickness = ::gpui::px(2.);
            }
        }),
        "4" => Some(quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.thickness = ::gpui::px(4.);
            }
        }),
        "8" => Some(quote! {
            if let ::std::option::Option::Some(__u) = s.text.get_or_insert_with(::core::default::Default::default).underline.as_mut() {
                __u.thickness = ::gpui::px(8.);
            }
        }),
        _ => None,
    }
}

fn emit_color_value(rest: &str, opacity: Option<u8>) -> Option<TokenStream2> {
    // black/white/transparent
    match rest {
        "black" => {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = 0x00000000 | alpha;
                return Some(quote! { ::gpui::rgba(#packed) });
            }
            return Some(quote! { ::gpui::black() });
        }
        "white" => {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = 0xFFFFFF00 | alpha;
                return Some(quote! { ::gpui::rgba(#packed) });
            }
            return Some(quote! { ::gpui::white() });
        }
        "transparent" => return Some(quote! { ::gpui::rgba(0) }),
        _ => {}
    }

    // Try color-shade (e.g., "red-500", "blue-400")
    if let Some((color, shade)) = rest.split_once('-') {
        if let Some((r, g, b)) = colors::color_rgb(color, Some(shade)) {
            if let Some(op) = opacity {
                let alpha = (op as f32 / 100.0 * 255.0).round() as u32;
                let packed = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | alpha;
                return Some(quote! { ::gpui::rgba(#packed) });
            }
            let packed = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
            return Some(quote! { ::gpui::rgb(#packed) });
        }
    }

    None
}

fn emit_arbitrary(util: &str, arb: &str, _opacity: Option<u8>) -> syn::Result<TokenStream2> {
    // Try color first (for bg-, text-, border-)
    if util == "bg" || util == "text" || util == "border" {
        if let Some(color) = parse_arbitrary_color(arb) {
            if util == "bg" {
                return Ok(quote! { s.background = Some((#color).into()); });
            } else if util == "text" {
                return Ok(
                    quote! { s.text.get_or_insert_with(::core::default::Default::default).color = Some((#color).into()); },
                );
            } else {
                return Ok(quote! { s.border_color = Some((#color).into()); });
            }
        }
    }

    // Try length (for w, h, p, m, gap, etc.)
    if let Some(len) = parse_arbitrary_length(arb) {
        return Ok(emit_arbitrary_length(util, len));
    }

    // Unknown arbitrary value - skip
    Ok(quote! {})
}

fn emit_arbitrary_length(util: &str, len: TokenStream2) -> TokenStream2 {
    match util {
        "w" => quote! { s.size.width = Some(::core::convert::Into::<::gpui::Length>::into(#len)); },
        "h" => {
            quote! { s.size.height = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "p" => {
            let d = quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#len) };
            quote! {
                s.padding.top = Some(#d);
                s.padding.right = Some(#d);
                s.padding.bottom = Some(#d);
                s.padding.left = Some(#d);
            }
        }
        "px" => {
            let d = quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#len) };
            quote! { s.padding.left = Some(#d); s.padding.right = Some(#d); }
        }
        "py" => {
            let d = quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#len) };
            quote! { s.padding.top = Some(#d); s.padding.bottom = Some(#d); }
        }
        "pt" => {
            quote! { s.padding.top = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#len)); }
        }
        "pr" => {
            quote! { s.padding.right = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#len)); }
        }
        "pb" => {
            quote! { s.padding.bottom = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#len)); }
        }
        "pl" => {
            quote! { s.padding.left = Some(::core::convert::Into::<::gpui::DefiniteLength>::into(#len)); }
        }
        "m" => {
            let l = quote! { ::core::convert::Into::<::gpui::Length>::into(#len) };
            quote! {
                s.margin.top = Some(#l);
                s.margin.right = Some(#l);
                s.margin.bottom = Some(#l);
                s.margin.left = Some(#l);
            }
        }
        "mx" => {
            let l = quote! { ::core::convert::Into::<::gpui::Length>::into(#len) };
            quote! { s.margin.left = Some(#l); s.margin.right = Some(#l); }
        }
        "my" => {
            let l = quote! { ::core::convert::Into::<::gpui::Length>::into(#len) };
            quote! { s.margin.top = Some(#l); s.margin.bottom = Some(#l); }
        }
        "mt" => {
            quote! { s.margin.top = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "mr" => {
            quote! { s.margin.right = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "mb" => {
            quote! { s.margin.bottom = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "ml" => {
            quote! { s.margin.left = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "gap" => {
            let d = quote! { ::core::convert::Into::<::gpui::DefiniteLength>::into(#len) };
            quote! { s.gap.width = Some(#d); s.gap.height = Some(#d); }
        }
        "rounded" => {
            let a = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#len) };
            quote! {
                s.corner_radii.top_left = Some(#a);
                s.corner_radii.top_right = Some(#a);
                s.corner_radii.bottom_right = Some(#a);
                s.corner_radii.bottom_left = Some(#a);
            }
        }
        "text" => {
            let a = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#len) };
            quote! { s.text.get_or_insert_with(::core::default::Default::default).font_size = Some(#a); }
        }
        "border" => {
            let a = quote! { ::core::convert::Into::<::gpui::AbsoluteLength>::into(#len) };
            quote! {
                s.border_widths.top = Some(#a);
                s.border_widths.right = Some(#a);
                s.border_widths.bottom = Some(#a);
                s.border_widths.left = Some(#a);
            }
        }
        "min-w" => {
            quote! { s.min_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "min-h" => {
            quote! { s.min_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "max-w" => {
            quote! { s.max_size.width = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "max-h" => {
            quote! { s.max_size.height = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "top" => {
            quote! { s.inset.top = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "right" => {
            quote! { s.inset.right = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "bottom" => {
            quote! { s.inset.bottom = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "left" => {
            quote! { s.inset.left = Some(::core::convert::Into::<::gpui::Length>::into(#len)); }
        }
        "inset" => {
            let l = quote! { ::core::convert::Into::<::gpui::Length>::into(#len) };
            quote! {
                s.inset.top = Some(#l);
                s.inset.right = Some(#l);
                s.inset.bottom = Some(#l);
                s.inset.left = Some(#l);
            }
        }
        _ => quote! {},
    }
}

fn parse_arbitrary_length(s: &str) -> Option<TokenStream2> {
    let s = s.trim();
    if let Some(rest) = s.strip_suffix("px") {
        let n: f32 = rest.parse().ok()?;
        return Some(quote! { ::gpui::px(#n) });
    }
    if let Some(rest) = s.strip_suffix("rem") {
        let n: f32 = rest.parse().ok()?;
        return Some(quote! { ::gpui::rems(#n) });
    }
    if let Some(rest) = s.strip_suffix('%') {
        let n: f32 = rest.parse().ok()?;
        let frac = n / 100.0;
        return Some(quote! { ::gpui::relative(#frac) });
    }
    // Try bare number (assume px)
    if let Ok(n) = s.parse::<f32>() {
        return Some(quote! { ::gpui::px(#n) });
    }
    None
}

fn parse_arbitrary_color(s: &str) -> Option<TokenStream2> {
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
            return Some(quote! { ::gpui::rgb(#packed) });
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
            return Some(quote! { ::gpui::rgba(#packed) });
        }
    }
    None
}

fn parse_hex_color(s: &str) -> Option<TokenStream2> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut out = String::new();
            for c in hex.chars() {
                out.push(c);
                out.push(c);
            }
            let v = u32::from_str_radix(&out, 16).ok()?;
            Some(quote! { ::gpui::rgb(#v) })
        }
        6 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(quote! { ::gpui::rgb(#v) })
        }
        8 => {
            let v = u32::from_str_radix(hex, 16).ok()?;
            Some(quote! { ::gpui::rgba(#v) })
        }
        _ => None,
    }
}

fn format_ident(name: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(name, proc_macro2::Span::call_site())
}
