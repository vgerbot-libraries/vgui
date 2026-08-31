use crate::theme::{themed_box, Mode, MODE};
use vgui::prelude::*;

/// A sub-component living in its own module. It imports the `MODE` marker
/// from `theme.rs`, reads the ancestor provider set in `main.rs`, and nests
/// its own override provider — demonstrating that context resolution is
/// per-render, not per-module.
#[allow(non_snake_case)]
pub fn ThemePanel() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-2">
            {themed_box("panel: inherits root context")}
            <Provider context={MODE} value={Mode::Dark}>
                {themed_box("panel: overridden to dark")}
            </Provider>
        </div>
    }
}
