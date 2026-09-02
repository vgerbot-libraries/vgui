use std::ops::Range;
use std::time::Duration;
use gpui::{
    canvas, fill, px, size, AnyElement, App, AppContext, Bounds, ClipboardItem, Context,
    ElementInputHandler, Entity, EntityInputHandler, Focusable, HighlightStyle,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, ParentElement, Point, Render, SharedString, StatefulInteractiveElement, Styled,
    StyledText, Task, TextLayout, UTF16Selection, UnderlineStyle, Window, hsla,
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
    multiline: bool,
    value: String,
    placeholder: Option<SharedString>,
    disabled: bool,
    readonly: bool,
    min: Option<f64>,
    max: Option<f64>,
    step: Option<f64>,
    rows: Option<u32>,
    required: bool,
    pattern: Option<String>,
    minlength: Option<usize>,
    maxlength: Option<usize>,
    list: Option<String>,
    form_submit: Option<crate::form::FormHandler>,
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
    picker_hour: u32,
    picker_minute: u32,
    picker_highlight_day: Option<u32>,
    picker_hue: f32,
    picker_sat: f32,
    picker_val: f32,
    sv_scroll_handle: gpui::ScrollHandle,
    hue_scroll_handle: gpui::ScrollHandle,
}

impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            kind: TextKind::Text,
            multiline: false,
            value: String::new(),
            placeholder: None,
            disabled: false,
            readonly: false,
            min: None,
            max: None,
            step: None,
            rows: None,
            required: false,
            pattern: None,
            minlength: None,
            maxlength: None,
            form_submit: None,
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
            picker_hour: 0,
            picker_minute: 0,
            picker_highlight_day: None,
            picker_hue: 0.0,
            picker_sat: 1.0,
            picker_val: 1.0,
            sv_scroll_handle: gpui::ScrollHandle::new(),
            hue_scroll_handle: gpui::ScrollHandle::new(),
            list: None,
        }
    }

    /// Sync mutable state from props. Called every render so prop changes
    /// (e.g. a signal-driven `value`) take effect. While focused the `value`
    /// is not overwritten — the user's in-progress edit wins.
    pub fn sync_from_props(&mut self, props: TextInputProps, _cx: &mut Context<Self>) {
        self.kind = props.kind;
        self.multiline = props.multiline;
        if props.placeholder.is_some() {
            self.placeholder = props.placeholder;
        }
        self.disabled = props.disabled;
        self.readonly = props.readonly;
        self.min = props.min;
        self.max = props.max;
        self.step = props.step;
        self.rows = props.rows;
        self.required = props.required;
        self.pattern = props.pattern;
        self.minlength = props.minlength;
        self.list = props.list;
        self.form_submit = crate::form::current_form_submit();
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
        if let Some(idx) = props.tabindex {
            if idx >= 0 {
                self.focus_handle_field = self.focus_handle_field.clone().tab_index(idx).tab_stop(true);
            } else {
                self.focus_handle_field = self.focus_handle_field.clone().tab_stop(false);
            }
        }
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
                let start = sel.start.min(sel.end).min(self.value.len());
                let end = sel.start.max(sel.end).min(self.value.len());
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
            TextKind::Date | TextKind::DateTime | TextKind::Month | TextKind::Time
        )
    }

    fn supports_time_picker(&self) -> bool {
        matches!(self.kind, TextKind::DateTime | TextKind::Time)
    }

    fn supports_color_picker(&self) -> bool {
        matches!(self.kind, TextKind::Color)
    }

    fn select_color(&mut self, hex: &str, cx: &mut Context<Self>) {
        self.value = hex.to_string();
        self.cursor = self.value.len();
        self.selection = None;
        self.picker_open = false;
        self.fire_on_input(cx);
        self.fire_on_change(cx);
    }

    fn open_picker(&mut self) {
        if self.supports_time_picker() {
            if let Some((h, m)) = parse_time(&self.value) {
                self.picker_hour = h;
                self.picker_minute = m;
            } else {
                self.picker_hour = 0;
                self.picker_minute = 0;
            }
        }
        if self.kind == TextKind::Time {
            // Time-only: no calendar needed
            self.picker_open = true;
            return;
        }
        if let Some((y, m, _)) = parse_date(&self.value) {
            self.picker_year = y;
            self.picker_month = m;
        } else if let Some((y, m, _)) = today_date() {
            self.picker_year = y;
            self.picker_month = m;
        }
        // Initialize highlight to selected day or today
        if let Some((_, _, d)) = parse_date(&self.value) {
            self.picker_highlight_day = Some(d);
        } else if let Some((_, _, d)) = today_date() {
            self.picker_highlight_day = Some(d);
        } else {
            self.picker_highlight_day = Some(1);
        }
        self.picker_open = true;
    }

    fn select_date(&mut self, year: i32, month: u32, day: u32, cx: &mut Context<Self>) {
        if self.kind == TextKind::DateTime {
            self.value = format!(
                "{:04}-{:02}-{:02}T{:02}:{:02}",
                year, month, day, self.picker_hour, self.picker_minute
            );
        } else {
            self.value = format!("{:04}-{:02}-{:02}", year, month, day);
        }
        self.cursor = self.value.len();
        self.selection = None;
        self.picker_open = false;
        self.fire_on_input(cx);
        self.fire_on_change(cx);
    }

    fn select_time(&mut self, hour: u32, minute: u32, cx: &mut Context<Self>) {
        if self.kind == TextKind::DateTime {
            if let Some((y, m, d)) = parse_date(&self.value) {
                self.value = format!(
                    "{:04}-{:02}-{:02}T{:02}:{:02}",
                    y, m, d, hour, minute
                );
            } else {
                self.value = format!("0000-01-01T{:02}:{:02}", hour, minute);
            }
        } else {
            self.value = format!("{:02}:{:02}", hour, minute);
        }
        self.cursor = self.value.len();
        self.selection = None;
        self.picker_open = false;
        self.fire_on_input(cx);
        self.fire_on_change(cx);
    }

    fn picker_move_highlight(&mut self, delta: i32, cx: &mut Context<Self>) {
        let dim = days_in_month(self.picker_year, self.picker_month) as i32;
        let cur = self.picker_highlight_day.unwrap_or(1) as i32;
        let mut new_day = cur + delta;
        // Handle month boundary crossing
        while new_day < 1 {
            // Go to previous month
            if self.picker_month == 1 {
                self.picker_month = 12;
                self.picker_year -= 1;
            } else {
                self.picker_month -= 1;
            }
            new_day += days_in_month(self.picker_year, self.picker_month) as i32;
        }
        let cur_dim = days_in_month(self.picker_year, self.picker_month) as i32;
        while new_day > cur_dim {
            // Go to next month
            new_day -= cur_dim;
            if self.picker_month == 12 {
                self.picker_month = 1;
                self.picker_year += 1;
            } else {
                self.picker_month += 1;
            }
        }
        self.picker_highlight_day = Some(new_day as u32);
        cx.notify();
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
        // Ignore tab — handled by on_key_down (Tab = focus traversal).
        // Newline: ignore for single-line (Enter = commit), allow for multiline.
        if text == "\t" {
            return;
        }
        if text == "\n" && !self.multiline {
            return;
        }
        let filtered = self.filter_text(text);
        let (start, end) = match range {
            Some(r) => {
                let s = self.utf16_to_value_byte(r.start.min(r.end));
                let e = self.utf16_to_value_byte(r.start.max(r.end));
                (s, e)
            }
            // No explicit range: replace the active IME marked (preedit) text if
            // present, otherwise the selection, otherwise insert at the cursor.
            // This matches the platform IME contract where a `None` range targets
            // the in-progress composition — without it, each preedit update appends
            // to the value instead of replacing it, corrupting CJK input.
            None => {
                if let Some((marked, _)) = self.marked.take() {
                    (marked.start.min(marked.end), marked.start.max(marked.end))
                } else if let Some(sel) = self.selection.take() {
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
                if self.multiline {
                    self.replace_range(None, "\n", cx);
                    cx.notify();
                } else {
                    self.fire_on_change(cx);
                    crate::form::__form_submit(&mut **cx);
                }
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
        self.focus_handle_field.focus(window, cx);
        let position = event.position;
        let index = self
            .layout
            .index_for_position(position)
            .map(|i| self.display_byte_to_value_byte(i))
            .unwrap_or(self.value.len());
        if event.modifiers.shift {
            self.move_cursor(index, true);
        } else {
            // Clamp to `value.len()`: `index_for_position` can report an index
            // past the end of the text (e.g. clicking in the padding area),
            // which would later panic the word-boundary helpers.
            self.cursor = index.min(self.value.len());
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


    // ── Color picker palette ────────────────────────────────────────

    fn render_color_palette(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Initialize HSV from current hex value if not yet set
        let current_hex = self.value.clone();
        let (hue, sat, val) = if let Some(c) = parse_hex_color(&current_hex) {
            let (h, s, v) = hsl_to_hsv(c.h, c.s, c.l);
            (h, s, v)
        } else {
            (self.picker_hue, self.picker_sat, self.picker_val)
        };

        let sv_size = px(140.);
        let sv_scroll_handle = self.sv_scroll_handle.clone();
        let sv_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                // Paint SV square: x-axis = saturation (0→1), y-axis = value (1→0)
                let origin = bounds.origin;
                let w = bounds.size.width;
                let h = bounds.size.height;
                let cols = 32;
                let rows = 32;
                let cell_w = w / cols as f32;
                let cell_h = h / rows as f32;
                for cy in 0..rows {
                    for cx_idx in 0..cols {
                        let s = cx_idx as f32 / cols as f32;
                        let v = 1.0 - cy as f32 / rows as f32;
                        let (hsl_h, hsl_s, hsl_l) = hsv_to_hsl(hue, s, v);
                        let color = hsla(hsl_h, hsl_s, hsl_l, 1.0);
                        let cell_bounds = Bounds::new(
                            Point::new(
                                origin.x + cx_idx as f32 * cell_w,
                                origin.y + cy as f32 * cell_h,
                            ),
                            size(cell_w + px(1.), cell_h + px(1.)),
                        );
                        window.paint_quad(fill(cell_bounds, color));
                    }
                }
            },
        );

        let hue_bar_h = px(12.);
        let hue_scroll_handle = self.hue_scroll_handle.clone();
        let hue_canvas = canvas(
            move |_bounds, _window, _cx| {},
            move |bounds, _state, window, _cx| {
                let origin = bounds.origin;
                let w = bounds.size.width;
                let h = bounds.size.height;
                let cols = 64;
                let cell_w = w / cols as f32;
                for cx_idx in 0..cols {
                    let h_val = cx_idx as f32 / cols as f32;
                    let color = hsla(h_val, 1.0, 0.5, 1.0);
                    let cell_bounds = Bounds::new(
                        Point::new(origin.x + cx_idx as f32 * cell_w, origin.y),
                        size(cell_w + px(1.), h),
                    );
                    window.paint_quad(fill(cell_bounds, color));
                }
            },
        );

        // SV indicator position
        let sv_indicator_x = sat * 140.0;
        let sv_indicator_y = (1.0 - val) * 140.0;
        // Hue indicator position
        let hue_indicator_x = hue * 180.0;

        let (cur_hsl_h, cur_hsl_s, cur_hsl_l) = hsv_to_hsl(hue, sat, val);
        let current_color = hsla(cur_hsl_h, cur_hsl_s, cur_hsl_l, 1.0);
        let current_hex_upper = current_hex.to_uppercase();
        let hex_display = if current_hex_upper.starts_with('#') {
            current_hex_upper.clone()
        } else if current_hex_upper.is_empty() {
            "#000000".to_string()
        } else {
            format!("#{}", current_hex_upper)
        };

        let popup = gpui::div()
            .id("color-palette")
            .absolute()
            .top(px(32.))
            .left_0()
            .w(px(200.))
            .bg(gpui::white())
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.7, 0.3))
            .rounded(px(4.))
            .shadow_md()
            .p_2()
            .flex()
            .flex_col()
            .gap_2()
            .on_mouse_down_out(cx.listener(|this, _event, _window, cx| {
                this.picker_open = false;
                cx.notify();
            }))
            // SV square with drag interaction
            .child(
                gpui::div()
                    .id("sv-square")
                    .track_scroll(&sv_scroll_handle)
                    .w(sv_size)
                    .h(sv_size)
                    .relative()
                    .child(sv_canvas)
                    .child(
                        gpui::div()
                            .absolute()
                            .left(px(sv_indicator_x - 5.0))
                            .top(px(sv_indicator_y - 5.0))
                            .w(px(10.))
                            .h(px(10.))
                            .rounded_full()
                            .border_2()
                            .border_color(gpui::white())
                            .shadow_sm(),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                            let bounds = this.sv_scroll_handle.bounds();
                            let local_x = (f32::from(event.position.x - bounds.origin.x)).max(0.0).min(140.0);
                            let local_y = (f32::from(event.position.y - bounds.origin.y)).max(0.0).min(140.0);
                            this.picker_sat = local_x / 140.0;
                            this.picker_val = 1.0 - local_y / 140.0;
                            let (h, s, l) = hsv_to_hsl(this.picker_hue, this.picker_sat, this.picker_val);
                            let hex = hsl_to_hex(h, s, l);
                            this.value = hex;
                            this.cursor = this.value.len();
                            this.fire_on_input(cx);
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
                        if event.pressed_button == Some(MouseButton::Left) {
                            let bounds = this.sv_scroll_handle.bounds();
                            let local_x = (f32::from(event.position.x - bounds.origin.x)).max(0.0).min(140.0);
                            let local_y = (f32::from(event.position.y - bounds.origin.y)).max(0.0).min(140.0);
                            this.picker_sat = local_x / 140.0;
                            this.picker_val = 1.0 - local_y / 140.0;
                            let (h, s, l) = hsv_to_hsl(this.picker_hue, this.picker_sat, this.picker_val);
                            let hex = hsl_to_hex(h, s, l);
                            this.value = hex;
                            this.cursor = this.value.len();
                            this.fire_on_input(cx);
                            cx.notify();
                        }
                    })),
            )
            // Hue bar with drag interaction
            .child(
                gpui::div()
                    .id("hue-bar")
                    .track_scroll(&hue_scroll_handle)
                    .w(px(180.))
                    .h(hue_bar_h)
                    .relative()
                    .rounded(px(6.))
                    .overflow_hidden()
                    .child(hue_canvas)
                    .child(
                        gpui::div()
                            .absolute()
                            .left(px(hue_indicator_x - 4.0))
                            .top(px(0.))
                            .w(px(4.))
                            .h(hue_bar_h)
                            .bg(gpui::white())
                            .border_1()
                            .border_color(hsla(0.0, 0.0, 0.3, 1.0)),
                    )
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                            let bounds = this.hue_scroll_handle.bounds();
                            let local_x = (f32::from(event.position.x - bounds.origin.x)).max(0.0).min(180.0);
                            this.picker_hue = local_x / 180.0;
                            let (h, s, l) = hsv_to_hsl(this.picker_hue, this.picker_sat, this.picker_val);
                            let hex = hsl_to_hex(h, s, l);
                            this.value = hex;
                            this.cursor = this.value.len();
                            this.fire_on_input(cx);
                            cx.notify();
                        }),
                    )
                    .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
                        if event.pressed_button == Some(MouseButton::Left) {
                            let bounds = this.hue_scroll_handle.bounds();
                            let local_x = (f32::from(event.position.x - bounds.origin.x)).max(0.0).min(180.0);
                            this.picker_hue = local_x / 180.0;
                            let (h, s, l) = hsv_to_hsl(this.picker_hue, this.picker_sat, this.picker_val);
                            let hex = hsl_to_hex(h, s, l);
                            this.value = hex;
                            this.cursor = this.value.len();
                            this.fire_on_input(cx);
                            cx.notify();
                        }
                    })),
            )
            // Current color preview + hex display
            .child(
                gpui::div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_2()
                    .child(
                        gpui::div()
                            .w(px(24.))
                            .h(px(24.))
                            .rounded(px(4.))
                            .bg(current_color)
                            .border_1()
                            .border_color(hsla(0.0, 0.0, 0.7, 0.3)),
                    )
                    .child(
                        gpui::div()
                            .text_size(px(13.))
                            .text_color(gpui::black())
                            .child(SharedString::from(hex_display)),
                    ),
            )
            // Preset colors label
            .child(
                gpui::div()
                    .text_size(px(11.))
                    .text_color(hsla(0.0, 0.0, 0.5, 1.0))
                    .child(SharedString::from("Presets")),
            )
            // Preset palette grid
            .child({
                let mut grid = gpui::div().flex().flex_wrap().gap_1();
                for (i, preset) in COLOR_PRESETS.iter().enumerate() {
                    let color = parse_hex_color(preset).unwrap_or(hsla(0.0, 0.0, 0.5, 1.0));
                    let hex = preset.to_string();
                    grid = grid.child(
                        gpui::div()
                            .id(("color-preset", i as u64))
                            .w(px(20.))
                            .h(px(20.))
                            .rounded(px(3.))
                            .bg(color)
                            .border_1()
                            .border_color(hsla(0.0, 0.0, 0.7, 0.2))
                            .cursor_pointer()
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _event, _window, cx| {
                                    this.select_color(&hex, cx);
                                    cx.notify();
                                }),
                            ),
                    );
                }
                grid
            });

        popup
    }

    // ── Datalist suggestions ────────────────────────────────────────

    fn datalist_suggestions(&self) -> Option<Vec<String>> {
        let id = self.list.as_ref()?;
        let options = crate::input_widgets::get_datalist(id)?;
        if !self.focused || self.value.is_empty() {
            return None;
        }
        let lower = self.value.to_lowercase();
        let matches: Vec<String> = options
            .into_iter()
            .filter(|o| o.to_lowercase().starts_with(&lower))
            .take(8)
            .collect();
        Some(matches)
    }

    fn render_datalist_popup(
        &self,
        suggestions: Vec<String>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let mut list = gpui::div()
            .id("datalist-popup")
            .absolute()
            .top(px(32.))
            .left_0()
            .w_full()
            .max_h(px(200.))
            .overflow_y_scroll()
            .bg(gpui::white())
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.7, 0.3))
            .rounded(px(4.))
            .shadow_md()
            .flex()
            .flex_col();

        for (idx, suggestion) in suggestions.into_iter().enumerate() {
            list = list.child(
                gpui::div()
                    .id(("datalist-suggestion", idx as u64))
                    .w_full()
                    .px_2()
                    .py_1()
                    .text_size(px(14.))
                    .text_color(gpui::black())
                    .hover(|s| s.bg(hsla(0.6, 0.8, 0.95, 1.0)))
                    .child(SharedString::from(suggestion.clone()))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _event, _window, cx| {
                            this.value = suggestion.clone();
                            this.cursor = this.value.len();
                            this.selection = None;
                            this.fire_on_input(cx);
                            this.fire_on_change(cx);
                            cx.notify();
                        }),
                    ),
            );
        }

        list
    }

    // ── Date picker popup ────────────────────────────────────────────

    fn render_picker_popup(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        // Time-only picker (no calendar)
        if self.kind == TextKind::Time {
            return self.render_time_picker_popup(cx);
        }

        let year = self.picker_year;
        let month = self.picker_month;
        let today = today_date();
        let selected = parse_date(&self.value);
        let highlight_day = self.picker_highlight_day.unwrap_or(1);

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
            let is_highlighted = day == highlight_day && !is_selected;
            let bg = if is_selected {
                hsla(0.6, 0.8, 0.5, 1.0)
            } else if is_highlighted {
                hsla(0.6, 0.5, 0.85, 1.0)
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
            } else if is_highlighted {
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

        // Time picker section (for DateTime only)
        let time_section = if self.supports_time_picker() {
            Some(self.render_time_section(cx))
        } else {
            None
        };

        let _ = &header; // ensure header is used
        let mut popup = gpui::div()
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
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                let key = event.keystroke.key.as_str();
                match key {
                    "left" => this.picker_move_highlight(-1, cx),
                    "right" => this.picker_move_highlight(1, cx),
                    "up" => this.picker_move_highlight(-7, cx),
                    "down" => this.picker_move_highlight(7, cx),
                    "enter" => {
                        if let Some(day) = this.picker_highlight_day {
                            this.select_date(this.picker_year, this.picker_month, day, cx);
                        }
                    }
                    "escape" => {
                        this.picker_open = false;
                        cx.notify();
                    }
                    _ => {}
                }
            }))
            .child(header)
            .child(weekday_row)
            .child(day_grid);

        if let Some(ts) = time_section {
            popup = popup.child(ts);
        }

        popup.into_any_element()
    }

    fn render_time_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let hour = self.picker_hour;
        let minute = self.picker_minute;

        gpui::div()
            .mt_2()
            .pt_2()
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap_1()
            .child(
                gpui::div()
                    .id("time-hour-down")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25C0}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.picker_hour = (this.picker_hour + 23) % 24;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                gpui::div()
                    .text_size(px(14.))
                    .text_color(gpui::black())
                    .w(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(SharedString::from(format!("{:02}", hour))),
            )
            .child(
                gpui::div()
                    .id("time-hour-up")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25B6}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.picker_hour = (this.picker_hour + 1) % 24;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                gpui::div()
                    .text_size(px(14.))
                    .text_color(gpui::black())
                    .child(SharedString::from(":")),
            )
            .child(
                gpui::div()
                    .id("time-minute-down")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25C0}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.picker_minute = (this.picker_minute + 59) % 60;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                gpui::div()
                    .text_size(px(14.))
                    .text_color(gpui::black())
                    .w(px(24.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(SharedString::from(format!("{:02}", minute))),
            )
            .child(
                gpui::div()
                    .id("time-minute-up")
                    .cursor_pointer()
                    .px_1()
                    .text_color(hsla(0.0, 0.0, 0.3, 1.0))
                    .child(SharedString::from("\u{25B6}"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.picker_minute = (this.picker_minute + 1) % 60;
                            cx.notify();
                        }),
                    ),
            )
            .child(
                gpui::div()
                    .id("time-confirm")
                    .ml_2()
                    .px_2()
                    .py_1()
                    .bg(hsla(0.6, 0.8, 0.5, 1.0))
                    .rounded(px(4.))
                    .cursor_pointer()
                    .text_size(px(12.))
                    .text_color(gpui::white())
                    .child(SharedString::from("OK"))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _e, _w, cx| {
                            this.select_time(this.picker_hour, this.picker_minute, cx);
                        }),
                    ),
            )
    }

    fn render_time_picker_popup(&self, cx: &mut Context<Self>) -> gpui::AnyElement {
        gpui::div()
            .id("time-picker-popup")
            .absolute()
            .top(px(34.))
            .left_0()
            .w(px(180.))
            .p_2()
            .bg(gpui::white())
            .border_1()
            .border_color(hsla(0.0, 0.0, 0.7, 0.3))
            .rounded(px(6.))
            .shadow_md()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                gpui::div()
                    .text_size(px(12.))
                    .text_color(hsla(0.0, 0.0, 0.5, 1.0))
                    .child(SharedString::from("Select Time")),
            )
            .child(self.render_time_section(cx))
            .into_any_element()
    }
    /// Check the current value against required/pattern/minlength/maxlength/kind rules.
    fn is_valid(&self) -> bool {
        validate_text(
            self.kind,
            &self.value,
            self.required,
            self.pattern.as_deref(),
            self.minlength,
            self.maxlength,
            self.min,
            self.max,
        )
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
        let display_len = display.len();
        let mut highlights: Vec<(Range<usize>, HighlightStyle)> = Vec::new();
        if let Some(sel) = &self.selection {
            let start = self
                .value_byte_to_display_byte(sel.start.min(sel.end))
                .min(display_len);
            let end = self
                .value_byte_to_display_byte(sel.start.max(sel.end))
                .min(display_len);
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
            let start = self.value_byte_to_display_byte(marked_range.start).min(display_len);
            let end = self.value_byte_to_display_byte(marked_range.end).min(display_len);
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
            .min_h(if self.multiline {
                if let Some(rows) = self.rows {
                    px(rows as f32 * f32::from(line_height).max(16.0) + 8.)
                } else {
                    px(80.)
                }
            } else {
                px(28.)
            })
            .border_1()
            .border_color(if self.is_valid() {
                hsla(0.0, 0.0, 0.6, 0.4)
            } else {
                hsla(0., 0.8, 0.5, 1.)
            })
            .rounded(px(4.))
            .bg(gpui::white());
        if self.multiline {
            div = div.whitespace_normal();
        }
        div = div
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

        // Color picker swatch for color inputs
        if self.supports_color_picker() {
            let swatch_bg = parse_hex_color(&self.value).unwrap_or(hsla(0.0, 0.0, 0.5, 1.0));
            div = div.child(
                gpui::div()
                    .id("color-swatch")
                    .absolute()
                    .top_0()
                    .right_0()
                    .w(px(24.))
                    .h_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .bg(swatch_bg)
                    .border_1()
                    .border_color(hsla(0.0, 0.0, 0.7, 0.3))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _event, _window, cx| {
                            this.picker_open = !this.picker_open;
                            cx.notify();
                        }),
                    ),
            );
        }

        // Color picker palette popup
        if self.picker_open && self.supports_color_picker() {
            div = div.child(self.render_color_palette(cx));
        }

        // Datalist autocomplete suggestions
        if let Some(suggestions) = self.datalist_suggestions() {
            if !suggestions.is_empty() {
                div = div.child(self.render_datalist_popup(suggestions, cx));
            }
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
                sm,
                md,
                lg,
                xl,
                animation: _,
                transition: _,
            } = class;
            base(div.style());
            crate::__apply_breakpoint_styles(div.style(), sm, md, lg, xl);
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
        // Resolve the byte range to replace. With no explicit range the platform
        // is updating the in-progress composition, so target the existing marked
        // (preedit) text — otherwise each preedit refresh appends instead of
        // replacing, corrupting CJK input.
        let (start, end) = match range {
            Some(r) => {
                let s = self.utf16_to_value_byte(r.start.min(r.end));
                let e = self.utf16_to_value_byte(r.start.max(r.end));
                (s, e)
            }
            None => {
                if let Some((marked, _)) = self.marked.take() {
                    (marked.start.min(marked.end), marked.start.max(marked.end))
                } else if let Some(sel) = self.selection.take() {
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

        // `new_selected_range` is given in UTF-16 offsets relative to the marked
        // text, so convert against `filtered` (not the whole value) and offset by
        // the marked start.
        if let Some(sel_utf16) = new_selected_range {
            let sel_start = start + utf16_to_byte(&filtered, sel_utf16.start.min(sel_utf16.end));
            let sel_end = start + utf16_to_byte(&filtered, sel_utf16.start.max(sel_utf16.end));
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
        let start = self.utf16_to_value_byte(range_utf16.start.min(range_utf16.end));
        let end = self.utf16_to_value_byte(range_utf16.start.max(range_utf16.end));
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
    pub multiline: bool,
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
    pub id: Option<String>,
    pub tabindex: Option<isize>,
    pub required: bool,
    pub pattern: Option<String>,
    pub minlength: Option<usize>,
    pub maxlength: Option<usize>,
    pub list: Option<String>,
    pub rows: Option<u32>,
}

/// Get-or-create the persistent `TextInput` entity and sync props.
/// Called by the `view!` macro's expanded code every render.
pub fn text_input(props: TextInputProps) -> Entity<TextInput> {
    let id = props.id.clone();
    let entity = get_or_create_view(|cx| cx.new(|cx| TextInput::new(cx)));
    with_root_cx(|cx| {
        entity.update(cx, |view, cx| view.sync_from_props(props, cx));
        let handle = entity.focus_handle(cx);
        if let Some(id) = &id {
            crate::label::register_label_target(id, handle.clone());
        }
        crate::label::register_in_label_scope(handle);
    });
    entity
}

/// Props for constructing a `<textarea>`.
pub struct TextAreaProps {
    pub value: String,
    pub placeholder: Option<SharedString>,
    pub disabled: bool,
    pub readonly: bool,
    pub on_input: Option<Box<dyn FnMut(&str, &mut App)>>,
    pub on_change: Option<Box<dyn FnMut(&str, &mut App)>>,
    pub style: Option<Css>,
    pub class: Option<TwStyle>,
    pub id: Option<String>,
    pub tabindex: Option<isize>,
    pub rows: Option<u32>,
}

/// Get-or-create the persistent `TextInput` entity for a `<textarea>`,
/// configured for multi-line editing.
pub fn text_area(props: TextAreaProps) -> Entity<TextInput> {
    let id = props.id.clone();
    let entity = get_or_create_view(|cx| cx.new(|cx| TextInput::new(cx)));
    with_root_cx(|cx| {
        entity.update(cx, |view, cx| {
            view.sync_from_props(
                TextInputProps {
                    kind: TextKind::Text,
                    multiline: true,
                    value: props.value,
                    placeholder: props.placeholder,
                    disabled: props.disabled,
                    readonly: props.readonly,
                    min: None,
                    max: None,
                    step: None,
                    on_input: props.on_input,
                    on_change: props.on_change,
                    style: props.style,
                    class: props.class,
                    id: props.id,
                    tabindex: props.tabindex,
                    rows: props.rows,
                    required: false,
                    pattern: None,
                    minlength: None,
                    maxlength: None,
                    list: None,
                },
                cx,
            );
        });
        let handle = entity.focus_handle(cx);
        if let Some(id) = &id {
            crate::label::register_label_target(id, handle.clone());
        }
        crate::label::register_in_label_scope(handle);
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

// ── Validation ──────────────────────────────────────────────────────

/// Check a text value against HTML-like constraint rules.
///
/// `pattern` is not a JS RegExp — it supports an exact literal or a single
/// trailing/leading `*` wildcard (`foo*`, `*bar`, `foo`).
pub fn validate_text(
    kind: TextKind,
    value: &str,
    required: bool,
    pattern: Option<&str>,
    minlength: Option<usize>,
    maxlength: Option<usize>,
    min: Option<f64>,
    max: Option<f64>,
) -> bool {
    if required && value.is_empty() {
        return false;
    }
    if let Some(n) = minlength {
        if value.chars().count() < n {
            return false;
        }
    }
    if let Some(n) = maxlength {
        if value.chars().count() > n {
            return false;
        }
    }
    if let Some(pat) = pattern {
        if !match_pattern(pat, value) {
            return false;
        }
    }
    match kind {
        TextKind::Email => {
            let at = value.matches('@').count();
            if at != 1 {
                return false;
            }
            let mut parts = value.splitn(2, '@');
            let local = parts.next().unwrap_or("");
            let domain = parts.next().unwrap_or("");
            !local.is_empty() && !domain.is_empty()
        }
        TextKind::Url => value.starts_with("http://") || value.starts_with("https://"),
        TextKind::Number => {
            if value.parse::<f64>().is_err() {
                return false;
            }
            let n: f64 = value.parse().unwrap();
            if let Some(lo) = min {
                if n < lo {
                    return false;
                }
            }
            if let Some(hi) = max {
                if n > hi {
                    return false;
                }
            }
            true
        }
        _ => true,
    }
}

/// Match `pat` against `value` with single `*` wildcard support.
fn match_pattern(pat: &str, value: &str) -> bool {
    if let Some(prefix) = pat.strip_suffix('*') {
        value.starts_with(prefix)
    } else if let Some(suffix) = pat.strip_prefix('*') {
        value.ends_with(suffix)
    } else {
        value == pat
    }
}

// ── Free functions ───────────────────────────────────────────────────
/// Map a UTF-16 code-unit offset to a byte offset within `s`, clamping to
/// `s.len()`. Used to convert IME selection offsets (relative to marked text)
/// into byte offsets.
fn utf16_to_byte(s: &str, utf16_offset: usize) -> usize {
    let mut utf16 = 0;
    for (i, c) in s.char_indices() {
        if utf16 >= utf16_offset {
            return i;
        }
        utf16 += c.len_utf16();
    }
    s.len()
}


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
    // Clamp `pos` to `s.len()`: callers may pass a cursor that has fallen out
    // of sync with `value` (e.g. after an external signal clear while the
    // input is focused, or a mouse click past the end of the text). Without
    // this, `s[prev..p]` below panics with "range end index N out of range
    // for slice of length M" when `pos > s.len()`.
    let mut p = pos.min(s.len());
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
    // Clamp `pos` to `s.len()` — see `prev_word_boundary` for rationale.
    let mut p = pos.min(s.len());
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

/// Parse a `HH:MM` time string into `(hour, minute)`.
fn parse_time(s: &str) -> Option<(u32, u32)> {
    // Handle both "HH:MM" and "YYYY-MM-DDTHH:MM" formats
    let time_part = if let Some(idx) = s.find('T') {
        &s[idx + 1..]
    } else {
        s
    };
    let mut parts = time_part.split(':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

/// Convert HSL (h: 0-1, s: 0-1, l: 0-1) to HSV (h: 0-1, s: 0-1, v: 0-1).
fn hsl_to_hsv(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    let v = l + s * l.min(1.0 - l);
    let s_out = if v > 0.0 { 2.0 * (v - l) / v } else { 0.0 };
    (h, s_out, v)
}

/// Convert HSV (h: 0-1, s: 0-1, v: 0-1) to HSL (h: 0-1, s: 0-1, l: 0-1).
fn hsv_to_hsl(h: f32, s: f32, v: f32) -> (f32, f32, f32) {
    let l = v * (1.0 - s / 2.0);
    let s_out = if l > 0.0 && l < 1.0 {
        (v - l) / l.min(1.0 - l)
    } else {
        0.0
    };
    (h, s_out, l)
}

/// Convert HSL (h: 0-1, s: 0-1, l: 0-1) to a `#RRGGBB` hex string.
fn hsl_to_hex(h: f32, s: f32, l: f32) -> String {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let hp = h * 6.0;
    let x = c * (1.0 - (hp % 2.0 - 1.0).abs());
    let m = l - c / 2.0;
    let (r, g, b) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    let r = ((r + m) * 255.0).round() as u8;
    let g = ((g + m) * 255.0).round() as u8;
    let b = ((b + m) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

/// Parse a `#RRGGBB` hex color string into an `Hsla` color.
/// Accepts `#rgb` (short form) and `#rrggbb` (long form), case-insensitive.
pub fn parse_hex_color(s: &str) -> Option<gpui::Hsla> {
    let hex = s.strip_prefix('#')?;
    let (r, g, b) = if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        (r, g, b)
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
        (r * 17, g * 17, b * 17)
    } else {
        return None;
    };
    Some(gpui::hsla(
        rgb_to_hsl_h(r, g, b),
        rgb_to_hsl_s(r, g, b),
        rgb_to_hsl_l(r, g, b),
        1.0,
    ))
}

fn rgb_to_hsl_h(r: u8, g: u8, b: u8) -> f32 {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    if delta < 1e-6 {
        return 0.0;
    }
    let h = if max == r {
        ((g - b) / delta) % 6.0
    } else if max == g {
        (b - r) / delta + 2.0
    } else {
        (r - g) / delta + 4.0
    };
    (h * 60.0).rem_euclid(360.0) / 360.0
}

fn rgb_to_hsl_s(r: u8, g: u8, b: u8) -> f32 {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    let delta = max - min;
    if delta < 1e-6 {
        0.0
    } else if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    }
}

fn rgb_to_hsl_l(r: u8, g: u8, b: u8) -> f32 {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    (max + min) / 2.0
}

/// 24 preset colors for the color picker palette (8 columns × 3 rows).
const COLOR_PRESETS: [&str; 24] = [
    "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
    "#808080", "#C0C0C0", "#800000", "#008000", "#000080", "#808000", "#800080", "#008080",
    "#FFA500", "#FFC0CB", "#A52A2A", "#DAA520", "#90EE90", "#87CEEB", "#DDA0DD", "#F5DEB3",
];

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

#[cfg(test)]
mod validate_tests {
    use super::*;

    #[test]
    fn prev_word_boundary_with_empty_string_and_stale_cursor_does_not_panic() {
        // Reproduces the dashboard panic: "range end index 14 out of range for
        // slice of length 0". When the input's `value` is cleared externally
        // (e.g. form submit / reset) while focused, the cached cursor can be
        // left pointing past the now-empty value. `prev_word_boundary` must
        // clamp `pos` instead of slicing `s[prev..pos]`.
        assert_eq!(prev_word_boundary("", 14), 0);
        assert_eq!(prev_word_boundary("abc", 99), 0);
        assert_eq!(prev_word_boundary("build dashboard", 15), 6);
    }

    #[test]
    fn next_word_boundary_with_pos_beyond_len_is_clamped() {
        assert_eq!(next_word_boundary("", 14), 0);
        assert_eq!(next_word_boundary("abc", 99), 3);
        assert_eq!(next_word_boundary("build dashboard", 0), 6);
    }

    #[test]
    fn empty_required_fails() {
        assert!(!validate_text(
            TextKind::Text,
            "",
            true,
            None,
            None,
            None,
            None,
            None
        ));
        assert!(validate_text(
            TextKind::Text,
            "x",
            true,
            None,
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn email_validation() {
        assert!(validate_text(
            TextKind::Email,
            "a@b.co",
            false,
            None,
            None,
            None,
            None,
            None
        ));
        assert!(!validate_text(
            TextKind::Email,
            "nope",
            false,
            None,
            None,
            None,
            None,
            None
        ));
        assert!(!validate_text(
            TextKind::Email,
            "@b.co",
            false,
            None,
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn url_validation() {
        assert!(validate_text(
            TextKind::Url,
            "https://example.com",
            false,
            None,
            None,
            None,
            None,
            None
        ));
        assert!(!validate_text(
            TextKind::Url,
            "example.com",
            false,
            None,
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn minlength_validation() {
        assert!(!validate_text(
            TextKind::Text,
            "ab",
            false,
            None,
            Some(3),
            None,
            None,
            None
        ));
        assert!(validate_text(
            TextKind::Text,
            "abc",
            false,
            None,
            Some(3),
            None,
            None,
            None
        ));
    }

    #[test]
    fn pattern_literal() {
        assert!(validate_text(
            TextKind::Text,
            "abc",
            false,
            Some("abc"),
            None,
            None,
            None,
            None
        ));
        assert!(!validate_text(
            TextKind::Text,
            "abd",
            false,
            Some("abc"),
            None,
            None,
            None,
            None
        ));
        assert!(validate_text(
            TextKind::Text,
            "abcdef",
            false,
            Some("abc*"),
            None,
            None,
            None,
            None
        ));
        assert!(validate_text(
            TextKind::Text,
            "foobar",
            false,
            Some("*bar"),
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn number_with_min_max() {
        assert!(validate_text(
            TextKind::Number,
            "5",
            false,
            None,
            None,
            None,
            Some(0.),
            Some(10.)
        ));
        assert!(!validate_text(
            TextKind::Number,
            "15",
            false,
            None,
            None,
            None,
            Some(0.),
            Some(10.)
        ));
        assert!(!validate_text(
            TextKind::Number,
            "abc",
            false,
            None,
            None,
            None,
            None,
            None
        ));
    }

    #[test]
    fn hex_color_parse() {
        // Valid 6-digit hex
        let red = parse_hex_color("#ff0000");
        assert!(red.is_some());
        let c = red.unwrap();
        assert_eq!(c.a, 1.0);

        // Valid 3-digit hex (short form)
        let short = parse_hex_color("#f00");
        assert!(short.is_some());
        let cs = short.unwrap();
        // #f00 expands to #ff0000, so hue should be 0 (red)
        assert!((cs.h - 0.0).abs() < 1e-6 || (cs.h - 1.0).abs() < 1e-6);

        // Uppercase
        let upper = parse_hex_color("#FF0000");
        assert!(upper.is_some());

        // Invalid: no #
        assert!(parse_hex_color("ff0000").is_none());
        // Invalid: wrong length
        assert!(parse_hex_color("#ff00").is_none());
        assert!(parse_hex_color("#").is_none());
        // Invalid: non-hex chars
        assert!(parse_hex_color("#gg0000").is_none());
    }
}