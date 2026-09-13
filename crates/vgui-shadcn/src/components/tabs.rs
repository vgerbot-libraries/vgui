use vgui::for_each;
use vgui::prelude::*;

pub struct Tabs {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for Tabs {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
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
        .into_any_element()
    }
}

pub struct TabsList {
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for TabsList {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let children = self.children;
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
        .into_any_element()
    }
}

pub struct TabsTrigger {
    pub active: bool,
    pub on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
    pub children: Vec<gpui::AnyElement>,
}

impl gpui::IntoElement for TabsTrigger {
    type Element = gpui::AnyElement;
    fn into_element(self) -> Self::Element {
        let on_click = self.on_click;
        let children = self.children;
        let style = if self.active {
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
        .into_any_element()
    }
}
