use vgui::prelude::*;

pub struct Slider {
    pub value: f64,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub on_change: Box<dyn Fn(f64, &mut gpui::App) + 'static>,
}

impl gpui::IntoElement for Slider {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let on_change = self.on_change;
        view! {
            <div style={css! { width: 100%; }}>
                <input
                    type="range"
                    value={self.value}
                    min={self.min}
                    max={self.max}
                    step={self.step}
                    on:change={f64_change_cb(move |v, cx| on_change(v, cx))}
                />
            </div>
        }
        .into_any_element()
    }
}

pub fn slider(
    value: f64,
    min: f64,
    max: f64,
    on_change: impl Fn(f64, &mut gpui::App) + 'static,
) -> Slider {
    Slider {
        value,
        min,
        max,
        step: 1.0,
        on_change: Box::new(on_change),
    }
}
