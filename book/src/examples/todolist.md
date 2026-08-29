# Todo List Example

## Overview

The todo list is a larger application that demonstrates:

- `Vec<Todo>` state managed with a signal.
- `create_memo` for a filtered list (`visible_todos`) and a derived count
  (`remaining`).
- `<For>` with a `fallback` for list rendering.
- `css!` macro for styling (conditional styles based on state).
- `hover` pseudo-state attribute.
- Component functions (`todo_item`, `filter_button`).
- Signal updates via `.update(cx, |todos| ...)`.

## Source Code

```rust
use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[derive(Clone, PartialEq)]
struct Todo {
    id: u32,
    text: String,
    done: bool,
}

fn todo_item(todo: Todo, set_todos: WriteSignal<Vec<Todo>>) -> impl gpui::IntoElement {
    let id = todo.id;
    let text = todo.text.clone();
    let done = todo.done;
    let text_style = if done {
        css! {
            color: #888;
            text-decoration: line-through;
            flex: 1;
            font-size: 14px;
        }
    } else {
        css! {
            color: #fff;
            flex: 1;
            font-size: 14px;
        }
    };
    let checkbox_style = if done {
        css! {
            width: 24px;
            height: 24px;
            border: 2px solid #888;
            border-radius: 4px;
            background: rgb(34, 197, 94);
            color: #fff;
            font-size: 14px;
            text-align: center;
            line-height: 20px;
        }
    } else {
        css! {
            width: 24px;
            height: 24px;
            border: 2px solid #888;
            border-radius: 4px;
            background: #333;
            color: #fff;
            font-size: 14px;
            text-align: center;
            line-height: 20px;
        }
    };
    let set_todos_toggle = set_todos.clone();
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            background: #3a3a3a;
            border-radius: 4px;
        }}>
            <button
                style={checkbox_style}
                on:click={click(move |cx| {
                    set_todos_toggle.update(cx, |todos| {
                        if let Some(t) = todos.iter_mut().find(|t| t.id == id) {
                            t.done = !t.done;
                        }
                    });
                })}
            >
                {if done { "x" } else { "" }}
            </button>
            <span style={text_style}>
                {text}
            </span>
            <button
                style={css! {
                    padding: 4px 8px;
                    background: #dc2626;
                    border-width: 0px;
                    border-radius: 4px;
                    color: #fff;
                    font-size: 12px;
                }}
                hover={css! { background: #b91c1c; }}
                on:click={click(move |cx| {
                    set_todos.update(cx, |todos| {
                        todos.retain(|t| t.id != id);
                    });
                })}
            >
                {"Delete"}
            </button>
        </div>
    }
}

fn filter_button(
    label: &'static str,
    active: bool,
    on_click: impl Fn(&mut gpui::App) + 'static,
) -> impl gpui::IntoElement {
    let style = if active {
        css! {
            padding: 4px 12px;
            background: rgb(37, 83, 235);
            border-width: 0px;
            border-radius: 4px;
            color: #fff;
        }
    } else {
        css! {
            padding: 4px 12px;
            background: #444;
            border-width: 0px;
            border-radius: 4px;
            color: #fff;
        }
    };
    view! {
        <button
            style={style}
            on:click={click(on_click)}
        >
            {label}
        </button>
    }
}

fn app() -> impl gpui::IntoElement {
    let (todos, set_todos) = create_signal(vec![
        Todo { id: 0, text: "Learn vgui".into(), done: false },
        Todo { id: 1, text: "Build a todo app".into(), done: false },
        Todo { id: 2, text: "Ship it".into(), done: false },
    ]);
    let (next_id, set_next_id) = create_signal(3u32);
    let (filter, set_filter) = create_signal("all".to_string());

    let remaining = create_memo({
        let todos = todos.clone();
        move || todos.get().iter().filter(|t| !t.done).count()
    });

    let visible_todos = create_memo({
        let todos = todos.clone();
        let filter = filter.clone();
        move || {
            let f = filter.get();
            let all = todos.get();
            match f.as_str() {
                "active" => all.into_iter().filter(|t| !t.done).collect::<Vec<_>>(),
                "completed" => all.into_iter().filter(|t| t.done).collect::<Vec<_>>(),
                _ => all,
            }
        }
    });

    let current_filter = filter.get();
    let set_todos_add = set_todos.clone();
    let set_todos_clear = set_todos.clone();
    let set_filter_all = set_filter.clone();
    let set_filter_active = set_filter.clone();
    let set_filter_completed = set_filter.clone();

    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 12px;
            padding: 20px;
            background: rgb(30, 30, 30);
            width: 500px;
            height: 600px;
            color: #fff;
            font-size: 14px;
        }}>
            <span style={css! {
                font-size: 24px;
                font-weight: bold;
                text-align: center;
            }}>
                {"Todo List"}
            </span>

            <button
                style={css! {
                    padding: 8px 16px;
                    background: rgb(37, 83, 235);
                    border-width: 0px;
                    border-radius: 4px;
                    color: #fff;
                    font-size: 14px;
                    text-align: center;
                }}
                hover={css! { background: rgb(29, 78, 216); }}
                on:click={click(move |cx| {
                    let id = next_id.get_with(cx);
                    set_todos_add.update(cx, |todos| {
                        todos.push(Todo { id, text: format!("Task {}", id), done: false });
                    });
                    set_next_id.update(cx, |n| *n += 1);
                })}
            >
                {"+ Add Todo"}
            </button>

            <div style={css! {
                display: flex;
                flex-direction: row;
                gap: 8px;
                justify-content: center;
            }}>
                {filter_button("All", current_filter == "all", move |cx| set_filter_all.set(cx, "all".to_string()))}
                {filter_button("Active", current_filter == "active", move |cx| set_filter_active.set(cx, "active".to_string()))}
                {filter_button("Completed", current_filter == "completed", move |cx| set_filter_completed.set(cx, "completed".to_string()))}
            </div>

            <div style={css! {
                display: flex;
                flex-direction: column;
                gap: 8px;
                flex: 1;
                overflow: hidden;
            }}>
                <For each={visible_todos.get()} fallback={view! {
                    <div style={css! {
                        text-align: center;
                        color: #888;
                        padding: 24px;
                    }}>
                        {"No todos here."}
                    </div>
                }}>
                    {move |todo: Todo, _i: usize| todo_item(todo, set_todos.clone())}
                </For>
            </div>

            <div style={css! {
                display: flex;
                flex-direction: row;
                justify-content: space-between;
                align-items: center;
                padding-top: 8px;
                border: 1px solid #444;
            }}>
                <span style={css! { color: #aaa; }}>
                    {format!("{} items left", remaining.get())}
                </span>
                <button
                    style={css! {
                        padding: 4px 12px;
                        background: #444;
                        border-width: 0px;
                        border-radius: 4px;
                        color: #ccc;
                        font-size: 12px;
                    }}
                    hover={css! { background: #555; }}
                    on:click={click(move |cx| {
                        set_todos_clear.update(cx, |todos| {
                            todos.retain(|t| !t.done);
                        });
                    })}
                >
                    {"Clear completed"}
                </button>
            </div>
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    });
}
```

## Key Concepts

### Conditional `css!` styles

The `todo_item` function selects different `css!` blocks for the text and
checkbox based on the `done` state — strikethrough + gray for completed, white
for active.

### Two memos: filtered list + count

`visible_todos` depends on both `todos` and `filter` — it recomputes when
either changes. `remaining` depends only on `todos`.

### `<For>` with fallback

When `visible_todos` is empty (e.g., filtering "active" with all todos done),
the fallback "No todos here." message renders.

### `hover` pseudo-state

The delete and add buttons use `hover={css! { ... }}` to change the background
color on mouse hover.

### `get_with` for non-tracking reads

`next_id.get_with(cx)` reads the signal's current value from the gpui entity
without registering a dependency — needed because this read happens inside a
click handler, not during render tracking.

### Running

```bash
cargo run -p vgui-todolist
```
