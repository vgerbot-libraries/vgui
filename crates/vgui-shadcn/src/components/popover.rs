use vgui::floating_at;
use vgui::for_each;
use vgui::prelude::*;
use vgui::show;

#[vgui_component]
pub fn popover(
    open: bool,
    anchor: NodeRef,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    show(
        open,
        floating_at(
            &anchor,
            view! {
                <div style={css! {
                    background: var(--popover);
                    color: var(--popover-foreground);
                    border-width: 1px;
                    border-style: solid;
                    border-color: var(--border);
                    border-radius: var(--radius);
                    padding: 12px;
                }}>
                    {for_each(children, |c, _| c)}
                </div>
            },
        ),
        gpui::Empty,
    )
}
