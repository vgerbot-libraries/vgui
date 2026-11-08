#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{App, Bounds, WindowBounds, WindowOptions, px, size};
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
use vgui::prelude::*;
use vgui_shadcn::prelude::{Progress, apply_theme};

fn progress() -> impl gpui::IntoElement {
    apply_theme(false);
    let (value, set_value) = create_signal(35.0f64);
    let progress = Progress {
        value: value.get(),
        max: 100.0,
    };

    view! {
        <div class="w-full max-w-md flex flex-col gap-4">
            {progress}
            <button on:click={click(move |cx| {
                set_value.update(cx, |value| *value = (*value + 10.0).min(100.0))
            })}>
                {"Advance"}
            </button>
        </div>
    }
}

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="w-full h-full flex items-center justify-center p-4">
            {progress()}
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(300.)), cx);
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
