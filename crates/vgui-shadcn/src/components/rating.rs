use vgui::for_each;
use vgui::prelude::*;

/// Star rating. Controlled by `value` (rounded to the nearest whole star).
/// When `on_change` is supplied the stars are interactive buttons, otherwise
/// the rating is read-only.
#[vgui_component]
pub fn rating(
    value: f32,
    count: usize,
    on_change: Option<Box<dyn Fn(usize, &mut gpui::App) + 'static>>,
) -> impl gpui::IntoElement {
    let count = count.max(1);
    let rounded = value.round().max(0.0) as usize;
    let interactive = on_change.is_some();
    let on_change = on_change.map(std::rc::Rc::new);

    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 2px;
        }}>
            {for_each(0..count, move |i: usize, _| {
                let filled = i < rounded;
                let glyph = if filled { "\u{2605}" } else { "\u{2606}" };
                let color = if filled {
                    css! { color: #f59e0b; }
                } else {
                    css! { color: var(--muted-foreground); }
                };
                let on_change = on_change.clone();
                if interactive {
                    let target = i + 1;
                    view! {
                        <button
                            style={css! {
                                border-width: 0px;
                                padding: 0px;
                                cursor: pointer;
                                font-size: 18px;
                                line-height: 1;
                            }}
                            hover={css! { opacity: 0.8; }}
                            on:click={click(move |cx| {
                                if let Some(cb) = on_change.as_ref() { cb(target, cx); }
                            })}
                        >
                            <span style={color}>{glyph}</span>
                        </button>
                    }.into_any_element()
                } else {
                    view! {
                        <span style={css! { font-size: 18px; line-height: 1; }}>
                            <span style={color}>{glyph}</span>
                        </span>
                    }.into_any_element()
                }
            })}
        </div>
    }
}
