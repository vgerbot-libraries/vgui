//! Regression test for the fcitx5 (text-input-v3) IME preedit-doubling bug.
//!
//! Drives the full IME sequence for typing "你好":
//!   SetMarkedText("n") -> SetMarkedText("ni") -> SetMarkedText("你好")
//!   -> DeleteText (clear preedit) -> InsertText("你好")
//!
//! The final input value must be exactly "你好" — not doubled or corrupted.

#![cfg(not(target_family = "wasm"))]

use std::sync::Arc;

use gpui::{
    px, size, App, Bounds, Entity, EntityInputHandler, Focusable, Render, Window, WindowBounds,
    WindowOptions,
};
use parking_lot::Mutex;
use vgui::prelude::*;
use vgui::{TextInput, TextInputProps};

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

/// Drive the fcitx5 (text-input-v3) IME sequence for typing "你好".
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

#[test]
fn ime_preedit_not_doubled() {
    let gpui_app = application();
    let result: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let result_clone = result.clone();
    let launch = move |cx: &mut App| {
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

        let value = window
            .update(cx, |root, window, cx| {
                drive_ime(root, window, cx);
                root.last_value.lock().clone()
            })
            .unwrap();
        *result_clone.lock() = Some(value);
        // Defer `quit` onto the running event loop. Calling `cx.quit()` synchronously
        // here would be a no-op: calloop's `EventLoop::run` resets the stop signal to
        // `false` *after* this launch callback returns and before it enters the
        // dispatch loop, so the quit would be lost and the visible window would hang
        // until manually closed. Spawning the quit lets it run on the foreground
        // executor (a calloop source) once the loop is actually turning, so the stop
        // flag is set while the loop is live and the app exits promptly.
        cx.spawn(async move |cx| {
            cx.update(|cx| cx.quit());
        })
        .detach();
    };

    gpui_app.run(launch);

    let value = result.lock().clone().expect("IME sequence did not produce a value");
    assert_eq!(
        value, "你好",
        "IME sequence should produce \"你好\" without preedit doubling"
    );
}
