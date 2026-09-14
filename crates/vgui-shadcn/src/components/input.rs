use vgui::prelude::*;

#[vgui_component]
pub fn text_field(
    value: String,
    placeholder: String,
    on_input: Box<dyn Fn(String, &mut gpui::App) + 'static>,
    class: Option<String>,
) -> impl gpui::IntoElement {
    let on_input = on_input;
    let class = class.unwrap_or_default();
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
                value={value}
                placeholder={placeholder}
                class="flex-1 w-full"
                style={css! {
                    border-width: 0;
                    background: var(--background);
                    color: var(--foreground);
                    padding: 0;
                    min-height: 0;
                }}
                on:input={input_cb(move |v, cx| on_input(v.to_string(), cx))}
            />
        </div>
    }
}
