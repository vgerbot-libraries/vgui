---
title: Quick Start
description: Create a complete native vgui counter application from an empty directory.
---

This guide creates a native counter with one button. The example includes every import and initialization step, so you can copy it into a new project without additional framework setup.

Complete the [Installation](../installation/) prerequisites before starting.

## 1. Create a project

```bash
cargo new vgui-counter --bin
cd vgui-counter
```

## 2. Add the dependencies

Replace `Cargo.toml` with the following manifest. vgui and GPUI currently use Git dependencies:

```toml
[package]
name = "vgui-counter"
version = "0.1.0"
edition = "2021"

[dependencies]
vgui = { git = "https://github.com/vgerbot-libraries/vgui" }
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-platform = { git = "https://github.com/zed-industries/zed", package = "gpui_platform" }
```

The `gpui-platform` dependency supplies the native application entry point used below. Keep the generated `Cargo.lock` so later builds use the same resolved revisions until you update them.

## 3. Create the application

Replace `src/main.rs` with this complete program:

```rust
use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use gpui_platform::application;
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (count, set_count) = create_signal(0i32);
    view! {
        <div class="flex flex-col gap-3 p-4 w-[500px] h-[500px] justify-center items-center text-white">
            <span>{format!("count = {}", count.get())}</span>
            <button
                class="p-2 bg-[#0000ff] hover:bg-[#000088] text-white rounded"
                on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
            >
                {"Increment"}
            </button>
        </div>
    }
}

fn main() {
    application().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(500.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
        )
        .unwrap();
    });
}
```

## 4. Run it

```bash
cargo +nightly run
```

The first build may take a while because Cargo also compiles GPUI and its native dependencies. Run the command in a desktop session; a headless build environment is not enough to display a window.

## How the pieces fit together

- `application()` creates the GPUI application for the current native platform, and `run` starts its event loop.
- `open_window` configures a 500 × 500 window and gives vgui the window and application contexts.
- `vgui::mount` installs `app` as the root view rendered inside that window.
- `create_signal` creates the count, `count.get()` displays it, and the click handler uses `set_count.update` to increment it.
- The `class` strings contain vgui-supported Tailwind-style utilities; they do not require a browser stylesheet.

The same program is compiled in the documentation CI as a native workspace example, and the manifest above has been checked from an empty external consumer project. These are compile checks, not a claim that native button interaction or a browser build has been verified.

Next, read [Your First Reactive View](../first-reactive-view/) to understand how state reads, writes, and rendering fit together.
