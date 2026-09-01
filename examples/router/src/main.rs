#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::for_each;
use vgui::prelude::*;
use vgui::router::RouteMatch;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn route_content(router: &Router) -> gpui::AnyElement {
    if router.match_route("/").is_some() {
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Home"}</h2>
                <p class="text-sm text-[#aaa]">{"Welcome to the router example."}</p>
            </div>
        }.into_any_element();
    }
    if router.match_route("/users").is_some() {
        let users = vec![
            ("1", "Alice"),
            ("2", "Bob"),
            ("3", "Charlie"),
        ];
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Users"}</h2>
                <ul class="flex flex-col gap-1 mt-2">
                    {for_each(users, |(id, name), _| {
                        let router = router.clone();
                        view! {
                            <li
                                class="p-2 cursor-pointer hover:bg-[#333] rounded text-sm"
                                on:click={click(move |cx| router.navigate(cx, &format!("/users/{id}")))}
                            >
                                {name.to_string()}
                            </li>
                        }
                    })}
                </ul>
            </div>
        }.into_any_element();
    }
    if let Some(m) = router.match_route("/users/:id") {
        let id = m.params.get("id").cloned().unwrap_or_else(|| "?".to_string());
        let router = router.clone();
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{format!("User #{}", id)}</h2>
                <p class="text-sm text-[#aaa]">{format!("Details for user {}", id)}</p>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm mt-2"
                    on:click={click(move |cx| router.navigate(cx, "/users"))}
                >
                    {"Back to list"}
                </button>
            </div>
        }.into_any_element();
    }
    if router.match_route("/settings").is_some() {
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Settings"}</h2>
                <p class="text-sm text-[#aaa]">{"Settings page content."}</p>
            </div>
        }.into_any_element();
    }
    // Fallback 404
    view! {
        <div class="p-4">
            <h2 class="text-lg font-bold">{"404"}</h2>
            <p class="text-sm text-[#aaa]">{"Page not found."}</p>
        </div>
    }.into_any_element()
}

fn app() -> impl gpui::IntoElement {
    let router = create_router("/");
    let r_home = router.clone();
    let r_users = router.clone();
    let r_settings = router.clone();
    let r_path = router.clone();

    view! {
        <div class="flex flex-col bg-[#1a1a2e] text-white h-full">
            // Nav bar
            <div class="flex flex-row gap-2 p-3 bg-[#2d2d44]">
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_home.navigate(cx, "/"))}
                >
                    {"Home"}
                </button>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_users.navigate(cx, "/users"))}
                >
                    {"Users"}
                </button>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_settings.navigate(cx, "/settings"))}
                >
                    {"Settings"}
                </button>
            </div>

            // Current path display
            <div class="text-sm text-gray-400 p-2">
                {format!("Current path: {}", r_path.path())}
            </div>

            // Route content
            <div class="flex-1">
                {route_content(&router)}
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
