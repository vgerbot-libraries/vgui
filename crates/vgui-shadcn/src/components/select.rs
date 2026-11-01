//! Strict single-select with shadcn styling.
//!
//! The committed value is controlled by the caller. Search text, popup
//! visibility and the keyboard cursor are local signals; confirming an option
//! publishes a new value and dismissing never changes the committed value.

use std::rc::Rc;

use gpui::{
    accesskit::Role, px, AnyElement, App, ElementId, Entity, Focusable,
    InteractiveElement, IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, ParentElement,
    Pixels, Point, SharedString, StatefulInteractiveElement, Styled, Window,
};
use vgui::{
    create_signal, css, floating, input_cb, next_auto_id, show, use_scroll_handle, NodeRef,
    ReadSignal, ScrollStrategy, TextInput, TextInputProps, TextKind, UniformListScrollHandle,
    WriteSignal,
};

use crate::components::icon::icon;

/// One selectable value.
pub struct SelectItem<V> {
    title: String,
    value: V,
    disabled: bool,
    display_title: Option<Rc<dyn Fn() -> AnyElement>>,
}

impl<V> SelectItem<V> {
    pub fn new(title: impl Into<String>, value: V) -> Self {
        Self {
            title: title.into(),
            value,
            disabled: false,
            display_title: None,
        }
    }

    /// Disabled options may receive the keyboard highlight but cannot be confirmed.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Override the content shown in the trigger for this option.
    pub fn display_title<E>(mut self, renderer: impl Fn() -> E + 'static) -> Self
    where
        E: IntoElement + 'static,
    {
        self.display_title = Some(Rc::new(move || renderer().into_any_element()));
        self
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn is_disabled(&self) -> bool {
        self.disabled
    }
}

/// A titled group of options. Empty groups are hidden while filtering.
pub struct SelectGroup<V> {
    title: String,
    items: Vec<SelectItem<V>>,
}

impl<V> SelectGroup<V> {
    pub fn new(title: impl Into<String>, items: Vec<SelectItem<V>>) -> Self {
        Self {
            title: title.into(),
            items,
        }
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn items(&self) -> &[SelectItem<V>] {
        &self.items
    }
}

/// Flat or grouped select data.
pub enum SelectItems<V> {
    Flat(Vec<SelectItem<V>>),
    Grouped(Vec<SelectGroup<V>>),
}

impl<V> SelectItems<V> {
    pub fn flat(items: Vec<SelectItem<V>>) -> Self {
        Self::Flat(items)
    }

    pub fn grouped(groups: Vec<SelectGroup<V>>) -> Self {
        Self::Grouped(groups)
    }

    fn get(&self, index: usize) -> Option<&SelectItem<V>> {
        match self {
            Self::Flat(items) => items.get(index),
            Self::Grouped(groups) => {
                let mut remaining = index;
                for group in groups {
                    if remaining < group.items.len() {
                        return group.items.get(remaining);
                    }
                    remaining -= group.items.len();
                }
                None
            }
        }
    }

    fn position(&self, value: &V) -> Option<usize>
    where
        V: PartialEq,
    {
        match self {
            Self::Flat(items) => items.iter().position(|item| &item.value == value),
            Self::Grouped(groups) => {
                let mut base = 0;
                for group in groups {
                    if let Some(index) = group
                        .items
                        .iter()
                        .position(|item| &item.value == value)
                    {
                        return Some(base + index);
                    }
                    base += group.items.len();
                }
                None
            }
        }
    }
}

impl<V> From<Vec<SelectItem<V>>> for SelectItems<V> {
    fn from(items: Vec<SelectItem<V>>) -> Self {
        Self::Flat(items)
    }
}

impl<V> From<Vec<SelectGroup<V>>> for SelectItems<V> {
    fn from(groups: Vec<SelectGroup<V>>) -> Self {
        Self::Grouped(groups)
    }
}

/// Trigger and option heights, matching gpui-kit's four select sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectSize {
    Xs,
    Sm,
    #[default]
    Md,
    Lg,
}

impl SelectSize {
    fn height(self) -> Pixels {
        match self {
            Self::Xs => px(20.),
            Self::Sm => px(24.),
            Self::Md => px(32.),
            Self::Lg => px(44.),
        }
    }

    fn font_size(self) -> Pixels {
        match self {
            Self::Xs | Self::Sm => px(12.),
            Self::Md | Self::Lg => px(14.),
        }
    }

    fn horizontal_padding(self) -> Pixels {
        match self {
            Self::Xs => px(6.),
            Self::Sm => px(8.),
            Self::Md | Self::Lg => px(12.),
        }
    }
}

type Matcher<V> = Rc<dyn Fn(&SelectItem<V>, &str) -> bool>;
type RowRenderer<V> = Rc<dyn Fn(&SelectItem<V>, bool, bool) -> AnyElement>;
type EmptyRenderer = Rc<dyn Fn() -> AnyElement>;

/// Props for [`select`]. Construct with [`ShadcnSelectProps::new`] and chain
/// optional configuration; fields are private so new options remain compatible.
pub struct ShadcnSelectProps<V> {
    items: SelectItems<V>,
    value: Option<V>,
    on_change: Rc<dyn Fn(Option<V>, &mut App)>,
    on_dismiss: Option<Rc<dyn Fn(&mut App)>>,
    id: Option<ElementId>,
    searchable: bool,
    search_placeholder: SharedString,
    placeholder: SharedString,
    accessibility_label: Option<SharedString>,
    menu_width: Option<Pixels>,
    menu_max_h: Pixels,
    icon: Option<AnyElement>,
    title_prefix: Option<String>,
    clearable: bool,
    disabled: bool,
    empty: Option<EmptyRenderer>,
    appearance: bool,
    focus_ring: bool,
    size: SelectSize,
    matcher: Option<Matcher<V>>,
    render_item: Option<RowRenderer<V>>,
}

impl<V> ShadcnSelectProps<V> {
    pub fn new(
        items: impl Into<SelectItems<V>>,
        value: Option<V>,
        on_change: Rc<dyn Fn(Option<V>, &mut App)>,
    ) -> Self {
        Self {
            items: items.into(),
            value,
            on_change,
            on_dismiss: None,
            id: None,
            searchable: false,
            search_placeholder: "Search…".into(),
            placeholder: "Select an option".into(),
            accessibility_label: None,
            menu_width: None,
            menu_max_h: px(320.),
            icon: None,
            title_prefix: None,
            clearable: false,
            disabled: false,
            empty: None,
            appearance: true,
            focus_ring: true,
            size: SelectSize::Md,
            matcher: None,
            render_item: None,
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = placeholder.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn on_dismiss(mut self, on_dismiss: Rc<dyn Fn(&mut App)>) -> Self {
        self.on_dismiss = Some(on_dismiss);
        self
    }

    /// Set an explicit popup width. The default follows the trigger width.
    pub fn menu_width(mut self, width: Pixels) -> Self {
        self.menu_width = Some(width);
        self
    }

    pub fn menu_max_h(mut self, height: Pixels) -> Self {
        self.menu_max_h = height;
        self
    }

    pub fn icon(mut self, icon: impl IntoElement) -> Self {
        self.icon = Some(icon.into_any_element());
        self
    }

    pub fn title_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.title_prefix = Some(prefix.into());
        self
    }

    pub fn clearable(mut self, clearable: bool) -> Self {
        self.clearable = clearable;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn focus_ring(mut self, enabled: bool) -> Self {
        self.focus_ring = enabled;
        self
    }

    pub fn size(mut self, size: SelectSize) -> Self {
        self.size = size;
        self
    }

    /// Override case-insensitive title substring matching.
    pub fn matcher(
        mut self,
        matcher: impl Fn(&SelectItem<V>, &str) -> bool + 'static,
    ) -> Self {
        self.matcher = Some(Rc::new(matcher));
        self
    }

    /// Customize an option row. Arguments are `(item, selected, highlighted)`.
    /// The outer row still owns sizing, events and accessibility.
    pub fn render_item<E>(
        mut self,
        renderer: impl Fn(&SelectItem<V>, bool, bool) -> E + 'static,
    ) -> Self
    where
        E: IntoElement + 'static,
    {
        self.render_item = Some(Rc::new(move |item, selected, highlighted| {
            renderer(item, selected, highlighted).into_any_element()
        }));
        self
    }

    pub fn empty<E>(mut self, renderer: impl Fn() -> E + 'static) -> Self
    where
        E: IntoElement + 'static,
    {
        self.empty = Some(Rc::new(move || renderer().into_any_element()));
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum VisibleRow {
    Header(String),
    Item(usize),
}

fn filtered_rows<V>(
    items: &SelectItems<V>,
    query: &str,
    matcher: Option<&Matcher<V>>,
) -> Vec<VisibleRow> {
    let query = query.to_lowercase();
    let matches = |item: &SelectItem<V>| {
        if query.is_empty() {
            true
        } else if let Some(matcher) = matcher {
            matcher(item, &query)
        } else {
            item.title.to_lowercase().contains(&query)
        }
    };

    let mut rows = Vec::new();
    match items {
        SelectItems::Flat(items) => {
            rows.extend(
                items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| matches(item))
                    .map(|(index, _)| VisibleRow::Item(index)),
            );
        }
        SelectItems::Grouped(groups) => {
            let mut base = 0;
            for group in groups {
                let matched: Vec<usize> = group
                    .items
                    .iter()
                    .enumerate()
                    .filter(|(_, item)| matches(item))
                    .map(|(index, _)| base + index)
                    .collect();
                if !matched.is_empty() {
                    rows.push(VisibleRow::Header(group.title.clone()));
                    rows.extend(matched.into_iter().map(VisibleRow::Item));
                }
                base += group.items.len();
            }
        }
    }
    rows
}

fn first_item(rows: &[VisibleRow]) -> Option<usize> {
    rows.iter().find_map(|row| match row {
        VisibleRow::Item(index) => Some(*index),
        VisibleRow::Header(_) => None,
    })
}

fn effective_item<V: PartialEq>(
    items: &SelectItems<V>,
    rows: &[VisibleRow],
    cursor: Option<&V>,
    committed: Option<&V>,
) -> Option<usize> {
    cursor
        .and_then(|value| items.position(value))
        .filter(|index| rows.contains(&VisibleRow::Item(*index)))
        .or_else(|| {
            committed
                .and_then(|value| items.position(value))
                .filter(|index| rows.contains(&VisibleRow::Item(*index)))
        })
        .or_else(|| first_item(rows))
}

fn moved_item(rows: &[VisibleRow], current: Option<usize>, forward: bool) -> Option<usize> {
    let positions: Vec<usize> = rows
        .iter()
        .filter_map(|row| match row {
            VisibleRow::Item(index) => Some(*index),
            VisibleRow::Header(_) => None,
        })
        .collect();
    if positions.is_empty() {
        return None;
    }
    let position = current.and_then(|index| positions.iter().position(|candidate| *candidate == index));
    let next = match (position, forward) {
        (Some(position), true) => (position + 1) % positions.len(),
        (Some(position), false) => (position + positions.len() - 1) % positions.len(),
        (None, true) => 0,
        (None, false) => positions.len() - 1,
    };
    positions.get(next).copied()
}

#[derive(Clone)]
struct Interaction<V> {
    items: Rc<SelectItems<V>>,
    committed: Option<V>,
    disabled: bool,
    searchable: bool,
    matcher: Option<Matcher<V>>,
    open: ReadSignal<bool>,
    set_open: WriteSignal<bool>,
    query: ReadSignal<String>,
    set_query: WriteSignal<String>,
    cursor: ReadSignal<Option<V>>,
    set_cursor: WriteSignal<Option<V>>,
    on_change: Rc<dyn Fn(Option<V>, &mut App)>,
    on_dismiss: Option<Rc<dyn Fn(&mut App)>>,
    trigger_ref: NodeRef,
    popup_ref: NodeRef,
    search: Entity<TextInput>,
    scroll: UniformListScrollHandle,
}

impl<V: Clone + PartialEq + 'static> Interaction<V> {
    fn rows(&self, query: &str) -> Vec<VisibleRow> {
        filtered_rows(&self.items, query, self.matcher.as_ref())
    }

    fn current_item(&self, rows: &[VisibleRow]) -> Option<usize> {
        effective_item(
            &self.items,
            rows,
            self.cursor.get().as_ref(),
            self.committed.as_ref(),
        )
    }

    fn scroll_to(&self, rows: &[VisibleRow], item: Option<usize>) {
        if let Some(row) = item.and_then(|item| {
            rows.iter()
                .position(|candidate| *candidate == VisibleRow::Item(item))
        }) {
            self.scroll.scroll_to_item(row, ScrollStrategy::Nearest);
        }
    }

    fn open(&self, window: &mut Window, cx: &mut App) {
        if self.disabled {
            return;
        }
        self.set_query.set(cx, String::new());
        let rows = self.rows("");
        let item = self.current_item(&rows);
        self.set_cursor.set(
            cx,
            item.and_then(|index| self.items.get(index))
                .map(|item| item.value.clone()),
        );
        self.scroll_to(&rows, item);
        self.set_open.set(cx, true);
        if self.searchable {
            self.search.focus_handle(cx).focus(window, cx);
        } else {
            self.popup_ref.focus(window, cx);
        }
    }

    fn close(&self, cursor: Option<V>, window: &mut Window, cx: &mut App) {
        if !self.open.get() {
            return;
        }
        self.set_open.set(cx, false);
        self.set_query.set(cx, String::new());
        self.set_cursor.set(cx, cursor);
        self.trigger_ref.focus(window, cx);
        if let Some(on_dismiss) = &self.on_dismiss {
            on_dismiss(cx);
        }
    }

    fn toggle(&self, window: &mut Window, cx: &mut App) {
        if self.open.get() {
            self.close(self.committed.clone(), window, cx);
        } else {
            self.open(window, cx);
        }
    }

    fn cancel(&self, window: &mut Window, cx: &mut App) {
        self.close(self.committed.clone(), window, cx);
    }

    fn confirm(&self, index: usize, window: &mut Window, cx: &mut App) {
        let Some(item) = self.items.get(index) else {
            return;
        };
        if item.disabled {
            return;
        }
        let value = item.value.clone();
        (self.on_change)(Some(value.clone()), cx);
        self.close(Some(value), window, cx);
    }

    fn confirm_cursor(&self, window: &mut Window, cx: &mut App) {
        let rows = self.rows(&self.query.get());
        if let Some(index) = self.current_item(&rows) {
            self.confirm(index, window, cx);
        }
    }

    fn move_cursor(&self, forward: bool, cx: &mut App) {
        let rows = self.rows(&self.query.get());
        let current = self.current_item(&rows);
        let next = moved_item(&rows, current, forward);
        if let Some(index) = next {
            if let Some(item) = self.items.get(index) {
                self.set_cursor.set(cx, Some(item.value.clone()));
            }
        }
        self.scroll_to(&rows, next);
    }

    fn clear(&self, cx: &mut App) {
        if self.disabled {
            return;
        }
        self.set_cursor.set(cx, None);
        (self.on_change)(None, cx);
    }
}

/// Render a controlled, shadcn-style single select.
pub fn select<V>(props: ShadcnSelectProps<V>) -> AnyElement
where
    V: Clone + PartialEq + 'static,
{
    let ShadcnSelectProps {
        items,
        value,
        on_change,
        on_dismiss,
        id,
        searchable,
        search_placeholder,
        placeholder,
        accessibility_label,
        menu_width,
        menu_max_h,
        icon: custom_icon,
        title_prefix,
        clearable,
        disabled,
        empty,
        appearance,
        focus_ring,
        size,
        matcher,
        render_item,
    } = props;

    let items = Rc::new(items);
    let (open, set_open) = create_signal(false);
    let (query, set_query) = create_signal(String::new());
    let (cursor, set_cursor) = create_signal::<Option<V>>(None);
    let trigger_ref = NodeRef::new();
    let popup_ref = NodeRef::new();
    let scroll = use_scroll_handle();
    let root_id = id.unwrap_or_else(|| ("shadcn-select", next_auto_id()).into());
    let list_id = ("shadcn-select-list", next_auto_id());

    let search = vgui::text_input(TextInputProps {
        kind: TextKind::Search,
        multiline: false,
        value: query.get(),
        placeholder: Some(search_placeholder),
        disabled: false,
        readonly: false,
        min: None,
        max: None,
        step: None,
        on_input: Some(input_cb({
            let items = items.clone();
            let matcher = matcher.clone();
            let set_query = set_query.clone();
            let set_cursor = set_cursor.clone();
            let scroll = scroll.clone();
            move |raw, cx| {
                set_query.set(cx, raw.to_string());
                let rows = filtered_rows(&items, raw, matcher.as_ref());
                let first = first_item(&rows);
                if let Some(index) = first {
                    if let Some(item) = items.get(index) {
                        set_cursor.set(cx, Some(item.value.clone()));
                    }
                } else {
                    set_cursor.set(cx, None);
                }
                if first.is_some() {
                    scroll.scroll_to_item(0, ScrollStrategy::Top);
                }
            }
        })),
        on_change: None,
        style: Some(css! {
            width: 100%;
            height: 34px;
            padding: 4px 8px 4px 30px;
            background: rgba(0, 0, 0, 0);
            color: var(--popover-foreground);
            border-width: 0px;
            font-size: 13px;
        }),
        class: None,
        id: None,
        tabindex: None,
        required: false,
        pattern: None,
        minlength: None,
        maxlength: None,
        list: None,
        rows: None,
    });

    let interaction = Interaction {
        items: items.clone(),
        committed: value.clone(),
        disabled,
        searchable,
        matcher: matcher.clone(),
        open: open.clone(),
        set_open: set_open.clone(),
        query: query.clone(),
        set_query: set_query.clone(),
        cursor: cursor.clone(),
        set_cursor: set_cursor.clone(),
        on_change: on_change.clone(),
        on_dismiss,
        trigger_ref: trigger_ref.clone(),
        popup_ref: popup_ref.clone(),
        search: search.clone(),
        scroll: scroll.clone(),
    };

    let selected = value.as_ref().and_then(|value| items.position(value));
    let selected_item = selected.and_then(|index| items.get(index));
    let display = selected_item
        .and_then(|item| item.display_title.as_ref().map(|renderer| renderer()))
        .unwrap_or_else(|| {
            gpui::div()
                .child(
                    selected_item
                        .map(|item| {
                            title_prefix
                                .as_ref()
                                .map(|prefix| format!("{prefix}{}", item.title))
                                .unwrap_or_else(|| item.title.clone())
                        })
                        .unwrap_or_else(|| placeholder.to_string()),
                )
                .into_any_element()
        });
    let show_clear = clearable && selected_item.is_some();

    let title = gpui::div()
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .truncate()
        .child(display);
    let title = if selected_item.is_none() || disabled {
        css! { color: var(--muted-foreground); }.apply(title)
    } else {
        css! { color: var(--foreground); }.apply(title)
    };

    let mut trigger = gpui::div()
        .id(root_id)
        .relative()
        .w_full()
        .h(size.height())
        .px(size.horizontal_padding())
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_2()
        .text_size(size.font_size())
        .role(Role::ComboBox)
        .aria_expanded(open.get())
        .aria_label(
            accessibility_label
                .clone()
                .unwrap_or_else(|| placeholder.to_string().into()),
        )
        .aria_value(display_for_a11y(selected_item, title_prefix.as_deref(), &placeholder))
        .cursor_pointer()
        .on_mouse_down(MouseButton::Left, {
            let interaction = interaction.clone();
            move |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                interaction.toggle(window, cx);
                cx.stop_propagation();
            }
        })
        .on_key_down({
            let interaction = interaction.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if interaction.open.get() || interaction.disabled {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "enter" | "space" | " " | "down" | "up" => {
                        interaction.open(window, cx);
                        cx.stop_propagation();
                    }
                    _ => {}
                }
            }
        })
        .child(title);

    if vgui::__bind_ref(&trigger_ref) {
        trigger = trigger
            .track_focus(&trigger_ref.focus_handle())
            .track_scroll(&trigger_ref.scroll_handle())
            .focusable();
    }
    trigger = if appearance {
        css! {
            background: var(--background);
            border-width: 1px;
            border-style: solid;
            border-color: var(--input);
            border-radius: var(--radius);
        }
        .apply(trigger)
    } else {
        css! {
            background: rgba(0, 0, 0, 0);
            border-width: 0px;
            border-radius: var(--radius);
        }
        .apply(trigger)
    };
    if disabled {
        trigger = css! { opacity: 0.5; cursor: default; }.apply(trigger);
    }
    if focus_ring && appearance && !disabled {
        trigger = trigger.focus(|style| {
            style.border_color(vgui::__var_color("--ring", Some(gpui::hsla(0., 0., 0., 1.))))
        });
    }
    if open.get() && appearance {
        trigger = trigger.border_color(vgui::__var_color(
            "--ring",
            Some(gpui::hsla(0., 0., 0., 1.)),
        ));
    }

    if show_clear {
        let clear = gpui::div()
            .id(("shadcn-select-clear", next_auto_id()))
            .flex()
            .items_center()
            .justify_center()
            .on_mouse_down(MouseButton::Left, {
                let interaction = interaction.clone();
                move |_: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                    interaction.clear(cx);
                    cx.stop_propagation();
                }
            })
            .child(icon("close").size_px(14.).color(vgui::__var_color(
                "--muted-foreground",
                Some(gpui::hsla(0., 0., 0.45, 1.)),
            )));
        trigger = trigger.child(clear);
    } else {
        trigger = trigger.child(custom_icon.unwrap_or_else(|| {
            icon("chevron_down")
                .size_px(16.)
                .color(vgui::__var_color(
                    "--muted-foreground",
                    Some(gpui::hsla(0., 0., 0.45, 1.)),
                ))
                .into_any_element()
        }));
    }

    let is_open = open.get() && !disabled;
    let query_text = query.get();
    let rows = filtered_rows(&items, if searchable { &query_text } else { "" }, matcher.as_ref());
    let current = effective_item(&items, &rows, cursor.get().as_ref(), value.as_ref());
    interaction.scroll_to(&rows, current);

    let bounds = if trigger_ref.is_bound() {
        trigger_ref.bounds()
    } else {
        gpui::Bounds::default()
    };
    let popup_width = menu_width.unwrap_or_else(|| {
        if bounds.size.width > px(0.) {
            bounds.size.width
        } else {
            px(240.)
        }
    });
    let popup_position = Point::new(bounds.origin.x, bounds.origin.y + bounds.size.height + px(6.));

    let search_box = gpui::div()
        .relative()
        .w_full()
        .child(
            gpui::div()
                .absolute()
                .left(px(10.))
                .top(px(9.))
                .child(icon("search").size_px(14.).color(vgui::__var_color(
                    "--muted-foreground",
                    Some(gpui::hsla(0., 0., 0.45, 1.)),
                ))),
        )
        .child(search.clone());
    let search_box = search_box.border_b_1().border_color(vgui::__var_color(
        "--border",
        Some(gpui::hsla(0., 0., 0.9, 1.)),
    ));

    let list = if rows.is_empty() {
        let content = empty.map(|renderer| renderer()).unwrap_or_else(|| {
            css! {
                padding: 24px 12px;
                text-align: center;
                font-size: 13px;
                color: var(--muted-foreground);
            }
            .apply(gpui::div().child("No options found"))
            .into_any_element()
        });
        gpui::div().w_full().child(content).into_any_element()
    } else {
        let list_rows = rows.clone();
        let list_items = items.clone();
        let list_interaction = interaction.clone();
        let renderer = render_item.clone();
        let committed = value.clone();
        let row_height = size.height();
        let font_size = size.font_size();
        gpui::uniform_list(list_id, rows.len(), move |range, _window, _cx| {
            range
                .filter_map(|row_index| {
                    let row = list_rows.get(row_index)?.clone();
                    match row {
                        VisibleRow::Header(title) => Some(
                            css! {
                                display: flex;
                                align-items: center;
                                padding: 0px 12px;
                                color: var(--muted-foreground);
                                font-size: 12px;
                                font-weight: 500;
                            }
                            .apply(
                                gpui::div()
                                    .id(("shadcn-select-header", row_index as u64))
                                    .w_full()
                                    .h(row_height)
                                    .child(title),
                            )
                            .into_any_element(),
                        ),
                        VisibleRow::Item(index) => {
                            let item = list_items.get(index)?;
                            let selected = committed.as_ref() == Some(&item.value);
                            let highlighted = current == Some(index);
                            let content = renderer
                                .as_ref()
                                .map(|renderer| renderer(item, selected, highlighted))
                                .unwrap_or_else(|| {
                                    gpui::div()
                                        .flex_1()
                                        .min_w_0()
                                        .overflow_hidden()
                                        .whitespace_nowrap()
                                        .truncate()
                                        .child(item.title.clone())
                                        .into_any_element()
                                });
                            let mut element = gpui::div()
                                .id(("shadcn-select-option", row_index as u64))
                                .w_full()
                                .h(row_height)
                                .px_3()
                                .flex()
                                .flex_row()
                                .items_center()
                                .gap_2()
                                .text_size(font_size)
                                .role(Role::ListBoxOption)
                                .aria_selected(selected)
                                .on_hover({
                                    let interaction = list_interaction.clone();
                                    let value = item.value.clone();
                                    move |hovered: &bool, _window: &mut Window, cx: &mut App| {
                                        if *hovered {
                                            interaction.set_cursor.set(cx, Some(value.clone()));
                                        }
                                    }
                                })
                                .on_mouse_down(MouseButton::Left, {
                                    let interaction = list_interaction.clone();
                                    move |_: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                                        interaction.confirm(index, window, cx);
                                        cx.stop_propagation();
                                    }
                                })
                                .child(content)
                                .child(if selected {
                                    icon("check")
                                        .size_px(14.)
                                        .color(vgui::__var_color(
                                            "--accent-foreground",
                                            Some(gpui::hsla(0., 0., 0.1, 1.)),
                                        ))
                                        .into_any_element()
                                } else {
                                    gpui::div().w(px(14.)).into_any_element()
                                });
                            if highlighted {
                                element = css! {
                                    background: var(--accent);
                                    color: var(--accent-foreground);
                                }
                                .apply(element);
                            } else {
                                element = css! {
                                    background: var(--popover);
                                    color: var(--popover-foreground);
                                }
                                .apply(element);
                            }
                            if item.disabled {
                                element = css! { opacity: 0.5; cursor: default; }.apply(element);
                            } else {
                                element = element.cursor_pointer();
                            }
                            Some(element.into_any_element())
                        }
                    }
                })
                .collect()
        })
        .track_scroll(&scroll)
        .w_full()
        .max_h(menu_max_h)
        .into_any_element()
    };

    let mut popup = gpui::div()
        .id(("shadcn-select-popup", next_auto_id()))
        .w(popup_width)
        .max_h(if searchable { menu_max_h + px(35.) } else { menu_max_h })
        .overflow_hidden()
        .flex()
        .flex_col()
        .occlude()
        .role(Role::ListBox)
        .on_key_down({
            let interaction = interaction.clone();
            move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                match event.keystroke.key.as_str() {
                    "down" => interaction.move_cursor(true, cx),
                    "up" => interaction.move_cursor(false, cx),
                    "enter" => interaction.confirm_cursor(window, cx),
                    "escape" => interaction.cancel(window, cx),
                    _ => return,
                }
                cx.stop_propagation();
            }
        })
        .on_mouse_down_out({
            let interaction = interaction.clone();
            move |event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                if interaction.trigger_ref.is_bound()
                    && interaction.trigger_ref.bounds().contains(&event.position)
                {
                    return;
                }
                interaction.cancel(window, cx);
                cx.stop_propagation();
            }
        });
    if vgui::__bind_ref(&popup_ref) {
        popup = popup.track_focus(&popup_ref.focus_handle()).focusable();
    }
    popup = css! {
        background: var(--popover);
        color: var(--popover-foreground);
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
        padding: 4px;
    }
    .apply(popup)
    .shadow_md();
    if searchable {
        popup = popup.child(search_box);
    }
    popup = popup.child(list);

    gpui::div()
        .relative()
        .w_full()
        .child(trigger)
        .child(show(is_open, floating(popup_position, popup), gpui::Empty))
        .into_any_element()
}

fn display_for_a11y<V>(
    selected: Option<&SelectItem<V>>,
    prefix: Option<&str>,
    placeholder: &SharedString,
) -> SharedString {
    selected
        .map(|item| {
            prefix
                .map(|prefix| format!("{prefix}{}", item.title))
                .unwrap_or_else(|| item.title.clone())
        })
        .unwrap_or_else(|| placeholder.to_string())
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grouped() -> SelectItems<u32> {
        SelectItems::grouped(vec![
            SelectGroup::new(
                "Fruit",
                vec![
                    SelectItem::new("Apple", 1),
                    SelectItem::new("Banana", 2).disabled(true),
                ],
            ),
            SelectGroup::new("Empty", vec![SelectItem::new("Celery", 3)]),
            SelectGroup::new(
                "Other",
                vec![SelectItem::new("Apricot", 4), SelectItem::new("Cherry", 5)],
            ),
        ])
    }

    #[test]
    fn filtering_hides_empty_groups_and_preserves_full_indices() {
        let rows = filtered_rows(&grouped(), "ap", None);
        assert_eq!(
            rows,
            vec![
                VisibleRow::Header("Fruit".into()),
                VisibleRow::Item(0),
                VisibleRow::Header("Other".into()),
                VisibleRow::Item(3),
            ]
        );
        assert_eq!(first_item(&rows), Some(0));
        assert_eq!(moved_item(&rows, Some(0), true), Some(3));
        assert_eq!(moved_item(&rows, Some(3), true), Some(0));
        assert_eq!(moved_item(&rows, Some(0), false), Some(3));
    }

    #[test]
    fn cursor_uses_value_after_items_are_reordered() {
        let rows = filtered_rows(&grouped(), "", None);
        let items = SelectItems::flat(vec![
            SelectItem::new("Cherry", 5),
            SelectItem::new("Apple", 1),
        ]);
        let reordered = filtered_rows(&items, "", None);
        assert_eq!(effective_item(&items, &reordered, Some(&1), None), Some(1));
        assert_eq!(effective_item(&items, &reordered, Some(&99), None), Some(0));
        assert_eq!(effective_item(&grouped(), &rows, None, Some(&4)), Some(3));
    }

    #[test]
    fn custom_matcher_receives_the_normalized_query() {
        let items = SelectItems::flat(vec![SelectItem::new("Alpha", 1)]);
        let matcher: Matcher<u32> = Rc::new(|item, query| item.title == "Alpha" && query == "zz");
        assert_eq!(filtered_rows(&items, "ZZ", Some(&matcher)), vec![VisibleRow::Item(0)]);
        assert!(filtered_rows(&items, "other", Some(&matcher)).is_empty());
    }

    #[test]
    fn empty_data_has_no_cursor_or_navigation_target() {
        let items = SelectItems::<u32>::flat(Vec::new());
        let rows = filtered_rows(&items, "", None);
        assert_eq!(first_item(&rows), None);
        assert_eq!(moved_item(&rows, None, true), None);
        assert_eq!(effective_item(&items, &rows, None, Some(&1)), None);
    }
}
