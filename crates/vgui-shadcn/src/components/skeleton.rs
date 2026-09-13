use vgui::prelude::*;

pub struct Skeleton {
    pub width: f32,
    pub height: f32,
}

impl gpui::IntoElement for Skeleton {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let w = self.width;
        let h = self.height;
        let style = css! {
            background: var(--muted);
            border-radius: var(--radius);
        };
        let el = gpui::div()
            .w(gpui::px(w))
            .h(gpui::px(h));
        style.apply(el).into_any_element()
    }
}

pub fn skeleton(width: f32, height: f32) -> Skeleton {
    Skeleton { width, height }
}
