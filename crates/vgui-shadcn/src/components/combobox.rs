//! Searchable free-text combo box.
//!
//! A text field with a drop-down list that filters its options as the user
//! types. Unlike a strict `select`, the value is free-form: the user may pick
//! a filtered option (which fills the box) or type an arbitrary value. Both
//! publish through `on_change`.

use std::rc::Rc;

use gpui::{
    deferred, px, AnyElement, App, ClickEvent, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use vgui::{create_signal, input_cb, next_auto_id};

/// Controlled-ish combo box.
///
/// - `options` — selectable suggestions.
/// - `initial` — starting text (may be empty or a non-option value).
/// - `placeholder` — hint shown while empty.
/// - `on_change(text, cx)` — fired on every edit and on option pick.
pub fn combobox(
    options: Vec<String>,
    initial: impl Into<String>,
    placeholder: impl Into<String>,
    on_change: Rc<dyn Fn(String, &mut App)>,
) -> AnyElement {
    let options = Rc::new(options);
    let (text, set_text) = create_signal(initial.into());
    let (open, set_open) = create_signal(false);

    let current = text.get();
    let is_open = open.get();

    let toggle_read = open.clone();
    let toggle_write = set_open.clone();
    let mut root = gpui::div()
        .id(("combobox", next_auto_id()))
        .relative()
        .w(px(240.))
        .on_mouse_down(
            MouseButton::Left,
            move |_: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                toggle_write.set(cx, true);
                let _ = toggle_read;
                cx.stop_propagation();
            },
        );

    // Text field: filter + publish free-form value.
    let edit_cb = {
        let set_text = set_text.clone();
        let set_open = set_open.clone();
        let on_change = on_change.clone();
        input_cb(move |raw, cx| {
            set_text.set(cx, raw.to_string());
            set_open.set(cx, true);
            on_change(raw.to_string(), cx);
        })
    };

    let field = vgui::view! {
        <input
            type="text"
            value={current.clone()}
            placeholder={placeholder.into()}
            style={vgui::css! {
                width: 100%;
                height: 36px;
                padding: 4px 12px;
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--input);
                border-radius: var(--radius);
            }}
            on:input={edit_cb}
        />
    };
    root = root.child(field);

    // Filtered suggestion list (case-insensitive substring).
    if is_open {
        let query = current.to_lowercase();
        let matches: Vec<String> = options
            .iter()
            .filter(|o| o.to_lowercase().contains(&query))
            .take(8)
            .cloned()
            .collect();

        if !matches.is_empty() {
            let dismiss = set_open.clone();
            let mut panel = gpui::div()
                .id(("combobox-panel", next_auto_id()))
                .absolute()
                .top(px(40.))
                .left_0()
                .right_0()
                .py_1()
                .on_mouse_down_out(
                    move |_: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                        dismiss.set(cx, false);
                        cx.stop_propagation();
                    },
                );
            panel = vgui::css! {
                background: var(--popover);
                color: var(--popover-foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: var(--radius);
            }
            .apply(panel)
            .shadow_md();

            for option in matches {
                let set_text = set_text.clone();
                let set_open = set_open.clone();
                let on_change = on_change.clone();
                let label = option.clone();
                let row = gpui::div()
                    .id(("combobox-option", next_auto_id()))
                    .w_full()
                    .px_3()
                    .py_1()
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                        set_text.set(cx, option.clone());
                        set_open.set(cx, false);
                        on_change(option.clone(), cx);
                    })
                    .child(label);
                let row = vgui::css! {
                    font-size: 13px;
                }
                .apply(row);
                panel = panel.child(row);
            }
            root = root.child(deferred(panel));
        }
    }

    root.into_any_element()
}
