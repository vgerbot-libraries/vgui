# Installation

## Prerequisites

- A recent stable **Rust** toolchain (edition 2021, ≥ 1.74 recommended).
- [`pkg-config`](https://www.freedesktop.org/wiki/Software/pkg-config/).
- A C/C++ build toolchain: `build-essential`, `cmake`.
- `libclang` for `bindgen` (used by `gpui` and its transitive deps).

## System Libraries

`vgui` does not depend on system libraries directly, but `gpui` does — it
talks to the native window system (Wayland and X11 on Linux, Cocoa/Metal on
macOS, Win32/DirectX on Windows). The following development packages are
required to build `gpui` on a Debian/Ubuntu Linux host:

```bash
sudo apt-get install -y \
  build-essential \
  cmake \
  pkg-config \
  libclang-dev \
  libssl-dev \
  libzstd-dev \
  libfontconfig1-dev \
  libfreetype6-dev \
  libglib2.0-dev \
  libgtk-3-dev \
  libasound2-dev \
  libdbus-1-dev \
  libxkbcommon-dev \
  libxkbcommon-x11-dev \
  libx11-dev \
  libxext-dev \
  libxrandr-dev \
  libxinerama-dev \
  libxcursor-dev \
  libxi-dev \
  libwayland-dev \
  libgl-dev \
  libegl-dev
```

### Other distributions

<details>
<summary>Fedora / RHEL</summary>

```bash
sudo dnf install -y \
  clang-devel openssl-devel libzstd-devel fontconfig-devel freetype-devel \
  glib2-devel gtk3-devel alsa-lib-devel dbus-devel \
  libxkbcommon-devel libxkbcommon-x11-devel \
  libX11-devel libXext-devel libXrandr-devel libXinerama-devel \
  libXcursor-devel libXi-devel wayland-devel \
  mesa-libGL-devel mesa-libEGL-devel \
  cmake pkg-config
```

</details>

<details>
<summary>Arch Linux</summary>

```bash
sudo pacman -S --needed \
  base-devel clang cmake pkgconf \
  openssl zstd fontconfig freetype2 glib2 gtk3 alsa-lib dbus \
  libxkbcommon libxkbcommon-x11 \
  libx11 libxext libxrandr libxinerama libxcursor libxi wayland \
  mesa
```

</details>

<details>
<summary>macOS</summary>

No extra system libraries are required beyond Xcode Command Line Tools:

```bash
xcode-select --install
```

`gpui` uses Metal/Cocoa natively on macOS.

</details>

<details>
<summary>Windows</summary>

Build with the MSVC toolchain (`rustup default stable-x86_64-pc-windows-msvc`)
and the [Visual Studio C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/).
The Windows SDK provides the rest.

</details>

## Adding vgui to Your Project

Add `vgui` (and `gpui`) to your `Cargo.toml`. Both are used as git
dependencies — neither crate is published to crates.io:

```toml
[dependencies]
vgui = { git = "https://github.com/vgerbot-libraries/vgui" }
gpui = { git = "https://github.com/zed-industries/zed" }
gpui-platform = { git = "https://github.com/zed-industries/zed", package = "gpui_platform" }
```

`vgui` can also be used as a path dependency if you have a local checkout.

Then bring the prelude into scope:

```rust
use vgui::prelude::*;
```

The prelude exports `view!`, `css!`, `tw!`, `twc!`, `variants!`, `theme!`, the
reactive primitives (`create_signal`, `create_memo`, `create_effect`,
`create_router`, `ReadSignal`, `WriteSignal`, `Router`, `RouteMatch`), the
`click` helper, `mount`, context API (`Context`, `use_context`,
`use_context_or`, `provide_context`), `NodeRef`, input widget constructors and
props types, styling types (`TwClass`, `TwClassSource`, `IntoTwStyle`,
`Theme`), overlay helpers (`portal`, `floating`), and `Breakpoint`. It also
re-exports `gpui::prelude::*` for convenience.

Verify the build compiles:

```bash
cargo build
```

The first build compiles `gpui` and its graphics backends, so expect a longer
initial compile. Subsequent incremental builds are fast.
