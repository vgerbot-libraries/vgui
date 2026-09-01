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
