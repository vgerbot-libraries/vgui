use vgui::for_each;
use vgui::prelude::*;

/// Two-state toggle button. Renders pressed (`pressed=true`) in the accent
/// style and emits the new state on click. Mirrors shadcn `Toggle`.
#[vgui_component]
pub fn toggle(
    pressed: bool,
    on_toggle: Box<dyn Fn(bool, &mut gpui::App) + 'static>,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    let style = if pressed {
        css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            gap: 8px;
            height: 36px;
            min-width: 36px;
            padding: 8px 12px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            background: var(--accent);
            color: var(--accent-foreground);
            cursor: pointer;
            font-size: 14px;
        }
    } else {
        css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            gap: 8px;
            height: 36px;
            min-width: 36px;
            padding: 8px 12px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            background: var(--background);
            color: var(--foreground);
            cursor: pointer;
            font-size: 14px;
        }
    };
    view! {
        <button
            style={style}
            hover={css! { background: var(--accent); }}
            aria:toggled={if pressed { "true" } else { "false" }}
            on:click={click(move |cx| on_toggle(!pressed, cx))}
        >
            {for_each(children, |c, _| c)}
        </button>
    }
}
