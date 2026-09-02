#![cfg_attr(target_family = "wasm", no_main)]

mod theme;
mod views;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

#[derive(Clone, PartialEq)]
pub struct Todo {
    pub id: u32,
    pub text: String,
    pub done: bool,
}

fn route_content(
    router: &Router,
    todos: ReadSignal<Vec<Todo>>,
    set_todos: WriteSignal<Vec<Todo>>,
    dialog_open: ReadSignal<bool>,
    set_dialog_open: WriteSignal<bool>,
    name: ReadSignal<String>,
    set_name: WriteSignal<String>,
    email: ReadSignal<String>,
    set_email: WriteSignal<String>,
) -> gpui::AnyElement {
    if router.match_route("/").is_some() {
        enter_child_scope("route:/");
        let el = views::dashboard_view(todos).into_any_element();
        exit_child_scope();
        return el;
    }
    if router.match_route("/tasks").is_some() {
        enter_child_scope("route:/tasks");
        let el = views::tasks_view(todos, set_todos, dialog_open, set_dialog_open).into_any_element();
        exit_child_scope();
        return el;
    }
    if router.match_route("/settings").is_some() {
        enter_child_scope("route:/settings");
        let el = views::settings_view(name, set_name, email, set_email).into_any_element();
        exit_child_scope();
        return el;
    }
    enter_child_scope("route:404");
    let el = view! {
        <div class="p-4" style={css!{ flex: 1; }}>
            <h2 class="text-xl font-bold" style={css!{ color: var(--text); }}>
                {"404"}
            </h2>
            <p style={css!{ color: var(--text-muted); }}>
                {"Page not found."}
            </p>
        </div>
    }.into_any_element();
    exit_child_scope();
    el
}

fn app() -> impl gpui::IntoElement {
    let router = create_router("/");
    let (theme_mode, set_theme_mode) = create_signal(theme::ThemeMode::Light);
    let (todos, set_todos) = create_signal(vec![
        Todo { id: 0, text: "Learn vgui".into(), done: true },
        Todo { id: 1, text: "Build dashboard".into(), done: false },
        Todo { id: 2, text: "Ship it".into(), done: false },
    ]);
    let (dialog_open, set_dialog_open) = create_signal(false);
    let (name, set_name) = create_signal(String::new());
    let (email, set_email) = create_signal(String::new());

    // Install the theme — reading theme_mode.get() registers a reactive
    // dependency so toggling re-runs render and re-themes everything.
    theme::apply_theme(theme_mode.get());

    let mode = theme_mode.get();
    let set_theme_toggle = set_theme_mode.clone();
    let r_dash = router.clone();
    let r_tasks = router.clone();
    let r_settings = router.clone();

    view! {
        <Provider context={theme::THEME_CTX} value={mode}>
            <div class="flex flex-col" style={css!{
                background: var(--bg);
                color: var(--text);
                height: 100%;
            }}>
                // Top bar
                <div class="flex flex-row items-center justify-between p-3" style={css!{
                    background: var(--surface);
                    border-bottom-width: 1px;
                    border-style: solid;
                    border-color: var(--border);
                }}>
                    <span style={css!{ font-weight: bold; font-size: 18px; color: var(--text); }}>
                        {"vgui Dashboard"}
                    </span>
                    <button
                        style={css!{
                            padding: 6px 12px;
                            background: var(--primary);
                            color: #ffffff;
                            border-width: 0px;
                            border-radius: var(--radius);
                            cursor: pointer;
                            font-size: 12px;
                        }}
                        on:click={click(move |cx| set_theme_toggle.update(cx, |m|
                            *m = match *m {
                                theme::ThemeMode::Light => theme::ThemeMode::Dark,
                                theme::ThemeMode::Dark => theme::ThemeMode::Light,
                            }
                        ))}
                    >
                        {if matches!(mode, theme::ThemeMode::Dark) { "Light" } else { "Dark" }}
                    </button>
                </div>

                // Body: side nav + content
                <div class="flex flex-row" style={css!{ flex: 1; overflow: hidden; }}>
                    // Side nav
                    <div class="flex flex-col gap-1 p-2" style={css!{
                        background: var(--surface);
                        border-right-width: 1px;
                        border-style: solid;
                        border-color: var(--border);
                        width: 140px;
                    }}>
                        <button
                            style={css!{
                                padding: 8px 12px;
                                background: var(--surface);
                                color: var(--text);
                                border-width: 0px;
                                border-radius: var(--radius);
                                cursor: pointer;
                                text-align: left;
                                font-size: 14px;
                            }}
                            on:click={click(move |cx| r_dash.navigate(cx, "/"))}
                        >
                            {"Dashboard"}
                        </button>
                        <button
                            style={css!{
                                padding: 8px 12px;
                                background: var(--surface);
                                color: var(--text);
                                border-width: 0px;
                                border-radius: var(--radius);
                                cursor: pointer;
                                text-align: left;
                                font-size: 14px;
                            }}
                            on:click={click(move |cx| r_tasks.navigate(cx, "/tasks"))}
                        >
                            {"Tasks"}
                        </button>
                        <button
                            style={css!{
                                padding: 8px 12px;
                                background: var(--surface);
                                color: var(--text);
                                border-width: 0px;
                                border-radius: var(--radius);
                                cursor: pointer;
                                text-align: left;
                                font-size: 14px;
                            }}
                            on:click={click(move |cx| r_settings.navigate(cx, "/settings"))}
                        >
                            {"Settings"}
                        </button>
                    </div>

                    // Content area
                    {route_content(
                        &router,
                        todos.clone(),
                        set_todos.clone(),
                        dialog_open.clone(),
                        set_dialog_open.clone(),
                        name.clone(),
                        set_name.clone(),
                        email.clone(),
                        set_email.clone(),
                    )}
                </div>
            </div>
        </Provider>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(800.), px(600.0)), cx);
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
