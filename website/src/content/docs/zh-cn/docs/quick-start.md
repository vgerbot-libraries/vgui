---
title: 快速开始
description: 从空目录创建一个完整的原生 vgui 计数器应用。
---

本指南创建一个带单个按钮的原生计数器。示例包含所有 import 和初始化步骤，可以直接复制到新项目，不需要额外的框架设置。

开始之前，请先完成[安装](../installation/)中的前置准备。

## 1. 创建项目

```bash
cargo new vgui-counter --bin
cd vgui-counter
```

## 2. 添加依赖

将 `Cargo.toml` 替换为以下清单。vgui 和 GPUI 目前使用 Git 依赖：

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

`gpui-platform` 提供下面使用的原生应用入口。保留生成的 `Cargo.lock`，使后续构建在主动更新之前使用同一组已解析版本。

## 3. 创建应用

将 `src/main.rs` 替换为以下完整程序：

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

## 4. 运行

```bash
cargo +nightly run
```

首次构建可能需要较长时间，因为 Cargo 还会编译 GPUI 及其原生依赖。请在桌面会话中运行该命令；无显示器的构建环境不能显示窗口。

## 各部分如何协作

- `application()` 为当前原生平台创建 GPUI 应用，`run` 启动事件循环。
- `open_window` 配置一个 500 × 500 的窗口，并将窗口与应用上下文传给 vgui。
- `vgui::mount` 将 `app` 安装为该窗口内渲染的根视图。
- `create_signal` 创建计数，`count.get()` 显示计数，点击处理器通过 `set_count.update` 将其递增。
- `class` 字符串包含 vgui 支持的 Tailwind 风格工具类，不需要浏览器样式表。

同一程序会作为原生 workspace 示例在文档 CI 中编译，以上清单也已在空的外部使用方项目中通过检查。这些都是编译检查，不表示已经验证原生按钮交互或浏览器构建。

接下来阅读[第一个响应式视图](../first-reactive-view/)，理解状态读取、写入和渲染如何协同工作。
