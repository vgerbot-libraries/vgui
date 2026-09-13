use vgui::for_each;
use vgui::prelude::*;

pub struct Sidebar {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Sidebar {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
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
        .into_any_element()
    }
}

pub struct SidebarItem {
    pub label: String,
    pub icon: String,
    pub active: bool,
    pub on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
}

impl gpui::IntoElement for SidebarItem {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let on_click = self.on_click;
        let bg = if self.active {
            css! { background: var(--sidebar-accent); color: var(--sidebar-accent-foreground); }
        } else {
            css! { background: var(--sidebar); color: var(--sidebar-foreground); }
        };
        let icon = self.icon;
        let label = self.label;
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
        .into_any_element()
    }
}

pub fn sidebar_item(
    icon: impl Into<String>,
    label: impl Into<String>,
    active: bool,
    on_click: impl Fn(&mut gpui::App) + 'static,
) -> SidebarItem {
    SidebarItem {
        icon: icon.into(),
        label: label.into(),
        active,
        on_click: Box::new(on_click),
    }
}
