//! Command palette — a fuzzy command runner toggled with Ctrl/Cmd+K.
//!
//! Mount [`command_palette`] once near the root of a [`vgui::VguiRoot`]. It
//! registers global key handlers: Ctrl+K (or Cmd+K on macOS) opens a centered
//! overlay listing commands, typing filters by title/keywords, ArrowUp/Down
//! moves the highlight, Enter runs the highlighted command, and Escape closes.
//! Running a command closes the palette and invokes its callback.

use std::rc::Rc;

use gpui::{
    deferred, px, AnyElement, App, ClickEvent, InteractiveElement, MouseButton, MouseDownEvent,
    ParentElement, StatefulInteractiveElement, Styled, Window,
};

use vgui::{create_signal, input_cb, next_auto_id, use_key_down, KeyboardEvent};

/// One runnable command.
pub struct CommandItem {
    /// Display title, also the primary match text.
    pub title: String,
    /// Extra lowercase words used for matching (shortcuts, synonyms…).
    pub keywords: Vec<String>,
    /// Invoked when the command is chosen.
    pub run: Rc<dyn Fn(&mut App)>,
}

impl CommandItem {
    /// Build a command with no extra keywords.
    pub fn new(title: impl Into<String>, run: impl Fn(&mut App) + 'static) -> Self {
        Self {
            title: title.into(),
            keywords: Vec::new(),
            run: Rc::new(run),
        }
    }

    /// Append match keywords.
    pub fn keywords(mut self, words: &[&str]) -> Self {
        self.keywords = words.iter().map(|s| (*s).to_string()).collect();
        self
    }

    fn matches(&self, ql: &str) -> bool {
        if ql.is_empty() {
            return true;
        }
        if self.title.to_lowercase().contains(ql) {
            return true;
        }
        self.keywords.iter().any(|k| k.to_lowercase().contains(ql))
    }
}

/// Filter + rank commands for a lowercased query; at most ten rows.
fn ranked(commands: &[CommandItem], ql: &str) -> Vec<usize> {
    let mut v: Vec<(usize, usize)> = commands
        .iter()
        .enumerate()
        .filter(|(_, c)| c.matches(ql))
        .map(|(idx, c)| {
            let t = c.title.to_lowercase();
            let score = t.find(ql).unwrap_or(0) + t.len();
            (idx, score)
        })
        .collect();
    v.sort_by_key(|(_, score)| *score);
    v.truncate(10);
    v.into_iter().map(|(idx, _)| idx).collect()
}

/// Mount a Ctrl/Cmd+K command palette over `commands`.
pub fn command_palette(commands: Vec<CommandItem>) -> AnyElement {
    let commands = Rc::new(commands);
    let (open, set_open) = create_signal(false);
    let (query, set_query) = create_signal(String::new());
    let (selected, set_selected) = create_signal(0usize);

    // Global hotkey (registered unconditionally): Ctrl/Cmd+K toggles, Esc closes.
    {
        let open_r = open.clone();
        let open_w = set_open.clone();
        use_key_down(move |e: &KeyboardEvent, _w: &mut Window, cx: &mut App| {
            let toggle = (e.ctrl_key || e.meta_key) && e.key.eq_ignore_ascii_case("k");
            if toggle {
                open_w.set(cx, !open_r.get());
                e.stop_propagation();
            } else if e.key == "Escape" && open_r.get() {
                open_w.set(cx, false);
                e.stop_propagation();
            }
        });
    }

    // In-palette navigation (registered unconditionally to keep hook slots
    // stable); no-ops while closed.
    {
        let is_open_r = open.clone();
        let commands = commands.clone();
        let query = query.clone();
        let selected = selected.clone();
        let set_selected = set_selected.clone();
        let set_open = set_open.clone();
        let set_query = set_query.clone();
        use_key_down(move |e: &KeyboardEvent, _w: &mut Window, cx: &mut App| {
            if !is_open_r.get() {
                return;
            }
            let ql = query.get().to_lowercase();
            let rows = ranked(&commands, &ql);
            let count = rows.len();
            match e.key.as_str() {
                "ArrowDown" => {
                    if count > 0 {
                        set_selected.set(cx, (selected.get() + 1) % count);
                    }
                    e.stop_propagation();
                }
                "ArrowUp" => {
                    if count > 0 {
                        set_selected.set(cx, (selected.get() + count - 1) % count);
                    }
                    e.stop_propagation();
                }
                "Enter" => {
                    if let Some(cmd_idx) = rows.get(selected.get()) {
                        let run = commands[*cmd_idx].run.clone();
                        run(cx);
                    }
                    set_open.set(cx, false);
                    set_query.set(cx, String::new());
                    set_selected.set(cx, 0);
                    e.stop_propagation();
                }
                _ => {}
            }
        });
    }

    let is_open = open.get();
    let ql = query.get().to_lowercase();
    let rows = ranked(&commands, &ql);
    let sel = selected.get().min(rows.len().saturating_sub(1));

    // Backdrop: a press outside the panel dismisses.
    let backdrop_dismiss = set_open.clone();
    let backdrop_query = set_query.clone();
    let backdrop_sel = set_selected.clone();
    let mut backdrop = gpui::div()
        .id(("command-backdrop", next_auto_id()))
        .absolute()
        .top_0()
        .left_0()
        .right_0()
        .bottom_0()
        .flex()
        .justify_center()
        .pt(px(80.))
        .on_mouse_down(
            MouseButton::Left,
            move |_: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                backdrop_dismiss.set(cx, false);
                backdrop_query.set(cx, String::new());
                backdrop_sel.set(cx, 0);
            },
        );
    backdrop = vgui::css! {
        background: rgba(0, 0, 0, 0.45);
    }
    .apply(backdrop);

    // Panel stops presses from reaching the backdrop.
    let mut panel = gpui::div()
        .id(("command-panel", next_auto_id()))
        .w(px(520.))
        .on_mouse_down(
            MouseButton::Left,
            move |ev: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                let _ = ev;
                cx.stop_propagation();
            },
        );
    panel = vgui::css! {
        background: var(--popover);
        color: var(--popover-foreground);
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
    }
    .apply(panel)
    .shadow_lg();

    // Search box.
    let search_value = query.clone();
    let search = vgui::view! {
        <input
            type="text"
            value={search_value.get()}
            placeholder="Type a command\u{2026}"
            style={vgui::css! {
                width: 100%;
                height: 40px;
                padding: 8px 14px;
                background: rgba(0, 0, 0, 0);
                color: var(--popover-foreground);
                border-width: 0;
                border-style: solid;
                border-color: var(--border);
                font-size: 14px;
            }}
            on:input={input_cb({
                let set_query = set_query.clone();
                let set_selected = set_selected.clone();
                move |raw, cx| {
                    set_query.set(cx, raw.to_string());
                    set_selected.set(cx, 0);
                }
            })}
        />
    };
    panel = panel.child(search);

    // Result list.
    let mut list = gpui::div().py_1().min_h(px(48.));
    if rows.is_empty() {
        list = list.child(
            vgui::css! {
                padding: 10px 14px;
                font-size: 13px;
                color: var(--muted-foreground);
            }
            .apply(gpui::div().child("No matching commands")),
        );
    } else {
        for (row, cmd_idx) in rows.iter().enumerate() {
            let title = commands[*cmd_idx].title.clone();
            let run = commands[*cmd_idx].run.clone();
            let set_open = set_open.clone();
            let set_query = set_query.clone();
            let set_selected = set_selected.clone();
            let item = gpui::div()
                .id(("command-item", next_auto_id()))
                .w_full()
                .px_3()
                .py_1()
                .cursor_pointer()
                .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                    run(cx);
                    set_open.set(cx, false);
                    set_query.set(cx, String::new());
                    set_selected.set(cx, 0);
                })
                .child(title);
            let item = vgui::css! {
                font-size: 13px;
                border-radius: var(--radius);
                margin: 1px 6px;
            }
            .apply(item);
            let item = if row == sel {
                vgui::css! { background: var(--accent); color: var(--accent-foreground); }
                    .apply(item)
            } else {
                vgui::css! { color: var(--popover-foreground); }.apply(item)
            };
            list = list.child(item);
        }
    }
    panel = panel.child(list);

    // Construct the overlay on every render so the search input's reactive
    // slot count is stable; mount it only while open.
    vgui::show(is_open, deferred(backdrop.child(panel)), gpui::Empty)
}
