use vgui::floating_at;
use vgui::prelude::*;
use vgui::show;

pub struct Tooltip {
    pub open: bool,
    pub anchor: NodeRef,
    pub text: String,
}

impl gpui::IntoElement for Tooltip {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        show(
            self.open,
            floating_at(
                &self.anchor,
                view! {
                    <div style={css! {
                        background: var(--foreground);
                        color: var(--background);
                        border-radius: 6px;
                        padding: 4px 8px;
                        font-size: 12px;
                    }}>
                        {self.text}
                    </div>
                },
            ),
            gpui::Empty,
        )
    }
}
