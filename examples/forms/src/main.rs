#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

#[derive(Clone, PartialEq)]
struct FormData {
    name: String,
    email: String,
    age: String,
    country: String,
    subscribe: bool,
}

fn app() -> impl gpui::IntoElement {
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());
    let (age, set_age) = create_signal(String::new());
    let (country, set_country) = create_signal("cn".to_string());
    let (subscribe, set_subscribe) = create_signal(false);
    let (submitted, set_submitted) = create_signal(Option::<FormData>::None);

    // Clones for the on:submit closure (the view! attributes below
    // consume separate clones).
    let s_name = name.clone();
    let s_email = email.clone();
    let s_age = age.clone();
    let s_country = country.clone();
    let s_subscribe = subscribe.clone();
    let r_name = set_name.clone();
    let r_email = set_email.clone();
    let r_age = set_age.clone();
    let r_country = set_country.clone();
    let r_subscribe = set_subscribe.clone();
    let r_submitted = set_submitted.clone();
    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] text-white" style={css!{ width: 500px; height: 600px; overflow-y: auto; }}>
            <h2 class="text-lg font-bold">{"Form Example"}</h2>
            <form
                on:submit={move |cx: &mut App| {
                    set_submitted.set(cx, Some(FormData {
                        name: s_name.get(),
                        email: s_email.get(),
                        age: s_age.get(),
                        country: s_country.get(),
                        subscribe: s_subscribe.get(),
                    }));
                }}
                on:reset={move |cx: &mut App| {
                    r_name.set(cx, String::new());
                    r_email.set(cx, String::new());
                    r_age.set(cx, String::new());
                    r_country.set(cx, "cn".to_string());
                    r_subscribe.set(cx, false);
                    r_submitted.set(cx, None);
                }}
            >
                <div class="flex flex-col gap-3">
                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Name"}</span>
                        <input
                            type="text"
                            placeholder="Enter your name"
                            value={name.get()}
                            on:input={move |v: &str, cx: &mut App| set_name.set(cx, v.to_string())}
                        />
                    </label>

                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Email"}</span>
                        <input
                            type="email"
                            placeholder="you@example.com"
                            value={email.get()}
                            on:input={move |v: &str, cx: &mut App| set_email.set(cx, v.to_string())}
                        />
                    </label>

                    <label class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Age"}</span>
                        <input
                            type="number"
                            min={0.0f64}
                            max={150.0f64}
                            placeholder="30"
                            value={age.get()}
                            on:input={move |v: &str, cx: &mut App| set_age.set(cx, v.to_string())}
                        />
                    </label>

                    <div class="flex flex-col gap-1">
                        <span class="text-sm text-[#888]">{"Country"}</span>
                        <select
                            options={vec![
                                ("cn".to_string(), "China".to_string()),
                                ("us".to_string(), "United States".to_string()),
                                ("jp".to_string(), "Japan".to_string()),
                            ]}
                            value={country.get()}
                            on:change={move |v: &str, cx: &mut App| set_country.set(cx, v.to_string())}
                        />
                    </div>

                    <label class="flex flex-row gap-2 items-center">
                        <input
                            type="checkbox"
                            checked={subscribe.get()}
                            on:change={move |v: bool, cx: &mut App| set_subscribe.set(cx, v)}
                        />
                        <span class="text-sm">{"Subscribe to newsletter"}</span>
                    </label>

                    <div class="flex flex-row gap-3">
                        <input type="submit" value="Submit" class="px-4 py-2 bg-[#2563ff] text-white rounded cursor-pointer" />
                        <input type="reset" value="Reset" class="px-4 py-2 bg-[#6c757d] text-white rounded cursor-pointer" />
                    </div>
                </div>
            </form>

            <Show when={submitted.get().is_some()}>
                <div class="bg-[#2d2d44] p-4 rounded-lg">
                    <span class="text-sm font-bold">{"Submitted data:"}</span>
                    {if let Some(data) = submitted.get() {
                        view! {
                            <div class="text-sm text-[#0f0] mt-2">
                                <div>{format!("Name: {}", data.name)}</div>
                                <div>{format!("Email: {}", data.email)}</div>
                                <div>{format!("Age: {}", data.age)}</div>
                                <div>{format!("Country: {}", data.country)}</div>
                                <div>{format!("Subscribed: {}", data.subscribe)}</div>
                            </div>
                        }.into_any_element()
                    } else {
                        gpui::div().into_any_element()
                    }}
                </div>
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
        let bounds = Bounds::centered(None, size(px(500.), px(600.0)), cx);
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
