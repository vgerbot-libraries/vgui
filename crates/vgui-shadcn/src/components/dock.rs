//! Dockable panels: nested resizable splits and tabbed panels.
//!
//! A [`DockArea`] lays out a tree of [`DockNode`]s over a fixed-size region:
//! leaves are tabbed [`PanelView`] stacks, and internal nodes split their area
//! horizontally or vertically at a draggable divider. Dividers use the same
//! delta-based pointer capture as [`crate::components::resizable`]; the tree
//! shape is fixed after construction, while the split ratios and active tab
//! are reactive and persist across re-renders.
//!
//! Edge panels (left/right/bottom sidebars) are just splits composed around a
//! centre leaf — use [`DockNode::split_horizontal`] / [`DockNode::split_vertical`].
//!
//! ```ignore
//! let area = dock_area(
//!     DockNode::split_horizontal(
//!         DockNode::leaf(vec![PanelView::new("Files", Rc::new(|| files()))]),
//!         DockNode::leaf(vec![PanelView::new("Editor", Rc::new(|| editor()))]),
//!         0.25,
//!     ),
//!     800.,
//!     600.,
//! );
//! let el: AnyElement = dock(area);
//! ```

use std::rc::Rc;

use gpui::{
    div, px, AnyElement, App, FontWeight, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, MouseMoveEvent, MouseUpEvent, ParentElement, StatefulInteractiveElement, Styled,
    Window,
};
use vgui::{create_signal, next_auto_id, __var_color};

const MIN_RATIO: f32 = 0.1;
const HANDLE: f32 = 6.0;
const TAB_H: f32 = 30.0;

/// A body factory for one dock panel.
pub type PanelBody = Rc<dyn Fn() -> AnyElement>;

/// One tabbed panel: a title and a factory producing its contents.
pub struct PanelView {
    /// Tab title.
    pub title: String,
    /// Builds the panel body each render.
    pub body: PanelBody,
}

impl PanelView {
    /// Create a named panel whose contents are produced by `body`.
    pub fn new(title: impl Into<String>, body: PanelBody) -> Rc<Self> {
        Rc::new(Self {
            title: title.into(),
            body,
        })
    }
}

/// A node in the dock layout: a tabbed leaf or a split of two children.
#[derive(Clone)]
pub enum DockNode {
    /// A tabbed panel stack; `active` is the initially selected index.
    Leaf {
        panels: Vec<Rc<PanelView>>,
        active: usize,
    },
    /// A split of `a` then `b`.
    Split {
        /// true = left/right, false = top/bottom.
        horizontal: bool,
        /// Fraction (0..1) of the extent given to `a`.
        ratio: f32,
        a: Rc<DockNode>,
        b: Rc<DockNode>,
    },
}

impl DockNode {
    /// A leaf showing the first of `panels`.
    pub fn leaf(panels: Vec<Rc<PanelView>>) -> Rc<Self> {
        Rc::new(DockNode::Leaf { panels, active: 0 })
    }

    /// A left/right split, `ratio` going to `left`.
    pub fn split_horizontal(left: Rc<DockNode>, right: Rc<DockNode>, ratio: f32) -> Rc<Self> {
        Rc::new(DockNode::Split {
            horizontal: true,
            ratio: ratio.clamp(MIN_RATIO, 1.0 - MIN_RATIO),
            a: left,
            b: right,
        })
    }

    /// A top/bottom split, `ratio` going to `top`.
    pub fn split_vertical(top: Rc<DockNode>, bottom: Rc<DockNode>, ratio: f32) -> Rc<Self> {
        Rc::new(DockNode::Split {
            horizontal: false,
            ratio: ratio.clamp(MIN_RATIO, 1.0 - MIN_RATIO),
            a: top,
            b: bottom,
        })
    }
}

/// A dock layout plus the fixed region it fills.
pub struct DockArea {
    root: Rc<DockNode>,
    width: f32,
    height: f32,
}

/// Build the dock area descriptor.
pub fn dock_area(root: Rc<DockNode>, width: f32, height: f32) -> DockArea {
    DockArea {
        root,
        width,
        height,
    }
}

/// The dock widget (the active renderer of a [`DockArea`]).
pub struct Dock;

/// Render a [`DockArea`] to an interactive element.
pub fn dock(area: DockArea) -> AnyElement {
    render_node(&area.root, 0., 0., area.width, area.height, true)
}

/// Geometry of a split along its main axis. Pure so it can be unit tested.
fn split_geometry(origin: f32, extent: f32, ratio: f32) -> SplitGeometry {
    let first = extent * ratio;
    let handle_at = origin + first;
    let second_origin = handle_at + HANDLE;
    let second = (extent - first - HANDLE).max(0.);
    SplitGeometry {
        first,
        handle_at,
        second_origin,
        second,
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct SplitGeometry {
    first: f32,
    handle_at: f32,
    second_origin: f32,
    second: f32,
}

/// Clamp a dragged ratio into the allowed range.
fn clamp_ratio(ratio: f32) -> f32 {
    ratio.clamp(MIN_RATIO, 1.0 - MIN_RATIO)
}

fn render_node(
    node: &Rc<DockNode>,
    origin_x: f32,
    origin_y: f32,
    width: f32,
    height: f32,
    root: bool,
) -> AnyElement {
    match node.as_ref() {
        DockNode::Leaf { panels, active } => render_leaf(panels, *active, width, height),
        DockNode::Split {
            horizontal,
            ratio,
            a,
            b,
        } => {
            // Per-node reactive ratio, seeded on first render and retained in
            // the scope slot on later renders (traversal order is stable
            // because the tree shape is fixed).
            let (ratio_sig, set_ratio) = create_signal(clamp_ratio(*ratio));
            let (anchor, set_anchor) = create_signal(Option::<f32>::None);

            let extent = if *horizontal { width } else { height };
            let origin = if *horizontal { origin_x } else { origin_y };
            let geo = split_geometry(origin, extent, ratio_sig.get());

            let on_down_ratio = ratio_sig.clone();
            let on_down_anchor = set_anchor.clone();
            let move_anchor_read = anchor.clone();
            let move_ratio_read = ratio_sig.clone();
            let move_ratio_write = set_ratio.clone();
            let move_anchor_write = set_anchor.clone();
            let up_anchor = set_anchor.clone();
            let horiz = *horizontal;

            let mut split_el = div()
                .id(("dock-split", next_auto_id()))
                .relative()
                .flex()
                .w(px(width))
                .h(px(height))
                .on_mouse_down(
                    MouseButton::Left,
                    move |ev: &MouseDownEvent, _w: &mut Window, cx: &mut App| {
                        let p = if horiz {
                            f32::from(ev.position.x)
                        } else {
                            f32::from(ev.position.y)
                        };
                        if (p - geo.handle_at).abs() <= HANDLE {
                            on_down_anchor.set(cx, Some(p));
                            let _ = on_down_ratio;
                        }
                    },
                )
                .on_mouse_move(move |ev: &MouseMoveEvent, _w: &mut Window, cx: &mut App| {
                    if ev.pressed_button != Some(MouseButton::Left) {
                        return;
                    }
                    let Some(last) = move_anchor_read.get() else {
                        return;
                    };
                    let p = if horiz {
                        f32::from(ev.position.x)
                    } else {
                        f32::from(ev.position.y)
                    };
                    let current_extent = extent.max(1.);
                    let next = clamp_ratio(move_ratio_read.get() + (p - last) / current_extent);
                    move_ratio_write.set(cx, next);
                    move_anchor_write.set(cx, Some(p));
                })
                .on_mouse_up(
                    MouseButton::Left,
                    move |_ev: &MouseUpEvent, _w: &mut Window, cx: &mut App| {
                        up_anchor.set(cx, None);
                    },
                );
            split_el = if *horizontal {
                split_el.flex_row()
            } else {
                split_el.flex_col()
            };
            if root {
                split_el = split_el.border_1().border_color(__var_color(
                    "--border",
                    Some(gpui::hsla(0., 0., 0.85, 1.)),
                ));
            }

            let (a_w, a_h) = if *horizontal {
                (geo.first, height)
            } else {
                (width, geo.first)
            };
            split_el = split_el.child(render_node(a, origin_x, origin_y, a_w, a_h, false));

            // Visual divider (pointer logic lives on the split root).
            let mut handle_el = div().flex_shrink_0();
            handle_el = if *horizontal {
                handle_el.w(px(HANDLE)).h_full().cursor_col_resize()
            } else {
                handle_el.h(px(HANDLE)).w_full().cursor_row_resize()
            };
            handle_el = vgui::css! {
                background: var(--border);
            }
            .apply(handle_el);
            split_el = split_el.child(handle_el);

            let (b_x, b_y, b_w, b_h) = if *horizontal {
                (geo.second_origin, origin_y, geo.second, height)
            } else {
                (origin_x, geo.second_origin, width, geo.second)
            };
            split_el = split_el.child(render_node(b, b_x, b_y, b_w, b_h, false));

            split_el.into_any_element()
        }
    }
}

fn render_leaf(panels: &[Rc<PanelView>], initial_active: usize, width: f32, height: f32) -> AnyElement {
    let (active, set_active) = create_signal(initial_active.min(panels.len().saturating_sub(1)));

    let border = __var_color("--border", Some(gpui::hsla(0., 0., 0.85, 1.)));
    let fg = __var_color("--foreground", Some(gpui::hsla(0., 0., 0.1, 1.)));
    let muted = __var_color("--muted-foreground", Some(gpui::hsla(0., 0., 0.4, 1.)));

    let mut leaf = div()
        .relative()
        .flex()
        .flex_col()
        .w(px(width))
        .h(px(height))
        .overflow_hidden();

    // Tab bar: fixed-width tabs so each is a reliable hit target.
    let mut tab_bar = div().flex().flex_row().h(px(TAB_H)).w_full().flex_shrink_0();
    for (i, panel) in panels.iter().enumerate() {
        let is_active = i == active.get();
        let setter = set_active.clone();
        let (bg, color, weight) = if is_active {
            (
                __var_color("--background", Some(gpui::hsla(0., 0., 1., 1.))),
                fg,
                FontWeight::SEMIBOLD,
            )
        } else {
            (
                __var_color("--muted", Some(gpui::hsla(0., 0., 0.95, 1.))),
                muted,
                FontWeight::NORMAL,
            )
        };
        let tab = div()
            .id(("dock-tab", next_auto_id()))
            .w(px(108.))
            .h(px(TAB_H))
            .flex()
            .items_center()
            .px_2()
            .bg(bg)
            .text_color(color)
            .font_weight(weight)
            .border_b_1()
            .border_color(if is_active {
                __var_color("--primary", Some(gpui::hsla(0.6, 0.6, 0.45, 1.)))
            } else {
                border
            })
            .on_click(move |_ev, _w: &mut Window, cx: &mut App| setter.set(cx, i))
            .child(panel.title.clone());
        tab_bar = tab_bar.child(tab);
    }
    leaf = leaf.child(tab_bar);

    // Active panel body.
    let body_h = (height - TAB_H).max(0.);
    if let Some(panel) = panels.get(active.get()) {
        let body = div()
            .flex_1()
            .w_full()
            .h(px(body_h))
            .overflow_hidden()
            .child((panel.body)());
        leaf = leaf.child(body);
    }

    leaf.into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_geometry_divides_extent() {
        let g = split_geometry(0., 100., 0.25);
        assert_eq!(g.first, 25.);
        assert_eq!(g.handle_at, 25.);
        assert_eq!(g.second_origin, 31.);
        assert_eq!(g.second, 69.);
    }

    #[test]
    fn split_geometry_respects_offset_origin() {
        let g = split_geometry(80., 200., 0.5);
        assert_eq!(g.handle_at, 180.);
        assert_eq!(g.second_origin, 186.);
    }

    #[test]
    fn ratio_never_collapses_a_pane() {
        assert_eq!(clamp_ratio(0.0), MIN_RATIO);
        assert_eq!(clamp_ratio(1.0), 1.0 - MIN_RATIO);
        assert_eq!(clamp_ratio(0.5), 0.5);
    }

    #[test]
    fn node_builders_clamp_ratio() {
        let leaf = DockNode::leaf(vec![]);
        let split = DockNode::split_horizontal(leaf.clone(), leaf.clone(), 9.0);
        match split.as_ref() {
            DockNode::Split { ratio, .. } => assert_eq!(*ratio, 1.0 - MIN_RATIO),
            _ => panic!("expected split"),
        }
    }
}
