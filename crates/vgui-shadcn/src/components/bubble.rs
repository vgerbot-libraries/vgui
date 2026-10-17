//! Chat message bubbles and a scrolling message list.
//!
//! [`bubble`] renders one message aligned to a side with author/timestamp
//! meta; [`message_scroller`] stacks bubbles in a vertically scrolling column
//! and pins to the latest message (like a chat transcript).

use gpui::{px, AnyElement, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled};
use vgui::next_auto_id;

/// Which side of the transcript a bubble belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BubbleSide {
    /// Sent by the local user (right-aligned, emphasized).
    Sent,
    /// Received from a peer (left-aligned, neutral).
    Received,
}

/// One chat message.
#[derive(Clone)]
pub struct Message {
    /// Display name of the author.
    pub author: String,
    /// Message body.
    pub text: String,
    /// Free-form timestamp label (e.g. `"10:32"`).
    pub time: String,
    /// Sent vs received, controlling alignment and color.
    pub side: BubbleSide,
}

impl Message {
    /// Build a message.
    pub fn new(
        author: impl Into<String>,
        text: impl Into<String>,
        time: impl Into<String>,
        side: BubbleSide,
    ) -> Self {
        Self {
            author: author.into(),
            text: text.into(),
            time: time.into(),
            side,
        }
    }
}

/// Render a single chat [`Message`] as a bubble.
pub fn bubble(message: Message) -> AnyElement {
    let Message {
        author,
        text,
        time,
        side,
    } = message;
    let sent = side == BubbleSide::Sent;

    // Row aligns the bubble left or right.
    let mut row = gpui::div()
        .w_full()
        .flex()
        .flex_row()
        .mb_2();
    row = if sent {
        row.justify_end()
    } else {
        row.justify_start()
    };

    let mut col = gpui::div().flex().flex_col().max_w(px(360.));
    col = if sent {
        col.items_end()
    } else {
        col.items_start()
    };

    // Author + time meta line.
    let meta = vgui::css! {
        font-size: 11px;
        color: var(--muted-foreground);
        margin-bottom: 2px;
    }
    .apply(gpui::div().child(format!("{}  ·  {}", author, time)));
    col = col.child(meta);

    // Bubble body.
    let mut body = gpui::div().px_3().py_2();
    body = if sent {
        vgui::css! {
            background: var(--primary);
            color: var(--primary-foreground);
            border-radius: var(--radius);
            font-size: 13px;
        }
        .apply(body)
    } else {
        vgui::css! {
            background: var(--muted);
            color: var(--foreground);
            border-radius: var(--radius);
            font-size: 13px;
        }
        .apply(body)
    };
    body = body.child(text);
    col = col.child(body);

    row.child(col).into_any_element()
}

/// A scrolling transcript of [`Message`]s, newest last.
pub fn message_scroller(messages: Vec<Message>) -> AnyElement {
    let mut list = gpui::div()
        .id(("message-scroller", next_auto_id()))
        .flex()
        .flex_col()
        .w_full()
        .h_full()
        .overflow_y_scroll()
        .px_3()
        .py_2();
    for message in messages {
        list = list.child(bubble(message));
    }
    vgui::css! {
        background: var(--background);
    }
    .apply(list)
    .into_any_element()
}
