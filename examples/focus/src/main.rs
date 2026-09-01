#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (radio_val, set_radio) = create_signal(0i32);
    let (field1, set_field1) = create_signal(String::new());
    let (field2, set_field2) = create_signal(String::new());
    let (size_sig, set_size) = create_signal((0f64, 0f64));

    // Individual setters for radio on:change closures (each needs its own
    // WriteSignal clone with a 'static lifetime).
    let sr0 = set_radio.clone();
    let sr1 = set_radio.clone();
    let sr2 = set_radio.clone();
    let set_dialog_open_btn = set_dialog_open.clone();
    let set_dialog_close = set_dialog_open.clone();
    let set_dialog_close_btn = set_dialog_open.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#505050] w-[600px] h-[500px] text-white"
            on:resize={move |ev: &ResizeEvent, _w, _cx| { set_size.update(_cx, |_| (ev.width, ev.height)); }}
        >
            <span class="text-sm text-[#0f0]">{format!("{:.0} x {:.0}", size_sig.get().0, size_sig.get().1)}</span>

            // ── Dialog with focus trap + restore ──────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">
                    {"Click the button, then Tab/Shift+Tab to cycle within the dialog. Escape or click-outside closes it and restores focus."}
                </span>
                <button
                    class="px-3 py-2 bg-[#0066cc] hover:bg-[#004499] rounded text-sm"
                    on:click={click(move |cx| set_dialog_open_btn.set(cx, true))}
                >
                    {"Open Dialog"}
                </button>
            </div>

            // ── Radio group with roving tabindex ──────────────────────
            <div class="flex flex-col gap-2">
                <span class="text-sm text-[#aaa]">
                    {"Tab reaches only the checked radio. Arrow keys move between radios."}
                </span>
                <radiogroup>
                    <div class="flex flex-row gap-4 items-center">
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 0} on:change={move |_v: bool, cx: &mut App| sr0.set(cx, 0)} />
                            <span class="text-sm">{"Option A"}</span>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 1} on:change={move |_v: bool, cx: &mut App| sr1.set(cx, 1)} />
                            <span class="text-sm">{"Option B"}</span>
                        </div>
                        <div class="flex flex-row gap-1 items-center">
                            <input type="radio" checked={radio_val.get() == 2} on:change={move |_v: bool, cx: &mut App| sr2.set(cx, 2)} />
                            <span class="text-sm">{"Option C"}</span>
                        </div>
                    </div>
                </radiogroup>
                <span class="text-sm text-[#0f0]">{format!("selected: {}", radio_val.get())}</span>
            </div>

            // ── Dialog content ────────────────────────────────────────
            <dialog open={dialog_open.get()} on:close={move |cx| set_dialog_close.set(cx, false)}>
                <div class="bg-white text-black p-5 rounded-lg flex flex-col gap-3 w-[350px]">
                    <h3 class="font-bold">{"Dialog with Focus Trap"}</h3>
                    <div class="flex flex-col gap-1">
                        <span class="text-sm text-[#666]">{"Field 1 (type text)"}</span>
                        <input
                            type="text"
                            placeholder="First field"
                            value={field1.get()}
                            on:input={move |v: &str, cx: &mut App| set_field1.set(cx, v.to_string())}
                        />
                    </div>
                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#666]">{"Field 2 (type text)"}</span>
                        <input
                            type="text"
                            placeholder="Second field"
                            value={field2.get()}
                            on:input={move |v: &str, cx: &mut App| set_field2.set(cx, v.to_string())}
                        />
                    </label>
                    <div class="flex flex-row gap-2 justify-end">
                        <button
                            class="px-3 py-2 bg-[#ccc] hover:bg-[#aaa] rounded text-sm"
                            on:click={click(move |cx| set_dialog_close_btn.set(cx, false))}
                        >
                            {"Close"}
                        </button>
                    </div>
                </div>
            </dialog>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(500.0)), cx);
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
