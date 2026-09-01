# Dashboard Example

## Live Demo

<iframe src="../wasm/dashboard/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The dashboard is the capstone `vgui` application — it combines router, theming,
context, forms, overlays, and list rendering into a single multi-module project.
It demonstrates:

- Router-driven view switching between Dashboard, Tasks, and Settings pages.
- Theme toggle with `set_theme()` + `var()` for reactive light/dark theming.
- `Context<ThemeMode>` propagated through the tree via `<Provider>`.
- Form for adding todos with enter-to-submit inside `<form>`.
- `dialog()` overlay for delete confirmation.
- `<For>` (via `for_each`) for todo list rendering.
- `<Show>` for conditional empty state and saved-settings feedback.
- `create_memo` for the filtered todo list and completion stats.
- Multi-module structure (`main.rs` + `theme.rs` + `views.rs`).

## Source Code

```rust
// --- main.rs ---
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
        return views::dashboard_view(todos).into_any_element();
    }
    if router.match_route("/tasks").is_some() {
        return views::tasks_view(todos, set_todos, dialog_open, set_dialog_open).into_any_element();
    }
    if router.match_route("/settings").is_some() {
        return views::settings_view(name, set_name, email, set_email).into_any_element();
    }
    view! {
        <div class="p-4" style={css!{ flex: 1; }}>
            <h2 class="text-xl font-bold" style={css!{ color: var(--text); }}>
                {"404"}
            </h2>
            <p style={css!{ color: var(--text-muted); }}>
                {"Page not found."}
            </p>
        </div>
    }.into_any_element()
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

// --- theme.rs ---
use vgui::prelude::*;

/// Light theme built with the `theme!` macro.
fn light_theme() -> Theme {
    theme! {
        --bg: #f5f5f5;
        --surface: #ffffff;
        --primary: #2563ff;
        --text: #111111;
        --text-muted: #666666;
        --border: #dddddd;
        --radius: 8px;
    }
}

/// Dark theme — same variable names, different values.
fn dark_theme() -> Theme {
    theme! {
        --bg: #1a1a2e;
        --surface: #2d2d44;
        --primary: #2563ff;
        --text: #ffffff;
        --text-muted: #aaaaaa;
        --border: #444444;
        --radius: 8px;
    }
}

/// Theme mode propagated through the element tree via `<Provider>`.
#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// The context marker for the theme mode.
pub static THEME_CTX: Context<ThemeMode> = Context::new();

/// Install the active theme based on the mode signal.
pub fn apply_theme(mode: ThemeMode) {
    set_theme(if matches!(mode, ThemeMode::Dark) {
        dark_theme()
    } else {
        light_theme()
    });
}

// --- views.rs ---
use vgui::{dialog, for_each};
use vgui::prelude::*;

use crate::Todo;

/// Dashboard view — summary stat cards + progress bar.
pub fn dashboard_view(todos: ReadSignal<Vec<Todo>>) -> impl gpui::IntoElement {
    let total = todos.get().len();
    let done = todos.get().iter().filter(|t| t.done).count();
    let active = total - done;
    let pct = if total == 0 { 0.0 } else { done as f64 / total as f64 };

    view! {
        <div class="flex flex-col gap-4 p-4" style={css!{ flex: 1; overflow-y: auto; }}>
            <h2 class="text-xl font-bold" style={css!{ color: var(--text); }}>
                {"Dashboard"}
            </h2>

            // Stat cards
            <div class="flex flex-row gap-3">
                {stat_card("Total Tasks", &total.to_string())}
                {stat_card("Active", &active.to_string())}
                {stat_card("Completed", &done.to_string())}
            </div>

            // Progress bar
            <div style={css!{
                background: var(--surface);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: var(--radius);
                padding: 16px;
            }}>
                <span style={css!{ color: var(--text-muted); font-size: 12px; }}>
                    {format!("Completion: {:.0}%", pct * 100.0)}
                </span>
                <div style={css!{
                    background: var(--border);
                    border-radius: 4px;
                    height: 8px;
                    margin-top: 8px;
                    overflow: hidden;
                }}>
                    <div style={css!{
                        background: var(--primary);
                        height: 8px;
                    }} class={tw_dynamic(&format!("w-[{}%]", (pct * 100.0) as u32))} />
                </div>
            </div>
        </div>
    }
}

fn stat_card(label: &str, value: &str) -> impl gpui::IntoElement {
    view! {
        <div style={css!{
            background: var(--surface);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 16px;
            flex: 1;
        }}>
            <span style={css!{ color: var(--text-muted); font-size: 12px; }}>
                {label.to_string()}
            </span>
            <div style={css!{ color: var(--text); font-size: 24px; font-weight: bold; }}>
                {value.to_string()}
            </div>
        </div>
    }
}

/// Tasks view — todo list with add form, filter buttons, and delete confirmation.
pub fn tasks_view(
    todos: ReadSignal<Vec<Todo>>,
    set_todos: WriteSignal<Vec<Todo>>,
    dialog_open: ReadSignal<bool>,
    set_dialog_open: WriteSignal<bool>,
) -> impl gpui::IntoElement {
    let (new_text, set_new_text) = create_signal(String::new());
    let (filter, set_filter) = create_signal("all".to_string());
    let (next_id, set_next_id) = create_signal(1u32);
    let (delete_id, set_delete_id) = create_signal(0u32);

    let visible = create_memo({
        let todos = todos.clone();
        let filter = filter.clone();
        move || {
            let f = filter.get();
            let all = todos.get();
            match f.as_str() {
                "active" => all.into_iter().filter(|t| !t.done).collect::<Vec<_>>(),
                "done" => all.into_iter().filter(|t| t.done).collect::<Vec<_>>(),
                _ => all,
            }
        }
    });

    let remaining = create_memo({
        let todos = todos.clone();
        move || todos.get().iter().filter(|t| !t.done).count()
    });

    let set_todos_add = set_todos.clone();
    let set_todos_toggle = set_todos.clone();
    let set_todos_delete = set_todos.clone();
    let set_todos_delete_confirm = set_todos.clone();
    let set_filter_all = set_filter.clone();
    let set_filter_active = set_filter.clone();
    let set_filter_done = set_filter.clone();
    let current_filter = filter.get();
    let delete_id_read = delete_id.clone();
    let set_dialog_close = set_dialog_open.clone();
    let set_dialog_confirm = set_dialog_open.clone();
    let new_text_read = new_text.clone();
    let set_new_text_input = set_new_text.clone();
    let set_dialog_cancel = set_dialog_open.clone();

    view! {
        <div class="flex flex-col gap-3 p-4" style={css!{ flex: 1; overflow-y: auto; }}>
            <h2 class="text-xl font-bold" style={css!{ color: var(--text); }}>
                {"Tasks"}
            </h2>

            // Add form
            <form
                on:submit={move |cx: &mut gpui::App| {
                    let text = new_text.get();
                    if !text.is_empty() {
                        let id = next_id.get_with(cx);
                        set_todos_add.update(cx, |todos| {
                            todos.push(Todo { id, text, done: false });
                        });
                        set_next_id.update(cx, |n| *n += 1);
                        set_new_text.set(cx, String::new());
                    }
                }}
            >
                <div class="flex flex-row gap-2">
                    <input
                        type="text"
                        placeholder="Add a task..."
                        value={new_text_read.get()}
                        on:input={move |v: &str, cx: &mut gpui::App| set_new_text_input.set(cx, v.to_string())}
                        style={css!{
                            flex: 1;
                            background: var(--surface);
                            color: var(--text);
                            border-width: 1px;
                            border-style: solid;
                            border-color: var(--border);
                            border-radius: var(--radius);
                            padding: 8px 12px;
                        }}
                    />
                    <input
                        type="submit"
                        value="Add"
                        style={css!{
                            background: var(--primary);
                            color: #ffffff;
                            border-width: 0px;
                            border-radius: var(--radius);
                            padding: 8px 16px;
                            cursor: pointer;
                        }}
                    />
                </div>
            </form>

            // Filter buttons
            <div class="flex flex-row gap-2">
                {filter_button("All", current_filter == "all", move |cx| set_filter_all.set(cx, "all".to_string()))}
                {filter_button("Active", current_filter == "active", move |cx| set_filter_active.set(cx, "active".to_string()))}
                {filter_button("Done", current_filter == "done", move |cx| set_filter_done.set(cx, "done".to_string()))}
            </div>

            // Todo list
            <div class="flex flex-col gap-2" style={css!{ flex: 1; }}>
                <Show when={!visible.get().is_empty()} fallback={view! {
                    <div style={css!{ color: var(--text-muted); text-align: center; padding: 24px; }}>
                        {"No tasks here."}
                    </div>
                }}>
                    {for_each(visible.get(), move |todo: Todo, _| {
                        let set_todos_t = set_todos_toggle.clone();
                        let set_todos_d = set_todos_delete.clone();
                        let set_dialog_o = set_dialog_open.clone();
                        let set_did = set_delete_id.clone();
                        let id = todo.id;
                        let text = todo.text.clone();
                        let done = todo.done;
                        view! {
                            <div class="flex flex-row items-center gap-2" style={css!{
                                background: var(--surface);
                                border-width: 1px;
                                border-style: solid;
                                border-color: var(--border);
                                border-radius: var(--radius);
                                padding: 8px 12px;
                            }}>
                                <button
                                    class={twc!(
                                        "text-white text-xs cursor-pointer",
                                        done.then_some("bg-[#2563ff]"),
                                        (!done).then_some("bg-transparent")
                                    )}
                                    style={css!{
                                        width: 20px;
                                        height: 20px;
                                        border-width: 2px;
                                        border-style: solid;
                                        border-color: var(--border);
                                        border-radius: 4px;
                                    }}
                                    on:click={click(move |cx| set_todos_t.update(cx, |todos| {
                                        if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
                                            t.done = !t.done;
                                        }
                                    }))}
                                >
                                    {if done { "x" } else { "" }}
                                </button>
                                <span class={twc!(
                                    "flex-1 text-sm",
                                    done.then_some("line-through"),
                                    (!done).then_some("no-underline")
                                )} style={css!{ color: var(--text); }}>
                                    {text}
                                </span>
                                <button
                                    style={css!{
                                        background: #dc2626;
                                        color: #ffffff;
                                        border-width: 0px;
                                        border-radius: 4px;
                                        padding: 4px 8px;
                                        font-size: 12px;
                                        cursor: pointer;
                                    }}
                                    on:click={click(move |cx| {
                                        set_did.set(cx, id);
                                        set_dialog_o.set(cx, true);
                                    })}
                                >
                                    {"Delete"}
                                </button>
                            </div>
                        }
                    })}
                </Show>
            </div>

            // Footer
            <div class="flex flex-row justify-between items-center" style={css!{
                border-top-width: 1px;
                border-style: solid;
                border-color: var(--border);
            }}>
                <span style={css!{ color: var(--text-muted); font-size: 12px; }}>
                    {format!("{} items left", remaining.get())}
                </span>
            </div>

            // Delete confirmation dialog
            {dialog(dialog_open.get(), move |cx| set_dialog_close.set(cx, false), view! {
                <div style={css!{
                    background: var(--surface);
                    color: var(--text);
                    padding: 20px;
                    border-radius: var(--radius);
                    max-width: 300px;
                }}>
                    <h3 style={css!{ font-weight: bold; font-size: 16px; margin-bottom: 8px; }}>
                        {"Delete task?"}
                    </h3>
                    <p style={css!{ color: var(--text-muted); font-size: 14px; margin-bottom: 16px; }}>
                        {"This action cannot be undone."}
                    </p>
                    <div class="flex flex-row gap-2 justify-end">
                        <button
                            style={css!{
                                padding: 6px 16px;
                                background: var(--border);
                                color: var(--text);
                                border-width: 0px;
                                border-radius: 4px;
                                cursor: pointer;
                            }}
                            on:click={click(move |cx| set_dialog_cancel.set(cx, false))}
                        >
                            {"Cancel"}
                        </button>
                        <button
                            style={css!{
                                padding: 6px 16px;
                                background: #dc2626;
                                color: #ffffff;
                                border-width: 0px;
                                border-radius: 4px;
                                cursor: pointer;
                            }}
                            on:click={click(move |cx| {
                                let id = delete_id_read.get();
                                set_todos_delete_confirm.update(cx, |todos| {
                                    todos.retain(|t| t.id != id);
                                });
                                set_dialog_confirm.set(cx, false);
                            })}
                        >
                            {"Delete"}
                        </button>
                    </div>
                </div>
            })}
        </div>
    }
}

fn filter_button(
    label: &'static str,
    active: bool,
    on_click: impl Fn(&mut gpui::App) + 'static,
) -> impl gpui::IntoElement {
    let bg = if active { "var(--primary)" } else { "var(--surface)" };
    let color = if active { "#ffffff" } else { "var(--text-muted)" };
    view! {
        <button
            style={css!{
                padding: 4px 12px;
                background: #444444;
                color: #aaaaaa;
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: 4px;
                cursor: pointer;
                font-size: 12px;
            }}
            class={twc!(active.then_some("bg-[#2563ff] text-white"), (!active).then_some("bg-[#2d2d44] text-[#aaa]"))}
            on:click={click(on_click)}
        >
            {label}
        </button>
    }
}

/// Settings view — form with name/email fields.
pub fn settings_view(
    name: ReadSignal<String>,
    set_name: WriteSignal<String>,
    email: ReadSignal<String>,
    set_email: WriteSignal<String>,
) -> impl gpui::IntoElement {
    let (saved, set_saved) = create_signal(false);
    let set_name_clone = set_name.clone();
    let set_email_clone = set_email.clone();
    let set_saved_reset = set_saved.clone();

    view! {
        <div class="flex flex-col gap-4 p-4" style={css!{ flex: 1; overflow-y: auto; }}>
            <h2 class="text-xl font-bold" style={css!{ color: var(--text); }}>
                {"Settings"}
            </h2>

            <form
                on:submit={move |cx: &mut gpui::App| {
                    set_saved.set(cx, true);
                }}
                on:reset={move |cx: &mut gpui::App| {
                    set_name_clone.set(cx, String::new());
                    set_saved_reset.set(cx, false);
                }}
            >
                <div class="flex flex-col gap-3">
                    <label class="flex flex-col gap-1">
                        <span style={css!{ color: var(--text-muted); font-size: 12px; }}>
                            {"Name"}
                        </span>
                        <input
                            type="text"
                            placeholder="Your name"
                            value={name.get()}
                            on:input={move |v: &str, cx: &mut gpui::App| set_name.set(cx, v.to_string())}
                            style={css!{
                                background: var(--surface);
                                color: var(--text);
                                border-width: 1px;
                                border-style: solid;
                                border-color: var(--border);
                                border-radius: var(--radius);
                                padding: 8px 12px;
                            }}
                        />
                    </label>

                    <label class="flex flex-col gap-1">
                        <span style={css!{ color: var(--text-muted); font-size: 12px; }}>
                            {"Email"}
                        </span>
                        <input
                            type="email"
                            placeholder="you@example.com"
                            value={email.get()}
                            on:input={move |v: &str, cx: &mut gpui::App| set_email.set(cx, v.to_string())}
                            style={css!{
                                background: var(--surface);
                                color: var(--text);
                                border-width: 1px;
                                border-style: solid;
                                border-color: var(--border);
                                border-radius: var(--radius);
                                padding: 8px 12px;
                            }}
                        />
                    </label>

                    <div class="flex flex-row gap-3">
                        <input
                            type="submit"
                            value="Save"
                            style={css!{
                                background: var(--primary);
                                color: #ffffff;
                                border-width: 0px;
                                border-radius: var(--radius);
                                padding: 8px 16px;
                                cursor: pointer;
                            }}
                        />
                        <input
                            type="reset"
                            value="Reset"
                            style={css!{
                                background: var(--surface);
                                color: var(--text);
                                border-width: 1px;
                                border-style: solid;
                                border-color: var(--border);
                                border-radius: var(--radius);
                                padding: 8px 16px;
                                cursor: pointer;
                            }}
                        />
                    </div>
                </div>
            </form>

            <Show when={saved.get()}>
                <div style={css!{
                    background: var(--surface);
                    border-width: 1px;
                    border-style: solid;
                    border-color: var(--border);
                    border-radius: var(--radius);
                    padding: 12px;
                    color: var(--text);
                    font-size: 14px;
                }}>
                    {"Settings saved!"}
                </div>
            </Show>
        </div>
    }
}
```

## Key Concepts

### Router + `match_route` cascade for view dispatch

`create_router("/")` creates a router backed by a path signal. Because
`router.render(cx, ...)` requires an `&App` context that is unavailable inside
the `app()` render function, the dashboard dispatches views with a cascade of
`router.match_route(pattern)` calls inside `route_content`. `match_route` reads
`router.path()` reactively (no `cx` needed) and returns `Option<RouteMatch>`.
Each branch returns an `impl IntoElement` converted via `.into_any_element()`,
falling through to a 404 view. Navigation happens in event handlers with
`router.navigate(cx, "/tasks")`, which updates the path signal and triggers a
re-render.

### `theme!` macro with CSS variables

`theme.rs` defines `light_theme()` and `dark_theme()` using the `theme!` macro,
which declares CSS custom properties (`--bg`, `--surface`, `--primary`,
`--text`, `--text-muted`, `--border`, `--radius`). `apply_theme(mode)` calls
`set_theme(...)` to install the active theme. Because `theme_mode.get()` is read
inside `app()`, toggling the mode re-runs render, re-installs the theme, and
every `var(--name)` reference in `css!` blocks picks up the new value — the
entire UI re-themes reactively.

### `Context<T>` + `<Provider>` for theme mode propagation

`THEME_CTX: Context<ThemeMode>` is a static context marker. The root view wraps
the app in `<Provider context={THEME_CTX} value={mode}>`, making the current
theme mode available to any descendant via `use_context`. This demonstrates how
cross-cutting state propagates through the element tree without prop drilling.

### Form `on:submit` for todo creation

The tasks view wraps the text input and submit button in a `<form>` with an
`on:submit` handler. Pressing Enter inside the text input submits the form
automatically, firing the handler which reads `new_text`, pushes a new `Todo`,
increments the next id, and clears the input. The settings view uses the same
pattern with `on:submit` (to show a saved confirmation) and `on:reset` (to clear
the fields).

### `dialog()` for delete confirmation

Clicking a todo's Delete button stores the target id in a signal and opens a
`dialog()` overlay. `dialog(open, on_close, content)` renders the content on a
deferred layer with a backdrop and Escape/click-outside dismissal. The dialog's
Confirm button removes the todo by id and closes the dialog; Cancel just closes
it.

### `<For>` + `<Show>` for list rendering

The todo list uses `for_each(visible.get(), |todo, _| view!{ ... })` to render
each item. The whole list is wrapped in `<Show when={!visible.get().is_empty()}
fallback={...}>` so an empty filtered list displays a "No tasks here." message
instead of nothing. The settings view uses `<Show when={saved.get()}>` to reveal
a saved-confirmation card after submit.

### `create_memo` for derived state

Two memos derive from the todos signal: `visible` filters the list by the
current filter string (`all`/`active`/`done`), and `remaining` counts incomplete
items. The dashboard view computes completion stats (`total`, `done`, `active`,
`pct`) directly from `todos.get()` and renders a progress bar whose width is a
runtime `tw_dynamic` class. Memos recompute only when their dependencies change,
not on every render.

### Multi-module project structure

The example is split across three files to show a realistic project layout:
`main.rs` owns the `Todo` struct, router setup, state signals, theme
application, the `<Provider>` wrapper, and the `route_content` dispatch;
`theme.rs` owns the theme definitions, `ThemeMode` enum, the context marker, and
`apply_theme`; `views.rs` owns the three page views (`dashboard_view`,
`tasks_view`, `settings_view`) and helpers (`stat_card`, `filter_button`).
Modules are declared with `mod theme;` / `mod views;` and referenced as
`theme::THEME_CTX`, `views::dashboard_view`, etc.

### Running

**Native:**

    cargo run -p vgui-dashboard

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-dashboard --release
    wasm-bindgen --target web --out-dir examples/dashboard/dist \
        --no-typescript target/wasm32-unknown-unknown/release/dashboard.wasm
    python3 scripts/serve_plain.py 8080 examples/dashboard
