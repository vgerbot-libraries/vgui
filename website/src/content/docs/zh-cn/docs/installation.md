---
title: 安装
description: 为 vgui 准备 Rust nightly 工具链和原生 GPUI 构建依赖。
---

vgui 应用需要 Rust nightly 工具链、C/C++ 构建环境，以及 GPUI 所需的原生库。具体软件包取决于操作系统。

## 安装 Rust 工具

安装 [rustup](https://rustup.rs/)，然后安装 nightly 及开发组件：

```bash
rustup toolchain install nightly
rustup component add rustfmt clippy --toolchain nightly
```

如果准备编译到 WebAssembly，还要安装对应 target：

```bash
rustup target add wasm32-unknown-unknown --toolchain nightly
```

安装 target 只代表准备好编译器，并不表示应用已经通过浏览器运行验证。

仓库的 `rust-toolchain.toml` 选择 `nightly`，但没有固定具体日期。示例清单中的 vgui 和 GPUI Git 依赖也没有固定 `rev`。请将应用的 `Cargo.lock` 纳入版本控制，以便在主动更新之前持续使用同一组依赖解析结果。

## 原生系统依赖

vgui 从 GPUI 及相关原生 crate 继承窗口系统、图形、字体、音频和构建时依赖要求。

### Debian 与 Ubuntu

安装构建工具和开发软件包：

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

原生构建会用到 `pkg-config` 和 `libclang`。GPUI 后端需要 X11、Wayland、OpenGL、字体和桌面相关库。

### macOS

安装 Xcode Command Line Tools：

```bash
xcode-select --install
```

GPUI 在 macOS 上使用原生 Cocoa 和 Metal 平台。这里说明的是前置要求，并不表示 vgui 的全部功能已在 macOS 上验证。

### Windows

使用 MSVC Rust 工具链，并安装带 C++ 工作负载和 Windows SDK 的 [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)。GPUI 在 Windows 上使用 Win32 和 DirectX 平台。安装前置要求不等于 Windows 上的功能已经验证。

## 选择源码使用方式

创建新应用时，使用[快速开始](../quick-start/)中的 Git 依赖，不需要将 vgui 克隆到应用内部。

开发 vgui 本身时，克隆仓库并构建 workspace：

```bash
git clone https://github.com/vgerbot-libraries/vgui.git
cd vgui
cargo build --workspace
```

首次构建会编译 GPUI 及其图形后端，因此可能比后续构建慢得多。

## 下一步

继续阅读[快速开始](../quick-start/)，创建计数器应用。如果编译器或窗口系统报错，请参阅[兼容性与故障排查](../compatibility-troubleshooting/)。
