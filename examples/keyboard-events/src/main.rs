#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;
use vgui::show_when;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

/// A command entry in the palette.
struct Command {
    name: &'static str,
    shortcut: &'static str,
}

const COMMANDS: &[Command] = &[
    Command { name: "New File",        shortcut: "Ctrl+N" },
    Command { name: "Open File",       shortcut: "Ctrl+O" },
    Command { name: "Save",            shortcut: "Ctrl+S" },
    Command { name: "Save As",         shortcut: "Ctrl+Shift+S" },
    Command { name: "Find",            shortcut: "Ctrl+F" },
    Command { name: "Replace",         shortcut: "Ctrl+H" },
    Command { name: "Toggle Terminal", shortcut: "Ctrl+`" },
    Command { name: "Format Document", shortcut: "Shift+Alt+F" },
    Command { name: "Command Palette", shortcut: "Ctrl+K" },
    Command { name: "Close Tab",       shortcut: "Ctrl+W" },
    Command { name: "Undo",            shortcut: "Ctrl+Z" },
    Command { name: "Redo",            shortcut: "Ctrl+Y" },
];

fn app() -> impl gpui::IntoElement {
    // ── State ──────────────────────────────────────────────────────
    let (palette_open, set_palette) = create_signal(false);
    let (selected, set_selected) = create_signal(0usize);
    let (last_key, set_last_key) = create_signal(String::new());
    let (last_action, set_last_action) = create_signal(String::from("Press ? for help"));
    let (help_open, set_help) = create_signal(false);

    // ── Global keyboard shortcuts (use_key_down) ───────────────────
    // These fire regardless of which element has focus. They form the
    // application-level keyboard layer: palette toggle, help, and
    // navigation within the palette.
    let set_palette_open = set_palette.clone();
    let set_palette_close = set_palette.clone();
    let set_help_open = set_help.clone();
    let set_help_close = set_help.clone();
    let set_sel = set_selected.clone();
    let set_act = set_last_action.clone();
    let set_key = set_last_key.clone();
    let palette_open_r = palette_open.clone();
    let selected_r = selected.clone();

    use_key_down(move |e: &KeyboardEvent, _w, cx| {
        // Live key inspector — record every key press.
        set_key.set(cx, format_key_event(e));

        // Ctrl+K toggles the command palette.
        if e.ctrl_key && e.key == "k" {
            set_palette_open.update(cx, |v| *v = !*v);
            set_sel.set(cx, 0);
            return; // don't process further
        }

        // Escape closes whatever is open.
        if e.key == "Escape" {
            set_palette_close.set(cx, false);
            set_help_close.set(cx, false);
            return;
        }

        // "?" toggles help (Shift+/ produces "?").
        if e.key == "?" {
            set_help_open.update(cx, |v| *v = !*v);
            return;
        }

        // Arrow / j/k navigation inside the palette.
        if palette_open_r.get() {
            let count = COMMANDS.len();
            match e.key.as_str() {
                "ArrowDown" | "j" => {
                    set_sel.update(cx, |s| *s = (*s + 1) % count);
                }
                "ArrowUp" | "k" => {
                    set_sel.update(cx, |s| *s = if *s == 0 { count - 1 } else { *s - 1 });
                }
                "Enter" => {
                    let idx = selected_r.get();
                    let cmd = &COMMANDS[idx.min(count - 1)];
                    set_act.set(cx, format!("Executed: {}", cmd.name));
                    set_palette_close.set(cx, false);
                }
                _ => {}
            }
        }
    });

    // ── use_key_up: track key release for modifier display ─────────
    let set_key_up = set_last_key.clone();
    use_key_up(move |e: &KeyboardEvent, _w, cx| {
        // Append " ↑" on key release so the inspector shows down/up cycle.
        if e.key == "Shift" || e.key == "Control" || e.key == "Alt" || e.key == "Meta" {
            set_key_up.update(cx, |s| {
                *s = format!("{} ↑", e.key);
            });
        }
    });

    // ── View ───────────────────────────────────────────────────────
    let set_pal = set_palette.clone();
    let set_act_btn = set_last_action.clone();
    let set_pal_btn = set_palette.clone();

    view! {
        <div class="flex flex-col gap-3 p-6 bg-[#1e1e2e] w-full h-full text-white rounded-lg overflow-hidden">
            // ── Header ──────────────────────────────────────────────
            <div class="flex flex-row items-center justify-between">
                <h2 class="text-lg font-bold text-[#cdd6f4]">{"Keyboard Events"}</h2>
                <span class="text-xs text-[#6c7086]">
                    {"Ctrl+K palette · ? help · Esc close"}
                </span>
            </div>

            // ── Last action banner ──────────────────────────────────
            <div class="px-3 py-2 rounded bg-[#313244] text-sm text-[#a6e3a1] font-mono">
                {last_action.get()}
            </div>

            // ── Key event inspector (element-level on:keydown) ──────
            // This div has its own on:keydown handler that shows the
            // raw KeyboardEvent fields. It calls stop_propagation on
            // Tab so the global handler doesn't also process it —
            // demonstrating the bubbling control.
            <div
                class="px-3 py-2 rounded bg-[#181825] border border-[#313244] text-xs font-mono text-[#94e2d5] cursor_pointer"
                tabindex="0"
                on:keydown={move |e: &KeyboardEvent, _w, _cx| {
                    // When the inspector itself is focused, swallow Tab
                    // so focus stays here and the global handler doesn't
                    // re-process it.
                    if e.key == "Tab" {
                        e.stop_propagation();
                    }
                }}
            >
                <div class="text-[#6c7086] mb-1">{"Click here, then press keys — Tab is captured locally:"}</div>
                <div class="text-[#cdd6f4] text-sm">
                    {if last_key.get().is_empty() {
                        "—".to_string()
                    } else {
                        last_key.get()
                    }}
                </div>
            </div>

            // ── Palette toggle button ───────────────────────────────
            <div class="flex flex-row gap-2">
                <button
                    class="px-3 py-2 rounded bg-[#89b4fa] hover:bg-[#74a0f0] text-[#1e1e2e] text-sm font-medium"
                    on:click={click(move |cx| {
                        set_pal.update(cx, |v| *v = !*v);
                        set_selected.set(cx, 0);
                    })}
                >
                    {"Open Palette (Ctrl+K)"}
                </button>
                <button
                    class="px-3 py-2 rounded bg-[#313244] hover:bg-[#45475a] text-[#cdd6f4] text-sm"
                    on:click={click(move |cx| set_act_btn.set(cx, "Button clicked".to_string()))}
                >
                    {"Dummy Action"}
                </button>
            </div>

            // ── Command palette overlay ─────────────────────────────
            // Rendered inline (not a <dialog>) so the keyboard event
            // layering is visible: global use_key_down handles arrow
            // navigation and Enter, while the palette is just a visual
            {show_when(palette_open.get(), command_palette(selected.get()))}

            // ── Help overlay ────────────────────────────────────────
            {show_when(help_open.get(), help_panel(set_pal_btn.clone()))}
        </div>
    }
}

/// Command palette overlay — a visual surface for keyboard navigation.
/// All keyboard handling (arrows, j/k, Enter, Esc) is done by the global
/// `use_key_down` handler in `app()`. This component is purely presentational.
#[allow(non_snake_case)]
fn command_palette(selected: usize) -> impl gpui::IntoElement {
    let cmds: Vec<(usize, &'static Command)> = COMMANDS.iter().enumerate().collect();
    view! {
        <div class="absolute inset-0 bg-black/40 flex items-start justify-center pt-12 z-10">
            <div class="bg-[#1e1e2e] border border-[#313244] rounded-lg w-[400px] flex flex-col gap-1 p-2">
                <div class="px-3 py-2 text-xs text-[#6c7086] border-b border-[#313244] mb-1">
                    {"Type Up/Down or j/k to navigate - Enter to execute - Esc to close"}
                </div>
                {vgui::for_each(cmds, move |(idx, cmd), _| {
                    let class = if idx == selected {
                        "px-3 py-2 rounded text-sm flex flex-row justify-between items-center cursor_pointer bg-[#89b4fa] text-[#1e1e2e]"
                    } else {
                        "px-3 py-2 rounded text-sm flex flex-row justify-between items-center cursor_pointer text-[#cdd6f4] hover:bg-[#313244]"
                    };
                    view! {
                        <div class={class}>
                            <span>{cmd.name}</span>
                            <span class="text-xs font-mono opacity-60">{cmd.shortcut}</span>
                        </div>
                    }
                })}
            </div>
        </div>
    }
}

/// Help overlay explaining the three keyboard event layers.
#[allow(non_snake_case)]
fn help_panel(close_setter: WriteSignal<bool>) -> impl gpui::IntoElement {
    view! {
        <div class="absolute inset-0 bg-black/40 flex items-center justify-center z-20">
            <div class="bg-[#1e1e2e] border border-[#313244] rounded-lg w-[380px] p-5 flex flex-col gap-3">
                <h3 class="text-sm font-bold text-[#cdd6f4]">{"Keyboard Event Layers"}</h3>
                <div class="flex flex-col gap-2 text-xs text-[#a6adc8]">
                    <div>
                        <span class="text-[#89b4fa] font-mono">{"use_key_down"}</span>
                        {" — global handler, fires on every key press regardless of focus. Handles Ctrl+K, Escape, ?, arrows, j/k, Enter."}
                    </div>
                    <div>
                        <span class="text-[#f9e2af] font-mono">{"on:keydown"}</span>
                        {" — element-level handler on the inspector div. Calls stop_propagation() on Tab to prevent the global handler from reprocessing it."}
                    </div>
                    <div>
                        <span class="text-[#a6e3a1] font-mono">{"use_key_up"}</span>
                        {" — global key-release handler, tracks modifier key up events."}
                    </div>
                    <div>
                        <span class="text-[#f38ba8] font-mono">{"stop_propagation"}</span>
                        {" — sets a flag on KeyboardEvent that halts dispatch to subsequent global handlers and prevents gpui event bubbling."}
                    </div>
                </div>
                <button
                    class="px-3 py-2 rounded bg-[#313244] hover:bg-[#45475a] text-[#cdd6f4] text-sm self-end"
                    on:click={click(move |cx| close_setter.set(cx, false))}
                >
                    {"Close (Esc)"}
                </button>
            </div>
        </div>
    }
}

/// Format a `KeyboardEvent` into a compact inspector string showing all
/// fields: key, code, modifiers, repeat, key_char.
fn format_key_event(e: &KeyboardEvent) -> String {
    let mut mods = String::new();
    if e.ctrl_key { mods.push_str("Ctrl+"); }
    if e.shift_key { mods.push_str("Shift+"); }
    if e.alt_key { mods.push_str("Alt+"); }
    if e.meta_key { mods.push_str("Meta+"); }
    let repeat = if e.repeat { " (repeat)" } else { "" };
    let kc = e.key_char.as_deref().unwrap_or("");
    format!(
        "{}{}  [code:{}]  char:{:?}{}",
        mods, e.key, e.code, kc, repeat,
    )
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(560.), px(440.0)), cx);
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
