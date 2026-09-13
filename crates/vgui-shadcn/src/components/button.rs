use vgui::for_each;
use vgui::prelude::*;

variants! {
    Button {
        base => css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            justify-content: center;
            gap: 8px;
            border-radius: var(--radius);
            border-width: 0px;
            cursor: pointer;
            font-size: 14px;
            font-weight: 500;
        },

        variant {
            default => css! {
                background: var(--primary);
                color: var(--primary-foreground);
            },
            destructive => css! {
                background: var(--destructive);
                color: var(--primary-foreground);
            },
            outline => css! {
                background: var(--background);
                color: var(--foreground);
                border-width: 1px;
                border-style: solid;
                border-color: var(--border);
            },
            secondary => css! {
                background: var(--secondary);
                color: var(--secondary-foreground);
            },
            ghost => css! {
                background: var(--background);
                color: var(--foreground);
            },
            link => css! {
                background: var(--background);
                color: var(--primary);
            },
        },

        size {
            default => css! { height: 36px; padding: 8px 16px; },
            sm => css! { height: 32px; padding: 4px 12px; font-size: 12px; },
            lg => css! { height: 40px; padding: 10px 24px; },
            icon => css! { width: 36px; height: 36px; padding: 0px; },
        },
    }
}

#[vgui_component]
pub fn button(
    variant: ButtonVariant,
    size: ButtonSize,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    let variants = ButtonVariants::default()
        .variant(variant)
        .size(size);
    let on_click = on_click;
    let children = children;
    let hover = match variant {
        ButtonVariant::Ghost | ButtonVariant::Outline => css! { background: var(--accent); },
        ButtonVariant::Secondary => css! { opacity: 0.8; },
        ButtonVariant::Link => css! { opacity: 0.8; },
        _ => css! { opacity: 0.9; },
    };
    view! {
        <button style={variants} hover={hover} on:click={click(move |cx| on_click(cx))}>
            {for_each(children, |c, _| c)}
        </button>
    }
}

#[vgui_component]
pub fn IconButton(
    glyph: &'static str,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    view! {
        <Button variant={ButtonVariant::Ghost} size={ButtonSize::Icon} on:click={on_click}>
            {glyph.to_string()}
        </Button>
    }
}
