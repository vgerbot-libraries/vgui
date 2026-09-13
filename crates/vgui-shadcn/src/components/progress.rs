use vgui::prelude::*;

pub struct Progress {
    pub value: f64,
    pub max: f64,
}

impl gpui::IntoElement for Progress {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let pct = if self.max <= 0.0 {
            0.0
        } else {
            (self.value / self.max).clamp(0.0, 1.0)
        };
        let outer = css! {
            width: 100%;
            height: 8px;
            background: var(--muted);
            border-radius: 999px;
            overflow: hidden;
        };
        let inner_style = css! {
            height: 100%;
            background: var(--primary);
            border-radius: 999px;
        };
        let inner = inner_style.apply(
            gpui::div().w(gpui::relative(pct as f32))
        );
        outer.apply(
            gpui::div().child(inner)
        ).into_any_element()
    }
}

pub fn progress(value: f64, max: f64) -> Progress {
    Progress { value, max }
}
