use crate::components::IconButton;
use crate::icons;
use vgui::prelude::*;

#[vgui_component]
pub fn mode_toggle(
    dark: bool,
    on_toggle: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let glyph = if dark { icons::SUN } else { icons::MOON };
    view! {
        <IconButton glyph={glyph} on:click={on_toggle} />
    }
}
