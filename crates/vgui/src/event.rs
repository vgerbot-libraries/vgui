//! Web-aligned DOM event layer.
//!
//! Normalized event structs ([`KeyboardEvent`], [`PointerEvent`], [`ResizeEvent`],
//! [`WheelEvent`]) carrying web-style field names, mapped from gpui's native
//! events. Exposed to users via the `on:keydown` / `on:keyup` / `on:pointerdown`
//! / `on:pointerup` / `on:pointermove` / `on:resize` / `on:wheel` / `on:dblclick`
//! / `on:contextmenu` `view!` attributes, which hand user closures references to
//! these pure-data structs.
//!
//! [`KeyboardEvent`] carries a web-style [`KeyboardEvent::stop_propagation`]
//! method that sets an internal flag; the macro-facing wrappers
//! ([`__dom_key_down`], [`__dom_key_up`]) translate that flag into
//! `cx.stop_propagation()` after the user closure returns.

use std::cell::Cell;

use gpui::{
    Keystroke, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Size,
    Window, px,
};

/// The physical input type that produced a [`PointerEvent`].
///
/// Only [`PointerType::Mouse`] is populated today (gpui exposes no touch/pen
/// events); the variants exist so the web-aligned field can be extended later.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PointerType {
    Mouse,
    Pen,
    Touch,
}

/// A web-style keyboard event (`on:keydown` / `on:keyup`).
///
/// Call [`stop_propagation`](Self::stop_propagation) to prevent the event
/// from bubbling to parent elements and global `use_key_down` / `use_key_up`
/// listeners.
#[derive(Clone, Debug)]
pub struct KeyboardEvent {
    /// Web `key`: the logical key value (`"a"`, `"A"`, `"Enter"`, `" "`,
    /// `"ArrowUp"`…). Falls back to the raw gpui `key` when no mapping is known.
    pub key: String,
    /// Web `code`: the physical key identifier (`"KeyA"`, `"Digit5"`,
    /// `"Enter"`…). Best-effort — gpui exposes no physical code, so unknown
    /// keys yield `""`.
    pub code: String,
    /// `true` when the key is auto-repeating (gpui `KeyDownEvent::is_held`).
    /// Always `false` for `keyup`.
    pub repeat: bool,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
    /// gpui `Modifiers::platform` (cmd on macOS, win/super elsewhere).
    pub meta_key: bool,
    /// The typed character, passthrough of `Keystroke::key_char`.
    pub key_char: Option<String>,
    /// Internal flag set by [`stop_propagation`](Self::stop_propagation).
    /// Checked by the macro-facing wrappers and the global dispatch loop.
    propagation_stopped: Cell<bool>,
}

/// A web-style pointer event (`on:pointerdown` / `on:pointerup` /
/// `on:pointermove`). Mouse-only for now.
#[derive(Clone, Debug)]
pub struct PointerEvent {
    pub pointer_type: PointerType,
    /// Mouse always `1` (web convention).
    pub pointer_id: i32,
    /// Window-space pixel coordinates.
    pub client_x: f64,
    pub client_y: f64,
    /// `-1` on move (no button change); `0`=left, `1`=middle, `2`=right; `-1`
    /// for Navigate buttons.
    pub button: i32,
    /// Bitmask: left=`1`, right=`2`, middle=`4`. `0` on up (gpui gives no
    /// remaining-pressed state) and on move with no button pressed.
    pub buttons: i32,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
    /// From gpui down/up `click_count`; `0` on move.
    pub click_count: usize,
    /// `true` for mouse (the primary pointer).
    pub is_primary: bool,
}

/// A web-style window resize event (`on:resize`). Fires on window viewport-size
/// change with the window's new size — web `window.onresize` semantics, not
/// per-element `ResizeObserver`.
#[derive(Clone, Debug)]
pub struct ResizeEvent {
    pub width: f64,
    pub height: f64,
}

/// A web-style wheel event (`on:wheel`).
#[derive(Clone, Debug)]
pub struct WheelEvent {
    pub delta_x: f64,
    pub delta_y: f64,
    pub shift_key: bool,
    pub ctrl_key: bool,
    pub alt_key: bool,
    pub meta_key: bool,
}

// ── keyboard normalization ──────────────────────────────────────────

impl KeyboardEvent {
    /// Construct a `KeyboardEvent` from web-style fields. Useful for testing
    /// and custom event dispatching.
    pub fn new(
        key: impl Into<String>,
        code: impl Into<String>,
        repeat: bool,
        shift_key: bool,
        ctrl_key: bool,
        alt_key: bool,
        meta_key: bool,
        key_char: Option<String>,
    ) -> Self {
        Self {
            key: key.into(),
            code: code.into(),
            repeat,
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
            key_char,
            propagation_stopped: Cell::new(false),
        }
    }

    /// Map a gpui `Keystroke` to a web-style [`KeyboardEvent`].
    pub fn from_keystroke(ks: &Keystroke, is_held: bool) -> Self {
        // Clone key_char once; reuse for both the `key` field (when
        // non-empty) and the `key_char` field.
        let key_char = ks.key_char.clone();
        let key = web_key_with_char(ks, key_char.as_ref());
        Self {
            key,
            code: web_code(ks),
            repeat: is_held,
            shift_key: ks.modifiers.shift,
            ctrl_key: ks.modifiers.control,
            alt_key: ks.modifiers.alt,
            meta_key: ks.modifiers.platform,
            key_char,
            propagation_stopped: Cell::new(false),
        }
    }

    /// Web-style `stopPropagation()`. Sets an internal flag that prevents the
    /// event from bubbling to parent elements and global listeners. Subsequent
    /// handlers on the same target are also skipped (matching
    /// `stopImmediatePropagation` semantics).
    pub fn stop_propagation(&self) {
        self.propagation_stopped.set(true);
    }

    /// Returns `true` if [`stop_propagation`](Self::stop_propagation) was called.
    pub fn is_propagation_stopped(&self) -> bool {
        self.propagation_stopped.get()
    }
}

/// Map a gpui `Keystroke` to the web `key` value, using a pre-cloned
/// `key_char` reference to avoid re-cloning from `ks.key_char`.
fn web_key_with_char(ks: &Keystroke, key_char: Option<&String>) -> String {
    if let Some(c) = key_char {
        if !c.is_empty() {
            return c.clone();
        }
    }
    web_key_named(ks)
}

/// Map a gpui `Keystroke` to the web `key` value.
fn web_key(ks: &Keystroke) -> String {
    web_key_with_char(ks, ks.key_char.as_ref())
}

/// Match-only `web_key` without the `key_char` shortcut.
fn web_key_named(ks: &Keystroke) -> String {
    // Avoid the intermediate `to_lowercase()` allocation by comparing
    // case-insensitively against the known names. Only the fallback path
    // allocates.
    match ks.key.as_str() {
        "space" | "SPACE" | "Space" => " ".to_string(),
        "enter" | "ENTER" | "Enter" => "Enter".to_string(),
        "escape" | "ESCAPE" | "Escape" => "Escape".to_string(),
        "tab" | "TAB" | "Tab" => "Tab".to_string(),
        "backspace" | "BACKSPACE" | "Backspace" => "Backspace".to_string(),
        "delete" | "DELETE" | "Delete" => "Delete".to_string(),
        "arrowup" | "ARROWUP" | "ArrowUp" => "ArrowUp".to_string(),
        "arrowdown" | "ARROWDOWN" | "ArrowDown" => "ArrowDown".to_string(),
        "arrowleft" | "ARROWLEFT" | "ArrowLeft" => "ArrowLeft".to_string(),
        "arrowright" | "ARROWRIGHT" | "ArrowRight" => "ArrowRight".to_string(),
        "home" | "HOME" | "Home" => "Home".to_string(),
        "end" | "END" | "End" => "End".to_string(),
        "pageup" | "PAGEUP" | "PageUp" => "PageUp".to_string(),
        "pagedown" | "PAGEDOWN" | "PageDown" => "PageDown".to_string(),
        "capslock" | "CAPSLOCK" | "CapsLock" => "CapsLock".to_string(),
        "control" | "CONTROL" | "Control" => "Control".to_string(),
        "alt" | "ALT" | "Alt" => "Alt".to_string(),
        "shift" | "SHIFT" | "Shift" => "Shift".to_string(),
        "meta" | "META" | "Meta" => "Meta".to_string(),
        "fn" | "FN" | "Fn" => "Fn".to_string(),
        "f1" | "F1" => "F1".to_string(),
        "f2" | "F2" => "F2".to_string(),
        "f3" | "F3" => "F3".to_string(),
        "f4" | "F4" => "F4".to_string(),
        "f5" | "F5" => "F5".to_string(),
        "f6" | "F6" => "F6".to_string(),
        "f7" | "F7" => "F7".to_string(),
        "f8" | "F8" => "F8".to_string(),
        "f9" | "F9" => "F9".to_string(),
        "f10" | "F10" => "F10".to_string(),
        "f11" | "F11" => "F11".to_string(),
        "f12" | "F12" => "F12".to_string(),
        other => other.to_string(),
    }
}
/// Map a gpui `Keystroke` to the web `code` value (best-effort; gpui exposes
/// no physical key code).
fn web_code(ks: &Keystroke) -> String {
    // Avoid the intermediate `to_lowercase()` allocation by matching
    // case-insensitively. Named keys return static slices; single-char
    // keys build "KeyX" / "DigitN" only for the fallback.
    match ks.key.as_str() {
        "enter" | "ENTER" | "Enter" => "Enter".to_string(),
        "space" | "SPACE" | "Space" => "Space".to_string(),
        "tab" | "TAB" | "Tab" => "Tab".to_string(),
        "escape" | "ESCAPE" | "Escape" => "Escape".to_string(),
        "backspace" | "BACKSPACE" | "Backspace" => "Backspace".to_string(),
        "delete" | "DELETE" | "Delete" => "Delete".to_string(),
        "arrowup" | "ARROWUP" | "ArrowUp" => "ArrowUp".to_string(),
        "arrowdown" | "ARROWDOWN" | "ArrowDown" => "ArrowDown".to_string(),
        "arrowleft" | "ARROWLEFT" | "ArrowLeft" => "ArrowLeft".to_string(),
        "arrowright" | "ARROWRIGHT" | "ArrowRight" => "ArrowRight".to_string(),
        "home" | "HOME" | "Home" => "Home".to_string(),
        "end" | "END" | "End" => "End".to_string(),
        "pageup" | "PAGEUP" | "PageUp" => "PageUp".to_string(),
        "pagedown" | "PAGEDOWN" | "PageDown" => "PageDown".to_string(),
        "f1" | "F1" => "F1".to_string(),
        "f2" | "F2" => "F2".to_string(),
        "f3" | "F3" => "F3".to_string(),
        "f4" | "F4" => "F4".to_string(),
        "f5" | "F5" => "F5".to_string(),
        "f6" | "F6" => "F6".to_string(),
        "f7" | "F7" => "F7".to_string(),
        "f8" | "F8" => "F8".to_string(),
        "f9" | "F9" => "F9".to_string(),
        "f10" | "F10" => "F10".to_string(),
        "f11" | "F11" => "F11".to_string(),
        "f12" | "F12" => "F12".to_string(),
        _ if ks.key.len() == 1 => {
            let ch = ks.key.chars().next().unwrap();
            if ch.is_ascii_alphabetic() {
                let mut s = String::with_capacity(4);
                s.push_str("Key");
                s.push(ch.to_ascii_uppercase());
                s
            } else if ch.is_ascii_digit() {
                let mut s = String::with_capacity(6);
                s.push_str("Digit");
                s.push(ch);
                s
            } else {
                String::new()
            }
        }
        _ => String::new(),
    }
}

// ── pointer normalization ───────────────────────────────────────────

fn mouse_button_to_web(b: MouseButton) -> i32 {
    match b {
        MouseButton::Left => 0,
        MouseButton::Middle => 1,
        MouseButton::Right => 2,
        MouseButton::Navigate(_) => -1,
    }
}

fn mouse_button_to_mask(b: MouseButton) -> i32 {
    match b {
        MouseButton::Left => 1,
        MouseButton::Right => 2,
        MouseButton::Middle => 4,
        MouseButton::Navigate(_) => 0,
    }
}

fn mods(m: Modifiers) -> (bool, bool, bool, bool) {
    (m.shift, m.control, m.alt, m.platform)
}

impl PointerEvent {
    pub(crate) fn from_mouse_down(e: &MouseDownEvent) -> Self {
        let (shift_key, ctrl_key, alt_key, meta_key) = mods(e.modifiers);
        Self {
            pointer_type: PointerType::Mouse,
            pointer_id: 1,
            client_x: f64::from(e.position.x),
            client_y: f64::from(e.position.y),
            button: mouse_button_to_web(e.button),
            buttons: mouse_button_to_mask(e.button),
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
            click_count: e.click_count,
            is_primary: true,
        }
    }

    pub(crate) fn from_mouse_up(e: &MouseUpEvent) -> Self {
        let (shift_key, ctrl_key, alt_key, meta_key) = mods(e.modifiers);
        Self {
            pointer_type: PointerType::Mouse,
            pointer_id: 1,
            client_x: f64::from(e.position.x),
            client_y: f64::from(e.position.y),
            button: mouse_button_to_web(e.button),
            // gpui gives no remaining-pressed state on up.
            buttons: 0,
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
            click_count: e.click_count,
            is_primary: true,
        }
    }

    pub(crate) fn from_mouse_move(e: &MouseMoveEvent) -> Self {
        let (shift_key, ctrl_key, alt_key, meta_key) = mods(e.modifiers);
        Self {
            pointer_type: PointerType::Mouse,
            pointer_id: 1,
            client_x: f64::from(e.position.x),
            client_y: f64::from(e.position.y),
            button: -1,
            buttons: e.pressed_button.map(mouse_button_to_mask).unwrap_or(0),
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
            click_count: 0,
            is_primary: true,
        }
    }

    pub(crate) fn from_click_mouse_up(e: &gpui::ClickEvent) -> Option<Self> {
        match e {
            gpui::ClickEvent::Mouse(mouse) => Some(Self::from_mouse_up(&mouse.up)),
            gpui::ClickEvent::Keyboard(_) | gpui::ClickEvent::Touch(_) => None,
        }
    }
}

// ── resize normalization ────────────────────────────────────────────

impl ResizeEvent {
    pub(crate) fn from_size(s: Size<Pixels>) -> Self {
        Self {
            width: f64::from(s.width),
            height: f64::from(s.height),
        }
    }

    pub(crate) fn from_window(window: &Window) -> Self {
        Self::from_size(window.viewport_size())
    }
}

impl WheelEvent {
    pub(crate) fn from_scroll_wheel(e: &gpui::ScrollWheelEvent) -> Self {
        let delta = e.delta.pixel_delta(px(16.));
        let (shift_key, ctrl_key, alt_key, meta_key) = mods(e.modifiers);
        Self {
            delta_x: f64::from(delta.x),
            delta_y: f64::from(delta.y),
            shift_key,
            ctrl_key,
            alt_key,
            meta_key,
        }
    }
}

// ── macro-facing converters ─────────────────────────────────────────
//
// Each wraps a user closure (receiving a normalized `&*Event`) and returns the
// gpui-listener closure gpui's `InteractiveElement` methods expect.

pub fn __dom_key_down<H: Fn(&KeyboardEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&gpui::KeyDownEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| {
        let ke = KeyboardEvent::from_keystroke(&e.keystroke, e.is_held);
        h(&ke, w, cx);
        if ke.is_propagation_stopped() {
            cx.stop_propagation();
        }
    }
}

pub fn __dom_key_up<H: Fn(&KeyboardEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&gpui::KeyUpEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| {
        let ke = KeyboardEvent::from_keystroke(&e.keystroke, false);
        h(&ke, w, cx);
        if ke.is_propagation_stopped() {
            cx.stop_propagation();
        }
    }
}

pub fn __dom_pointer_down<H: Fn(&PointerEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&MouseDownEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| h(&PointerEvent::from_mouse_down(e), w, cx)
}

pub fn __dom_pointer_up<H: Fn(&PointerEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&MouseUpEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| h(&PointerEvent::from_mouse_up(e), w, cx)
}

pub fn __dom_pointer_move<H: Fn(&PointerEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&MouseMoveEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| h(&PointerEvent::from_mouse_move(e), w, cx)
}

pub fn __dom_wheel<H: Fn(&WheelEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&gpui::ScrollWheelEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| h(&WheelEvent::from_scroll_wheel(e), w, cx)
}

pub fn __dom_dblclick<H: Fn(&PointerEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| {
        if e.click_count() >= 2 {
            if let Some(p) = PointerEvent::from_click_mouse_up(e) {
                h(&p, w, cx);
            }
        }
    }
}

pub fn __dom_contextmenu<H: Fn(&PointerEvent, &mut Window, &mut gpui::App) + 'static>(
    h: H,
) -> impl Fn(&gpui::ClickEvent, &mut Window, &mut gpui::App) + 'static {
    move |e, w, cx| {
        if e.is_secondary() {
            if let Some(p) = PointerEvent::from_click_mouse_up(e) {
                h(&p, w, cx);
            }
        }
    }
}

/// Whether a drag operation is currently active. Callable from any
/// reactive scope (create_memo, create_effect, event handlers).
pub fn has_active_drag() -> bool {
    crate::reactive::with_root_cx(|cx| cx.has_active_drag())
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Pixels, Point, Size, px};

    fn ks(key: &str, key_char: Option<&str>, mods: Modifiers) -> Keystroke {
        Keystroke {
            modifiers: mods,
            key: key.to_string(),
            key_char: key_char.map(str::to_string),
        }
    }

    #[test]
    fn web_key_named_keys() {
        let none = Modifiers::default();
        assert_eq!(web_key(&ks("enter", None, none)), "Enter");
        assert_eq!(web_key(&ks("space", None, none)), " ");
        assert_eq!(web_key(&ks("arrowup", None, none)), "ArrowUp");
        assert_eq!(web_key(&ks("f1", None, none)), "F1");
        assert_eq!(web_key(&ks("f12", None, none)), "F12");
        assert_eq!(web_key(&ks("escape", None, none)), "Escape");
    }

    #[test]
    fn web_key_letter_with_key_char() {
        // key_char takes precedence when non-empty.
        let k = ks("a", Some("a"), Modifiers::default());
        assert_eq!(web_key(&k), "a");
        // Shifted letter: gpui keeps key "a" but key_char "A".
        let k = ks("a", Some("A"), Modifiers { shift: true, ..Default::default() });
        assert_eq!(web_key(&k), "A");
    }

    #[test]
    fn web_key_letter_without_key_char() {
        // Modifier-only press: key_char None, key "a" → fallback to key.
        let k = ks("a", None, Modifiers::default());
        assert_eq!(web_key(&k), "a");
    }

    #[test]
    fn web_key_unknown_falls_back_to_raw() {
        let k = ks("someunknownkey", None, Modifiers::default());
        assert_eq!(web_key(&k), "someunknownkey");
    }

    #[test]
    fn web_code_letters_digits_named() {
        let none = Modifiers::default();
        assert_eq!(web_code(&ks("a", None, none)), "KeyA");
        assert_eq!(web_code(&ks("z", None, none)), "KeyZ");
        assert_eq!(web_code(&ks("5", None, none)), "Digit5");
        assert_eq!(web_code(&ks("enter", None, none)), "Enter");
        assert_eq!(web_code(&ks("space", None, none)), "Space");
        assert_eq!(web_code(&ks("arrowleft", None, none)), "ArrowLeft");
        assert_eq!(web_code(&ks("f5", None, none)), "F5");
    }

    #[test]
    fn web_code_unknown_is_empty() {
        let k = ks("someunknownkey", None, Modifiers::default());
        assert_eq!(web_code(&k), "");
    }

    #[test]
    fn keyboard_event_from_keystroke_flags_and_repeat() {
        let m = Modifiers { shift: true, control: true, alt: true, platform: true, function: false };
        let k = ks("a", Some("A"), m);
        let ev = KeyboardEvent::from_keystroke(&k, true);
        assert_eq!(ev.key, "A");
        assert_eq!(ev.code, "KeyA");
        assert!(ev.repeat);
        assert!(ev.shift_key && ev.ctrl_key && ev.alt_key && ev.meta_key);
        assert_eq!(ev.key_char, Some("A".to_string()));
    }

    #[test]
    fn keyboard_event_stop_propagation_flag() {
        let k = ks("escape", None, Modifiers::default());
        let ev = KeyboardEvent::from_keystroke(&k, false);
        assert!(!ev.is_propagation_stopped());
        ev.stop_propagation();
        assert!(ev.is_propagation_stopped());
    }

    #[test]
    fn keyboard_event_new_constructor() {
        let ev = KeyboardEvent::new(
            "Enter", "Enter", true, false, true, false, false, None,
        );
        assert_eq!(ev.key, "Enter");
        assert_eq!(ev.code, "Enter");
        assert!(ev.repeat);
        assert!(!ev.shift_key);
        assert!(ev.ctrl_key);
        assert!(!ev.alt_key);
        assert!(!ev.meta_key);
        assert!(ev.key_char.is_none());
        assert!(!ev.is_propagation_stopped());
    }

    fn pt(x: f32, y: f32) -> Point<Pixels> {
        Point { x: px(x), y: px(y) }
    }

    #[test]
    fn pointer_down_button_mapping() {
        let mods = Modifiers::default();
        for (button, web, mask) in [
            (MouseButton::Left, 0, 1),
            (MouseButton::Right, 2, 2),
            (MouseButton::Middle, 1, 4),
        ] {
            let e = MouseDownEvent { button, position: pt(10.0, 20.0), modifiers: mods, click_count: 2, first_mouse: false };
            let p = PointerEvent::from_mouse_down(&e);
            assert_eq!(p.button, web);
            assert_eq!(p.buttons, mask);
            assert_eq!(p.click_count, 2);
            assert_eq!(p.pointer_type, PointerType::Mouse);
            assert_eq!(p.pointer_id, 1);
            assert!(p.is_primary);
            assert_eq!(p.client_x, 10.0);
            assert_eq!(p.client_y, 20.0);
        }
    }

    #[test]
    fn pointer_up_buttons_zero() {
        let e = MouseUpEvent { button: MouseButton::Left, position: pt(1.0, 2.0), modifiers: Modifiers::default(), click_count: 1 };
        let p = PointerEvent::from_mouse_up(&e);
        assert_eq!(p.button, 0);
        assert_eq!(p.buttons, 0);
    }

    #[test]
    fn pointer_move_button_minus_one_and_mask() {
        // No button pressed.
        let e = MouseMoveEvent { position: pt(5.0, 6.0), pressed_button: None, modifiers: Modifiers::default() };
        let p = PointerEvent::from_mouse_move(&e);
        assert_eq!(p.button, -1);
        assert_eq!(p.buttons, 0);
        assert_eq!(p.click_count, 0);

        // Left dragged.
        let e = MouseMoveEvent { position: pt(5.0, 6.0), pressed_button: Some(MouseButton::Left), modifiers: Modifiers::default() };
        let p = PointerEvent::from_mouse_move(&e);
        assert_eq!(p.button, -1);
        assert_eq!(p.buttons, 1);
    }

    #[test]
    fn resize_event_from_size() {
        let s: Size<Pixels> = Size { width: px(800.0), height: px(600.0) };
        let ev = ResizeEvent::from_size(s);
        assert_eq!(ev.width, 800.0);
        assert_eq!(ev.height, 600.0);
    }

    #[test]
    fn wheel_event_from_pixel_and_line_delta() {
        let e = gpui::ScrollWheelEvent {
            position: pt(0.0, 0.0),
            delta: gpui::ScrollDelta::Pixels(pt(3.0, -8.0)),
            modifiers: Modifiers {
                shift: true,
                control: true,
                alt: false,
                platform: true,
                function: false,
            },
            touch_phase: gpui::TouchPhase::default(),
        };
        let w = WheelEvent::from_scroll_wheel(&e);
        assert_eq!(w.delta_x, 3.0);
        assert_eq!(w.delta_y, -8.0);
        assert!(w.shift_key && w.ctrl_key && w.meta_key);
        assert!(!w.alt_key);

        let e = gpui::ScrollWheelEvent {
            position: pt(0.0, 0.0),
            delta: gpui::ScrollDelta::Lines(Point { x: 1.0, y: -2.0 }),
            modifiers: Modifiers::default(),
            touch_phase: gpui::TouchPhase::default(),
        };
        let w = WheelEvent::from_scroll_wheel(&e);
        assert_eq!(w.delta_x, 16.0);
        assert_eq!(w.delta_y, -32.0);
    }

    #[test]
    fn from_click_mouse_up_ignores_keyboard_and_touch() {
        let up = MouseUpEvent {
            button: MouseButton::Left,
            position: pt(1.0, 2.0),
            modifiers: Modifiers::default(),
            click_count: 2,
        };
        let down = MouseDownEvent {
            button: MouseButton::Left,
            position: pt(1.0, 2.0),
            modifiers: Modifiers::default(),
            click_count: 2,
            first_mouse: false,
        };
        let e = gpui::ClickEvent::Mouse(gpui::MouseClickEvent { down, up });
        let p = PointerEvent::from_click_mouse_up(&e).unwrap();
        assert_eq!(p.click_count, 2);
        assert_eq!(p.client_x, 1.0);
        assert_eq!(p.client_y, 2.0);

        let e = gpui::ClickEvent::Keyboard(gpui::KeyboardClickEvent::default());
        assert!(PointerEvent::from_click_mouse_up(&e).is_none());

        let e = gpui::ClickEvent::Touch(gpui::TouchClickEvent::default());
        assert!(PointerEvent::from_click_mouse_up(&e).is_none());
    }
}
