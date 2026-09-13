use vgui::for_each;
use vgui::prelude::*;
use vgui::show;

pub struct Collapsible {
    pub open: bool,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Collapsible {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <div style={css! { display: flex; flex-direction: column; gap: 4px; }}>
                {for_each(children, |c, _| c)}
            </div>
        }
        .into_any_element()
    }
}

pub struct CollapsibleContent {
    pub open: bool,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for CollapsibleContent {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        show(
            self.open,
            view! {
                <div style={css! { display: flex; flex-direction: column; }}>
                    {for_each(children, |c, _| c)}
                </div>
            },
            gpui::Empty,
        )
    }
}
