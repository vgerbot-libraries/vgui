#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{App, Bounds, WindowBounds, WindowOptions, px, size};
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
use std::rc::Rc;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn language_select() -> gpui::AnyElement {
    apply_theme(false);
    let (language, set_language) = create_signal(Some("rust".to_string()));

    let props = ShadcnSelectProps::new(
        vec![
            SelectItem::new("Rust", "rust".to_string()),
            SelectItem::new("TypeScript", "typescript".to_string()),
            SelectItem::new("Python", "python".to_string()).disabled(true),
        ],
        language.get(),
        Rc::new(move |value, cx| set_language.set(cx, value)),
    )
    .placeholder("Choose a language")
    .searchable(true)
    .clearable(true)
    .accessibility_label("Programming language")
    .size(SelectSize::Md);

    select(props)
}

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="w-full h-full flex items-start justify-center p-4">
            {language_select()}
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(450.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);
    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
