//! Web (WASM) platform helpers.
//!
//! These functions are only available when compiling for `target_family =
//! "wasm"`. They bridge vgui to the browser DOM via `web-sys`.

#[cfg(target_family = "wasm")]
mod imp {
    use std::cell::Cell;
    use wasm_bindgen::prelude::*;
    use web_sys::{KeyboardEvent, Window};

    thread_local! {
        static INSTALLED: Cell<bool> = const { Cell::new(false) };
    }

    /// Install bubbling-phase listeners on `window` for `keydown` and `keyup`
    /// that call `preventDefault()` and `stopPropagation()` on every keyboard
    /// event.
    ///
    /// gpui_web registers its own keyboard listeners on the hidden IME mirror
    /// `<textarea>` (a child of `<body>`). Those fire first during the target
    /// and early bubble phases, so gpui still receives and processes every
    /// keystroke. By the time the event bubbles to `window`, our listener
    /// calls `preventDefault()` — which works at any propagation phase — to
    /// stop the browser's default action (Tab moving focus, Space scrolling,
    /// arrow-key scrolling, etc.). `stopPropagation()` prevents any further
    /// window-level listeners from reacting.
    ///
    /// Idempotent: safe to call multiple times; only the first call installs
    /// listeners.
    pub fn intercept_keyboard_events() {
        INSTALLED.with(|installed| {
            if installed.get() {
                return;
            }
            installed.set(true);
        });

        let Some(window) = web_sys::window() else {
            return;
        };

        let keydown_closure = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
            event.prevent_default();
            event.stop_propagation();
        });
        let keyup_closure = Closure::<dyn FnMut(KeyboardEvent)>::new(|event: KeyboardEvent| {
            event.prevent_default();
            event.stop_propagation();
        });

        register(&window, "keydown", &keydown_closure);
        register(&window, "keyup", &keyup_closure);

        // Forget the closures so they live for the page lifetime. The
        // listeners are never removed — the canvas owns the page.
        keydown_closure.forget();
        keyup_closure.forget();
    }

    fn register(window: &Window, name: &str, closure: &Closure<dyn FnMut(KeyboardEvent)>) {
        window
            .add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
            .ok();
    }
}

#[cfg(target_family = "wasm")]
pub use imp::intercept_keyboard_events;

#[cfg(not(target_family = "wasm"))]
#[doc(hidden)]
pub fn intercept_keyboard_events() {}
