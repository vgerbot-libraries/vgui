//! Sortable, selectable data table.
//!
//! [`DataTable`] renders columns of string cells with a sticky-style header
//! row. Clicking a sortable header cycles ascending → descending → unsorted;
//! clicking a body row toggles its selection. Ordering and selection live in a
//! reactive [`Store`] that is returned alongside the element so callers can
//! read the current sort and selection. Rows are laid out with explicit
//! column widths, so arbitrary content stays aligned.

use gpui::{
    px, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, Window,
};
use vgui::{create_store, next_auto_id, Store};

/// One column definition.
#[derive(Clone)]
pub struct TableColumn {
    /// Header title.
    pub title: String,
    /// Fixed column width in px.
    pub width: f32,
    /// Whether clicking the header sorts by this column.
    pub sortable: bool,
}

impl TableColumn {
    /// A non-sortable column.
    pub fn new(title: impl Into<String>, width: f32) -> Self {
        Self {
            title: title.into(),
            width,
            sortable: false,
        }
    }

    /// Allow header-click sorting on this column.
    pub fn sortable(mut self) -> Self {
        self.sortable = true;
        self
    }
}

/// Reactive table state observable by the host.
#[derive(Clone, Default)]
pub struct TableState {
    /// Column currently sorted, if any.
    pub sort_col: Option<usize>,
    /// Sort direction (`true` = ascending).
    pub sort_asc: bool,
    /// Selected row indices (in the original input order).
    pub selected: Vec<usize>,
}

impl TableState {
    fn is_selected(&self, row: usize) -> bool {
        self.selected.contains(&row)
    }

    fn toggle(&mut self, row: usize) {
        if let Some(slot) = self.selected.iter().position(|r| *r == row) {
            self.selected.remove(slot);
        } else {
            self.selected.push(row);
        }
    }
}

/// A data-grid configuration.
pub struct DataTable {
    columns: Vec<TableColumn>,
    rows: Vec<Vec<String>>,
}

impl DataTable {
    /// Start a table with the given columns and row data.
    ///
    /// Each row must have at least as many cells as there are columns.
    pub fn new(columns: Vec<TableColumn>, rows: Vec<Vec<String>>) -> Self {
        Self { columns, rows }
    }
}

/// Render the table, returning the element and the backing reactive
/// [`Store`] for reading sort/selection state.
pub fn table(data: DataTable) -> (gpui::AnyElement, Store<TableState>) {
    let DataTable { columns, rows } = data;
    let (store, set_store): (Store<TableState>, vgui::SetStore<TableState>) =
        create_store(TableState::default());
    let snap = store.get();

    // Compute the displayed order from the current sort state.
    let mut order: Vec<usize> = (0..rows.len()).collect();
    if let Some(col) = snap.sort_col {
        let asc = snap.sort_asc;
        order.sort_by(|&a, &b| {
            let va = rows.get(a).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
            let vb = rows.get(b).and_then(|r| r.get(col)).map(String::as_str).unwrap_or("");
            let cmp = va.cmp(vb);
            if asc { cmp } else { cmp.reverse() }
        });
    }

    // ── Header ──────────────────────────────────────────────────────────
    let mut header_cells = gpui::div().flex().flex_row();
    for (ci, col) in columns.iter().enumerate() {
        let set_store = set_store.clone();
        let sortable = col.sortable;
        let indicator = match snap.sort_col {
            Some(sc) if sc == ci => if snap.sort_asc { " \u{25B2}" } else { " \u{25BC}" },
            _ => "",
        };
        let title = format!("{}{}", col.title, indicator);
        let cell = vgui::css! {
            height: 32px; padding-left: 10px; padding-right: 10px;
            border-bottom-width: 1px; border-style: solid; border-color: var(--border);
            color: var(--foreground); font-size: 12px; font-weight: 600;
            display: flex; flex-direction: row; align-items: center; flex-shrink: 0;
        }
        .apply(gpui::div().w(px(col.width)).h(px(32.)));
        let cell = if sortable {
            cell.id(("table-head", next_auto_id())).cursor_pointer().on_click(
                move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                    set_store.update(cx, |s| {
                        match s.sort_col {
                            Some(current) if current == ci => {
                                if s.sort_asc {
                                    s.sort_asc = false;
                                } else {
                                    s.sort_col = None;
                                    s.sort_asc = true;
                                }
                            }
                            _ => {
                                s.sort_col = Some(ci);
                                s.sort_asc = true;
                            }
                        }
                    });
                },
            )
        } else {
            cell.id(("table-head-static", next_auto_id()))
        };
        header_cells = header_cells.child(cell.child(title));
    }
    let header = vgui::css! {
        display: flex; flex-direction: row; background: var(--muted); flex-shrink: 0;
    }
    .apply(gpui::div())
    .child(header_cells);

    // ── Body (fold rows with .child so every row keeps hit testing) ─────
    let mut body = vgui::css! { display: flex; flex-direction: column; }.apply(gpui::div());
    for row_idx in order {
        let set_store = set_store.clone();
        let selected = snap.is_selected(row_idx);
        let mut cells = gpui::div().flex().flex_row();
        for (ci, col) in columns.iter().enumerate() {
            let text = rows
                .get(row_idx)
                .and_then(|r| r.get(ci))
                .cloned()
                .unwrap_or_default();
            let cell = vgui::css! {
                padding-left: 10px; padding-right: 10px;
                color: var(--foreground); font-size: 12px;
                display: flex; flex-direction: row; align-items: center; flex-shrink: 0;
            }
            .apply(gpui::div().w(px(col.width)).h(px(30.)))
            .child(text);
            cells = cells.child(cell);
        }
        let row_bg = if selected {
            vgui::css! {
                display: flex; flex-direction: row; flex-shrink: 0;
                background: var(--accent);
            }
        } else {
            vgui::css! {
                display: flex; flex-direction: row; flex-shrink: 0;
                background: var(--card);
            }
        };
        let row = row_bg
            .apply(gpui::div().w(px(columns.iter().map(|c| c.width).sum::<f32>())).h(px(30.)))
            .id(("table-row", next_auto_id()))
            .cursor_pointer()
            .on_click(move |_: &ClickEvent, _w: &mut Window, cx: &mut App| {
                set_store.update(cx, |s| s.toggle(row_idx));
            })
            .child(cells);
        body = body.child(row);
    }

    let element = vgui::css! {
        display: flex; flex-direction: column;
        border-width: 1px; border-style: solid; border-color: var(--border);
        border-radius: var(--radius); background: var(--card); overflow: hidden;
    }
    .apply(gpui::div())
    .child(header)
    .child(body)
    .into_any_element();

    (element, store)
}
