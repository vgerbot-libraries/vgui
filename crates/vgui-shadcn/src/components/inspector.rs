//! Runtime UI / component inspector.
//!
//! [`Inspector`] renders a live tree of [`InspectNode`]s (component name, kind
//! badge, optional state value) with collapsible branches and a live filter
//! box. Expand/collapse and filtering are reactive.
use vgui::{click, create_store, for_each, input_cb, SetStore, Store};
use vgui::prelude::*;

/// One node in the inspected component tree.
#[derive(Clone, Default)]
pub struct InspectNode {
    /// Display name (component / element type).
    pub name: String,
    /// Short kind badge, e.g. "div", "signal", "view".
    pub kind: String,
    /// Optional live state summary, e.g. `open = true`.
    pub state: Option<String>,
    /// Nested children.
    pub children: Vec<InspectNode>,
}

impl InspectNode {
    /// A leaf node with a name and kind.
    pub fn new(name: impl Into<String>, kind: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: kind.into(),
            state: None,
            children: Vec::new(),
        }
    }

    /// Attach a live-state summary.
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Append a child node.
    pub fn child(mut self, child: InspectNode) -> Self {
        self.children.push(child);
        self
    }

    fn matches(&self, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        self.name.to_lowercase().contains(&needle.to_lowercase())
            || self
                .children
                .iter()
                .any(|c| c.matches(needle))
    }
}

/// A configuration describing the tree to inspect.
pub struct Inspector {
    title: String,
    roots: Vec<InspectNode>,
}

impl Inspector {
    /// Create an inspector with a panel title and root nodes.
    pub fn new(title: impl Into<String>, roots: Vec<InspectNode>) -> Self {
        Self {
            title: title.into(),
            roots,
        }
    }
}

#[derive(Clone)]
struct InspectorState {
    expanded: Vec<String>,
    filter: String,
}

/// Render the interactive inspector panel.
pub fn inspector(inspector: Inspector) -> gpui::AnyElement {
    let Inspector { title, roots } = inspector;
    // Root branches start expanded.
    let initial_expanded: Vec<String> = roots
        .iter()
        .enumerate()
        .map(|(i, _)| format!("{}", i))
        .collect();
    let (state, set_state): (Store<InspectorState>, vgui::SetStore<InspectorState>) =
        create_store(InspectorState {
            expanded: initial_expanded,
            filter: String::new(),
        });

    let snap = state.get();
    let filter = snap.filter.clone();

    let rows: Vec<gpui::AnyElement> = roots
        .iter()
        .enumerate()
        .flat_map(|(i, node)| {
            render_node(node, &format!("{}", i), 0, &state, &set_state, &filter)
        })
        .collect();

    let list_set = set_state.clone();
    vgui::view! {
        <div style={vgui::css!{
            display: flex; flex-direction: column;
            border-width: 1px; border-style: solid; border-color: var(--border);
            border-radius: var(--radius); background: var(--card);
            min-width: 280px; font-size: 12px;
        }}>
            <div style={vgui::css!{
                padding: 8px 10px; border-bottom-width: 1px; border-style: solid;
                border-color: var(--border); color: var(--foreground); font-weight: 600;
            }}>{title}</div>
            <input
                value={filter.clone()}
                placeholder={"Filter nodes"}
                style={vgui::css!{
                    height: 28px; margin: 8px; padding-left: 8px; padding-right: 8px;
                    border-width: 1px; border-style: solid; border-color: var(--border);
                    border-radius: var(--radius); background: var(--background);
                    color: var(--foreground);
                }}
                on:input={input_cb(move |raw, cx| {
                    list_set.update(cx, |s| s.filter = raw.to_string());
                })}
            />
            <div style={vgui::css!{ display: flex; flex-direction: column; padding-bottom: 6px; }}>
                {for_each(rows.into_iter().enumerate(), move |(i, el), _key| {
                    let _ = i;
                    el
                })}
            </div>
        </div>
    }
    .into_any_element()
}

fn is_expanded(state: &Store<InspectorState>, path: &str) -> bool {
    state.get().expanded.iter().any(|p| p == path)
}

fn render_node(
    node: &InspectNode,
    path: &str,
    depth: usize,
    state: &Store<InspectorState>,
    set_state: &SetStore<InspectorState>,
    filter: &str,
) -> Vec<gpui::AnyElement> {
    if !node.matches(filter) {
        return Vec::new();
    }
    // When a filter is active, force expansion so matches stay visible.
    let expanded = !filter.is_empty() || is_expanded(state, path);
    let has_children = !node.children.is_empty();

    let indent = depth * 12;
    let chevron = if has_children {
        if expanded {
            "▾".to_string()
        } else {
            "▸".to_string()
        }
    } else {
        " ".to_string()
    };

    let toggle_set = set_state.clone();
    let toggle_path = path.to_string();
    let label = node.name.clone();
    let kind = node.kind.clone();
    let state_text = node.state.clone();

    let spacer = gpui::div()
        .w(gpui::px(indent as f32))
        .h(gpui::px(1.))
        .into_any_element();
    let row = vgui::view! {
        <div style={vgui::css!{
            display: flex; flex-direction: row; align-items: center; gap: 6px;
            padding-top: 3px; padding-bottom: 3px; padding-right: 10px;
            color: var(--foreground);
        }}>
            {spacer}
            <span style={vgui::css!{
                display: flex; flex-direction: row; width: 14px;
                color: var(--muted-foreground); cursor: pointer;
            }}
                on:click={click(move |cx| {
                    let p = toggle_path.clone();
                    toggle_set.update(cx, |s| {
                        if let Some(pos) = s.expanded.iter().position(|x| x == &p) {
                            s.expanded.remove(pos);
                        } else {
                            s.expanded.push(p.clone());
                        }
                    });
                })}
            >{chevron}</span>
            <span style={vgui::css!{ color: var(--foreground); }}>{label}</span>
            <span style={vgui::css!{
                padding-left: 5px; padding-right: 5px; border-radius: 999px;
                background: var(--muted); color: var(--muted-foreground); font-size: 10px;
            }}>{kind}</span>
            {if let Some(st) = state_text {
                vgui::view!{ <span style={vgui::css!{ color: var(--muted-foreground); }}>{st}</span> }
            } else {
                vgui::view!{ <span></span> }
            }}
        </div>
    }
    .into_any_element();

    let mut out = vec![row];
    if has_children && expanded {
        for (i, child) in node.children.iter().enumerate() {
            let child_path = format!("{}-{}", path, i);
            out.extend(render_node(child, &child_path, depth + 1, state, set_state, filter));
        }
    }
    out
}
