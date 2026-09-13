use vgui::prelude::*;

pub struct Separator {
    pub vertical: bool,
}

impl Default for Separator {
    fn default() -> Self {
        Self { vertical: false }
    }
}

impl gpui::IntoElement for Separator {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        if self.vertical {
            view! {
                <div style={css! {
                    width: 1px;
                    height: 100%;
                    background: var(--border);
                }} />
            }
            .into_any_element()
        } else {
            view! {
                <div style={css! {
                    width: 100%;
                    height: 1px;
                    background: var(--border);
                }} />
            }
            .into_any_element()
        }
    }
}

pub fn separator() -> Separator {
    Separator { vertical: false }
}

pub fn separator_vertical() -> Separator {
    Separator { vertical: true }
}
