#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    view! {
        <div class="w-full h-full flex flex-col gap-3 p-4 justify-center items-center text-white">
            <span>{format!("count = {}", count.get())}</span>
            <button
                class="h-12 p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"
                on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
            >
                {"Increment"}
            </button>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
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
