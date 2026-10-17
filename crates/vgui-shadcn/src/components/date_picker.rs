//! Calendar date-picker popover.
//!
//! A trigger button showing the selected date opens an anchored panel
//! containing a [`calendar`] month grid. Choosing a day updates the value and
//! closes the popover; pressing outside dismisses it (the same deferred-panel
//! + click-outside mechanism as the dropdown menu).
//!
//! The calendar owns signals, so it is constructed on every render and its
//! mounting is gated with [`vgui::show`] — never conditionally constructed — to
//! keep the component's reactive hook-slot count stable across open/close.

use std::rc::Rc;

use gpui::{
    deferred, px, App, InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement,
    Styled, Window,
};
use vgui::{create_signal, next_auto_id, show};
use vgui::prelude::*;

use crate::components::calendar::calendar;

fn format_date(y: i32, m: u32, d: u32) -> String {
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Controlled date picker.
///
/// - `value` — selected `(year, month, day)`, if any (shows a placeholder).
/// - `on_change(year, month, day, cx)` — fired when a day is chosen.
pub fn date_picker(
    value: Option<(i32, u32, u32)>,
    on_change: Rc<dyn Fn(i32, u32, u32, &mut App)>,
) -> gpui::AnyElement {
    let (open_read, open_write) = create_signal(false);
    let is_open = open_read.get();

    let label = value
        .map(|(y, m, d)| format_date(y, m, d))
        .unwrap_or_else(|| "Pick a date".to_string());

    let toggle_write = open_write.clone();
    let toggle_read = open_read.clone();
    let mut root = gpui::div()
        .id(("date-picker", next_auto_id()))
        .relative()
        .cursor_pointer()
        .on_mouse_down(
            MouseButton::Left,
            move |_: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                toggle_write.set(cx, !toggle_read.get());
                cx.stop_propagation();
            },
        );

    // Trigger: date label + calendar glyph.
    let glyph = gpui::div().text_size(px(13.)).child("\u{1F4C5}");
    let glyph = css! { color: var(--muted-foreground); }.apply(glyph);

    let trigger_inner = gpui::div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .w(px(168.))
        .h(px(36.))
        .px_3()
        .child(gpui::div().text_size(px(13.)).child(label))
        .child(glyph);
    let trigger = css! {
        background: var(--background);
        color: var(--foreground);
        border-width: 1px;
        border-style: solid;
        border-color: var(--input);
        border-radius: var(--radius);
    }
    .apply(trigger_inner);
    root = root.child(trigger);

    // Build the calendar unconditionally so its signal slots always exist.
    // Selecting a day forwards the value then closes the popover.
    let on_day = {
        let dismiss = open_write.clone();
        Rc::new(move |y: i32, m: u32, d: u32, cx: &mut App| {
            on_change(y, m, d, cx);
            dismiss.set(cx, false);
        })
    };
    let dismiss_out = open_write.clone();
    let panel = gpui::div()
        .id(("date-picker-panel", next_auto_id()))
        .absolute()
        .top(px(40.))
        .left_0()
        .on_mouse_down_out(
            move |_: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                dismiss_out.set(cx, false);
                cx.stop_propagation();
            },
        )
        .child(calendar(value, on_day));

    // Mount the deferred panel only while open, but always construct it.
    root = root.child(show(is_open, deferred(panel), gpui::Empty));

    root.into_any_element()
}
