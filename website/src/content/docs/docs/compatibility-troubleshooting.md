---
title: Compatibility & Troubleshooting
description: Diagnose common vgui toolchain, native dependency, window, and platform verification problems.
---

vgui is experimental and tracks moving Rust nightly and Git dependencies. When something fails, first identify whether the problem is dependency resolution, compilation, native window creation, or browser behavior.

## Toolchain and dependency changes

The repository selects `nightly` without pinning a date, and the example dependencies do not specify Git revisions. A fresh install can therefore resolve code that differs from an earlier build.

- Keep the application's `Cargo.lock` so dependency updates are deliberate.
- If a previously working unlocked project fails, compare the complete [Quick Start](../quick-start/) with your manifest and entry point.
- Use `cargo +nightly check` to separate compilation problems from window-runtime problems.
- Do not assume an example written for another GPUI revision still has the same application API.

## Native build failures

Use the error's first missing package, linker input, or build-script failure to guide the fix.

| Symptom | Things to check |
| --- | --- |
| `pkg-config` cannot find a package | Install `pkg-config` and the package's development files, not only its runtime library. |
| `bindgen`, Clang, or header errors | Install `libclang`, a C/C++ build toolchain, and the development headers listed in Installation. |
| Font, GTK, X11, Wayland, GL, audio, or D-Bus linker errors | Compare your installed development packages with the list for your distribution. |
| The first build takes a long time | GPUI and graphics backends are large native dependencies; subsequent builds reuse Cargo artifacts. |

Return to [Installation](../installation/) for the Debian/Ubuntu package list and macOS/Windows prerequisites. Package names vary on other distributions.

## The program compiles but no window appears

A successful build does not require a graphical session, but opening a native window does.

- Run the application from a desktop session rather than a headless CI or SSH environment without display forwarding.
- On Linux, check that the relevant graphical session and `DISPLAY` or `WAYLAND_DISPLAY` environment are available to the process.
- Check the terminal for errors from window creation or the native graphics backend.
- Do not treat a Linux result as validation on macOS or Windows; each platform needs its own run and interaction checks.

The counter can be compile-checked from a repository checkout with:

```bash
cargo check -p vgui --example counter --locked
```

That command checks code and dependencies. It does not open a window or click the button.

## WebAssembly and browser issues

Keep these questions separate:

1. Is `wasm32-unknown-unknown` installed for the selected nightly toolchain?
2. Did this specific application compile for that target?
3. Did the generated module and assets load in a browser?
4. Did rendering, sizing, keyboard input, text input, IME, clipboard, and accessibility behavior actually get tested?

A “yes” at one step does not imply the next. The current [Native & Web](../native-and-web/) page documents the available support code without claiming a verified browser counter.

## Get help

If you report an issue, include your operating system, Rust toolchain, relevant dependency revisions, the complete command, and the first full error. For a runtime problem, say whether the application compiled and whether a window or browser canvas appeared.

Open a [GitHub issue](https://github.com/vgerbot-libraries/vgui/issues) with that information, or return to the [Quick Start](../quick-start/) to compare a minimal project.
