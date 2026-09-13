use vgui::for_each;
use vgui::prelude::*;

/// Desktop/WASM adapter for the mobile picker: a vertical list of options.
#[vgui_component]
pub fn picker(
    options: Vec<String>,
    selected: usize,
    on_select: Box<dyn Fn(usize, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let on_select = std::rc::Rc::new(on_select);
    let items: Vec<gpui::AnyElement> = options
        .into_iter()
        .enumerate()
        .map(|(i, label)| {
            let on_select = on_select.clone();
            let style = if i == selected {
                css! {
                    background: var(--accent);
                    color: var(--accent-foreground);
                    padding: 8px 12px;
                    border-radius: var(--radius);
                    cursor: pointer;
                }
            } else {
                css! {
                    background: var(--background);
                    color: var(--foreground);
                    padding: 8px 12px;
                    border-radius: var(--radius);
                    cursor: pointer;
                }
            };
            view! {
                <button style={style} on:click={click(move |cx| on_select(i, cx))}>
                    {label}
                </button>
            }
            .into_any_element()
        })
        .collect();
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 4px;
            background: var(--popover);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 8px;
        }}>
            {for_each(items, |c, _| c)}
        </div>
    }
}
