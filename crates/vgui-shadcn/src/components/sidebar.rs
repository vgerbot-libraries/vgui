use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn sidebar(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! {
            width: 240px;
            height: 100%;
            background: var(--sidebar);
            color: var(--sidebar-foreground);
            border-right-width: 1px;
            border-style: solid;
            border-color: var(--sidebar-border);
            display: flex;
            flex-direction: column;
            padding: 12px;
            gap: 4px;
            overflow: hidden;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn sidebar_item(
    label: String,
    icon: String,
    active: bool,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let bg = if active {
        css! { background: var(--sidebar-accent); color: var(--sidebar-accent-foreground); }
    } else {
        css! { background: var(--sidebar); color: var(--sidebar-foreground); }
    };
    view! {
        <button
            style={bg}
            hover={css! { background: var(--sidebar-accent); }}
            on:click={click(move |cx| on_click(cx))}
        >
            <div style={css! {
                display: flex;
                flex-direction: row;
                align-items: center;
                gap: 8px;
                padding: 8px 12px;
                border-radius: var(--radius);
                width: 100%;
            }}>
                <span>{icon}</span>
                <span>{label}</span>
            </div>
        </button>
    }
}
