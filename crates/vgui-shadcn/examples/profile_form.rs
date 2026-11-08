#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{App, Bounds, WindowBounds, WindowOptions, px, size};
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn profile_form() -> impl gpui::IntoElement {
    apply_theme(false);
    let (name, set_name) = create_signal(String::new());
    let (notifications, set_notifications) = create_signal(false);

    view! {
        <div class="flex flex-col gap-3 p-4">
            {TextField {
                value: name.get(),
                placeholder: "Display name".to_string(),
                on_input: Box::new(move |value, cx| set_name.set(cx, value)),
                class: None::<String>,
            }}
            <div class="flex flex-row items-center gap-2">
                {Switch {
                    checked: notifications.get(),
                    on_change: Box::new(move |checked, cx| {
                        set_notifications.set(cx, checked)
                    }),
                }}
                <span>
                    {if notifications.get() { "Notifications on" } else { "Notifications off" }}
                </span>
            </div>
        </div>
    }
}

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="w-full h-full">
            {profile_form()}
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
