//! File attachment display card.
//!
//! A compact row showing a file glyph, name and human-readable size, with an
//! optional upload progress bar and a remove button. Suitable for chat
//! composers and upload dropzones.

use std::rc::Rc;

use gpui::{px, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window};
use vgui::next_auto_id;

/// Data describing one attached file.
#[derive(Clone)]
pub struct Attachment {
    /// File name shown on the card.
    pub name: String,
    /// File size in bytes (rendered compactly).
    pub size_bytes: u64,
}

impl Attachment {
    /// Build an attachment descriptor.
    pub fn new(name: impl Into<String>, size_bytes: u64) -> Self {
        Self {
            name: name.into(),
            size_bytes,
        }
    }
}

fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["B", "KB", "MB", "GB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[0])
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Render one attachment card.
///
/// - `file` — the name/size descriptor.
/// - `progress` — `Some(0.0..=1.0)` shows a progress bar; `None` shows the
///   card as fully uploaded.
/// - `on_remove` — invoked when the remove button is pressed.
pub fn attachment_view(
    file: Attachment,
    progress: Option<f32>,
    on_remove: Rc<dyn Fn(&mut App)>,
) -> gpui::AnyElement {
    let Attachment { name, size_bytes } = file;

    let mut card = gpui::div()
        .flex()
        .flex_row()
        .items_center()
        .gap_2()
        .w(px(280.))
        .px_3()
        .py_2();
    card = vgui::css! {
        background: var(--muted);
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
    }
    .apply(card);

    // File glyph.
    let glyph = vgui::css! {
        font-size: 18px;
        color: var(--muted-foreground);
    }
    .apply(gpui::div().child("\u{1F4CE}"));
    card = card.child(glyph);

    // Name + meta column.
    let mut meta = gpui::div().flex().flex_col().gap_1().flex_1().min_w(px(0.));
    let name_el = vgui::css! {
        font-size: 13px;
        color: var(--foreground);
        overflow: hidden;
        white-space: nowrap;
    }
    .apply(gpui::div().child(name));
    meta = meta.child(name_el);

    match progress {
        Some(p) => {
            let pct = p.clamp(0.0, 1.0);
            let track = vgui::css! {
                height: 4px;
                width: 200px;
                border-radius: 9999px;
                background: var(--border);
                overflow: hidden;
            }
            .apply(gpui::div());
            let fill = vgui::css! {
                height: 4px;
                border-radius: 9999px;
                background: var(--primary);
            }
            .apply(gpui::div())
            .w(px(200.0 * pct));
            let track = track.child(fill);
            meta = meta.child(track);
        }
        None => {
            let size_el = vgui::css! {
                font-size: 11px;
                color: var(--muted-foreground);
            }
            .apply(gpui::div().child(human_size(size_bytes)));
            meta = meta.child(size_el);
        }
    }
    card = card.child(meta);

    // Remove button.
    let remove = gpui::div()
        .id(("attachment-remove", next_auto_id()))
        .cursor_pointer()
        .px_1()
        .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
            on_remove(cx);
        });
    let remove = vgui::css! {
        font-size: 14px;
        color: var(--muted-foreground);
    }
    .apply(remove)
    .child("\u{2715}");
    card = card.child(remove);

    card.into_any_element()
}

/// An upload zone styled as a dashed drop target. `children` typically hold the
/// hint text/icon; press handling for opening a file dialog is left to the
/// host platform.
pub fn attachment_dropzone(children: Vec<gpui::AnyElement>) -> gpui::AnyElement {
    let mut zone = gpui::div()
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_2()
        .w_full()
        .py_6();
    for child in children {
        zone = zone.child(child);
    }
    vgui::css! {
        border-width: 1px;
        border-style: dashed;
        border-color: var(--border);
        border-radius: var(--radius);
        color: var(--muted-foreground);
        background: var(--muted);
    }
    .apply(zone)
    .into_any_element()
}
