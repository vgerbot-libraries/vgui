#![cfg_attr(target_family = "wasm", no_main)]

mod panel;
mod theme;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use panel::ThemePanel;
use theme::{themed_box, Mode, MODE};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (mode, set_mode) = create_signal(Mode::Light);
    view! {
        <Provider context={MODE} value={mode.get()}>
            <div class="flex flex-col p-4 gap-2 w-[400px] h-[400px]">
                {themed_box("root context (toggles)")}
                <ThemePanel />
                <button class="p-2 bg-[#0066cc] text-white rounded"
                    on:click={click(move |cx| set_mode.update(cx, |m|
                        *m = match *m { Mode::Light => Mode::Dark, Mode::Dark => Mode::Light }))}>
                    {"toggle root theme"}
                </button>
            </div>
        </Provider>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(400.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
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
