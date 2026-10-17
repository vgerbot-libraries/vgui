//! Tree view with expand/collapse and selection.
//!
//! Callers describe the data as recursive [`TreeNodeData`] nodes. The widget
//! owns the set of expanded node ids in a store and renders one indented row
//! per visible node; branch nodes get a chevron (▶ collapsed / ▼ expanded),
//! leaves get an aligned spacer.

use std::rc::Rc;

use gpui::{px, ClickEvent, InteractiveElement, IntoElement, ParentElement, Styled, Window};
use vgui::{create_store, for_each, next_auto_id, SetStore, Store};
use vgui::prelude::*;

/// One node in a [`tree`]. Children are rendered indented when expanded.
#[derive(Clone)]
pub struct TreeNodeData {
    pub id: u64,
    pub label: String,
    pub children: Vec<TreeNodeData>,
}

impl TreeNodeData {
    /// Leaf node (no children).
    pub fn leaf(id: u64, label: impl Into<String>) -> Self {
        TreeNodeData {
            id,
            label: label.into(),
            children: Vec::new(),
        }
    }

    /// Branch node with the given children.
    pub fn branch(id: u64, label: impl Into<String>, children: Vec<TreeNodeData>) -> Self {
        TreeNodeData {
            id,
            label: label.into(),
            children,
        }
    }

    fn is_branch(&self) -> bool {
        !self.children.is_empty()
    }
}

type OnSelect = Rc<dyn Fn(u64, &mut gpui::App)>;

fn chevron(open: bool) -> &'static str {
    if open { "\u{25BC}" } else { "\u{25B6}" }
}

/// One indented row plus (when expanded) its children.
fn tree_node(
    node: TreeNodeData,
    depth: usize,
    open_read: Store<Vec<u64>>,
    open_write: SetStore<Vec<u64>>,
    selected: u64,
    on_select: OnSelect,
) -> gpui::AnyElement {
    let is_open = open_read.get().contains(&node.id);
    let is_selected = selected == node.id;
    let has_children = node.is_branch();
    let indent = 12.0 * depth as f32;

    // Chevron / leaf spacer, both as AnyElement.
    let marker: gpui::AnyElement = if has_children {
        let id = node.id;
        let open_w = open_write.clone();
        let btn = gpui::div()
            .id(("tree-chevron", next_auto_id()))
            .w(px(16.))
            .h(px(22.))
            .flex()
            .items_center()
            .justify_center()
            .cursor_pointer()
            .on_click(move |_: &ClickEvent, _: &mut Window, cx: &mut gpui::App| {
                open_w.update(cx, |ids| {
                    if let Some(pos) = ids.iter().position(|x| *x == id) {
                        ids.remove(pos);
                    } else {
                        ids.push(id);
                    }
                });
                cx.stop_propagation();
            })
            .child(chevron(is_open));
        css! { color: var(--muted-foreground); font-size: 9px; }
            .apply(btn)
            .into_any_element()
    } else {
        gpui::div()
            .w(px(16.))
            .h(px(22.))
            .into_any_element()
    };

    let row_id = node.id;
    let on_select_row = on_select.clone();
    let row = gpui::div()
        .id(("tree-row", next_auto_id()))
        .flex()
        .flex_row()
        .items_center()
        .h(px(26.))
        .pl(px(indent + 4.))
        .pr(px(8.))
        .cursor_pointer()
        .on_click(move |_: &ClickEvent, _: &mut Window, cx: &mut gpui::App| {
            on_select_row(row_id, cx);
        })
        .child(marker)
        .child(gpui::div().text_size(px(13.)).child(node.label.clone()));

    let row = if is_selected {
        css! {
            background: var(--accent);
            color: var(--accent-foreground);
            border-radius: 4px;
        }
        .apply(row)
    } else {
        css! { color: var(--foreground); }.apply(row)
    };

    let mut column = gpui::div().flex().flex_col().child(row);

    if has_children && is_open {
        for child in node.children {
            column = column.child(tree_node(
                child,
                depth + 1,
                open_read.clone(),
                open_write.clone(),
                selected,
                on_select.clone(),
            ));
        }
    }

    column.into_any_element()
}

/// Render a tree of `nodes` with expand/collapse chevrons and row selection.
///
/// `selected` is the selected node id (use a value no node has for none).
/// `on_select(node_id, cx)` fires when a row is clicked.
pub fn tree(
    nodes: Vec<TreeNodeData>,
    selected: u64,
    on_select: Rc<dyn Fn(u64, &mut gpui::App)>,
) -> gpui::AnyElement {
    let (open_read, open_write) = create_store(Vec::<u64>::new());

    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            background: var(--popover);
            color: var(--popover-foreground);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 4px;
            min-width: 160px;
        }}>
            {for_each(nodes, move |n: TreeNodeData, _| {
                tree_node(
                    n, 0, open_read.clone(), open_write.clone(),
                    selected, on_select.clone(),
                )
            })}
        </div>
    }
    .into_any_element()
}
