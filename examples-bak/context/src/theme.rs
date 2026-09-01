use vgui::prelude::*;

/// A theme mode propagated through the element tree via `<Provider>`.
#[derive(Clone, PartialEq)]
pub enum Mode {
    Light,
    Dark,
}

/// The context marker. Zero-sized, stored in a plain `static`.
pub static MODE: Context<Mode> = Context::new();

/// A box that reads the nearest `MODE` provider, falling back to `Light`
/// when no provider is active. `css!` takes literal CSS, so the `if`/`else`
/// picks one of two literal blocks — no dynamic interpolation needed.
///
/// This leaf consumer defines no provider of its own; it reads whatever
/// ancestor provider was pushed in another module, proving a `Context<T>`
/// marker crosses file boundaries for free.
pub fn themed_box(label: &str) -> impl gpui::IntoElement {
    let mode = use_context_or(&MODE, || Mode::Light);
    let style = if matches!(mode, Mode::Dark) {
        css! {
            background: #1a1a2a;
            color: #ffffff;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    } else {
        css! {
            background: #f5f5f5;
            color: #111111;
            padding: 16px;
            margin: 8px;
            border-radius: 8px;
        }
    };
    view! {
        <div style={style}>{label.to_string()}</div>
    }
}
