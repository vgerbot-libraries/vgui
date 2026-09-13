use vgui::floating_at;
use vgui::for_each;
use vgui::prelude::*;
use vgui::show;

pub struct Popover {
    pub open: bool,
    pub anchor: NodeRef,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Popover {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        show(
            self.open,
            floating_at(
                &self.anchor,
                view! {
                    <div style={css! {
                        background: var(--popover);
                        color: var(--popover-foreground);
                        border-width: 1px;
                        border-style: solid;
                        border-color: var(--border);
                        border-radius: var(--radius);
                        padding: 12px;
                    }}>
                        {for_each(children, |c, _| c)}
                    </div>
                },
            ),
            gpui::Empty,
        )
    }
}
