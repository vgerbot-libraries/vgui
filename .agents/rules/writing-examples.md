---
name: writing-examples
description: Conventions for creating dual-target (native + WASM) examples with mdBook live demo pages. Applies when adding or modifying examples or their book pages.
globs:
  - "examples/**"
  - "book/src/examples/**"
  - "scripts/build_wasm.sh"
paths:
  - "examples/**"
  - "book/src/examples/**"
  - "scripts/build_wasm.sh"
trigger: auto
tags: [examples, wasm, mdbook, docs]
---

# Rule: Writing Examples

Every example in `examples/` must be dual-target (native + WASM) and have a
corresponding mdBook page with an embedded live demo.

## 1. Example binary structure

Each example lives at `examples/<name>/` with:

```
examples/<name>/
├── Cargo.toml
├── src/main.rs
├── index.html          # Trunk entry (optional, for `trunk serve`)
├── trunk.toml           # Trunk config (optional)
└── standalone.html      # Manual WASM test page (optional)
```

### Cargo.toml

```toml
[package]
name = "vgui-<name>"
version.workspace = true
edition.workspace = true
# ... other workspace fields

[[bin]]
name = "<name>"
path = "src/main.rs"

[dependencies]
vgui.workspace = true
gpui.workspace = true
gpui-platform.workspace = true

[target.'cfg(target_family = "wasm")'.dependencies]
wasm-bindgen = "0.2"
```

The `wasm-bindgen` dependency is **required** — it provides the
`#[wasm_bindgen(start)]` attribute for the WASM entry point.

### src/main.rs — dual entry point

Every example MUST use this pattern:

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    // ... example-specific UI ...
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(WIDTH.), px(HEIGHT.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);

    // On WASM, run_embedded returns an ApplicationHandle that keeps the
    // app alive. mem::forget prevents it from being dropped when start()
    // returns, since WASM's run() is non-blocking.
    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
```

**Key points:**

- `#![cfg_attr(target_family = "wasm", no_main)]` — disables native `main` on WASM.
- `single_threaded_web()` — vgui uses `Rc`/`RefCell`/`thread_local`, so it is
  single-threaded. Do NOT use `application()` on WASM (it enables
  multi-threading via web workers, which causes `Send`/`Sync` issues).
- `std::mem::forget(gpui_app.run_embedded(launch))` — WASM's `Platform::run`
  returns immediately (non-blocking). `run_embedded` returns an
  `ApplicationHandle` that keeps the app alive; `mem::forget` prevents it from
  being dropped when `start()` returns.
- `vgui::intercept_keyboard_events()` — **required** in every WASM `start()`.
  Installs `window`-level listeners that call `preventDefault()` and
  `stopPropagation()` on all keyboard events, preventing the browser from
  stealing focus (Tab) or scrolling (Space/arrows) when the canvas doesn't
  handle a key. gpui_web's own listeners on the IME mirror `<textarea>` fire
  first (target + early bubble), so gpui still receives every keystroke; the
  `window` listener only suppresses the browser default after gpui has
  processed it. On non-WASM targets this function is a no-op.
- `gpui_platform::web_init()` — sets up panic hooks and logging for WASM.

## 2. Building WASM for the book

Use the build script to compile, bindgen, and copy assets:

```bash
scripts/build_wasm.sh <name>
```

This produces:
- `examples/<name>/dist/<name>.js` + `<name>_bg.wasm` (wasm-bindgen output)
- `book/src/wasm/<name>/index.html` + `<name>.js` + `<name>_bg.wasm` (book assets)

The `book/src/wasm/` directory is gitignored — always run `build_wasm.sh` before
`mdbook build` when example code changes.

### Manual build (if build_wasm.sh doesn't exist)

```bash
# 1. Build release WASM
cargo +nightly build --target wasm32-unknown-unknown -p vgui-<name> --release

# 2. Generate JS bindings
wasm-bindgen --target web --out-dir examples/<name>/dist \
    --no-typescript target/wasm32-unknown-unknown/release/<name>.wasm

# 3. Copy to book
mkdir -p book/src/wasm/<name>
cp examples/<name>/dist/<name>.js book/src/wasm/<name>/
cp examples/<name>/dist/<name>_bg.wasm book/src/wasm/<name>/
```

The `book/src/wasm/<name>/index.html` loader file is created once and should
not need updating unless the JS module name changes.

## 3. mdBook page

Every example MUST have a page at `book/src/examples/<name>.md` with:

1. **Live Demo** section (immediately after the title) with an iframe:

```markdown
# <Title> Example

## Live Demo

<iframe src="../wasm/<name>/" width="100%" height="600" style="border:1px solid #444; border-radius:4px;"></iframe>
```

> **IMPORTANT:** The iframe `src` MUST end with a trailing slash
> (`../wasm/<name>/`), not `../wasm/<name>/index.html`. mdbook serve redirects
> `index.html` to a clean URL, which breaks relative path resolution for the
> WASM/JS assets inside the iframe.

2. **Source Code** section with the full `main.rs` content (including the
   dual entry point boilerplate).

3. **Key Concepts** section explaining the example's features.

4. **Running** section with both native and WASM commands:

```markdown
### Running

**Native:**

    cargo run -p vgui-<name>

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-<name> --release
    wasm-bindgen --target web --out-dir examples/<name>/dist \
        --no-typescript target/wasm32-unknown-unknown/release/<name>.wasm
    python3 scripts/serve_plain.py 8080 examples/<name>
```

5. The page MUST be listed in `book/src/SUMMARY.md` under the Examples section.

## 4. Checklist for adding a new example

- [ ] Create `examples/<name>/Cargo.toml` with `wasm-bindgen` target dep
- [ ] Write `examples/<name>/src/main.rs` with dual entry point pattern
- [ ] Add the example to the workspace `members` list in root `Cargo.toml`
- [ ] `cargo +nightly check` passes (native)
- [ ] `cargo +nightly check --target wasm32-unknown-unknown` passes (WASM)
- [ ] Run `scripts/build_wasm.sh <name>` to generate WASM assets
- [ ] Create `book/src/wasm/<name>/index.html` (or let build_wasm.sh create it)
- [ ] Create `book/src/examples/<name>.md` with Live Demo iframe + source
- [ ] Add the page to `book/src/SUMMARY.md`
- [ ] `mdbook build book` succeeds
- [ ] Open the page in a browser and verify the iframe renders the app
