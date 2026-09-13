use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn tabs(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 8px;
            width: 100%;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn tabs_list(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            gap: 4px;
            background: var(--muted);
            border-radius: var(--radius);
            padding: 4px;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn tabs_trigger(
    active: bool,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    let style = if active {
        css! {
            background: var(--background);
            color: var(--foreground);
            border-radius: 6px;
            padding: 6px 12px;
            border-width: 0px;
            cursor: pointer;
            font-size: 14px;
        }
    } else {
        css! {
            background: var(--muted);
            color: var(--muted-foreground);
            border-radius: 6px;
            padding: 6px 12px;
            border-width: 0px;
            cursor: pointer;
            font-size: 14px;
        }
    };
    view! {
        <button style={style} on:click={click(move |cx| on_click(cx))}>
            {for_each(children, |c, _| c)}
        </button>
    }
}
