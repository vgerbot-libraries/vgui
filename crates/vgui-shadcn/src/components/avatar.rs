use vgui::prelude::*;

pub struct Avatar {
    pub initials: String,
    pub size: f32,
}

impl gpui::IntoElement for Avatar {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let size = self.size;
        let initials = self.initials;
        let style = css! {
            border-radius: 999px;
            background: var(--muted);
            color: var(--muted-foreground);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            font-weight: 600;
        };
        let el = gpui::div()
            .w(gpui::px(size))
            .h(gpui::px(size))
            .child(initials);
        style.apply(el).into_any_element()
    }
}

pub fn avatar(initials: impl Into<String>) -> Avatar {
    Avatar {
        initials: initials.into(),
        size: 40.0,
    }
}
