//! Time-of-day segmented field.
//!
//! Three numeric boxes (hour, minute, optional second) joined by colons. Each
//! segment accepts up to two digits; when the active segments all parse, the
//! assembled `(hour, minute, second)` — clamped to its valid range — is
//! published through `on_change`.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement};
use vgui::{input_cb, ReadSignal, WriteSignal};

/// Controlled-ish `HH:MM` or `HH:MM:SS` field.
///
/// `on_change(hour, minute, second, cx)` fires when every visible segment
/// parses; `second` is `0` when `with_seconds` is false.
pub fn time_field(
    with_seconds: bool,
    on_change: Rc<dyn Fn(u32, u32, u32, &mut App)>,
) -> gpui::AnyElement {
    let (hour, set_hour) = vgui::create_signal(String::new());
    let (minute, set_minute) = vgui::create_signal(String::new());
    let (second, set_second) = vgui::create_signal(String::new());

    let emit: Rc<dyn Fn(&mut App)> = {
        let hour = hour.clone();
        let minute = minute.clone();
        let second = second.clone();
        Rc::new(move |cx: &mut App| {
            let h: Option<u32> = hour.get().parse().ok();
            let m: Option<u32> = minute.get().parse().ok();
            let s: Option<u32> = if with_seconds {
                second.get().parse().ok()
            } else {
                Some(0)
            };
            if let (Some(h), Some(m), Some(s)) = (h, m, s) {
                on_change(h.min(23), m.min(59), s.min(59), cx);
            }
        })
    };

    let hbox = segment_box("HH", hour, set_hour, emit.clone());
    let mbox = segment_box("MM", minute, set_minute, emit.clone());

    let sseg = if with_seconds {
        let sbox = segment_box("SS", second, set_second, emit.clone());
        vgui::view! {
            <div style={vgui::css! { display: flex; flex-direction: row; align-items: center; gap: 4px; }}>
                <span style={vgui::css!{ color: var(--muted-foreground); }}>{":"}</span>
                {sbox}
            </div>
        }
        .into_any_element()
    } else {
        gpui::Empty.into_any_element()
    };

    vgui::view! {
        <div style={vgui::css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
        }}>
            {hbox}
            <span style={vgui::css!{ color: var(--muted-foreground); }}>{":"}</span>
            {mbox}
            {sseg}
        </div>
    }
    .into_any_element()
}

fn segment_box(
    placeholder: &'static str,
    read: ReadSignal<String>,
    write: WriteSignal<String>,
    emit: Rc<dyn Fn(&mut App)>,
) -> gpui::AnyElement {
    let value = read.get();
    vgui::view! {
        <input
            type="text"
            value={value}
            placeholder={placeholder}
            style={vgui::css! {
                width: 36px;
                height: 36px;
                text-align: center;
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--input);
                border-radius: var(--radius);
            }}
            on:input={input_cb(move |raw, cx| {
                // Digits only, capped at two characters.
                let digits: String = raw.chars().filter(|c| c.is_ascii_digit()).take(2).collect();
                write.set(cx, digits);
                emit(cx);
            })}
        />
    }
    .into_any_element()
}
