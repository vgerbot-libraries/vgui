use vgui::prelude::*;

#[vgui_component]
pub fn switch(
    checked: bool,
    on_change: Box<dyn Fn(bool, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let track = if checked {
        css! { background: var(--primary); }
    } else {
        css! { background: var(--input); }
    };
    let inner = if checked {
        css! {
            width: 40px;
            height: 22px;
            border-radius: 999px;
            padding: 2px;
            display: flex;
            align-items: center;
            justify-content: flex-end;
        }
    } else {
        css! {
            width: 40px;
            height: 22px;
            border-radius: 999px;
            padding: 2px;
            display: flex;
            align-items: center;
            justify-content: flex-start;
        }
    };
    view! {
        <button
            style={track}
            on:click={click(move |cx| on_change(!checked, cx))}
        >
            <div style={inner}>
                <div style={css! {
                    width: 18px;
                    height: 18px;
                    border-radius: 999px;
                    background: var(--background);
                }} />
            </div>
        </button>
    }
}
