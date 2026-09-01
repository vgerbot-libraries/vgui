#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (active, set_active) = create_signal(false);

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] text-white" style={css!{ width: 700px; height: 600px; overflow-y: auto; }}>
            <h2 class="text-lg font-bold">{"Styling Showcase"}</h2>

            // ── css! macro ───────────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"css! macro"}</span>
                <div style={css!{
                    display: flex;
                    gap: 12px;
                    padding: 16px;
                    background: linear-gradient(135deg, #2563ff, #6c757d);
                    border-radius: 8px;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                }}>
                    <div style={css!{ background: #2563ff; padding: 12px; border-radius: 4px; }}>
                        {"Child A"}
                    </div>
                    <div style={css!{ background: #6c757d; padding: 12px; border-radius: 4px; }}>
                        {"Child B"}
                    </div>
                </div>
            </div>

            // ── Tailwind classes ─────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"tw! classes"}</span>
                <div class="flex gap-3 p-4 bg-gradient-to-br from-[#2563ff] to-[#6c757d] rounded-lg">
                    <div class="bg-[#2563ff] p-3 rounded">
                        {"Child A"}
                    </div>
                    <div class="bg-[#6c757d] p-3 rounded">
                        {"Child B"}
                    </div>
                </div>
            </div>

            // ── Pseudo-states ────────────────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Pseudo-states (hover / active / focus)"}</span>
                <div class="flex gap-2">
                    <button
                        class="px-4 py-2 bg-[#2563ff] text-white rounded"
                        hover={css!{ background: #0044cc; }}
                    >
                        {"Hover me"}
                    </button>
                    <button
                        class="px-4 py-2 bg-[#10b981] text-white rounded"
                        active={css!{ background: #34d399; }}
                    >
                        {"Active me"}
                    </button>
                    <button
                        class="px-4 py-2 bg-[#9933ff] text-white rounded"
                        focus={css!{ border: 2px solid #ffffff; }}
                    >
                        {"Focus me"}
                    </button>
                </div>
            </div>

            // ── Dynamic classes (twc!) ───────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Dynamic classes (twc!)"}</span>
                <button
                    class={twc!(
                        "p-3 rounded text-white transition-colors",
                        active.get().then_some("bg-[#2563ff]"),
                        (!active.get()).then_some("bg-[#6c757d]")
                    )}
                    on:click={click(move |cx| set_active.update(cx, |v| *v = !*v))}
                >
                    {if active.get() { "Active: ON" } else { "Active: OFF" }}
                </button>
            </div>

            // ── Responsive breakpoints ───────────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Responsive breakpoints (resize window)"}</span>
                <div class="flex flex-col lg:flex-row gap-2">
                    <div class="bg-[#2563ff] p-3 rounded text-white">{"Box 1"}</div>
                    <div class="bg-[#10b981] p-3 rounded text-white">{"Box 2"}</div>
                    <div class="bg-[#9933ff] p-3 rounded text-white">{"Box 3"}</div>
                </div>
                <span class="sm:text-sm lg:text-lg text-[#aaa]">{"sm:text-sm lg:text-lg"}</span>
            </div>

            // ── Runtime classes (tw_dynamic) ─────────────────────────
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"tw_dynamic() runtime"}</span>
                <div class={tw_dynamic("p-4 bg-[#2563ff] text-white rounded-lg")}>
                    {"Runtime-constructed class string"}
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
        let bounds = Bounds::centered(None, size(px(700.), px(600.0)), cx);
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
