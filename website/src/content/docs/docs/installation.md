---
title: Installation
description: Prepare a Rust nightly toolchain and native GPUI build dependencies for vgui.
---

A vgui application needs a Rust nightly toolchain, a C/C++ build environment, and the native libraries required by GPUI. The exact packages depend on your operating system.

## Install Rust tools

Install [rustup](https://rustup.rs/), then install nightly and its development components:

```bash
rustup toolchain install nightly
rustup component add rustfmt clippy --toolchain nightly
```

If you intend to compile for WebAssembly, also install the target:

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Installing the target only prepares the compiler. It does not by itself verify that an application runs in a browser.

The repository's `rust-toolchain.toml` selects `nightly` but does not pin a dated toolchain. vgui and GPUI are also consumed as Git dependencies without a fixed `rev` in the example manifest. Keep your application's `Cargo.lock` under version control to make dependency resolution reproducible until you deliberately update it.

## Native system dependencies

vgui inherits its window-system, graphics, font, audio, and build-time requirements from GPUI and related native crates.

### Debian and Ubuntu

Install the build tools and development packages:

```bash
sudo apt-get update
sudo apt-get install -y \
  build-essential cmake pkg-config libclang-dev \
  libssl-dev libzstd-dev libfontconfig1-dev libfreetype6-dev \
  libglib2.0-dev libgtk-3-dev libasound2-dev libdbus-1-dev \
  libxkbcommon-dev libxkbcommon-x11-dev libx11-dev libxext-dev \
  libxrandr-dev libxinerama-dev libxcursor-dev libxi-dev \
  libwayland-dev libgl-dev libegl-dev
```

`pkg-config` and `libclang` are used during native builds. X11, Wayland, OpenGL, font, and desktop libraries are needed by the GPUI backend.

### macOS

Install the Xcode Command Line Tools:

```bash
xcode-select --install
```

GPUI uses the native Cocoa and Metal platform on macOS. This command describes the prerequisite; it is not a claim that every vgui feature has been validated on macOS.

### Windows

Use the MSVC Rust toolchain and install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the C++ workload and a Windows SDK. GPUI uses the Win32 and DirectX platform on Windows. Feature validation on Windows is separate from installing these prerequisites.

## Choose how to use the source

For a new application, use the Git dependencies shown in the [Quick Start](../quick-start/). You do not need to clone vgui into your application.

To develop vgui itself, clone the repository and build the workspace:

```bash
git clone https://github.com/vgerbot-libraries/vgui.git
cd vgui
cargo build --workspace
```

The first build compiles GPUI and its graphics backends, so it can take substantially longer than later builds.

## Next steps

Continue to the [Quick Start](../quick-start/) to create the counter application. If the compiler or window system reports an error, see [Compatibility & Troubleshooting](../compatibility-troubleshooting/).
