---
title: 计数器
description: 了解维护中的原生 vgui 计数器及其 signal 驱动的更新流程。
---

本示例使用与[快速开始](../../quick-start/)及维护中的 `crates/vgui/examples/counter.rs` 原生示例相同的完整程序，不会再引入另一种计数器实现。

## 完整的原生示例

以下是完整的 `src/main.rs`。所需清单、项目设置和运行说明请参阅[快速开始](../../quick-start/)。

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

## 跟踪响应式更新

1. `create_signal(0i32)` 返回 `count` 读取句柄和 `set_count` 写入句柄。
2. 构造标签时，`count.get()` 读取当前整数。在响应式渲染作用域中，这次读取会建立视图对 signal 的依赖。
3. `click` 将闭包适配为 GPUI 点击监听器。闭包通过应用上下文调用 `set_count.update`，原地递增整数。
4. signal 的值变化后，vgui 会通知依赖方并请求重新渲染已挂载的根视图。下一次渲染读取新的计数并构造更新后的标签。

signal 在已挂载的响应式视图中创建，而不是在 `main` 中创建。重复渲染会复用它的位置式状态，不会把计数重置为零。请在多次渲染之间保持响应式原语的创建顺序和类型稳定。

写入经 `PartialEq` 比较后相等的值不会触发 signal 通知。这与[Store 与列表](../store-and-lists/)中介绍的 store 行为不同。

## 原生挂载与样式

`application()` 创建原生 GPUI 应用，`run` 启动其事件循环。`open_window` 配置原生窗口，`vgui::mount` 则将 `app` 安装为窗口的根视图。

`class` 属性使用 vgui 支持的 Tailwind 风格工具类为 GPUI 元素设置样式。它们不需要浏览器样式表，这些标记也不会创建浏览器 DOM 节点。

## 验证边界与后续阅读

这个维护中的程序会作为原生 workspace 示例进行编译检查。该检查不能证明窗口已经显示或 Increment 按钮已被点击，也不属于浏览器交互覆盖。

接下来阅读[第一个响应式视图](../../first-reactive-view/)，了解依赖跟踪、memo、effect 和更新问题排查。原生与浏览器各自需要哪些验证，请参阅[原生与 Web](../../native-and-web/)。
