use vgui::prelude::*;

pub struct TextField {
    pub value: String,
    pub placeholder: String,
    pub on_input: Box<dyn Fn(String, &mut gpui::App) + 'static>,
    pub class: Option<String>,
}

impl gpui::IntoElement for TextField {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let on_input = self.on_input;
        let class = self.class.unwrap_or_default();
        view! {
            <div class={class.clone()} style={css! {
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--input);
                border-radius: var(--radius);
                padding: 4px 12px;
                height: 36px;
                display: flex;
                align-items: center;
            }}>
                <input
                    type="text"
                    value={self.value}
                    placeholder={self.placeholder}
                    on:input={input_cb(move |v, cx| on_input(v.to_string(), cx))}
                />
            </div>
        }
        .into_any_element()
    }
}

pub fn text_field(
    value: String,
    placeholder: impl Into<String>,
    on_input: impl Fn(String, &mut gpui::App) + 'static,
) -> TextField {
    TextField {
        value,
        placeholder: placeholder.into(),
        on_input: Box::new(on_input),
        class: None,
    }
}
