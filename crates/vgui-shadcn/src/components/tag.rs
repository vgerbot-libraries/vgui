use vgui::prelude::*;

use crate::icons;

variants! {
    Tag {
        base => css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
            border-radius: var(--radius);
            padding: 2px 8px;
            font-size: 12px;
            font-weight: 500;
        },
        variant {
            default => css! { background: var(--primary); color: var(--primary-foreground); },
            secondary => css! { background: var(--secondary); color: var(--secondary-foreground); },
            outline => css! {
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
            },
            destructive => css! { background: var(--destructive); color: var(--primary-foreground); },
        },
    }
}

#[vgui_component]
pub fn tag(
    variant: TagVariant,
    label: String,
    closable: bool,
    on_close: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let variants = TagVariants::default().variant(variant);
    view! {
        <span style={variants}>
            <span>{label}</span>
            {if closable {
                view! {
                    <button
                        style={css! {
                            border-width: 0px;
                            cursor: pointer;
                            padding: 0px;
                            line-height: 1;
                        }}
                        on:click={click(move |cx| on_close(cx))}
                    >
                        {icons::X}
                    </button>
                }.into_any_element()
            } else {
                gpui::div().into_any_element()
            }}
        </span>
    }
}
