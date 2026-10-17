//! Headless test support for vgui apps.
//!
//! [`run_headless`] builds an in-memory gpui window hosting a [`VguiRoot`] and
//! hands the test a [`TestWindow`] with the small event-simulation set the
//! components in this workspace need: [`draw`](TestWindow::draw),
//! [`click`](TestWindow::click), [`mouse_down`](TestWindow::mouse_down) /
//! [`mouse_up`](TestWindow::mouse_up), [`mouse_move`](TestWindow::mouse_move)
//! and [`key_down`](TestWindow::key_down).
//!
//! It removes the per-test `TestAppContext` + `add_window_view` + event-struct
//! boilerplate every headless interaction test otherwise repeats.
//!
//! Enabled by the `test-support` cargo feature (or automatically inside the
//! crate's own `#[cfg(test)]` builds). The in-memory gpui platform is native
//! only, so the module is absent on `wasm32`.
//!
//! ```ignore
//! vgui::headless::run_headless(
//!     || view! { "hello" },
//!     |win| {
//!         win.draw();
//!         win.click(10.0, 10.0);
//!     },
//! );
//! ```

use gpui::{
    point, px, IntoElement, KeyDownEvent, Keystroke, Modifiers, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, TestAppContext, VisualTestContext,
};

use crate::VguiRoot;

/// A borrowed handle on the in-memory test window, valid for the duration of
/// the [`run_headless`] body closure.
pub struct TestWindow<'a> {
    cx: &'a mut VisualTestContext,
}

impl<'a> TestWindow<'a> {
    /// Force one layout+paint pass.
    pub fn draw(&mut self) {
        self.cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
    }

    /// Dispatch a left mouse-down at window coordinates `(x, y)`.
    pub fn mouse_down(&mut self, x: f32, y: f32) {
        self.mouse_down_button(x, y, MouseButton::Left, 1);
    }

    /// Dispatch a left mouse-up at window coordinates `(x, y)`.
    pub fn mouse_up(&mut self, x: f32, y: f32) {
        self.mouse_up_button(x, y, MouseButton::Left, 1);
    }

    /// Dispatch a mouse-down with an explicit button and click count.
    pub fn mouse_down_button(&mut self, x: f32, y: f32, button: MouseButton, click_count: usize) {
        self.cx.simulate_event(MouseDownEvent {
            position: point(px(x), px(y)),
            modifiers: Modifiers::default(),
            button,
            click_count,
            first_mouse: false,
        });
    }

    /// Dispatch a mouse-up with an explicit button and click count.
    pub fn mouse_up_button(&mut self, x: f32, y: f32, button: MouseButton, click_count: usize) {
        self.cx.simulate_event(MouseUpEvent {
            position: point(px(x), px(y)),
            modifiers: Modifiers::default(),
            button,
            click_count,
        });
    }

    /// A full press+release (a click) at `(x, y)`.
    pub fn click(&mut self, x: f32, y: f32) {
        self.mouse_down(x, y);
        self.mouse_up(x, y);
    }

    /// Move the pointer to `(x, y)`; `pressed` sends a held left button.
    pub fn mouse_move(&mut self, x: f32, y: f32, pressed: bool) {
        self.cx.simulate_event(MouseMoveEvent {
            position: point(px(x), px(y)),
            pressed_button: if pressed {
                Some(MouseButton::Left)
            } else {
                None
            },
            modifiers: Modifiers::default(),
        });
    }

    /// Dispatch a raw key-down (named keys like `"enter"` use `key_char=None`).
    pub fn key_down(&mut self, key: &str, modifiers: Modifiers) {
        self.cx.simulate_event(KeyDownEvent {
            keystroke: Keystroke {
                modifiers,
                key: key.into(),
                key_char: None,
            },
            is_held: false,
            prefer_character_input: false,
        });
    }

    /// Convenience: a key with a held control modifier (command on macOS).
    pub fn key_down_ctrl(&mut self, key: &str, control: bool) {
        self.key_down(
            key,
            Modifiers {
                control,
                ..Modifiers::default()
            },
        );
    }

    pub fn cx(&mut self) -> &mut VisualTestContext {
        self.cx
    }
}

/// Build an in-memory app/window rendering `render`, then run `body` against
/// it. The app is torn down when `body` returns.
pub fn run_headless<R, F, B>(mut render: F, mut body: B)
where
    R: IntoElement + 'static,
    F: FnMut() -> R + 'static,
    B: FnMut(&mut TestWindow),
{
    let mut app = TestAppContext::single();
    let (_view, cx) = app.add_window_view(|window, cx| {
        VguiRoot::new(cx, window.window_handle(), move || {
            render().into_any_element()
        })
    });
    let mut window = TestWindow { cx };
    body(&mut window);
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;
    use gpui::{div, px, InteractiveElement, IntoElement, StatefulInteractiveElement, Styled};

    use crate::reactive::next_auto_id;

    /// A click dispatched at the centre of a hit-tested div fires its handler.
    #[test]
    fn click_reaches_hit_tested_element() {
        let clicked = Rc::new(Cell::new(false));
        let c = clicked.clone();
        super::run_headless(
            move || {
                let c = c.clone();
                div()
                    .id(("hit", next_auto_id()))
                    .w(px(100.))
                    .h(px(100.))
                    .on_click(move |_, _, _| c.set(true))
                    .into_any_element()
            },
            |win| {
                win.draw();
                assert!(!clicked.get(), "no click before input");
                win.click(50., 50.);
                assert!(clicked.get(), "click should fire the div handler");
            },
        );
    }

    /// Drawing a minimal root must not panic.
    #[test]
    fn draw_minimal_root() {
        super::run_headless(|| div().w(px(10.)).h(px(10.)), |win| {
            win.draw();
        });
    }
}
