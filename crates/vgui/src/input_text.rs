use std::ops::Range;
use std::time::Duration;
use gpui::{
    canvas, fill, px, size, App, AppContext, Bounds, ClipboardItem, Context, ElementInputHandler,
    Entity, EntityInputHandler, Focusable, HighlightStyle, InteractiveElement, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, Point,
    Render, SharedString, StatefulInteractiveElement, Styled, StyledText, Task, TextLayout,
    UTF16Selection, UnderlineStyle, Window, hsla,
};

use crate::reactive::{get_or_create_view, with_root_cx};
use crate::style::{Css, TwStyle};

/// Which kind of text-based `<input>` to render.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextKind {
    Text,
    Password,
    Search,
    Email,
    Url,
    Tel,
    Number,
    Date,
    DateTime,
    Time,
    Month,
    Week,
    Color,
}

impl TextKind {
    fn is_masked(self) -> bool {
        matches!(self, TextKind::Password)
    }

    fn placeholder_default(self) -> &'static str {
        match self {
            TextKind::Text => "",
            TextKind::Password => "",
            TextKind::Search => "Search\u{2026}",
            TextKind::Email => "email@example.com",
            TextKind::Url => "https://example.com",
            TextKind::Tel => "+1 (555) 000-0000",
            TextKind::Number => "0",
            TextKind::Date => "YYYY-MM-DD",
            TextKind::DateTime => "YYYY-MM-DDTHH:MM",
            TextKind::Time => "HH:MM",
            TextKind::Month => "YYYY-MM",
            TextKind::Week => "YYYY-Www",
            TextKind::Color => "#RRGGBB",
        }
    }
}

/// The core text editing widget. A gpui `Entity` (view) cached in a reactive
/// scope slot so its editing state persists across re-renders.
pub struct TextInput {
    kind: TextKind,
    value: String,
    placeholder: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    cursor: usize,
    selection: Option<Range<usize>>,
    marked: Option<(Range<usize>, String)>,
    focused: bool,
    blink_on: bool,
    selecting: bool,
    last_committed: String,
    focus_handle_field: gpui::FocusHandle,
    on_input: Option<Box<dyn FnMut(&str, &mut App)>>,
    on_change: Option<Box<dyn FnMut(&str, &mut App)>>,
    style: Option<Css>,
    class: Option<TwStyle>,
    blink_task: Option<Task<()>>,
    layout: TextLayout,
    picker_open: bool,
    picker_year: i32,
    picker_month: u32,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            kind: TextKind::Text,
            value: String::new(),
            placeholder: None,
            disabled: false,
            readonly: false,
            min: None,
            max: None,
            step: None,
            cursor: 0,
            selection: None,
            marked: None,
            focused: false,
            blink_on: false,
            selecting: false,
            last_committed: String::new(),
            focus_handle_field: cx.focus_handle(),
            on_input: None,
            on_change: None,
            style: None,
            class: None,
            blink_task: None,
            layout: TextLayout::default(),
            picker_open: false,
            picker_year: 2026,
            picker_month: 1,
        }
    }

    /// Sync mutable state from props. Called every render so prop changes
    /// (e.g. a signal-driven `value`) take effect. While focused the `value`
    /// is not overwritten — the user's in-progress edit wins.
    pub fn sync_from_props(&mut self, props: TextInputProps, _cx: &mut Context<Self>) {
        self.kind = props.kind;
        if props.placeholder.is_some() {
            self.placeholder = props.placeholder;
        }
        self.disabled = props.disabled;
        self.readonly = props.readonly;
        self.min = props.min;
        self.max = props.max;
        self.step = props.step;
        if !self.focused && props.value != self.value {
            self.value = props.value.clone();
            self.cursor = self.value.len();
            self.selection = None;
            self.last_committed = props.value;
        }
        self.style = props.style;
        self.class = props.class;
        self.on_input = props.on_input;
        self.on_change = props.on_change;
    }

    // ── Displayed text ───────────────────────────────────────────────

    fn displayed_text(&self) -> SharedString {
        if self.kind.is_masked() {
            let count = self.value.chars().count();
            SharedString::from("\u{2022}".repeat(count))
        } else {
            SharedString::from(self.value.clone())
        }
    }

    /// Map a byte offset in `self.value` to a byte offset in the displayed string.
    fn value_byte_to_display_byte(&self, value_byte: usize) -> usize {
        if self.kind.is_masked() {
            let char_index = self.value[..value_byte.min(self.value.len())].chars().count();
            char_index * '\u{2022}'.len_utf8()
        } else {
            value_byte
        }
    }

    /// Map a byte offset in the displayed string to a byte offset in `self.value`.
    fn display_byte_to_value_byte(&self, display_byte: usize) -> usize {
        if self.kind.is_masked() {
            let char_index = display_byte / '\u{2022}'.len_utf8();
            self.value
                .char_indices()
                .nth(char_index)
                .map(|(i, _)| i)
                .unwrap_or(self.value.len())
        } else {
            display_byte
        }
    }

    // ── UTF-16 ↔ UTF-8 conversion (EntityInputHandler uses UTF-16) ────

    fn utf16_to_value_byte(&self, utf16_offset: usize) -> usize {
        let mut utf16 = 0;
        for (i, c) in self.value.char_indices() {
            if utf16 >= utf16_offset {
                return i;
            }
            utf16 += c.len_utf16();
        }
        self.value.len()
    }

    fn value_byte_to_utf16(&self, value_byte: usize) -> usize {
        let mut utf16 = 0;
        for (i, c) in self.value.char_indices() {
            if i >= value_byte {
                break;
            }
            utf16 += c.len_utf16();
        }
        utf16
    }

    // ── Cursor / selection helpers ───────────────────────────────────

    fn delete_selection(&mut self) {
        if let Some(sel) = self.selection.take() {
            let start = sel.start.min(sel.end);
            let end = sel.start.max(sel.end);
            self.value.replace_range(start..end, "");
            self.cursor = start;
        }
    }

    fn selected_text(&self) -> String {
        match &self.selection {
            Some(sel) => {
                let start = sel.start.min(sel.end);
                let end = sel.start.max(sel.end);
                self.value[start..end].to_string()
            }
            None => String::new(),
        }
    }

    // ── Blink ────────────────────────────────────────────────────────

    fn start_blink(&mut self, cx: &mut Context<Self>) {
        self.blink_on = true;
        self.blink_task = Some(cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(530))
                    .await;
                this.update(cx, |this, cx| {
                    this.blink_on = !this.blink_on;
                    cx.notify();
                })
                .ok();
            }
        }));
    }

    fn stop_blink(&mut self) {
        self.blink_task = None;
        self.blink_on = false;
    }

    // ── Callbacks ────────────────────────────────────────────────────

    fn fire_on_input(&mut self, cx: &mut Context<Self>) {
        let value = self.value.clone();
        if let Some(cb) = self.on_input.as_mut() {
            cb(&value, &mut **cx);
        }
        cx.notify();
    }

    fn fire_on_change(&mut self, cx: &mut Context<Self>) {
        self.last_committed = self.value.clone();
        let value = self.value.clone();
        if let Some(cb) = self.on_change.as_mut() {
            cb(&value, &mut **cx);
        }
    }

    // ── Date picker ──────────────────────────────────────────────────

    fn supports_picker(&self) -> bool {
        matches!(
            self.kind,
            TextKind::Date | TextKind::DateTime | TextKind::Month
        )
    }

    fn open_picker(&mut self) {
        if let Some((y, m, _)) = parse_date(&self.value) {
            self.picker_year = y;
            self.picker_month = m;
        } else if let Some((y, m, _)) = today_date() {
            self.picker_year = y;
            self.picker_month = m;
        }
        self.picker_open = true;
    }

    fn select_date(&mut self, year: i32, month: u32, day: u32, cx: &mut Context<Self>) {
        self.value = format!("{:04}-{:02}-{:02}", year, month, day);
        self.cursor = self.value.len();
        self.selection = None;
        self.picker_open = false;
        self.fire_on_input(cx);
        self.fire_on_change(cx);
    }

    // ── Editing primitives ───────────────────────────────────────────

    fn filter_text(&self, text: &str) -> String {
        match self.kind {
            TextKind::Number => text
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(c, '.' | '-' | '+' | 'e' | 'E'))
                .collect(),
            TextKind::Date | TextKind::Month => text
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == '-')
                .collect(),
            TextKind::DateTime => text
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(*c, '-' | 'T' | ':'))
                .collect(),
            TextKind::Time => text
                .chars()
                .filter(|c| c.is_ascii_digit() || *c == ':')
                .collect(),
            TextKind::Week => text
                .chars()
                .filter(|c| c.is_ascii_digit() || matches!(*c, '-' | 'W'))
                .collect(),
            TextKind::Color => text
                .chars()
                .filter(|c| c.is_ascii_hexdigit() || *c == '#')
                .collect(),
            _ => text.to_string(),
        }
    }

    fn replace_range(&mut self, range: Option<Range<usize>>, text: &str, cx: &mut Context<Self>) {
        if self.disabled || self.readonly {
            return;
        }
        // Ignore tab/newline — handled by on_key_down (Tab = focus traversal,
        // Enter = commit).
        if text == "\t" || text == "\n" {
            return;
        }
        let filtered = self.filter_text(text);
        let (start, end) = match range {
            Some(r) => {
                let s = self.utf16_to_value_byte(r.start.min(r.end));
                let e = self.utf16_to_value_byte(r.start.max(r.end));
                (s, e)
            }
            None => {
                if let Some(sel) = self.selection.take() {
                    (sel.start.min(sel.end), sel.start.max(sel.end))
                } else {
                    (self.cursor, self.cursor)
                }
            }
        };
        self.value.replace_range(start..end, &filtered);
        self.cursor = start + filtered.len();
        self.selection = None;
        self.marked = None;
        self.fire_on_input(cx);
    }

    // ── Key handling ─────────────────────────────────────────────────

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let mods = &event.keystroke.modifiers;
        let cmd = mods.control || mods.platform;

        match key {
            "backspace" => {
                if self.readonly {
                    return;
                }
                if self.selection.is_some() {
                    self.delete_selection();
                    self.fire_on_input(cx);
                } else if cmd {
                    let new_cursor = prev_word_boundary(&self.value, self.cursor);
                    self.value.replace_range(new_cursor..self.cursor, "");
                    self.cursor = new_cursor;
                    self.fire_on_input(cx);
                } else if self.cursor > 0 {
                    let new_cursor = prev_char_boundary(&self.value, self.cursor);
                    self.value.replace_range(new_cursor..self.cursor, "");
                    self.cursor = new_cursor;
                    self.fire_on_input(cx);
                }
            }
            "delete" => {
                if self.readonly {
                    return;
                }
                if self.selection.is_some() {
                    self.delete_selection();
                    self.fire_on_input(cx);
                } else if cmd {
                    let end = next_word_boundary(&self.value, self.cursor);
                    self.value.replace_range(self.cursor..end, "");
                    self.fire_on_input(cx);
                } else if self.cursor < self.value.len() {
                    let end = next_char_boundary(&self.value, self.cursor);
                    self.value.replace_range(self.cursor..end, "");
                    self.fire_on_input(cx);
                }
            }
            "left" => {
                if cmd {
                    self.move_cursor(prev_word_boundary(&self.value, self.cursor), mods.shift);
                } else {
                    self.move_cursor(prev_char_boundary(&self.value, self.cursor), mods.shift);
                }
                cx.notify();
            }
            "right" => {
                if cmd {
                    self.move_cursor(next_word_boundary(&self.value, self.cursor), mods.shift);
                } else {
                    self.move_cursor(next_char_boundary(&self.value, self.cursor), mods.shift);
                }
                cx.notify();
            }
            "up" | "home" => {
                self.move_cursor(0, mods.shift);
                cx.notify();
            }
            "down" | "end" => {
                self.move_cursor(self.value.len(), mods.shift);
                cx.notify();
            }
            "enter" => {
                self.fire_on_change(cx);
            }
            "escape" => {
                if self.picker_open {
                    self.picker_open = false;
                    cx.notify();
                } else {
                    self.value = self.last_committed.clone();
                    self.cursor = self.value.len();
                    self.selection = None;
                    cx.notify();
                }
            }
            "tab" => {
                // let gpui handle focus traversal
            }
            _ => {
                if cmd {
                    match key {
                        "a" => {
                            self.selection = Some(0..self.value.len());
                            cx.notify();
                        }
                        "c" => {
                            let text = self.selected_text();
                            if !text.is_empty() {
                                cx.write_to_clipboard(ClipboardItem::new_string(text));
                            }
                        }
                        "v" => {
                            if self.readonly {
                                return;
                            }
                            if let Some(item) = cx.read_from_clipboard() {
                                if let Some(text) = item.text() {
                                    self.replace_range(None, &text, cx);
                                }
                            }
                        }
                        "x" => {
                            if self.readonly {
                                return;
                            }
                            let text = self.selected_text();
                            if !text.is_empty() {
                                cx.write_to_clipboard(ClipboardItem::new_string(text));
                                self.delete_selection();
                                self.fire_on_input(cx);
                            }
                        }
                        _ => {}
                    }
                }
                // Printable keys arrive via replace_text_in_range (IME)
            }
        }
    }

    fn move_cursor(&mut self, new_cursor: usize, shift: bool) {
        let new_cursor = new_cursor.min(self.value.len());
        if shift {
            if let Some(sel) = &mut self.selection {
                let anchor = if sel.start <= sel.end {
                    sel.start
                } else {
                    sel.end
                };
                self.cursor = new_cursor;
                *sel = anchor..new_cursor;
            } else {
                let anchor = self.cursor;
                self.cursor = new_cursor;
                if anchor != new_cursor {
                    self.selection = Some(anchor..new_cursor);
                }
            }
        } else {
            self.cursor = new_cursor;
            self.selection = None;
        }
    }

    // ── Mouse handling ───────────────────────────────────────────────

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        self.focus_handle_field.focus(window);
        let position = event.position;
        let index = self
            .layout
            .index_for_position(position)
            .map(|i| self.display_byte_to_value_byte(i))
            .unwrap_or(self.value.len());
        if event.modifiers.shift {
            self.move_cursor(index, true);
        } else {
            self.cursor = index;
            self.selection = None;
        }
        self.selecting = true;
        cx.notify();
    }

    fn handle_mouse_move(&mut self, event: &MouseMoveEvent, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        let position = event.position;
        let index = self
            .layout
            .index_for_position(position)
            .map(|i| self.display_byte_to_value_byte(i))
            .unwrap_or(self.value.len());
        self.move_cursor(index, true);
        cx.notify();
    }

    fn handle_mouse_up(&mut self, cx: &mut Context<Self>) {
        self.selecting = false;
        cx.notify();
    }

    // ── Date picker popup ────────────────────────────────────────────

    fn render_picker_popup(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let year = self.picker_year;
        let month = self.picker_month;
        let today = today_date();
        let selected = parse_date(&self.value);

        let month_name = month_name(month);
        let first_weekday = weekday_of_first(year, month);
        let days_in_month = days_in_month(year, month);

        // Header: ◀  Month Year  ▶
        let header = gpui::div()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .mb_2()
            .child(
                gpui::div()
                    .id("picker-prev")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25C0}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            if this.picker_month == 1 {
                                this.picker_month = 12;
                                this.picker_year -= 1;
                            } else {
                                this.picker_month -= 1;
                            }
                            cx.notify();
                        }),
                    ),
            )
            .child(
                gpui::div()
                    .text_size(px(13.))
                    .text_color(gpui::black())
                    .child(SharedString::from(format!("{} {}", month_name, year))),
            )
            .child(
                gpui::div()
                    .id("picker-next")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25B6}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            if this.picker_month == 12 {
                                this.picker_month = 1;
                                this.picker_year += 1;
                            } else {
                                this.picker_month += 1;
                            }
                            cx.notify();
                        }),
                    ),
            );

        // Weekday header row
        let mut weekday_row = gpui::div().flex().flex_row().mb_1();
        for wd in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
            weekday_row = weekday_row.child(
                gpui::div()
                    .w(px(28.))
                    .h(px(20.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(11.))
                    .text_color(hsla(0.0, 0.0, 0.5, 1.0))
                    .child(SharedString::from(wd)),
            );
        }

        // Day grid
        let mut day_grid = gpui::div().flex().flex_col();
        let mut week = gpui::div().flex().flex_row();
        // Leading blanks
        for _ in 0..first_weekday {
            week = week.child(gpui::div().w(px(28.)).h(px(28.)));
        }
        for day in 1..=days_in_month {
            let is_today = today
                .map(|(ty, tm, td)| ty == year && tm == month && td == day)
                .unwrap_or(false);
            let is_selected = selected
                .map(|(sy, sm, sd)| sy == year && sm == month && sd == day)
                .unwrap_or(false);
            let bg = if is_selected {
                hsla(0.6, 0.8, 0.5, 1.0)
            } else if is_today {
                hsla(0.6, 0.6, 0.9, 1.0)
            } else {
                gpui::transparent_black()
            };
            let fg = if is_selected {
                gpui::white()
            } else {
                gpui::black()
            };
            let border = if is_today && !is_selected {
                hsla(0.6, 0.6, 0.5, 1.0)
            } else {
                gpui::transparent_black()
            };
            let day_id = ("picker-day", day as u64);
            week = week.child(
                gpui::div()
                    .id(day_id)
                    .w(px(28.))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(12.))
                    .text_color(fg)
                    .bg(bg)
                    .border_1()
                    .border_color(border)
                    .rounded(px(4.))
                    .cursor_pointer()
                    .child(SharedString::from(format!("{}", day)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.select_date(year, month, day, cx);
                        }),
                    ),
            );
            let weekday = (first_weekday + day as u32 - 1) % 7;
            if weekday == 6 {
                day_grid = day_grid.child(week);
                week = gpui::div().flex().flex_row();
            }
        }
        // Trailing partial week
        let last_weekday = (first_weekday + days_in_month as u32 - 1) % 7;
        if last_weekday != 6 {
            for _ in (last_weekday + 1)..7 {
                week = week.child(gpui::div().w(px(28.)).h(px(28.)));
            }
            day_grid = day_grid.child(week);
        }

        let _ = &header; // ensure header is used
        gpui::div()
            .id("date-picker-popup")
            .absolute()
            .top(px(34.))
            .left_0()
            .w(px(224.))
            .p_2()
            .bg(gpui::white())
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.7, 0.3))
            .rounded(px(6.))
            .shadow_md()
            .flex()
            .flex_col()
            .child(header)
            .child(weekday_row)
            .child(day_grid)
    }
}

// ── Render ───────────────────────────────────────────────────────────

impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Detect focus transitions
        let is_focused = self.focus_handle_field.is_focused(window);
        if is_focused && !self.focused {
            self.focused = true;
            self.start_blink(cx);
        } else if !is_focused && self.focused {
            self.focused = false;
            self.stop_blink();
            self.picker_open = false;
            self.fire_on_change(cx);
        }

        let mut text_style = window.text_style();
        // The input has a white background; force the text color to black so
        // it is always visible regardless of inherited color from ancestors
        // (e.g. a parent with `text-white`).
        text_style.color = gpui::black();
        let font_size = text_style.font_size.to_pixels(window.rem_size());
        let line_height = text_style
            .line_height
            .to_pixels(font_size.into(), window.rem_size());

        // Build displayed text + highlights
        let display = self.displayed_text();
        let mut highlights: Vec<(Range<usize>, HighlightStyle)> = Vec::new();

        // Selection highlight (in displayed-string coordinates)
        if let Some(sel) = &self.selection {
            let start = self.value_byte_to_display_byte(sel.start.min(sel.end));
            let end = self.value_byte_to_display_byte(sel.start.max(sel.end));
            if start < end {
                highlights.push((
                    start..end,
                    HighlightStyle {
                        background_color: Some(hsla(0.6, 0.8, 0.6, 0.4)),
                        ..Default::default()
                    },
                ));
            }
        }

        // IME marked text underline
        if let Some((marked_range, _)) = &self.marked {
            let start = self.value_byte_to_display_byte(marked_range.start);
            let end = self.value_byte_to_display_byte(marked_range.end);
            if start < end {
                highlights.push((
                    start..end,
                    HighlightStyle {
                        underline: Some(UnderlineStyle {
                            thickness: px(1.),
                            color: Some(hsla(0.0, 0.0, 0.0, 0.7)),
                            wavy: false,
                        }),
                        ..Default::default()
                    },
                ));
            }
        }

        let placeholder_text: Option<SharedString> = if self.value.is_empty() && !self.focused {
            self.placeholder
                .clone()
                .or_else(|| Some(SharedString::from(self.kind.placeholder_default())))
                .filter(|p| !p.is_empty())
        } else {
            None
        };

        let styled = if let Some(ph) = placeholder_text {
            StyledText::new(ph.clone()).with_default_highlights(
                &text_style,
                [(
                    0..ph.len(),
                    HighlightStyle {
                        color: Some(hsla(0.0, 0.0, 0.5, 0.6)),
                        ..Default::default()
                    },
                )],
            )
        } else {
            StyledText::new(display.clone()).with_default_highlights(&text_style, highlights)
        };

        // Clone the layout for cursor painting and click-to-index
        let layout = styled.layout().clone();
        self.layout = layout.clone();

        // Capture state for the canvas closures
        let entity = cx.entity();
        let focus_handle = self.focus_handle_field.clone();
        let focused = self.focused;
        let blink_on = self.blink_on;
        let has_selection = self.selection.is_some();
        let disp_cursor = self.value_byte_to_display_byte(self.cursor);
        let cursor_color = hsla(0.0, 0.0, 0.0, 0.8);
        let lh = line_height;

        let mut div = gpui::div()
            .id("text-input")
            .track_focus(&self.focus_handle_field)
            .focusable()
            .cursor_text()
            .text_color(gpui::black())
            .relative()
            .px_2()
            .py_1()
            .min_h(px(28.))
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.6, 0.4))
            .rounded(px(4.))
            .bg(gpui::white())
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, event: &MouseDownEvent, window, cx| {
                    this.handle_mouse_down(event, window, cx);
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                this.handle_mouse_move(event, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.handle_mouse_up(cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                this.handle_key_down(event, window, cx);
            }))
            .child(styled)
            .child(canvas(
                move |_bounds, _window, _cx| {},
                move |bounds, _state, window, cx| {
                    // Paint cursor
                    if focused && blink_on && !has_selection {
                        if let Some(p) = layout.position_for_index(disp_cursor) {
                            let cursor_bounds = Bounds::new(p, size(px(1.), lh));
                            window.paint_quad(fill(cursor_bounds, cursor_color));
                        }
                    }
                    // Register IME input handler
                    if focused {
                        window.handle_input(
                            &focus_handle,
                            ElementInputHandler::new(bounds, entity.clone()),
                            cx,
                        );
                    }
                },
            ));

        // Calendar toggle icon for date-like inputs
        if self.supports_picker() {
            div = div.child(
                gpui::div()
                    .id("calendar-icon")
                    .absolute()
                    .top_0()
                    .right_0()
                    .w(px(24.))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .text_color(hsla(0.0, 0.0, 0.4, 1.0))
                    .text_size(px(14.))
                    .child(SharedString::from("\u{1F4C5}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            if this.picker_open {
                                this.picker_open = false;
                            } else {
                                this.open_picker();
                            }
                            cx.notify();
                        }),
                    ),
            );
        }

        // Date picker popup calendar
        if self.picker_open && self.supports_picker() {
            div = div.child(self.render_picker_popup(cx));
        }

        // Apply user-provided style (overrides defaults)
        if let Some(style) = self.style.take() {
            div = style.apply(div);
        }
        // Apply user-provided class
        if let Some(class) = self.class.take() {
            let TwStyle {
                base,
                hover,
                focus,
                active,
            } = class;
            base(div.style());
            if let Some(h) = hover {
                div = div.hover(move |mut s| {
                    h(&mut s);
                    s
                });
            }
            if let Some(f) = focus {
                div = div.focus(move |mut s| {
                    f(&mut s);
                    s
                });
            }
            if let Some(a) = active {
                div = div.active(move |mut s| {
                    a(&mut s);
                    s
                });
            }
        }

        div
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle_field.clone()
    }
}

// ── EntityInputHandler (IME / platform text input) ───────────────────

impl EntityInputHandler for TextInput {
    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if !ignore_disabled_input && (self.disabled || self.readonly) {
            return None;
        }
        let (start, end) = match &self.selection {
            Some(sel) => (sel.start.min(sel.end), sel.start.max(sel.end)),
            None => (self.cursor, self.cursor),
        };
        let s = self.value_byte_to_utf16(start);
        let e = self.value_byte_to_utf16(end);
        Some(UTF16Selection {
            range: s..e,
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked.as_ref().map(|(r, _)| {
            self.value_byte_to_utf16(r.start)..self.value_byte_to_utf16(r.end)
        })
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_range(range, text, cx);
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled || self.readonly {
            return;
        }
        // First replace the text
        let (start, end) = match range {
            Some(r) => {
                let s = self.utf16_to_value_byte(r.start.min(r.end));
                let e = self.utf16_to_value_byte(r.start.max(r.end));
                (s, e)
            }
            None => {
                if let Some(sel) = self.selection.take() {
                    (sel.start.min(sel.end), sel.start.max(sel.end))
                } else {
                    (self.cursor, self.cursor)
                }
            }
        };
        let filtered = self.filter_text(new_text);
        self.value.replace_range(start..end, &filtered);
        let marked_start = start;
        let marked_end = start + filtered.len();
        self.marked = Some((marked_start..marked_end, filtered.clone()));

        // Set the selection within the marked text
        if let Some(sel_utf16) = new_selected_range {
            let sel_start = self.utf16_to_value_byte(sel_utf16.start) + start;
            let sel_end = self.utf16_to_value_byte(sel_utf16.end) + start;
            self.selection = Some(sel_start..sel_end);
            self.cursor = sel_end;
        } else {
            self.cursor = marked_end;
            self.selection = None;
        }

        self.fire_on_input(cx);
    }

    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        _adjusted_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let start = self.utf16_to_value_byte(range_utf16.start);
        let end = self.utf16_to_value_byte(range_utf16.end);
        Some(self.value[start..end].to_string())
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<gpui::Pixels>> {
        let start = self.value_byte_to_display_byte(self.utf16_to_value_byte(range_utf16.start));
        let end = self.value_byte_to_display_byte(self.utf16_to_value_byte(range_utf16.end));
        let p1 = self.layout.position_for_index(start)?;
        let p2 = self.layout.position_for_index(end)?;
        Some(Bounds::from_corners(p1, p2 + Point::new(px(0.), element_bounds.size.height)))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let byte_index = self.layout.index_for_position(point).ok()?;
        let value_byte = self.display_byte_to_value_byte(byte_index);
        Some(self.value_byte_to_utf16(value_byte))
    }
}

// ── Props + constructor ──────────────────────────────────────────────

/// Props for constructing a text-based `<input>`.
pub struct TextInputProps {
    pub kind: TextKind,
    pub value: String,
    pub placeholder: Option<SharedString>,
    pub disabled: bool,
    pub readonly: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub on_input: Option<Box<dyn FnMut(&str, &mut App)>>,
    pub on_change: Option<Box<dyn FnMut(&str, &mut App)>>,
    pub style: Option<Css>,
    pub class: Option<TwStyle>,
}

/// Get-or-create the persistent `TextInput` entity and sync props.
/// Called by the `view!` macro's expanded code every render.
pub fn text_input(props: TextInputProps) -> Entity<TextInput> {
    let entity = get_or_create_view(|cx| cx.new(|cx| TextInput::new(cx)));
    with_root_cx(|cx| {
        entity.update(cx, |view, cx| view.sync_from_props(props, cx));
    });
    entity
}

// ── Callback helpers ─────────────────────────────────────────────────

/// Wrap a closure as an `on:input` callback for text-based inputs.
pub fn input_cb(
    f: impl FnMut(&str, &mut App) + 'static,
) -> Box<dyn FnMut(&str, &mut App)> {
    Box::new(f)
}

/// Wrap a closure as an `on:change` callback for text-based inputs.
pub fn str_change_cb(
    f: impl FnMut(&str, &mut App) + 'static,
) -> Box<dyn FnMut(&str, &mut App)> {
    Box::new(f)
}

// ── Free functions ───────────────────────────────────────────────────

fn prev_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p.saturating_sub(s[..p].chars().last().map(|c| c.len_utf8()).unwrap_or(0))
}

fn next_char_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos + 1;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}

fn prev_word_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    // skip trailing whitespace
    while p > 0 {
        let prev = prev_char_boundary(s, p);
        if s[prev..p].chars().all(|c| c.is_whitespace()) {
            p = prev;
        } else {
            break;
        }
    }
    // skip non-whitespace
    while p > 0 {
        let prev = prev_char_boundary(s, p);
        if s[prev..p].chars().all(|c| !c.is_whitespace()) {
            p = prev;
        } else {
            break;
        }
    }
    p
}

fn next_word_boundary(s: &str, pos: usize) -> usize {
    let mut p = pos;
    // skip non-whitespace
    while p < s.len() {
        let next = next_char_boundary(s, p);
        if s[p..next].chars().all(|c| !c.is_whitespace()) {
            p = next;
        } else {
            break;
        }
    }
    // skip whitespace
    while p < s.len() {
        let next = next_char_boundary(s, p);
        if s[p..next].chars().all(|c| c.is_whitespace()) {
            p = next;
        } else {
            break;
        }
    }
    p
}

// ── Date helpers ─────────────────────────────────────────────────────

/// Parse a `YYYY-MM-DD` string into `(year, month, day)`.
fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let mut parts = s.split('-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if (1..=12).contains(&m) && (1..=31).contains(&d) {
        Some((y, m, d))
    } else {
        None
    }
}

/// Return today's date as `(year, month, day)` in the local timezone.
fn today_date() -> Option<(i32, u32, u32)> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).ok()?.as_secs();
    // Days since epoch
    let days = (secs / 86400) as i64;
    // Use a simple civil-from-days algorithm (Howard Hinnant).
    days_to_ymd(days)
}

/// Convert days since 1970-01-01 to (year, month, day).
fn days_to_ymd(days: i64) -> Option<(i32, u32, u32)> {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let year = (y + if m <= 2 { 1 } else { 0 }) as i32;
    Some((year, m, d))
}

/// Return the weekday (0=Sunday) of the first day of the given month.
fn weekday_of_first(year: i32, month: u32) -> u32 {
    // Use Zeller's congruence. January and February are counted as
    // months 13 and 14 of the previous year.
    let (y, m): (i32, i32) = if month < 3 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };
    let k = y % 100;
    let j = y / 100;
    let h = (1 + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    // Zeller: 0=Saturday → convert to 0=Sunday
    ((h + 6) % 7) as u32
}

/// Number of days in a given month.
fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}
