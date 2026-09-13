use vgui::for_each;
use vgui::prelude::*;
use vgui::show;

#[vgui_component]
pub fn collapsible(
    open: bool,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div style={css! { display: flex; flex-direction: column; gap: 4px; }}>
            {for_each(children, |c, _| c)}
        </div>
    }
}

#[vgui_component]
pub fn collapsible_content(
    open: bool,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    show(
        open,
        view! {
            <div style={css! { display: flex; flex-direction: column; }}>
                {for_each(children, |c, _| c)}
            </div>
        },
        gpui::Empty,
    )
}
