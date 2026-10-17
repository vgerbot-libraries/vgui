use vgui::for_each;
use vgui::prelude::*;

/// Visual severity of a [`Toast`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ToastVariant {
    Default,
    Destructive,
}

/// One transient notification. The owning view keeps the live list (typically
/// in a signal) and removes an entry in `on_dismiss` when its close button is
/// clicked.
#[derive(Clone)]
pub struct Toast {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub variant: ToastVariant,
}

impl Toast {
    pub fn new(id: u64, title: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            description: None,
            variant: ToastVariant::Default,
        }
    }

    pub fn description(mut self, text: impl Into<String>) -> Self {
        self.description = Some(text.into());
        self
    }

    pub fn destructive(mut self) -> Self {
        self.variant = ToastVariant::Destructive;
        self
    }
}

fn toast_card_style(variant: ToastVariant) -> vgui::Css {
    if variant == ToastVariant::Destructive {
        css! {
            display: flex;
            flex-direction: column;
            gap: 4px;
            padding: 12px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--destructive);
            background: var(--popover);
        }
    } else {
        css! {
            display: flex;
            flex-direction: column;
            gap: 4px;
            padding: 12px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            background: var(--popover);
        }
    }
}

/// Portal-layer viewport that renders the active toasts above the rest of the
/// UI (top-end stack). Mount once near the app root; drive `toasts` reactively
/// and remove a toast in `on_dismiss`.
#[vgui_component]
pub fn toast_viewport(
    toasts: Vec<Toast>,
    on_dismiss: Box<dyn Fn(u64, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let on_dismiss = std::rc::Rc::new(on_dismiss);
    portal(
        view! {
            <div style={css! {
                position: absolute;
                top: 16px;
                right: 16px;
                display: flex;
                flex-direction: column;
                gap: 8px;
                width: 320px;
            }}>
                {for_each(toasts.into_iter(), move |toast: Toast, _| {
                    let on_dismiss = on_dismiss.clone();
                    let id = toast.id;
                    let title_color = if toast.variant == ToastVariant::Destructive {
                        css! { color: var(--destructive); }
                    } else {
                        css! { color: var(--popover-foreground); }
                    };
                    let description = toast.description.clone();
                    view! {
                        <div style={toast_card_style(toast.variant)}>
                            <div style={css! {
                                display: flex;
                                flex-direction: row;
                                justify-content: space-between;
                                align-items: center;
                                gap: 8px;
                            }}>
                                <span style={css! { font-size: 14px; font-weight: 600; }}>
                                    <span style={title_color}>{toast.title}</span>
                                </span>
                                <button
                                    style={css! {
                                        border-width: 0px;
                                        cursor: pointer;
                                        padding: 0px;
                                        line-height: 1;
                                        color: var(--muted-foreground);
                                    }}
                                    on:click={click(move |cx| on_dismiss(id, cx))}
                                >
                                    {crate::icons::X}
                                </button>
                            </div>
                            {if let Some(text) = description {
                                view! {
                                    <span style={css! {
                                        font-size: 13px;
                                        color: var(--muted-foreground);
                                    }}>
                                        {text}
                                    </span>
                                }.into_any_element()
                            } else {
                                gpui::div().into_any_element()
                            }}
                        </div>
                    }
                })}
            </div>
        },
        200,
    )
}
