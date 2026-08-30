#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

/// A theme mode propagated through the element tree via `<Provider>`.
#[derive(Clone, PartialEq)]
enum Mode {
    Light,
    Dark,
}

/// The context marker. Zero-sized, stored in a plain `static`.
static MODE: Context<Mode> = Context::new();

/// A box that reads the nearest `MODE` provider, falling back to `Light`
/// when no provider is active. `css!` takes literal CSS, so the `if`/`else`
/// picks one of two literal blocks — no dynamic interpolation needed.
fn themed_box(label: &str) -> impl gpui::IntoElement {
    let mode = use_context_or(&MODE, || Mode::Light);
    let style = if matches!(mode, Mode::Dark) {
        css! {
            background: #1a1a2a;
            color: #ffffff;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    } else {
        css! {
            background: #f5f5f5;
            color: #111111;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    };
    view! {
        <div style={style}>{label.to_string()}</div>
    }
}

fn app() -> impl gpui::IntoElement {
    let (mode, set_mode) = create_signal(Mode::Light);
    view! {
        <Provider context={MODE} value={mode.get()}>
            <div class="flex flex-col p-4 gap-2 w-[400px] h-[400px]">
                {themed_box("root context (toggles)")}
                <Provider context={MODE} value={Mode::Dark}>
                    {themed_box("overridden to dark")}
                </Provider>
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
