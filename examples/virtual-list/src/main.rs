#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(10_000usize);
    let scroll_handle = use_scroll_handle();

    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#505050] w-full h-full text-white">
            <div class="flex gap-2 items-center">
                <span>{format!("{} items", count.get())}</span>
                <button
                    class="px-2 py-1 rounded bg-[#0000ff] hover:bg-[#000088] text-white"
                    on:click={click({
                        let set_count = set_count.clone();
                        move |cx| set_count.update(cx, |n| *n += 1000)
                    })}
                >
                    {"+1000"}
                </button>
                <button
                    class="px-2 py-1 rounded bg-[#FF0000] hover:bg-[#880000] text-white"
                    on:click={click({
                        let set_count = set_count.clone();
                        move |cx| set_count.update(cx, |n| *n = n.saturating_sub(1000))
                    })}
                >
                    {"-1000"}
                </button>
                <button
                    class="px-2 py-1 rounded bg-[#00aa00] hover:bg-[#006600] text-white"
                    on:click={click({
                        let scroll_handle = scroll_handle.clone();
                        move |cx| {
                            scroll_handle.scroll_to_item(5000, ScrollStrategy::Top);
                            cx.refresh_windows();
                        }
                    })}
                >
                    {"Scroll to 5000"}
                </button>
            </div>
            <div class="flex-1 min-h-0">
                <uniform_list
                    count={count.get()}
                    render={|range, _window, _cx| {
                        range
                            .map(|i| {
                                view! {
                                    <div class="h-7 px-2 flex items-center hover:bg-[#606060]">
                                        {format!("Item {i}")}
                                    </div>
                                }
                            })
                            .collect()
                    }}
                    scroll_handle={scroll_handle.clone()}
                    class="h-full overflow-y-scroll"
                />
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
