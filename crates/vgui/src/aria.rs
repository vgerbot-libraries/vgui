//! ARIA helper functions for the `view!` macro.
//!
//! These functions map string values (from `role="button"` or `aria:haspopup="menu"`)
//! to the corresponding accesskit enum variants.

use gpui::accesskit::{Role, Toggled};

/// Resolve a string role name to an accesskit::Role.
/// Called by macro-emitted code: `.role(::vgui::__resolve_aria_role("button"))`.
#[doc(hidden)]
pub fn __resolve_aria_role(s: &str) -> Role {
    match s {
        "button" => Role::Button,
        "link" => Role::Link,
        "checkbox" => Role::CheckBox,
        "radio" => Role::RadioButton,
        "switch" => Role::Switch,
        "textbox" => Role::TextInput,
        "searchbox" => Role::SearchInput,
        "slider" => Role::Slider,
        "spinbutton" => Role::SpinButton,
        "combobox" | "listbox" => Role::ComboBox,
        "menu" => Role::Menu,
        "menubar" => Role::MenuBar,
        "menuitem" => Role::MenuItem,
        "menuitemcheckbox" => Role::MenuItemCheckBox,
        "menuitemradio" => Role::MenuItemRadio,
        "tab" => Role::Tab,
        "tablist" => Role::TabList,
        "tabpanel" => Role::TabPanel,
        "tree" => Role::Tree,
        "treeitem" => Role::TreeItem,
        "toolbar" => Role::Toolbar,
        "list" => Role::List,
        "listitem" => Role::ListItem,
        "grid" => Role::Grid,
        "gridcell" => Role::Cell,
        "row" => Role::Row,
        "rowheader" => Role::RowHeader,
        "columnheader" => Role::ColumnHeader,
        "table" => Role::Table,
        "heading" => Role::Heading,
        "img" | "image" => Role::Image,
        "separator" => Role::Splitter,
        "progressbar" => Role::ProgressIndicator,
        "scrollbar" => Role::ScrollBar,
        "alert" => Role::Alert,
        "alertdialog" => Role::AlertDialog,
        "dialog" => Role::Dialog,
        "status" => Role::Status,
        "marquee" => Role::Marquee,
        "timer" => Role::Timer,
        "tooltip" => Role::Tooltip,
        "application" => Role::Application,
        "document" => Role::Document,
        "group" => Role::Group,
        "region" => Role::Region,
        "navigation" => Role::Navigation,
        "article" => Role::Article,
        "main" => Role::Main,
        "header" => Role::Header,
        "footer" => Role::Footer,
        "form" => Role::Form,
        "complementary" => Role::Complementary,
        "contentinfo" => Role::ContentInfo,
        "figure" => Role::Figure,
        "math" => Role::Math,
        "note" => Role::Note,
        "definition" => Role::Definition,
        "term" => Role::Term,
        "log" => Role::Log,
        "banner" => Role::Banner,
        "directory" => Role::Unknown,
        "none" | "presentation" => Role::GenericContainer,
        _ => Role::GenericContainer,
    }
}

/// Resolve a string to accesskit::Toggled.
#[doc(hidden)]
pub fn __resolve_toggled(s: &str) -> Toggled {
    match s {
        "true" => Toggled::True,
        "mixed" => Toggled::Mixed,
        _ => Toggled::False,
    }
}

/// Resolve a string to accesskit::AutoComplete.
#[doc(hidden)]
pub fn __resolve_autocomplete(s: &str) -> gpui::accesskit::AutoComplete {
    use gpui::accesskit::AutoComplete;
    match s {
        "inline" => AutoComplete::Inline,
        "list" => AutoComplete::List,
        "both" => AutoComplete::Both,
        _ => AutoComplete::Inline,
    }
}

/// Resolve a string to accesskit::HasPopup.
#[doc(hidden)]
pub fn __resolve_has_popup(s: &str) -> gpui::accesskit::HasPopup {
    use gpui::accesskit::HasPopup;
    match s {
        "menu" => HasPopup::Menu,
        "listbox" => HasPopup::Listbox,
        "tree" => HasPopup::Tree,
        "grid" => HasPopup::Grid,
        "dialog" => HasPopup::Dialog,
        "true" => HasPopup::Menu,
        _ => HasPopup::Menu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_common_roles() {
        assert_eq!(__resolve_aria_role("button"), Role::Button);
        assert_eq!(__resolve_aria_role("link"), Role::Link);
        assert_eq!(__resolve_aria_role("checkbox"), Role::CheckBox);
        assert_eq!(__resolve_aria_role("navigation"), Role::Navigation);
    }

    #[test]
    fn resolve_unknown_role_defaults_to_generic() {
        assert_eq!(__resolve_aria_role("nonexistent"), Role::GenericContainer);
    }

    #[test]
    fn resolve_toggled() {
        assert_eq!(__resolve_toggled("false"), Toggled::False);
        assert_eq!(__resolve_toggled("mixed"), Toggled::Mixed);
    }
}
