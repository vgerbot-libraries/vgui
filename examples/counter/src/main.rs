use gpui::{div, px, rgb, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

fn increment_button(set_count: WriteSignal<i32>) -> impl gpui::IntoElement {
    view! {
        <button
            style={css! { padding: 8px; background: #0000ff; }}
            hover={css! { background: #000088; }}
            on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
        >
            {"Increment"}
        </button>
    }
}

fn decrement_button(set_count: WriteSignal<i32>) -> impl gpui::IntoElement {
    view! {
        <button
            style={css! { padding: 8px; background: #FF0000; }}
            hover={css! { background: #880000; }}
            on:click={click(move |cx| set_count.update(cx, |n| *n -= 1))}
        >
            {"Decrement"}
        </button>
    }
}

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    let doubled = create_memo({
        let count = count.clone();
        move || count.get() * 2
    });
    view! {
        <div style={css! {
            display: flex;
            flex-direction: column;
            gap: 12px;
            padding: 16px;
            background: #505050;
            width: 500px;
            height: 500px;
            justify-content: center;
            align-items: center;
            color: #ffffff;
        }}>
            <span>{format!("Hello, {}!", count.get())}</span>
            <span>{format!("doubled {}", doubled.get())}</span>
            <Show when={count.get() > 0}>
                <span>{"positive"}</span>
            </Show>
            <Show when={count.get() < 0}>
                <span>{"negative"}</span>
            </Show>
            <Show when={count.get() == 0}>
                <span>{"zero"}</span>
            </Show>
            <Show when={count.get() & 1 == 1} fallback={view! { <span>{"even"}</span> }}>
                <span>{"odd"}</span>
            </Show>
            {increment_button(set_count.clone())}
            {decrement_button(set_count.clone())}
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    });
}
