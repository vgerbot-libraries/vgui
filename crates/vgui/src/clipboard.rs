//! Clipboard helpers.
//!
//! Thin wrappers over the GPUI platform clipboard so components and event
//! handlers can read and write plain text without importing `ClipboardItem`.

use gpui::{App, ClipboardItem};
/// Write `text` to the system clipboard.
pub fn write_text(text: impl Into<String>, cx: &mut App) {
    cx.write_to_clipboard(ClipboardItem::new_string(text.into()));
}

/// Read the concatenated plain-text content of the system clipboard.
/// Returns `None` when the clipboard holds no string entry.
pub fn read_text(cx: &App) -> Option<String> {
    cx.read_from_clipboard().and_then(|item| item.text())
}
