use vgui::dialog;
use vgui::for_each;
use vgui::prelude::*;

pub struct Dialog {
    pub open: bool,
    pub on_close: Box<dyn Fn(&mut gpui::App) + 'static>,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Dialog {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        dialog(self.open, self.on_close, view! {
            <div style={css! {
                background: var(--popover);
                color: var(--popover-foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
                border-radius: var(--radius);
                padding: 24px;
                display: flex;
                flex-direction: column;
                gap: 12px;
                min-width: 320px;
            }}>
                {for_each(children, |c, _| c)}
            </div>
        })
    }
}

pub struct DialogTitle {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for DialogTitle {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <span style={css! { font-size: 18px; font-weight: 600; color: var(--foreground); }}>
                {for_each(children, |c, _| c)}
            </span>
        }
        .into_any_element()
    }
}

pub struct DialogDescription {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for DialogDescription {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        view! {
            <span style={css! { font-size: 14px; color: var(--muted-foreground); }}>
                {for_each(children, |c, _| c)}
            </span>
        }
        .into_any_element()
    }
}
