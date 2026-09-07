#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{hsla, px, size, App, Bounds, Hsla, WindowBounds, WindowOptions};
use vgui::for_each;
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// ── Drag value types ─────────────────────────────────────────────────

/// A draggable color card. `Clone + Render` lets the default drag
/// preview constructor work without an explicit `drag:preview={...}`.
#[derive(Clone, Copy, PartialEq)]
struct DragItem {
    id: usize,
    color: Hsla,
}

impl Render for DragItem {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        let color = self.color;
        let id = self.id;
        view! {
            <div class="w-20 h-20 rounded-lg border-2 flex items-center justify-center text-white text-sm"
                 style={vgui::Css::new(move |s| {
                    s.background = Some(color.into());
                    s.border_color = Some(hsla(0.0, 0.0, 1.0, 0.5).into());
                })}>
                {format!("#{}", id)}
            </div>
        }
    }
}

/// Drag value for tab reordering. Implements `Render` so the default
/// drag preview works.
#[derive(Clone, Copy)]
struct TabDrag {
    ix: usize,
}

impl Render for TabDrag {
    fn render(&mut self, _window: &mut gpui::Window, _cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        view! {
            <div class="px-3 py-1 rounded bg-[#45475a] text-white text-xs">
                {"move tab"}
            </div>
        }
    }
}

// ── App ──────────────────────────────────────────────────────────────

fn app() -> impl gpui::IntoElement {
    // Last dropped color card.
    let (dropped, set_dropped) = create_signal::<Option<DragItem>>(None);

    // Dropped file paths (OS file drag — native only).
    let (files, set_files) = create_signal::<Vec<String>>(Vec::new());

    // Tab reordering.
    let tabs = vec!["Inbox", "Drafts", "Sent", "Archive", "Spam"];
    let (tab_order, set_tab_order) = create_signal::<Vec<usize>>((0..tabs.len()).collect());

    let colors = [
        hsla(0.0, 0.7, 0.5, 1.0),       // red
        hsla(0.33, 0.7, 0.5, 1.0),      // green
        hsla(0.58, 0.7, 0.5, 1.0),      // blue
    ];

    let drop_border = dropped.get().map(|d| d.color).unwrap_or(hsla(0.0, 0.0, 0.5, 1.0));
    let drop_bg = dropped.get().map(|d| hsla(d.color.h, d.color.s, d.color.l, 0.15)).unwrap_or(hsla(0.0, 0.0, 0.0, 0.0));

    view! {
        <div class="flex flex-col gap-4 p-4 bg-[#1e1e2e] w-full h-full text-white overflow-y-auto">
            // ── Draggable color cards ───────────────────────────────
            <div class="flex gap-3">
                {for_each(colors, move |color, ix| {
                    let id = ix + 1;
                    view! {
                        <div drag={DragItem { id, color }}
                             class="w-20 h-20 rounded-lg border-2 flex items-center justify-center text-white text-sm cursor-grab"
                             style={vgui::Css::new(move |s| {
                                s.background = Some(color.into());
                                s.border_color = Some(hsla(0.0, 0.0, 1.0, 0.5).into());
                            })}>
                            {format!("#{}", id)}
                        </div>
                    }
                })}
            </div>

            // ── Drop target ─────────────────────────────────────────
            <div
                class="h-32 rounded-lg border-2 border-dashed flex items-center justify-center text-lg"
                style={vgui::Css::new(move |s| {
                    s.border_color = Some(drop_border.into());
                    s.background = Some(drop_bg.into());
                })}
                on:drop={move |item: &DragItem, _window, cx| {
                    set_dropped.update(cx, |_| Some(*item));
                }}
            >
                {if let Some(d) = dropped.get() {
                    format!("Dropped #{}", d.id)
                } else {
                    "Drop a card here".to_string()
                }}
            </div>

            // ── File drop zone (OS file drag — native only) ─────────
            <div
                class="h-24 rounded-lg border-2 border-dashed border-[#666] flex flex-col items-center justify-center text-sm text-[#aaa] p-2"
                on:drop={move |paths: &ExternalPaths, _window, cx| {
                    set_files.update(cx, |_| paths.paths().iter().map(|p| p.display().to_string()).collect::<Vec<_>>());
                }}
            >
                {if files.get().is_empty() {
                    "Drop files from your file manager here".to_string()
                } else {
                    files.get().join("\n")
                }}
            </div>

            // ── Tab bar with drag reordering ────────────────────────
            <div class="flex gap-1 bg-[#181825] rounded-lg p-1">
                {for_each(tab_order.get(), move |ix, _| {
                    let label = tabs[ix].to_string();
                    let set_tab_order = set_tab_order.clone();
                    view! {
                        <div drag={TabDrag { ix }}
                             on:drop={move |d: &TabDrag, _w, cx| {
                                 set_tab_order.update(cx, |order| {
                                     let from = d.ix;
                                     let to = ix;
                                     if from != to {
                                         let item = order.remove(from);
                                         order.insert(to, item);
                                     }
                                 });
                             }}
                             class="px-4 py-2 rounded bg-[#313244] text-sm cursor-grab hover:bg-[#45475a]">
                            {label}
                        </div>
                    }
                })}
            </div>

            // ── Drag status indicator ───────────────────────────────
            <div class="text-sm text-[#888]">
                {format!("Drag active: {}", has_active_drag())}
            </div>
        </div>
    }
}

// ── Entry points ─────────────────────────────────────────────────────

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(520.), px(560.0)), cx);
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
