#![cfg_attr(target_family = "wasm", no_main)]

use std::time::Duration;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions, AnimationExt};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-6 p-6 bg-[#1a1a2e] w-[520px] h-[640px] text-white overflow-y-auto">
            <h2 class="text-lg font-bold">{"Animations & Transitions"}</h2>

            // ── Built-in keyframe animations ───────────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"animate-pulse"}</span>
                <div class="bg-[#3b82f6] rounded p-4 animate-pulse">
                    {"Pulsing"}
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"animate-bounce"}</span>
                <div class="bg-[#10b981] rounded p-4 animate-bounce">
                    {"Bouncing"}
                </div>
            </div>

            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"animate-ping"}</span>
                <div class="bg-[#ef4444] rounded p-4 animate-ping">
                    {"Pinging"}
                </div>
            </div>

            // ── Transitions on hover ───────────────────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"transition-opacity hover:opacity-50 duration-300"}</span>
                <button class="bg-[#6366f1] hover:opacity-50 rounded p-3 transition-opacity duration-300">
                    {"Fade on hover"}
                </button>
            </div>

            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"transition-colors hover:bg-[#2563eb] duration-300"}</span>
                <button class="bg-[#6366f1] hover:bg-[#2563eb] rounded p-3 transition-colors duration-300 ease-in-out">
                    {"Color on hover"}
                </button>
            </div>

            // ── Custom animation via animate={...} ─────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">{"animate={...} custom"}</span>
                <div
                    class="bg-[#f59e0b] rounded p-4"
                    animate={|el| el.with_animation(
                        "custom-breath",
                        gpui::Animation::new(Duration::from_millis(1500))
                            .repeat()
                            .with_easing(gpui::ease_in_out),
                        |mut el, delta| {
                            el = el.opacity(0.4 + 0.6 * (delta * std::f32::consts::PI).sin());
                            el
                        },
                    )}
                >
                    {"Custom breathing"}
                </div>
            </div>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(520.), px(640.0)), cx);
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
