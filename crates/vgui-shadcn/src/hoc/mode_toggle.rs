use crate::components::icon_button;
use crate::icons;

pub struct ModeToggle {
    pub dark: bool,
    pub on_toggle: Box<dyn Fn(&mut gpui::App) + 'static>,
}

impl gpui::IntoElement for ModeToggle {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let glyph = if self.dark { icons::SUN } else { icons::MOON };
        icon_button(glyph, self.on_toggle).into_any_element()
    }
}

pub fn mode_toggle(dark: bool, on_toggle: impl Fn(&mut gpui::App) + 'static) -> ModeToggle {
    ModeToggle {
        dark,
        on_toggle: Box::new(on_toggle),
    }
}
