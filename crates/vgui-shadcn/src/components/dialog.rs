use vgui::dialog;
use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn dialog_component(
    open: bool,
    on_close: Box<dyn Fn(&mut gpui::App) + 'static>,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    dialog(open, on_close, view! {
        <div style={css! {
            background: var(--popover);
            color: var(--popover-foreground);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            border-radius: var(--radius);
            padding: 24px;
            display: flex;
            flex-direction: column;
            gap: 12px;
            min-width: 320px;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    })
}

#[vgui_component]
pub fn dialog_title(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <span style={css! { font-size: 18px; font-weight: 600; color: var(--foreground); }}>
            {for_each(children, |c, _| c)}
        </span>
    }
}

#[vgui_component]
pub fn dialog_description(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <span style={css! { font-size: 14px; color: var(--muted-foreground); }}>
            {for_each(children, |c, _| c)}
        </span>
    }
}
