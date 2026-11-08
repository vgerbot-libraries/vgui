#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{App, Bounds, WindowBounds, WindowOptions, px, size};
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
use vgui::prelude::*;
use vgui_shadcn::components::dialog::{DialogComponent, DialogDescription, DialogTitle};
use vgui_shadcn::prelude::apply_theme;

fn dialog() -> impl gpui::IntoElement {
    apply_theme(false);
    let (open, set_open) = create_signal(false);
    let trigger = {
        let set_open = set_open.clone();
        view! {
            <button on:click={click(move |cx| set_open.set(cx, true))}>
                {"Delete item"}
            </button>
        }
    };
    let dialog = DialogComponent {
        open: open.get(),
        on_close: Box::new({
            let set_open = set_open.clone();
            move |cx| set_open.set(cx, false)
        }),
        children: vec![
            DialogTitle {
                children: vec![view! { <span>{"Delete item?"}</span> }.into_any_element()],
            }
            .into_any_element(),
            DialogDescription {
                children: vec![
                    view! { <span>{"This action cannot be undone."}</span> }.into_any_element(),
                ],
            }
            .into_any_element(),
            view! {
                <button on:click={click(move |cx| set_open.set(cx, false))}>
                    {"Cancel"}
                </button>
            }
            .into_any_element(),
        ],
    };

    view! { <div>{trigger}{dialog}</div> }
}

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="w-full h-full flex items-center justify-center p-4">
            {dialog()}
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
