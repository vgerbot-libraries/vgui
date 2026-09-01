use parking_lot::Mutex;
use std::sync::Arc;

use gpui::{
    px, size, App, Bounds, Entity, EntityInputHandler, Focusable, Render, Window, WindowBounds,
    WindowOptions,
};
use vgui::prelude::*;
use vgui::{TextInput, TextInputProps};

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
struct ReproRoot {
    input: Entity<TextInput>,
    focus: gpui::FocusHandle,
    last_value: Arc<Mutex<String>>,
}

impl ReproRoot {
    fn new(cx: &mut gpui::Context<Self>) -> Self {
        let last_value: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
        let captured = last_value.clone();
        let input = cx.new(|cx| {
            let mut ti = TextInput::new(cx);
            ti.sync_from_props(
                TextInputProps {
                    kind: TextKind::Text,
                    multiline: false,
                    value: String::new(),
                    placeholder: Some("type here".into()),
                    disabled: false,
                    readonly: false,
                    min: None,
                    max: None,
                    step: None,
                    on_input: Some(vgui::input_cb(move |v, _| {
                        *captured.lock() = v.to_string();
                    })),
                    on_change: None,
                    style: None,
                    class: None,
                    id: None,
                    tabindex: None,
                    required: false,
                    pattern: None,
                    minlength: None,
                    maxlength: None,
                    list: None,
                    rows: None,
                },
                cx,
            );
            ti
        });
        Self {
            input,
            focus: cx.focus_handle(),
            last_value,
        }
    }
}

impl Focusable for ReproRoot {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus.clone()
    }
}

impl Render for ReproRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        gpui::div().size_full().child(self.input.clone())
    }
}

/// Drive the fcitx5 (text-input-v3) IME sequence for typing "你好":
///   SetMarkedText("n") -> SetMarkedText("ni") -> SetMarkedText("你好")
///   -> DeleteText (clear preedit) -> InsertText("你好")
fn drive_ime(root: &mut ReproRoot, window: &mut Window, cx: &mut gpui::Context<ReproRoot>) {
    let input = root.input.clone();
    input.update(cx, |ti, cx| {
        ti.replace_and_mark_text_in_range(None, "n", None, window, cx);
    });
    input.update(cx, |ti, cx| {
        ti.replace_and_mark_text_in_range(None, "ni", None, window, cx);
    });
    input.update(cx, |ti, cx| {
        ti.replace_and_mark_text_in_range(None, "你好", None, window, cx);
    });
    input.update(cx, |ti, cx| {
        if let Some(marked) = ti.marked_text_range(window, cx) {
            ti.replace_text_in_range(Some(marked), "", window, cx);
        }
    });
    input.update(cx, |ti, cx| {
        ti.replace_text_in_range(None, "你好", window, cx);
    });
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(400.), px(120.0)), cx);
        let window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    ..Default::default()
                },
                |_, cx| cx.new(|cx| ReproRoot::new(cx)),
            )
            .unwrap();

        // Drive the IME sequence synchronously and read back the final value.
        let value = window
            .update(cx, |root, window, cx| {
                drive_ime(root, window, cx);
                root.last_value.lock().clone()
            })
            .unwrap();
        eprintln!("final value: {value:?}");
        if value == "你好" {
            eprintln!("PASS: IME sequence produced \"你好\" (no preedit doubling)");
        } else {
            eprintln!("FAIL: expected \"你好\", got {value:?}");
        }
        cx.quit();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}
