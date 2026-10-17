//! Programmatic AccessKit layer.
//!
//! The `view!` macro resolves string `role="..."` attributes directly onto
//! elements (see [`crate::aria`]). This module serves code that builds
//! elements imperatively: [`AccessibleNode`] is a typed description of an
//! accessible node's role and states that can be applied to any stateful
//! element, and [`A11yAction`] bridges the subset of `accesskit::Action` an
//! app can respond to (click, focus, expand/collapse, increment/decrement and
//! scroll-into-view) onto gpui's assistive-technology action dispatch.
//!
//! ```ignore
//! use gpui::{div, InteractiveElement, StatefulInteractiveElement};
//! use vgui::a11y::{AccessibleNode, A11yAction};
//!
//! let node = AccessibleNode::button("Submit");
//! node.apply_to(div().id(("submit", vgui::next_auto_id())))
//!     .on_a11y_action(A11yAction::Click, |_window, _cx| {
//!         // activate on assistive-tech click
//!     });
//! ```

use gpui::accesskit::{self, Role, Toggled};
use gpui::{StatefulInteractiveElement, Window, App};

/// The assistive-technology actions vgui exposes a typed handler for.
///
/// Each variant maps to one `accesskit::Action`; use
/// [`accesskit_action`](A11yAction::accesskit_action) for the raw value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum A11yAction {
    Click,
    Focus,
    Blur,
    Expand,
    Collapse,
    Increment,
    Decrement,
    ScrollIntoView,
}

impl A11yAction {
    /// The underlying [`accesskit::Action`].
    pub fn accesskit_action(self) -> accesskit::Action {
        match self {
            A11yAction::Click => accesskit::Action::Click,
            A11yAction::Focus => accesskit::Action::Focus,
            A11yAction::Blur => accesskit::Action::Blur,
            A11yAction::Expand => accesskit::Action::Expand,
            A11yAction::Collapse => accesskit::Action::Collapse,
            A11yAction::Increment => accesskit::Action::Increment,
            A11yAction::Decrement => accesskit::Action::Decrement,
            A11yAction::ScrollIntoView => accesskit::Action::ScrollIntoView,
        }
    }
}

/// A typed, reusable description of an accessible node: its role plus the
/// common states and labels assistive technology reads.
#[derive(Clone, Debug, Default)]
pub struct AccessibleNode {
    role: Role,
    label: Option<String>,
    description: Option<String>,
    toggled: Option<Toggled>,
    expanded: Option<bool>,
    selected: Option<bool>,
    focusable: bool,
}

impl AccessibleNode {
    /// A node with an explicit role.
    pub fn new(role: Role) -> Self {
        Self {
            role,
            ..Default::default()
        }
    }

    /// A `Button` node labelled `label`.
    pub fn button(label: impl Into<String>) -> Self {
        Self::new(Role::Button).label(label)
    }

    /// A `CheckBox`/toggle node with the given pressed state.
    pub fn checkbox(label: impl Into<String>, checked: bool) -> Self {
        Self::new(Role::CheckBox)
            .label(label)
            .toggled(if checked {
                Toggled::True
            } else {
                Toggled::False
            })
            .focusable()
    }

    /// Set the accessible name.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the accessible description (longer help text).
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the toggle state (checkbox / switch).
    pub fn toggled(mut self, toggled: Toggled) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Set the expanded state (disclosure / tree item / combobox).
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    /// Set the selected state (list / tab / gridcell).
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    /// Mark the node focusable by assistive technology / keyboard.
    pub fn focusable(mut self) -> Self {
        self.focusable = true;
        self
    }

    /// The node's role.
    pub fn role(&self) -> Role {
        self.role
    }

    /// Apply this node's role, labels and states to a stateful element.
    pub fn apply_to<E>(&self, el: E) -> E
    where
        E: StatefulInteractiveElement,
    {
        let mut el = el;
        if self.role != Role::GenericContainer {
            el = el.role(self.role);
        }
        if let Some(label) = &self.label {
            el = el.aria_label(label.clone());
        }
        if let Some(description) = &self.description {
            el = el.aria_description(description.clone());
        }
        if let Some(toggled) = self.toggled {
            el = el.aria_toggled(toggled);
        }
        if let Some(expanded) = self.expanded {
            el = el.aria_expanded(expanded);
        }
        if let Some(selected) = self.selected {
            el = el.aria_selected(selected);
        }
        if self.focusable {
            el = el.focusable();
        }
        el
    }
}

/// Register a typed handler for one assistive-technology [`A11yAction`] on a
/// stateful element. The raw `ActionData` is intentionally not exposed: these
/// actions carry no payload.
pub fn on_accessible_action<E, F>(el: E, action: A11yAction, mut handler: F) -> E
where
    E: StatefulInteractiveElement,
    F: FnMut(&mut Window, &mut App) + 'static,
{
    el.on_a11y_action(action.accesskit_action(), move |_data, window, cx| {
        handler(window, cx);
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actions_map_to_accesskit() {
        assert_eq!(A11yAction::Click.accesskit_action(), accesskit::Action::Click);
        assert_eq!(A11yAction::Expand.accesskit_action(), accesskit::Action::Expand);
        assert_eq!(
            A11yAction::ScrollIntoView.accesskit_action(),
            accesskit::Action::ScrollIntoView
        );
        assert_eq!(
            A11yAction::Decrement.accesskit_action(),
            accesskit::Action::Decrement
        );
    }

    #[test]
    fn node_records_states() {
        let node = AccessibleNode::checkbox("Agree", true).expanded(false);
        assert_eq!(node.role(), Role::CheckBox);
        assert_eq!(node.toggled, Some(Toggled::True));
        assert_eq!(node.selected, None);
        assert_eq!(node.expanded, Some(false));
        assert!(node.focusable);
    }

    /// Applying a node with an AccessKit action to an element must draw.
    #[test]
    fn accessible_element_draws() {
        use crate::reactive::next_auto_id;
        use gpui::{div, InteractiveElement, IntoElement, Styled};

        crate::headless::run_headless(
            || {
                let node = AccessibleNode::button("OK");
                let el = node.apply_to(div().id(("ok", next_auto_id())));
                let el = on_accessible_action(el, A11yAction::Click, |_w, _cx| {});
                el.w(gpui::px(40.)).h(gpui::px(40.)).into_any_element()
            },
            |win| {
                win.draw();
            },
        );
    }
}
