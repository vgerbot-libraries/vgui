use vgui::dialog;
use vgui::for_each;
use vgui::prelude::*;

#[vgui_component]
pub fn drawer(
    open: bool,
    on_close: Box<dyn Fn(&mut gpui::App) + 'static>,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    dialog(open, on_close, view! {
        <div style={css! {
            background: var(--sidebar);
            color: var(--sidebar-foreground);
            border-width: 1px;
            border-style: solid;
            border-color: var(--sidebar-border);
            border-radius: var(--radius);
            padding: 16px;
            width: 280px;
            height: 100%;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}>
            {for_each(children, |c, _| c)}
        </div>
    })
}
