use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

pub fn spacing_value(s: &str) -> Option<f32> {
    Some(match s {
        "0" => 0.0,
        "px" => 1.0,
        "0.5" => 2.0,
        "1" => 4.0,
        "1.5" => 6.0,
        "2" => 8.0,
        "2.5" => 10.0,
        "3" => 12.0,
        "3.5" => 14.0,
        "4" => 16.0,
        "5" => 20.0,
        "6" => 24.0,
        "7" => 28.0,
        "8" => 32.0,
        "9" => 36.0,
        "10" => 40.0,
        "11" => 44.0,
        "12" => 48.0,
        "14" => 56.0,
        "16" => 64.0,
        "20" => 80.0,
        "24" => 96.0,
        "28" => 112.0,
        "32" => 128.0,
        "36" => 144.0,
        "40" => 160.0,
        "44" => 176.0,
        "48" => 192.0,
        "52" => 208.0,
        "56" => 224.0,
        "60" => 240.0,
        "64" => 256.0,
        "72" => 288.0,
        "80" => 320.0,
        "96" => 384.0,
        _ => return None,
    })
}

pub fn font_size_value(s: &str) -> Option<(f32, f32)> {
    Some(match s {
        "xs" => (12.0, 16.0),
        "sm" => (14.0, 20.0),
        "base" => (16.0, 24.0),
        "lg" => (18.0, 28.0),
        "xl" => (20.0, 28.0),
        "2xl" => (24.0, 32.0),
        "3xl" => (30.0, 36.0),
        "4xl" => (36.0, 40.0),
        "5xl" => (48.0, 48.0),
        "6xl" => (60.0, 60.0),
        "7xl" => (72.0, 72.0),
        "8xl" => (96.0, 96.0),
        "9xl" => (128.0, 128.0),
        _ => return None,
    })
}

pub fn border_radius_value(s: &str) -> Option<f32> {
    Some(match s {
        "none" => 0.0,
        "sm" => 2.0,
        "" => 4.0, // "rounded" with no suffix
        "md" => 6.0,
        "lg" => 8.0,
        "xl" => 12.0,
        "2xl" => 16.0,
        "3xl" => 24.0,
        "full" => 9999.0,
        _ => return None,
    })
}

pub fn border_width_value(s: &str) -> Option<f32> {
    Some(match s {
        "0" => 0.0,
        "" => 1.0, // "border" with no suffix
        "2" => 2.0,
        "4" => 4.0,
        "8" => 8.0,
        _ => return None,
    })
}

pub fn shadow_value(s: &str) -> Option<TokenStream2> {
    Some(match s {
        "none" => quote! { ::std::vec::Vec::new() },
        "sm" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.05),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(2.),
                    spread_radius: ::gpui::px(0.),
                }
            ]
        },
        "" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(3.),
                    spread_radius: ::gpui::px(0.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(1.)),
                    blur_radius: ::gpui::px(2.),
                    spread_radius: ::gpui::px(-1.),
                }
            ]
        },
        "md" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-1.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(2.)),
                    blur_radius: ::gpui::px(4.),
                    spread_radius: ::gpui::px(-2.),
                }
            ]
        },
        "lg" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(10.)),
                    blur_radius: ::gpui::px(15.),
                    spread_radius: ::gpui::px(-3.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(4.)),
                    blur_radius: ::gpui::px(6.),
                    spread_radius: ::gpui::px(-4.),
                }
            ]
        },
        "xl" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(20.)),
                    blur_radius: ::gpui::px(25.),
                    spread_radius: ::gpui::px(-5.),
                },
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.1),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(8.)),
                    blur_radius: ::gpui::px(10.),
                    spread_radius: ::gpui::px(-6.),
                }
            ]
        },
        "2xl" => quote! {
            ::std::vec![
                ::gpui::BoxShadow {
                    color: ::gpui::hsla(0., 0., 0., 0.25),
                    offset: ::gpui::point(::gpui::px(0.), ::gpui::px(25.)),
                    blur_radius: ::gpui::px(50.),
                    spread_radius: ::gpui::px(-12.),
                }
            ]
        },
        _ => return None,
    })
}
