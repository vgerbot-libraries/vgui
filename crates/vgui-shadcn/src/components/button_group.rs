use vgui::for_each;
use vgui::prelude::*;

/// Horizontally connected group of buttons (segmented control). Buttons are
/// laid edge-to-edge with shared rounding on the first/last items.
#[vgui_component]
pub fn button_group(children: Vec<gpui::AnyElement>) -> impl gpui::IntoElement {
    view! {
        <div role="group" style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}
