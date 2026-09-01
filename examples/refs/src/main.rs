#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, Pixels, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    // Create NodeRefs before view! — they're empty shells until bound
    // during render by the `ref=` attribute.
    let scroll_ref = NodeRef::new();
    let focus_ref = NodeRef::new();
    let items: Vec<u32> = (0..20).collect();
    let (bounds_text, set_bounds_text) = create_signal(String::new());
    let bounds_ref = NodeRef::new();

    // Clone refs for the event-handler closures (the ref= attributes
    // below consume separate clones).
    let scroll_ref_btn1 = scroll_ref.clone();
    let scroll_ref_btn2 = scroll_ref.clone();
    let focus_ref_btn = focus_ref.clone();
    let bounds_ref_btn = bounds_ref.clone();

    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#505050] w-[400px] h-[500px] text-white">
            <h2 class="text-lg font-bold">{"Refs Demo"}</h2>

            // Buttons that call imperative methods on the refs.
            <div class="flex gap-2">
                <button
                    class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded text-white"
                    on:click={click(move |_cx| {
                        scroll_ref_btn1.scroll_to_bottom();
                    })}
                >
                    {"Scroll to bottom"}
                </button>
                <button
                    class="p-2 bg-[#006600] hover:bg-[#004400] rounded text-white"
                    on:click={click(move |_cx| {
                        scroll_ref_btn2.scroll_to(2);
                    })}
                >
                    {"Scroll to #2"}
                </button>
                <button
                    class="p-2 bg-[#660066] hover:bg-[#440044] rounded text-white"
                    on:click={move |_e, window, cx| {
                        focus_ref_btn.focus(window, cx);
                    }}
                >
                    {"Focus box"}
                </button>
                <button
                    class="p-2 bg-[#0066cc] hover:bg-[#004499] rounded text-white"
                    on:click={move |_e, _window, _cx| {
                        let b = bounds_ref_btn.bounds();
                        set_bounds_text.set(_cx, format!(
                            "x: {:.0} y: {:.0} w: {:.0} h: {:.0}",
                            f32::from(b.origin.x),
                            f32::from(b.origin.y),
                            f32::from(b.size.width),
                            f32::from(b.size.height),
                        ));
                    }}
                >
                    {"Get bounds"}
                </button>
            </div>

            // A scrollable list bound to scroll_ref via ref=.
            // ref= forces an auto-id and applies track_focus + track_scroll
            // so scroll_to/scroll_to_bottom/bounds all work.
            <div
                ref={scroll_ref.clone()}
                class="flex-1 overflow-y-scroll bg-[#3a3a3a] rounded p-2 gap-1 flex-col"
            >
                <For each={items}>
                    {move |i: u32, _idx: usize| view! {
                        <div class="p-2 bg-[#2a2a2a] rounded">
                            {format!("Item {}", i)}
                        </div>
                    }}
                </For>
            </div>

            // A focusable box bound to focus_ref.
            <div
                ref={focus_ref.clone()}
                class="p-3 bg-[#2a2a2a] rounded border-2 border-[#666] focus:border-[#0f0]"
                tabindex={0}
            >
                {"Click 'Focus box' to focus me."}
            </div>

            // A div bound to bounds_ref — click "Get bounds" to read its
            // painted Bounds<Pixels> (origin + size) from the previous frame.
            <div
                ref={bounds_ref.clone()}
                class="p-3 bg-[#2a2a2a] rounded border-2 border-[#444]"
            >
                {"Bounds target div"}
            </div>

            <Show when={!bounds_text.get().is_empty()}>
                <span class="text-sm text-[#0f0]">{bounds_text.get()}</span>
            </Show>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(500.0)), cx);
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
