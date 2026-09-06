#![cfg_attr(target_family = "wasm", no_main)]

use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;
use vgui::show_when;
use vgui::{CommandOptions, ContextOptions, KeymapOptions};

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
    Command { name: "New File",      shortcut: "Ctrl+N" },
    Command { name: "Save",          shortcut: "Ctrl+S" },
    Command { name: "Save As",       shortcut: "Ctrl+Shift+S" },
    Command { name: "Find",          shortcut: "Ctrl+F" },
    Command { name: "Open Palette",  shortcut: "Ctrl+K, Ctrl+P" },
    Command { name: "Quit",          shortcut: "Ctrl+Q" },
];

fn app() -> impl gpui::IntoElement {
    // ── State ──────────────────────────────────────────────────────
    let (palette_open, set_palette) = create_signal(false);
    let (selected, set_selected) = create_signal(0usize);
    let (last_action, set_last_action) = create_signal(String::from("Press Ctrl+K, Ctrl+P for palette"));
    let (partial_label, set_partial_label) = create_signal(String::new());

    // ── Shortcuts engine ───────────────────────────────────────────
    let sc = use_shortcuts();

    // Build the keymap: a "global" context with app-wide commands, and a
    // "palette" context (fallback to global) with navigation commands.
    let mut commands = HashMap::new();
    commands.insert("new_file".to_string(), CommandOptions {
        shortcut: "Ctrl+N".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("save".to_string(), CommandOptions {
        shortcut: "Ctrl+S".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("save_as".to_string(), CommandOptions {
        shortcut: "Ctrl+Shift+S".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("find".to_string(), CommandOptions {
        shortcut: "Ctrl+F".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("palette".to_string(), CommandOptions {
        shortcut: "Ctrl+K,Ctrl+P".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("quit".to_string(), CommandOptions {
        shortcut: "Ctrl+Q".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    // Palette-context commands
    commands.insert("next".to_string(), CommandOptions {
        shortcut: "Down".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("prev".to_string(), CommandOptions {
        shortcut: "Up".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("select".to_string(), CommandOptions {
        shortcut: "Enter".to_string(), event: None, prevent_default: None, interceptors: None,
    });
    commands.insert("close".to_string(), CommandOptions {
        shortcut: "Escape".to_string(), event: None, prevent_default: None, interceptors: None,
    });

    let mut contexts = HashMap::new();
    contexts.insert("global".to_string(), ContextOptions {
        commands: vec!["new_file".to_string(), "save".to_string(), "save_as".to_string(),
                       "find".to_string(), "palette".to_string(), "quit".to_string()],
        abstract_ctx: None, fallbacks: None,
    });
    contexts.insert("palette".to_string(), ContextOptions {
        commands: vec!["next".to_string(), "prev".to_string(), "select".to_string(), "close".to_string()],
        abstract_ctx: None, fallbacks: Some(vec!["global".to_string()]),
    });

    sc.keymap(KeymapOptions { commands, contexts });

    // ── Partial-match indicator ────────────────────────────────────
    // The on_partial_change handler runs without a gpui context, so we
    // store the names in a shared cell and update the signal from the
    // command handlers (which do have cx). The view re-renders on every
    // signal change, so the label stays current.
    let partial_names: Rc<RefCell<Vec<String>>> = Rc::new(RefCell::new(Vec::new()));
    let pn_clone = partial_names.clone();
    sc.on_partial_change(move |names: &[String]| {
        *pn_clone.borrow_mut() = names.to_vec();
    });

    // Helper to sync the partial-label signal from the shared cell.
    let sync_partial = {
        let pn = partial_names.clone();
        let set_pl = set_partial_label.clone();
        move |cx: &mut App| {
            let names = pn.borrow();
            if names.is_empty() {
                set_pl.set(cx, String::new());
            } else {
                set_pl.set(cx, format!("Partial: {}", names.join(", ")));
            }
        }
    };

    // ── Command handlers ───────────────────────────────────────────
    let set_act = set_last_action.clone();
    sc.on("new_file", move |_sev, _w, cx| {
        set_act.set(cx, "New File".to_string());
    });
    let set_act = set_last_action.clone();
    sc.on("save", move |_sev, _w, cx| {
        set_act.set(cx, "Save".to_string());
    });
    let set_act = set_last_action.clone();
    sc.on("save_as", move |_sev, _w, cx| {
        set_act.set(cx, "Save As".to_string());
    });
    let set_act = set_last_action.clone();
    sc.on("find", move |_sev, _w, cx| {
        set_act.set(cx, "Find".to_string());
    });
    let set_act = set_last_action.clone();
    sc.on("quit", move |_sev, _w, cx| {
        set_act.set(cx, "Quit".to_string());
    });

    // Palette command (sequence: Ctrl+K, Ctrl+P)
    let set_pal = set_palette.clone();
    let set_sel2 = set_selected.clone();
    let sc_pal_inner = sc.clone();
    sc.on("palette", move |_sev, _w, cx| {
        set_pal.set(cx, true);
        set_sel2.set(cx, 0);
        let _guard = sc_pal_inner.switch_context("palette");
        std::mem::forget(_guard);
    });

    // Palette-context commands
    let set_sel_p = set_selected.clone();
    let sync_p = sync_partial.clone();
    sc.on("next", move |_sev, _w, cx| {
        let count = COMMANDS.len();
        set_sel_p.update(cx, |s| *s = (*s + 1) % count);
        sync_p(cx);
    });
    let set_sel_p = set_selected.clone();
    let sync_p = sync_partial.clone();
    sc.on("prev", move |_sev, _w, cx| {
        let count = COMMANDS.len();
        set_sel_p.update(cx, |s| *s = if *s == 0 { count - 1 } else { *s - 1 });
        sync_p(cx);
    });
    let set_act_p = set_last_action.clone();
    let set_pal_p = set_palette.clone();
    let sc_close = sc.clone();
    let selected_clone = selected.clone();
    sc.on("select", move |_sev, _w, cx| {
        let idx = selected_clone.get();
        let cmd = &COMMANDS[idx.min(COMMANDS.len() - 1)];
        set_act_p.set(cx, format!("Executed: {}", cmd.name));
        set_pal_p.set(cx, false);
        let _guard = sc_close.switch_context("global");
        std::mem::forget(_guard);
    });
    let set_pal_c = set_palette.clone();
    let sc_close2 = sc.clone();
    sc.on("close", move |_sev, _w, cx| {
        set_pal_c.set(cx, false);
        let _guard = sc_close2.switch_context("global");
        std::mem::forget(_guard);
    });

    // Start in the global context
    let _global_guard = sc.switch_context("global");
    std::mem::forget(_global_guard);

    // ── View ───────────────────────────────────────────────────────
    let cmds: Vec<(usize, &'static Command)> = COMMANDS.iter().enumerate().collect();

    view! {
        <div class="flex flex-col gap-3 p-6 bg-[#1e1e2e] w-full h-full text-white rounded-lg overflow-hidden">
            // ── Header ──────────────────────────────────────────────
            <div class="flex flex-row items-center justify-between">
                <h2 class="text-lg font-bold text-[#cdd6f4]">{"Keyboard Shortcuts"}</h2>
                <span class="text-xs text-[#6c7086]">
                    {"Ctrl+K, Ctrl+P → palette · Esc → close"}
                </span>
            </div>

            // ── Last action banner ──────────────────────────────────
            <div class="px-3 py-2 rounded bg-[#313244] text-sm text-[#a6e3a1] font-mono">
                {last_action.get()}
            </div>

            // ── Partial-match indicator ─────────────────────────────
            {show_when(!partial_label.get().is_empty(), view! {
                <div class="px-3 py-1 rounded bg-[#181825] border border-[#f9e2af] text-xs font-mono text-[#f9e2af]">
                    {partial_label.get()}
                </div>
            })}

            // ── Shortcut reference ──────────────────────────────────
            <div class="flex flex-col gap-1 px-3 py-2 rounded bg-[#181825] border border-[#313244] text-xs font-mono">
                <div class="text-[#6c7086] mb-1">{"Shortcuts (try these):"}</div>
                {vgui::for_each(cmds, move |(_i, cmd), _| {
                    view! {
                        <div class="flex flex-row justify-between text-[#cdd6f4]">
                            <span>{cmd.name}</span>
                            <span class="text-[#89b4fa]">{cmd.shortcut}</span>
                        </div>
                    }
                })}
            </div>

            // ── Command palette overlay ─────────────────────────────
            {show_when(palette_open.get(), command_palette(selected.get()))}
        </div>
    }
}

/// Command palette overlay — a visual surface for keyboard navigation.
/// All keyboard handling is done by the `shortcuts` engine; this component
/// is purely presentational.
#[allow(non_snake_case)]
fn command_palette(selected: usize) -> impl gpui::IntoElement {
    let cmds: Vec<(usize, &'static Command)> = COMMANDS.iter().enumerate().collect();
    view! {
        <div class="absolute inset-0 bg-black/40 flex items-start justify-center pt-12 z-10">
            <div class="bg-[#1e1e2e] border border-[#313244] rounded-lg w-[400px] flex flex-col gap-1 p-2">
                <div class="px-3 py-2 text-xs text-[#6c7086] border-b border-[#313244] mb-1">
                    {"Up/Down to navigate · Enter to execute · Esc to close"}
                </div>
                {vgui::for_each(cmds, move |(idx, cmd), _| {
                    let class = if idx == selected {
                        "px-3 py-2 rounded text-sm flex flex-row justify-between items-center bg-[#89b4fa] text-[#1e1e2e]"
                    } else {
                        "px-3 py-2 rounded text-sm flex flex-row justify-between items-center text-[#cdd6f4] hover:bg-[#313244]"
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

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(560.), px(480.0)), cx);
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
