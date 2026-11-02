---
title: Native & Web
description: Understand vgui's native and WebAssembly targets and the current limits of browser verification.
---

vgui contains native GPUI integration and WebAssembly-specific support code, but these are not interchangeable entry points. A native compile check also does not establish that a browser demo renders or handles input correctly.

## Current capabilities and verification

| Path | What is available | What it does not prove |
| --- | --- | --- |
| Native counter | The Quick Start uses `gpui_platform::application()` and is compile-checked in the documentation workflow. | Compilation alone does not test the visible window or button interaction. |
| WebAssembly target | The toolchain can include `wasm32-unknown-unknown`, and vgui has WASM-only dependencies and browser keyboard helpers. | Installing the target or having WASM code does not validate a complete application in a browser. |
| Browser demo | Browser rendering, canvas hosting, input, and application lifetime require a separate platform entry point and asset setup. | There is currently no verified dual-target counter tutorial or published live demo in this documentation. |

## Native applications

The [Quick Start](../quick-start/) follows the native model:

1. Create the current platform's GPUI application.
2. Run the native event loop.
3. Open a GPUI window.
4. Mount the vgui root view into that window.

The markup still produces GPUI elements. Elements do not become browser DOM nodes, and window behavior comes from the native GPUI backend.

## What changes in a browser

A browser host must address concerns that a native window handles differently:

- **Entry point and lifetime:** WASM startup is non-blocking and uses the web platform rather than the native `application().run` path shown in the Quick Start.
- **Canvas sizing:** the host page and canvas determine the available viewport. A fixed native window size is not a responsive browser layout.
- **Keyboard behavior:** browser defaults such as Tab navigation and Space scrolling can conflict with canvas input. vgui exposes `intercept_keyboard_events()` to install bubbling-phase keyboard listeners for a browser canvas; on native targets it is a no-op.
- **Input and platform behavior:** text input, IME, clipboard, accessibility, and other OS integrations need their own browser validation.

The keyboard helper is one piece of browser integration, not a complete WASM startup or deployment recipe.

## Verification boundary

The P0 documentation verifies the counter's native compilation only. It does not provide an unverified `wasm-bindgen` command sequence or reuse the repository's old WASM script, which refers to removed example packages.

Treat native compilation, native interaction, WASM compilation, browser rendering, and browser interaction as separate checks. A future browser quick start should include its complete source and record the browser and interactions actually tested.

For build or display problems, continue to [Compatibility & Troubleshooting](../compatibility-troubleshooting/).
