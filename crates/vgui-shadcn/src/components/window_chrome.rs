//! Custom window chrome: title bar, status bar, and window border.
//!
//! [`title_bar`] is a draggable bar wired to the platform window move gesture
//! ([`Window::start_window_move`]) with working minimize / maximize / close
//! controls. [`status_bar`] is a bottom information strip. [`window_border`]
//! frames content with the theme border for borderless (custom-decorated)
//! windows.

use gpui::{
    div, px, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window,
};
use vgui::next_auto_id;

/// Configuration for [`title_bar`].
pub struct TitleBarOptions {
    /// Window title shown in the centre (or left when no controls).
    pub title: String,
    /// Show the minimize button.
    pub minimizable: bool,
    /// Show the maximize / restore button.
    pub maximizable: bool,
    /// Show the close button.
    pub closable: bool,
}

impl TitleBarOptions {
    /// A title bar showing the title and all three window controls.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            minimizable: true,
            maximizable: true,
            closable: true,
        }
    }
}

/// A draggable custom title bar with native window controls.
///
/// Pressing anywhere on the bar (other than a button) starts a platform
/// window move; the buttons minimize, toggle fullscreen, and close.
pub fn title_bar(options: TitleBarOptions) -> gpui::AnyElement {
    let TitleBarOptions {
        title,
        minimizable,
        maximizable,
        closable,
    } = options;

    // The bar owns the drag: mouse-down starts a platform window move.
    let mut bar = div()
        .id(("title-bar", next_auto_id()))
        .relative()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(34.))
        .px(px(10.))
        .on_mouse_down(gpui::MouseButton::Left, move |_ev, window: &mut Window, _cx| {
            window.start_window_move();
        });
    bar = vgui::css! {
        background: var(--card);
        border-bottom-width: 1px;
        border-style: solid;
        border-color: var(--border);
        color: var(--foreground);
        font-size: 13px;
    }
    .apply(bar);

    // Traffic-light / caption buttons on the right.
    let mut controls = div().flex().flex_row().items_center().ml_auto().gap(px(6.));
    if minimizable {
        controls = controls.child(caption_button("—", move |window: &mut Window, _cx| {
            window.minimize_window();
        }));
    }
    if maximizable {
        controls = controls.child(caption_button("□", move |window: &mut Window, _cx| {
            window.toggle_fullscreen();
        }));
    }
    if closable {
        controls = controls.child(caption_button("✕", move |window: &mut Window, _cx| {
            window.remove_window();
        }));
    }

    bar.child(div().flex().items_center().child(title))
        .child(controls)
        .into_any_element()
}

fn caption_button(
    glyph: &'static str,
    action: impl Fn(&mut Window, &mut App) + 'static,
) -> gpui::AnyElement {
    let action = std::rc::Rc::new(action);
    let click_action = action.clone();
    let mut b = div()
        .id((glyph, next_auto_id()))
        .flex()
        .items_center()
        .justify_center()
        .w(px(22.))
        .h(px(22.))
        .cursor_pointer()
        .on_click(move |_: &ClickEvent, window: &mut Window, cx: &mut App| {
            click_action(window, cx);
        });
    b = vgui::css! {
        border-radius: var(--radius);
        color: var(--muted-foreground);
        font-size: 12px;
    }
    .apply(b);
    b.child(glyph).into_any_element()
}

/// A bottom status bar. `items` are laid out left-to-right; the bar pins to
/// the bottom of its parent column.
pub struct StatusBar {
    items: Vec<gpui::AnyElement>,
}

impl StatusBar {
    /// Create an empty status bar.
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Append one status item (e.g. a position or mode readout).
    pub fn item(mut self, item: gpui::AnyElement) -> Self {
        self.items.push(item);
        self
    }
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

/// Render the bottom status bar.
pub fn status_bar(bar: StatusBar) -> gpui::AnyElement {
    let mut row = div()
        .flex()
        .flex_row()
        .items_center()
        .w_full()
        .h(px(26.))
        .px(px(10.))
        .gap(px(12.));
    row = vgui::css! {
        background: var(--card);
        border-top-width: 1px;
        border-style: solid;
        border-color: var(--border);
        color: var(--muted-foreground);
        font-size: 12px;
    }
    .apply(row);
    for item in bar.items {
        row = row.child(item);
    }
    row.into_any_element()
}

/// One labelled status segment, e.g. "Ln 3, Col 8".
pub fn status_item(text: impl Into<String>) -> gpui::AnyElement {
    div()
        .flex()
        .items_center()
        .h_full()
        .child(text.into())
        .into_any_element()
}

/// Frame `content` with the theme border for a borderless window.
pub fn window_border(content: gpui::AnyElement) -> gpui::AnyElement {
    let mut frame = div().flex().flex_col().w_full().h_full();
    frame = vgui::css! {
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
        background: var(--background);
        overflow: hidden;
    }
    .apply(frame);
    frame.child(content).into_any_element()
}
