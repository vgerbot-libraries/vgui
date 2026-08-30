#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

/// Light theme built with the `theme!` macro.
fn light_theme() -> Theme {
    theme! {
        --bg: #f8f9fa;
        --surface: #ffffff;
        --text: #1a1a1a;
        --text-muted: #6c757d;
        --primary: #2563ff;
        --primary-hover: #1d4eff;
        --border: #dee2e6;
        --radius: 8px;
        --spacing: 16px;
    }
}

/// Dark theme — same variable names, different values.
fn dark_theme() -> Theme {
    theme! {
        --bg: #1a1a2f;
        --surface: #0f1623;
        --text: #e0e0e0;
        --text-muted: #8892b0;
        --primary: #4dabff;
        --primary-hover: #339aff;
        --border: #2d3748;
        --radius: 8px;
        --spacing: 16px;
    }
}

fn app() -> impl gpui::IntoElement {
    let (dark, set_dark) = create_signal(false);

    // Install the theme inside the render closure. Reading `dark.get()`
    // registers a reactive dependency, so toggling the signal re-runs render,
    // re-sets the theme, and rebuilds all styled elements.
    set_theme(if dark.get() {
        dark_theme()
    } else {
        light_theme()
    });

    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: var(--spacing);
            padding: var(--spacing);
            background: var(--bg);
            color: var(--text);
            width: 480px;
            height: 400px;
            font-size: 16px;
        }}>
            // Header row: title + toggle button
            <div style={css! {
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}>
                <span style={css! {
                    font-size: 22px;
                    font-weight: bold;
                }}>
                    {"CSS Variables Theming"}
                </span>
                <button
                    style={css! {
                        padding: 8px 16px;
                        background: var(--primary);
                        color: #ffffff;
                        border-radius: var(--radius);
                        border-width: 0px;
                        cursor: pointer;
                        font-size: 14px;
                    }}
                    hover={css! {
                        background: var(--primary-hover);
                    }}
                    on:click={click(move |cx| set_dark.update(cx, |v| *v = !*v))}
                >
                    {if dark.get() { "🌙 Dark" } else { "☀ Light" }}
                </button>
            </div>

            // Card 1 — uses var() for background, border, radius
            <div style={css! {
                background: var(--surface);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: var(--radius);
                padding: var(--spacing);
                display: flex;
                flex-direction: column;
                gap: 8px;
            }}>
                <span style={css! {
                    font-weight: bold;
                    font-size: 18px;
                }}>
                    {"Theme via var()"}
                </span>
                <span style={css! {
                    color: var(--text-muted);
                    font-size: 14px;
                }}>
                    {"Every color, spacing, and radius on this page reads from CSS variables. Toggle the theme to see them update reactively."}
                </span>
            </div>

            // Card 2 — gradient with var() color args
            <div style={css! {
                background: linear-gradient(135deg, var(--primary), var(--surface));
                border-radius: var(--radius);
                padding: var(--spacing);
                display: flex;
                flex-direction: column;
                gap: 8px;
            }}>
                <span style={css! {
                    font-weight: bold;
                    font-size: 18px;
                    color: #ffffff;
                }}>
                    {"Gradient with var()"}
                </span>
                <span style={css! {
                    color: #ffffff;
                    font-size: 14px;
                }}>
                    {"linear-gradient(135deg, var(--primary), var(--surface))"}
                </span>
            </div>

            // Card 3 — keyword + number vars
            <div style={css! {
                background: var(--surface);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: var(--radius);
                padding: var(--spacing);
                display: flex;
                flex-direction: column;
                gap: 8px;
            }}>
                <span style={css! {
                    font-weight: bold;
                    font-size: 18px;
                }}>
                    {"Keyword & number vars"}
                </span>
                <span style={css! {
                    color: var(--text-muted);
                    font-size: 14px;
                    line-height: 1.5;
                }}>
                    {"--spacing is a length, --radius is a length, --primary is a color. All resolve at runtime from the thread-local theme."}
                </span>
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
        let bounds = Bounds::centered(None, size(px(480.), px(400.0)), cx);
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
