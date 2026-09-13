use vgui::dialog;
use vgui::for_each;
use vgui::prelude::*;

/// Side panel overlay. Uses the dialog portal (no slide transform).
pub struct Drawer {
    pub open: bool,
    pub on_close: Box<dyn Fn(&mut gpui::App) + 'static>,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Drawer {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
        dialog(self.open, self.on_close, view! {
            <div style={css! {
                background: var(--sidebar);
                color: var(--sidebar-foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--sidebar-border);
                border-radius: var(--radius);
                padding: 16px;
                width: 280px;
                height: 100%;
                display: flex;
                flex-direction: column;
                gap: 8px;
            }}>
                {for_each(children, |c, _| c)}
            </div>
        })
    }
}
