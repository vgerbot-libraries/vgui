use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (open, set_open) = create_signal(false);
    let (text, set_text) = create_signal("Hello".to_string());

    view! {
        <div class="flex flex-col gap-2 p-4 bg-[#1a1a2e] w-[600px] h-[700px] text-white overflow-y-auto">
            <h1>{"Heading 1"}</h1>
            <h2>{"Heading 2"}</h2>
            <h3>{"Heading 3"}</h3>
            <h4>{"Heading 4"}</h4>
            <h5>{"Heading 5"}</h5>
            <h6>{"Heading 6"}</h6>

            <hr />

            <p>{"Normal paragraph"}</p>
            <strong>{"Bold text"}</strong>
            <em>{"Italic text"}</em>
            <u>{"Underlined"}</u>
            <s>{"Strikethrough"}</s>
            <mark>{"Highlighted"}</mark>
            <code>{"monospace"}</code>
            <small>{"small text"}</small>

            <hr />

            <ul>
                <li>{"Item 1"}</li>
                <li>{"Item 2"}</li>
            </ul>
            <ol>
                <li>{"First"}</li>
                <li>{"Second"}</li>
            </ol>
            <dl>
                <dt>{"Term"}</dt>
                <dd>{"Definition"}</dd>
            </dl>

            <hr />

            <header>{"Header"}</header>
            <nav>{"Nav"}</nav>
            <main>{"Main content"}</main>
            <section>{"Section"}</section>
            <article>{"Article"}</article>
            <aside>{"Aside"}</aside>
            <footer>{"Footer"}</footer>

            <hr />

            <a on:click={click(move |_cx| {})}>{"Click link"}</a>

            <hr />

            <div style={css! {
                font-family: monospace;
                text-overflow: ellipsis;
                text-decoration-color: #ff0000;
                text-decoration-thickness: 2px;
                text-decoration-style: wavy;
                text-background: #ffff00;
                scrollbar-width: thin;
                background: linear-gradient(90deg, #ff0000, #0000ff);
                white-space: nowrap;
                overflow: hidden;
            }}>
                <span>{"CSS properties test"}</span>
            </div>

            <hr />

            <div class="truncate font-mono leading-none">{"Truncated monospace text with leading-none"}</div>
            <div class="text-ellipsis font-serif leading-loose">{"Ellipsis serif with leading-loose"}</div>
            <div class="underline decoration-wavy decoration-2">{"Wavy underline thickness 2"}</div>

            <hr />

            <progress value={0.5f64} max={1.0f64} />

            <meter value={0.7f64} max={1.0f64} />

            <hr />

            <textarea
                placeholder="Enter text"
                value={text.get()}
                on:input={move |v: &str, cx: &mut App| set_text.set(cx, v.to_string())}
            />

            <hr />

            <select
                options={vec![("1".to_string(), "One".to_string()), ("2".to_string(), "Two".to_string())]}
                value={"1".to_string()}
                on:change={move |v: &str, _cx: &mut App| {}}
            />

            <hr />

            <details open={open.get()}>
                <summary on:click={click(move |cx| set_open.update(cx, |v| *v = !*v))}>
                    {"Click to toggle"}
                </summary>
                <div>{"Hidden content"}</div>
            </details>

            <hr />

            <dialog open={false}>
                <div class="bg-white p-4 rounded">{"Dialog content"}</div>
            </dialog>

            <div on:modifiers_changed={move |_e, _w, _cx| {}} on:any_mouse_down={move |_e, _w, _cx| {}}>
                {"Events test"}
            </div>
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(700.0)), cx);
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
