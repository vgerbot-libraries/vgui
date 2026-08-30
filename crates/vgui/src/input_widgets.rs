use gpui::{
    canvas, fill, px, quad, size, App, AppContext, BorderStyle, Bounds, Context, Entity,
    FocusHandle, Focusable, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, PathPromptOptions, Point, Render,
    SharedString, Stateful, StatefulInteractiveElement, Styled, Window, hsla,
};

use crate::reactive::{get_or_create_slot, get_or_create_view, with_root_cx};
use crate::style::{Css, TwStyle};
use std::sync::Arc;
// ── Radio group scope ───────────────────────────────────────────────
//
// A thread_local stack mirroring `label.rs`'s `SCOPE_STACK`. While a
// `<radiogroup>` is rendering, child `radio()` calls register their
// `FocusHandle`s into the shared vector so `radiogroup()`'s arrow-key
// handler can move focus between them.

thread_local! {
    static RADIO_SCOPE: std::cell::RefCell<Vec<Arc<std::cell::RefCell<Vec<FocusHandle>>>>> =
        std::cell::RefCell::new(Vec::new());
}

/// Push a new handle-collection onto the radio scope stack. Returns the
/// shared handle vector for passing to `radiogroup()`.
#[doc(hidden)]
pub fn __radiogroup_scope_enter() -> Arc<std::cell::RefCell<Vec<FocusHandle>>> {
    let handles = Arc::new(std::cell::RefCell::new(Vec::new()));
    RADIO_SCOPE.with(|s| s.borrow_mut().push(handles.clone()));
    handles
}

/// Pop the radio scope stack.
#[doc(hidden)]
pub fn __radiogroup_scope_exit() {
    RADIO_SCOPE.with(|s| {
        s.borrow_mut().pop();
    });
}

/// Register a radio's `FocusHandle` with the current radiogroup scope.
/// No-op if not inside a `<radiogroup>`.
pub(crate) fn register_in_radio_scope(handle: FocusHandle) {
    RADIO_SCOPE.with(|s| {
        if let Some(top) = s.borrow_mut().last_mut() {
            top.borrow_mut().push(handle);
        }
    });
}

// ── Checkbox ─────────────────────────────────────────────────────────

/// Props for `<input type="checkbox">`.
pub struct CheckboxProps {
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<Box<dyn FnMut(bool, &mut App)>>,
}

/// Render a checkbox. Returns a `Stateful<Div>` so the macro can chain
/// `style`/`class`/`hover`/`active`/`focus`/`id` uniformly.
pub fn checkbox(props: CheckboxProps) -> Stateful<gpui::Div> {
    let checked = props.checked;
    let disabled = props.disabled;
    let on_change = std::cell::RefCell::new(props.on_change);

    let box_color = if disabled {
        hsla(0.0, 0.0, 0.8, 1.0)
    } else if checked {
        hsla(0.6, 0.8, 0.5, 1.0)
    } else {
        hsla(0.0, 0.0, 1.0, 1.0)
    };
    let border_color = if disabled {
        hsla(0.0, 0.0, 0.7, 1.0)
    } else {
        hsla(0.6, 0.6, 0.5, 1.0)
    };

    gpui::div()
        .id(("checkbox", crate::reactive::next_auto_id()))
        .cursor_pointer()
        .size(px(18.))
        .rounded(px(4.))
        .bg(box_color)
        .border_1()
        .border_color(border_color)
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
                if disabled {
                    return;
                }
                if let Some(cb) = on_change.borrow_mut().as_mut() {
                    cb(!checked, cx);
                }
            },
        )
        .child(if checked {
            // Checkmark glyph
            gpui::div()
                .text_color(gpui::white())
                .text_size(px(14.))
                .child(SharedString::from("\u{2713}"))
                .into_any_element()
        } else {
            gpui::Empty.into_any_element()
        })
}

// ── Radio ────────────────────────────────────────────────────────────

/// Props for `<input type="radio">`.
pub struct RadioProps {
    pub checked: bool,
    pub disabled: bool,
    pub on_change: Option<Box<dyn FnMut(bool, &mut App)>>,
}

/// Render a radio button. Returns a `Stateful<Div>`.
///
/// Each radio instance owns a persistent `FocusHandle` (cached in a reactive
/// scope slot) so focus survives re-renders. When inside a `<radiogroup>`,
/// the handle is registered for arrow-key navigation and roving tabindex
/// applies: the checked radio is a tab stop, unchecked radios are focusable
/// but not reachable via Tab. Disabled radios are not focusable at all.
pub fn radio(props: RadioProps) -> Stateful<gpui::Div> {
    let checked = props.checked;
    let disabled = props.disabled;
    let on_change = std::cell::RefCell::new(props.on_change);

    // Persistent focus handle for this radio instance.
    let handle = get_or_create_slot(|cx| cx.focus_handle());

    // Register with the current <radiogroup> scope (no-op if outside one).
    register_in_radio_scope(handle.clone());

    let border_color = if disabled {
        hsla(0.0, 0.0, 0.7, 1.0)
    } else {
        hsla(0.6, 0.6, 0.5, 1.0)
    };
    let bg = if disabled {
        hsla(0.0, 0.0, 0.9, 1.0)
    } else {
        hsla(0.0, 0.0, 1.0, 1.0)
    };

    let mut div = gpui::div()
        .id(("radio", crate::reactive::next_auto_id()))
        .track_focus(&handle)
        .cursor_pointer()
        .size(px(18.))
        .rounded(px(9.))
        .bg(bg)
        .border_1()
        .border_color(border_color)
        .flex()
        .items_center()
        .justify_center();

    // Roving tabindex: checked radio is a tab stop; unchecked is focusable
    // but not a tab stop. Disabled radios are not focusable at all.
    if !disabled {
        if checked {
            div = div.tab_index(0);
        } else {
            div = div.focusable().tab_stop(false);
        }
    }

    div = div.on_mouse_down(
        MouseButton::Left,
        move |_event: &MouseDownEvent, _window: &mut Window, cx: &mut App| {
            if disabled {
                return;
            }
            if let Some(cb) = on_change.borrow_mut().as_mut() {
                cb(true, cx);
            }
        },
    );

    div.child(if checked {
        gpui::div()
            .size(px(10.))
            .rounded(px(5.))
            .bg(hsla(0.6, 0.8, 0.5, 1.0))
            .into_any_element()
    } else {
        gpui::Empty.into_any_element()
    })
}

/// Render a radio group with roving tabindex. Only the checked radio is a
/// tab stop; arrow keys move focus between radios in the group.
///
/// `handles` is the shared vector populated by child `radio()` calls during
/// render (via `__radiogroup_scope_enter`/`register_in_radio_scope`). By the
/// time the key handler runs, all handles are collected.
pub fn radiogroup(
    handles: Arc<std::cell::RefCell<Vec<FocusHandle>>>,
    content: impl gpui::IntoElement,
) -> gpui::AnyElement {
    let handles_for_keys = handles.clone();
    gpui::div()
        .id(("radiogroup", crate::reactive::next_auto_id()))
        .tab_group()
        .on_key_down(move |event: &KeyDownEvent, window: &mut Window, cx: &mut gpui::App| {
            let handles = handles_for_keys.borrow();
            if handles.is_empty() {
                return;
            }
            let focused = window.focused(cx);
            let current_idx = focused
                .as_ref()
                .and_then(|f| handles.iter().position(|h| h == f));
            match event.keystroke.key.as_str() {
                "left" | "up" => {
                    let next_idx = match current_idx {
                        Some(0) => handles.len() - 1,
                        Some(i) => i - 1,
                        None => 0,
                    };
                    window.focus(&handles[next_idx], cx);
                    cx.stop_propagation();
                }
                "right" | "down" => {
                    let next_idx = match current_idx {
                        Some(i) => (i + 1) % handles.len(),
                        None => 0,
                    };
                    window.focus(&handles[next_idx], cx);
                    cx.stop_propagation();
                }
                _ => {}
            }
        })
        .child(content)
        .into_any_element()
}

// ── Range (slider) ───────────────────────────────────────────────────

/// The persistent slider view. Cached in a reactive scope slot so drag
/// state persists across re-renders.
pub struct RangeInput {
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    dragging: bool,
    on_change: Option<Box<dyn FnMut(f64, &mut App)>>,
    style: Option<Css>,
    class: Option<TwStyle>,
    bounds: Option<Bounds<gpui::Pixels>>,
    focus_handle_field: gpui::FocusHandle,
}

impl RangeInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            disabled: false,
            dragging: false,
            on_change: None,
            style: None,
            class: None,
            bounds: None,
            focus_handle_field: cx.focus_handle(),
        }
    }

    pub fn sync_from_props(&mut self, props: RangeProps, _cx: &mut Context<Self>) {
        self.min = props.min;
        self.max = props.max;
        self.step = props.step;
        self.disabled = props.disabled;
        if !self.dragging {
            self.value = props.value;
        }
        self.style = props.style;
        self.class = props.class;
        self.on_change = props.on_change;
        if let Some(idx) = props.tabindex {
            if idx >= 0 {
                self.focus_handle_field = self.focus_handle_field.clone().tab_index(idx).tab_stop(true);
            } else {
                self.focus_handle_field = self.focus_handle_field.clone().tab_stop(false);
            }
        }
    }

    fn ratio(&self) -> f32 {
        let span = (self.max - self.min).max(1e-9);
        ((self.value - self.min) / span).clamp(0.0, 1.0) as f32
    }

    fn value_from_x(&self, x: gpui::Pixels, bounds_width: gpui::Pixels) -> f64 {
        let ratio = (x / bounds_width.max(px(1.0))).clamp(0.0, 1.0) as f64;
        let raw = self.min + ratio * (self.max - self.min);
        // snap to step
        if self.step > 0.0 {
            let steps = ((raw - self.min) / self.step).round();
            (self.min + steps * self.step).clamp(self.min, self.max)
        } else {
            raw.clamp(self.min, self.max)
        }
    }

    fn fire_change(&mut self, cx: &mut Context<Self>) {
        let v = self.value;
        if let Some(cb) = self.on_change.as_mut() {
            cb(v, &mut **cx);
        }
    }

    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        bounds: Bounds<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        self.focus_handle_field.focus(window, cx);
        self.dragging = true;
        let x = event.position.x - bounds.origin.x;
        self.value = self.value_from_x(x, bounds.size.width);
        self.fire_change(cx);
        cx.notify();
    }

    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        bounds: Bounds<gpui::Pixels>,
        cx: &mut Context<Self>,
    ) {
        if !self.dragging {
            return;
        }
        let x = event.position.x - bounds.origin.x;
        self.value = self.value_from_x(x, bounds.size.width);
        self.fire_change(cx);
        cx.notify();
    }

    fn handle_mouse_up(&mut self, cx: &mut Context<Self>) {
        self.dragging = false;
        cx.notify();
    }

    fn handle_key_down(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let key = event.keystroke.key.as_str();
        let step = if event.keystroke.modifiers.shift {
            self.step * 10.0
        } else {
            self.step
        };
        match key {
            "left" | "down" => {
                self.value = (self.value - step).max(self.min);
                self.fire_change(cx);
                cx.notify();
            }
            "right" | "up" => {
                self.value = (self.value + step).min(self.max);
                self.fire_change(cx);
                cx.notify();
            }
            "home" => {
                self.value = self.min;
                self.fire_change(cx);
                cx.notify();
            }
            "end" => {
                self.value = self.max;
                self.fire_change(cx);
                cx.notify();
            }
            _ => {}
        }
    }
}

impl Render for RangeInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let ratio = self.ratio();
        let track_color = hsla(0.0, 0.0, 0.8, 1.0);
        let fill_color = hsla(0.6, 0.8, 0.5, 1.0);
        let thumb_color = gpui::white();
        let thumb_border = hsla(0.6, 0.6, 0.5, 1.0);
        let entity = cx.entity();

        // The canvas is absolutely positioned over the entire widget so it
        // captures the correct bounds (a non-absolute flex child would get
        // zero width because the track has `w_full`). It also paints the
        // fill bar and thumb using actual pixel bounds for correct positioning.
        let entity_for_bounds = entity.clone();

        let mut div = gpui::div()
            .id("range-input")
            .track_focus(&self.focus_handle_field)
            .focusable()
            .cursor_pointer()
            .h(px(28.))
            .relative()
            .flex()
            .items_center()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                    if let Some(bounds) = this.bounds {
                        this.handle_mouse_down(event, bounds, window, cx);
                    }
                }),
            )
            .on_mouse_move(
                cx.listener(move |this, event: &MouseMoveEvent, _window, cx| {
                    if let Some(bounds) = this.bounds {
                        this.handle_mouse_move(event, bounds, cx);
                    }
                }),
            )
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event: &MouseUpEvent, _window, cx| {
                    this.handle_mouse_up(cx);
                }),
            )
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, _window, cx| {
                this.handle_key_down(event, cx);
            }))
            .child(
                // Track (background only — fill & thumb are painted on the canvas)
                gpui::div()
                    .h(px(6.))
                    .w_full()
                    .rounded(px(3.))
                    .bg(track_color),
            )
            .child(
                canvas(
                    move |bounds, _window, _cx| bounds,
                    move |bounds, _state, window, cx| {
                        // Store bounds for mouse handlers (only notify on change)
                        entity_for_bounds.update(cx, |this, cx| {
                            if this.bounds != Some(bounds) {
                                this.bounds = Some(bounds);
                                cx.notify();
                            }
                        });
                        // Paint fill bar
                        let track_h = px(6.);
                        let track_y = bounds.origin.y
                            + px((bounds.size.height - track_h) / px(2.));
                        let fill_w = ratio * bounds.size.width;
                        let fill_bounds = Bounds::new(
                            Point::new(bounds.origin.x, track_y),
                            size(fill_w, track_h),
                        );
                        window.paint_quad(fill(fill_bounds, fill_color));
                        // Paint thumb (18px circle centered on the fill edge)
                        let thumb_size = px(18.);
                        let thumb_x = bounds.origin.x + fill_w
                            - px(thumb_size / px(2.));
                        let thumb_y = bounds.origin.y
                            + px((bounds.size.height - thumb_size) / px(2.));
                        let thumb_bounds = Bounds::new(
                            Point::new(thumb_x, thumb_y),
                            size(thumb_size, thumb_size),
                        );
                        window.paint_quad(quad(
                            thumb_bounds,
                            px(thumb_size / px(2.)),
                            thumb_color,
                            px(2.),
                            thumb_border,
                            BorderStyle::default(),
                        ));
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full(),
            );

        if let Some(style) = self.style.take() {
            div = style.apply(div);
        }
        if let Some(class) = self.class.take() {
            let TwStyle {
                base,
                hover,
                focus,
                active,
                animation: _,
                transition: _,
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

        let _ = entity;
        div
    }
}

impl Focusable for RangeInput {
    fn focus_handle(&self, _cx: &App) -> gpui::FocusHandle {
        self.focus_handle_field.clone()
    }
}

/// Props for `<input type="range">`.
pub struct RangeProps {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub disabled: bool,
    pub on_change: Option<Box<dyn FnMut(f64, &mut App)>>,
    pub style: Option<Css>,
    pub class: Option<TwStyle>,
    pub id: Option<String>,
    pub tabindex: Option<isize>,
}

/// Get-or-create the persistent `RangeInput` entity and sync props.
pub fn range_input(props: RangeProps) -> Entity<RangeInput> {
    let id = props.id.clone();
    let entity = get_or_create_view(|cx| cx.new(|cx| RangeInput::new(cx)));
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

// ── File ─────────────────────────────────────────────────────────────

/// Props for `<input type="file">`.
pub struct FileProps {
    pub multiple: bool,
    pub on_change: Option<Box<dyn FnMut(Vec<std::path::PathBuf>, &mut App)>>,
}

/// Render a file-input button. Returns a `Stateful<Div>`; the macro chains
/// style/class and emits children (the label) as `.child(..)`.
pub fn file_input(props: FileProps) -> Stateful<gpui::Div> {
    let multiple = props.multiple;
    let on_change = std::cell::RefCell::new(props.on_change);

    let id = ("file-input", crate::reactive::next_auto_id());
    let div = gpui::div()
        .id(id)
        .cursor_pointer()
        .px_3()
        .py_1()
        .rounded(px(4.))
        .bg(hsla(0.6, 0.6, 0.5, 1.0))
        .text_color(gpui::white())
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, window: &mut Window, cx: &mut App| {
                let receiver = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple,
                    prompt: None,
                });
                let mut on_change = on_change.borrow_mut().take();
                window
                    .spawn(cx, async move |cx| {
                        if let Ok(Ok(Some(paths))) = receiver.await {
                            if let Some(cb) = on_change.as_mut() {
                                cx.update(|_window, cx| cb(paths, cx)).ok();
                            }
                        }
                    })
                    .detach();
            },
        );
    div
}

// ── Callback helpers ─────────────────────────────────────────────────

/// Wrap a closure as an `on:change` callback for checkbox/radio.
pub fn bool_change_cb(
    f: impl FnMut(bool, &mut App) + 'static,
) -> Box<dyn FnMut(bool, &mut App)> {
    Box::new(f)
}

/// Wrap a closure as an `on:change` callback for range.
pub fn f64_change_cb(
    f: impl FnMut(f64, &mut App) + 'static,
) -> Box<dyn FnMut(f64, &mut App)> {
    Box::new(f)
}

/// Wrap a closure as an `on:change` callback for file.
pub fn files_cb(
    f: impl FnMut(Vec<std::path::PathBuf>, &mut App) + 'static,
) -> Box<dyn FnMut(Vec<std::path::PathBuf>, &mut App)> {
    Box::new(f)
}

// ── Select ───────────────────────────────────────────────────────────

/// Props for `<select>`.
pub struct SelectProps {
    pub options: Vec<(String, String)>,
    pub value: String,
    pub disabled: bool,
    pub on_change: Option<Box<dyn FnMut(&str, &mut App)>>,
}

/// Wrap a closure as an `on:change` callback for select.
pub fn str_select_change_cb(
    f: impl FnMut(&str, &mut App) + 'static,
) -> Box<dyn FnMut(&str, &mut App)> {
    Box::new(f)
}

/// Render a simple select dropdown. Returns a `Stateful<Div>` so the macro
/// can chain style/class/id uniformly.
pub fn select(props: SelectProps) -> Stateful<gpui::Div> {
    let options = props.options;
    let selected = props.value;
    let disabled = props.disabled;
    let on_change = std::cell::RefCell::new(props.on_change);
    let open = std::cell::Cell::new(false);

    let display_label = options
        .iter()
        .find(|(v, _)| *v == selected)
        .map(|(_, l)| l.clone())
        .unwrap_or_else(|| selected.clone());

    let border_color = if disabled {
        hsla(0.0, 0.0, 0.7, 1.0)
    } else {
        hsla(0.0, 0.0, 0.6, 0.4)
    };

    let _ = open; // TODO: popup dropdown

    gpui::div()
        .id(("select", crate::reactive::next_auto_id()))
        .cursor_pointer()
        .px_2()
        .py_1()
        .min_h(px(28.))
        .border_1()
        .border_color(border_color)
        .rounded(px(4.))
        .bg(gpui::white())
        .text_color(gpui::black())
        .text_size(px(14.))
        .child(SharedString::from(display_label))
        .on_mouse_down(
            MouseButton::Left,
            move |_event: &MouseDownEvent, _window: &mut Window, _cx: &mut App| {
                if disabled {
                    return;
                }
                // Simple cycle-through on click for now
                if let Some(idx) = options.iter().position(|(v, _)| *v == selected) {
                    let next = (idx + 1) % options.len();
                    if let Some((v, _)) = options.get(next) {
                        if let Some(cb) = on_change.borrow_mut().as_mut() {
                            cb(v, _cx);
                        }
                    }
                }
            },
        )
}
