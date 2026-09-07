#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, TitlebarOptions, WindowBounds, WindowDecorations, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// Define simple unit-struct actions for the app menu.
gpui::actions!(window_mgmt, [NewWindow, Close, ToggleFullscreen, Minimize]);

fn app() -> impl gpui::IntoElement {
    let (title, set_title) = create_signal("vgui Window Management".to_string());
    let (maximized, set_maximized) = create_signal(false);
    let (saved_size, set_saved_size) = create_signal(None::<gpui::Size<gpui::Pixels>>);

    // Set app menus (idempotent — safe to call every render).
    set_app_menus(vec![
        Menu::new("File").items(vec![
            MenuItem::Separator,
            MenuItem::action("New Window", NewWindow),
            MenuItem::action("Close", Close),
        ]),
        Menu::new("View").items(vec![
            MenuItem::action("Toggle Fullscreen", ToggleFullscreen),
            MenuItem::action("Minimize", Minimize),
        ]),
    ]);

    // Register close-interception handler once per window (slot-guarded).
    use_window_should_close(|_window, _cx| {
        // Return true to allow close, false to prevent.
        true
    });

    // Update window title reactively.
    let current_title = title.get();
    let _ = with_window(|window, _| window.set_window_title(&current_title));

    view! {
        <div class="w-full h-full flex flex-col bg-[#2b2b2b] text-white select-none">
            // Custom titlebar — drag to move window.
            <div
                class="h-10 flex items-center justify-between px-3 bg-[#1e1e1e] border-b border-[#3a3a3a] cursor-default"
                on:pointerdown={move |_e, window, _cx| window.start_window_move()}
            >
                <span class="text-sm font-medium truncate">{title.get()}</span>
                <div class="flex gap-2">
                    <button
                        class="w-7 h-6 rounded bg-[#3a3a3a] hover:bg-[#505050] text-xs flex items-center justify-center"
                        on:pointerdown={move |_e, _w, cx| cx.stop_propagation()}
                        on:click={move |_, window, _cx| window.minimize_window()}
                    >
                        {"–"}
                    </button>
                    <button
                        class="w-7 h-6 rounded bg-[#3a3a3a] hover:bg-[#505050] text-xs flex items-center justify-center"
                        on:pointerdown={move |_e, _w, cx| cx.stop_propagation()}
                        on:click={{
                            let maximized = maximized.clone();
                            let saved_size = saved_size.clone();
                            let set_maximized = set_maximized.clone();
                            let set_saved_size = set_saved_size.clone();
                            move |_, window, cx| {
                                if maximized.get() {
                                    if let Some(sz) = saved_size.get() {
                                        window.resize(sz);
                                    }
                                    set_maximized.set(cx, false);
                                } else {
                                    let old = window.bounds().size;
                                    set_saved_size.set(cx, Some(old));
                                    if let Some(display) = window.display(cx) {
                                        let vb = display.visible_bounds();
                                        window.resize(vb.size);
                                    }
                                    set_maximized.set(cx, true);
                                }
                            }
                        }}
                    >
                        {"□"}
                    </button>
                    <button
                        class="w-7 h-6 rounded bg-[#e02424] hover:bg-[#b01a1a] text-xs flex items-center justify-center"
                        on:pointerdown={move |_e, _w, cx| cx.stop_propagation()}
                        on:click={move |_, window, _cx| window.remove_window()}
                    >
                        {"×"}
                    </button>
                </div>
            </div>

            // Content area.
            <div class="flex-1 flex flex-col gap-4 p-6 overflow-auto">
                <h1 class="text-xl font-bold">{"Window Management"}</h1>

                <div class="flex flex-col gap-2">
                    <label class="text-sm text-gray-300">{"Window Title"}</label>
                    <div class="flex gap-2">
                        <input
                            class="flex-1 px-3 py-2 rounded bg-[#3a3a3a] border border-[#555] text-white text-sm outline-none focus:border-[#007acc]"
                            type="text"
                            value={title.get()}
                            on:input={move |v: &str, cx: &mut App| set_title.set(cx, v.to_string())}
                        />
                    </div>
                    <p class="text-xs text-gray-400">
                        {"The window title updates reactively as you type."}
                    </p>
                </div>

                <div class="flex flex-col gap-2">
                    <h2 class="text-sm font-semibold text-gray-300">{"Window Controls"}</h2>
                    <div class="flex gap-2 flex-wrap">
                        <button
                            class="px-3 py-2 rounded bg-[#007acc] hover:bg-[#005a9e] text-sm"
                            on:click={move |_, window, _cx| window.minimize_window()}
                        >
                            {"Minimize"}
                        </button>
                        <button
                            class="px-3 py-2 rounded bg-[#007acc] hover:bg-[#005a9e] text-sm"
                            on:click={{
                                let maximized = maximized.clone();
                                let saved_size = saved_size.clone();
                                let set_maximized = set_maximized.clone();
                                let set_saved_size = set_saved_size.clone();
                                move |_, window, cx| {
                                    if maximized.get() {
                                        if let Some(sz) = saved_size.get() {
                                            window.resize(sz);
                                        }
                                        set_maximized.set(cx, false);
                                    } else {
                                        let old = window.bounds().size;
                                        set_saved_size.set(cx, Some(old));
                                        if let Some(display) = window.display(cx) {
                                            let vb = display.visible_bounds();
                                            window.resize(vb.size);
                                        }
                                        set_maximized.set(cx, true);
                                    }
                                }
                            }}
                        >
                            {if maximized.get() { "Restore" } else { "Maximize" }}
                        </button>
                        <button
                            class="px-3 py-2 rounded bg-[#e02424] hover:bg-[#b01a1a] text-sm"
                            on:click={move |_, window, _cx| window.remove_window()}
                        >
                            {"Close Window"}
                        </button>
                    </div>
                    <p class="text-xs text-gray-400">
                        {"The titlebar area above can be dragged to move the window."}
                    </p>
                </div>

                <div class="flex flex-col gap-2">
                    <h2 class="text-sm font-semibold text-gray-300">{"Multi-Window"}</h2>
                    <button
                        class="px-3 py-2 rounded bg-[#2d7d46] hover:bg-[#236b3a] text-sm self-start"
                        on:click={click(move |cx| {
                            let _ = vgui::open_window(
                                cx,
                                WindowOptions {
                                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                                        None,
                                        size(px(400.), px(300.)),
                                        cx,
                                    ))),
                                    ..Default::default()
                                },
                                app,
                            );
                        })}
                    >
                        {"Open New Window"}
                    </button>
                    <p class="text-xs text-gray-400">
                        {"Opens a second independent window with the same app."}
                    </p>
                </div>

                <div class="flex flex-col gap-2">
                    <h2 class="text-sm font-semibold text-gray-300">{"App Menus"}</h2>
                    <p class="text-xs text-gray-400">
                        {"The native menu bar shows File and View menus (visible on macOS / Linux desktop)."}
                    </p>
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
        let bounds = Bounds::centered(None, size(px(600.), px(450.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(TitlebarOptions {
                    appears_transparent: true,
                    ..Default::default()
                }),
                window_min_size: Some(size(px(400.), px(300.))),
                window_decorations: Some(WindowDecorations::Client),
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
