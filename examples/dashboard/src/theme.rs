use vgui::prelude::*;

/// Light theme built with the `theme!` macro.
fn light_theme() -> Theme {
    theme! {
        --bg: #f5f5f5;
        --surface: #ffffff;
        --primary: #2563ff;
        --text: #111111;
        --text-muted: #666666;
        --border: #dddddd;
        --radius: 8px;
    }
}

/// Dark theme — same variable names, different values.
fn dark_theme() -> Theme {
    theme! {
        --bg: #1a1a2e;
        --surface: #2d2d44;
        --primary: #2563ff;
        --text: #ffffff;
        --text-muted: #aaaaaa;
        --border: #444444;
        --radius: 8px;
    }
}

/// Theme mode propagated through the element tree via `<Provider>`.
#[derive(Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
}

/// The context marker for the theme mode.
pub static THEME_CTX: Context<ThemeMode> = Context::new();

/// Install the active theme based on the mode signal.
pub fn apply_theme(mode: ThemeMode) {
    set_theme(if matches!(mode, ThemeMode::Dark) {
        dark_theme()
    } else {
        light_theme()
    });
}
