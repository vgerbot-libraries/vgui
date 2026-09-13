use vgui::prelude::*;

#[vgui_component]
pub fn slider(
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    on_change: Box<dyn Fn(f64, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! { width: 100%; }}>
            <input
                type="range"
                value={value}
                min={min}
                max={max}
                step={step}
                on:change={f64_change_cb(move |v, cx| on_change(v, cx))}
            />
        </div>
    }
}
