//! Settings / preferences panels.
//!
//! A [`SettingPanel`] is a titled, bordered stack of [`SettingRow`]s. Rows
//! carry a label, optional description, and a control (switch or choose-one
//! buttons). Values are read/written through a reactive
//! [`Store<Settings>`](vgui::Store), so toggling a control updates the shared
//! settings object and every other reader.

use gpui::{
    App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window,
};
use vgui::{create_store, Store};


/// Keyed bag of preference values.
#[derive(Clone, Default)]
pub struct Settings {
    /// Boolean toggles keyed by row id.
    pub toggles: Vec<(String, bool)>,
    /// Currently selected option index per choice row id.
    pub choices: Vec<(String, usize)>,
}

impl Settings {
    /// Empty settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Read a toggle, defaulting to `false`.
    pub fn toggle(&self, id: &str) -> bool {
        self.toggles
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| *v)
            .unwrap_or(false)
    }

    /// Read a choice index, defaulting to `0`.
    pub fn choice(&self, id: &str) -> usize {
        self.choices
            .iter()
            .find(|(k, _)| k == id)
            .map(|(_, v)| *v)
            .unwrap_or(0)
    }

    fn has_toggle(&self, id: &str) -> bool {
        self.toggles.iter().any(|(k, _)| k == id)
    }

    fn set_toggle(&mut self, id: &str, value: bool) {
        if let Some(slot) = self.toggles.iter_mut().find(|(k, _)| k == id) {
            slot.1 = value;
        } else {
            self.toggles.push((id.to_string(), value));
        }
    }

    fn set_choice(&mut self, id: &str, value: usize) {
        if let Some(slot) = self.choices.iter_mut().find(|(k, _)| k == id) {
            slot.1 = value;
        } else {
            self.choices.push((id.to_string(), value));
        }
    }
}

/// One labelled preference row.
pub enum SettingRow {
    /// A boolean toggle row.
    Toggle {
        id: String,
        label: String,
        description: Option<String>,
        default: bool,
    },
    /// A choose-one row rendered as option buttons.
    Choice {
        id: String,
        label: String,
        options: Vec<String>,
        default: usize,
    },
}

impl SettingRow {
    /// A switch-backed row.
    pub fn toggle(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self::Toggle {
            id: id.into(),
            label: label.into(),
            description: None,
            default: false,
        }
    }

    /// A choose-one row.
    pub fn choice(
        id: impl Into<String>,
        label: impl Into<String>,
        options: Vec<String>,
    ) -> Self {
        Self::Choice {
            id: id.into(),
            label: label.into(),
            options,
            default: 0,
        }
    }

    /// Attach explanatory text (toggle rows only).
    pub fn description(mut self, text: impl Into<String>) -> Self {
        if let SettingRow::Toggle { description, .. } = &mut self {
            *description = Some(text.into());
        }
        self
    }
}

/// A titled settings panel configuration.
pub struct SettingPanel {
    title: String,
    rows: Vec<SettingRow>,
    initial: Settings,
}

impl SettingPanel {
    /// Create a panel with an initial settings snapshot.
    pub fn new(title: impl Into<String>, initial: Settings) -> Self {
        Self {
            title: title.into(),
            rows: Vec::new(),
            initial,
        }
    }

    /// Append a preference row.
    pub fn row(mut self, row: SettingRow) -> Self {
        self.rows.push(row);
        self
    }
}

/// Render the settings panel, returning the element and the backing reactive
/// store so callers can read current preferences on save.
pub fn settings_panel(panel: SettingPanel) -> (gpui::AnyElement, Store<Settings>) {
    let SettingPanel {
        title,
        rows,
        initial,
    } = panel;
    let (store, set_store): (Store<Settings>, vgui::SetStore<Settings>) = create_store(initial);
    let snapshot = store.get();

    let rendered: Vec<gpui::AnyElement> = rows
        .into_iter()
        .map(|row| render_row(row, &store, &set_store, &snapshot))
        .collect();

    let mut body = vgui::css! { display: flex; flex-direction: column; }.apply(gpui::div());
    for row_el in rendered {
        body = body.child(row_el);
    }



    let header = vgui::css!{
        padding: 10px 12px; border-bottom-width: 1px;
        border-style: solid; border-color: var(--border);
        color: var(--foreground); font-size: 14px; font-weight: 600;
    }
    .apply(gpui::div())
    .child(title);

    let element = vgui::css!{
        display: flex; flex-direction: column;
        border-width: 1px; border-style: solid; border-color: var(--border);
        border-radius: var(--radius); background: var(--card);
        min-width: 300px;
    }
    .apply(gpui::div())
    .child(header)
    .child(body)
    .into_any_element();

    (element, store)
}

fn render_row(
    row: SettingRow,
    store: &Store<Settings>,
    set_store: &vgui::SetStore<Settings>,
    snapshot: &Settings,
) -> gpui::AnyElement {
    match row {
        SettingRow::Toggle {
            id,
            label,
            description,
            default,
        } => {
            let on = if snapshot.has_toggle(&id) {
                snapshot.toggle(&id)
            } else {
                default
            };
            let read_store = store.clone();
            let write_store = set_store.clone();
            let pill_style = if on {
                vgui::css!{
                    width: 34px; height: 20px; border-width: 0px; border-radius: 999px;
                    background: var(--primary); display: flex; flex-direction: row;
                    align-items: center; padding-left: 2px; padding-right: 2px;
                    justify-content: flex-end; flex-shrink: 0;
                }
            } else {
                vgui::css!{
                    width: 34px; height: 20px; border-width: 0px; border-radius: 999px;
                    background: var(--input); display: flex; flex-direction: row;
                    align-items: center; padding-left: 2px; padding-right: 2px;
                    justify-content: flex-start; flex-shrink: 0;
                }
            };
            let knob = vgui::css!{
                width: 16px; height: 16px; border-radius: 999px; background: #ffffff;
            }.apply(gpui::div());
            let pill = pill_style.apply(
                gpui::div()
                    .id(("setting-toggle", vgui::next_auto_id()))
                    .w(gpui::px(34.))
                    .h(gpui::px(20.))
                    .cursor_pointer()
                    .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                        let now = read_store.get().toggle(&id);
                        write_store.update(cx, |s| s.set_toggle(&id, !now));
                    }),
            )
            .child(knob)
            .into_any_element();
            let label_el = vgui::css!{
                color: var(--foreground); font-size: 13px;
            }
            .apply(gpui::div())
            .child(label);
            let mut text_col = vgui::css!{
                display: flex; flex-direction: column; gap: 2px;
            }
            .apply(gpui::div())
            .child(label_el);
            if let Some(d) = description {
                let d_el = vgui::css!{
                    color: var(--muted-foreground); font-size: 12px;
                }.apply(gpui::div()).child(d);
                text_col = text_col.child(d_el);
            }
            vgui::css!{
                display: flex; flex-direction: row; align-items: center;
                gap: 12px; padding: 10px 12px;
                border-bottom-width: 1px; border-style: solid; border-color: var(--border);
            }
            .apply(gpui::div())
            .child(text_col.flex_1())
            .child(pill)
            .into_any_element()
        }
        SettingRow::Choice {
            id,
            label,
            options,
            default,
        } => {
            let current = if snapshot.choices.iter().any(|(k, _)| k == &id) {
                snapshot.choice(&id)
            } else {
                default
            };
            let buttons: Vec<gpui::AnyElement> = options
                .iter()
                .enumerate()
                .map(|(i, opt)| {
                    let write_store = set_store.clone();
                    let choice_id = id.clone();
                    let selected = i == current;
                    let btn_style = if selected {
                        vgui::css!{
                            height: 26px; padding-left: 10px; padding-right: 10px;
                            border-width: 1px; border-style: solid; border-color: var(--primary);
                            border-radius: var(--radius); font-size: 12px;
                            background: var(--primary); color: var(--primary-foreground);
                        }
                    } else {
                        vgui::css!{
                            height: 26px; padding-left: 10px; padding-right: 10px;
                            border-width: 1px; border-style: solid; border-color: var(--border);
                            border-radius: var(--radius); font-size: 12px;
                            background: var(--background); color: var(--foreground);
                        }
                    };
                    btn_style.apply(
                        gpui::div()
                            .id(("setting-choice", vgui::next_auto_id()))
                            .cursor_pointer()
                            .on_click(move |_: &gpui::ClickEvent, _w, cx: &mut gpui::App| {
                                write_store.update(cx, |s| s.set_choice(&choice_id, i));
                            }),
                    )
                    .child(opt.clone())
                    .into_any_element()
                })
                .collect();
            let mut options_row =
                vgui::css!{ display: flex; flex-direction: row; gap: 6px; flex-wrap: wrap; }
                    .apply(gpui::div());
            for b in buttons {
                options_row = options_row.child(b);
            }
            let label_el = vgui::css!{ color: var(--foreground); font-size: 13px; }
                .apply(gpui::div()).child(label);
            vgui::css!{
                display: flex; flex-direction: column; gap: 6px; padding: 10px 12px;
            }
            .apply(gpui::div())
            .child(label_el)
            .child(options_row)
            .into_any_element()
        }
    }
}
