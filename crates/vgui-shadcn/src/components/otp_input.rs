//! Segmented one-time-code (OTP) entry.
//!
//! Renders one single-character box per digit. Typing into a box stores its
//! final character; once every box is filled the assembled code is published
//! through `on_complete`. Pasting a multi-character code into any box spreads
//! the characters across the remaining boxes.

use std::rc::Rc;

use gpui::{App, IntoElement, ParentElement};
use vgui::{create_store, for_each, input_cb};

/// Controlled-ish OTP field of `length` single-character boxes.
///
/// `on_complete(code, cx)` fires whenever all boxes hold a digit/character,
/// with the concatenated code. Editing a box to empty clears completion until
/// the code is whole again.
pub fn otp_input(length: usize, on_complete: Rc<dyn Fn(String, &mut App)>) -> gpui::AnyElement {
    let length = length.max(1);
    let (digits, set_digits) = create_store(vec![String::new(); length]);

    let cells = for_each(0..length, move |i: usize, _pos| {
        let set_digits = set_digits.clone();
        let digits_read = digits.clone();
        let on_complete = on_complete.clone();
        let value = digits_read.get().get(i).cloned().unwrap_or_default();

        vgui::view! {
            <input
                type="text"
                value={value}
                style={vgui::css! {
                    width: 40px;
                    height: 44px;
                    text-align: center;
                    font-size: 18px;
                    background: var(--background);
                    color: var(--foreground);
                    border-width: 1px;
                    border-style: solid;
                    border-color: var(--input);
                    border-radius: var(--radius);
                }}
                on:input={input_cb(move |raw, cx| {
                    // Keep only the trailing character(s); a paste of several
                    // characters fans out from this box.
                    let chars: Vec<char> = raw.chars().collect();
                    let completed = set_digits.update(cx, |buf| {
                        if chars.len() <= 1 {
                            if let Some(slot) = buf.get_mut(i) {
                                *slot = chars.last().map(|c| c.to_string()).unwrap_or_default();
                            }
                        } else {
                            for (offset, ch) in chars.iter().take(length - i).enumerate() {
                                if let Some(slot) = buf.get_mut(i + offset) {
                                    *slot = ch.to_string();
                                }
                            }
                        }
                        if buf.iter().all(|s| !s.is_empty()) {
                            Some(buf.concat())
                        } else {
                            None
                        }
                    });
                    if let Some(code) = completed {
                        on_complete(code, cx);
                    }
                })}
            />
        }
    });

    vgui::view! {
        <div style={vgui::css! {
            display: flex;
            flex-direction: row;
            gap: 8px;
        }}>{cells}</div>
    }
    .into_any_element()
}
