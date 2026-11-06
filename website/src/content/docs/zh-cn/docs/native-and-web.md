---
title: 原生与 Web
description: 了解 vgui 的原生和 WebAssembly 目标、实时计数器预览及浏览器验证。
---

vgui 提供原生 GPUI 集成和 WebAssembly 专用支持，两者使用不同的入口。维护中的计数器展示了双端实现，原生与浏览器行为仍需要分别验证。

## 当前能力与验证范围

| 路径 | 已具备的内容 | 不能据此推断的内容 |
| --- | --- | --- |
| 原生计数器 | 维护中的双端计数器使用 `gpui_platform::application()`，可通过 `cargo run -p vgui --example counter` 运行。 | 文档不声称已验证原生窗口显示或按钮交互。 |
| WebAssembly target | 同一个计数器使用 `single_threaded_web()`、`run_embedded`、`web_init()` 和浏览器键盘事件拦截。网站构建会生成其预编译 WASM 资源。 | 仅构建 WASM 不会执行浏览器渲染或输入验证。 |
| 浏览器 demo | [计数器页面](../examples/counter/)并排展示真实的只读源码与实时预编译 WASM 预览。网站浏览器测试覆盖 canvas 渲染和 Increment 交互。 | 这不能验证原生 UI 交互或所有浏览器平台集成。 |

## 原生应用

[快速开始](../quick-start/)采用原生模型：

1. 创建当前平台的 GPUI 应用。
2. 运行原生事件循环。
3. 打开 GPUI 窗口。
4. 将 vgui 根视图挂载到该窗口。

标记仍会生成 GPUI 元素。元素不会变成浏览器 DOM 节点，窗口行为来自原生 GPUI 后端。

## 浏览器中有哪些不同

浏览器宿主需要处理一些与原生窗口不同的问题：

- **入口与生命周期：**WASM 启动是非阻塞的，并使用 Web 平台，而不是快速开始中的原生 `application().run` 路径。
- **canvas 尺寸：**宿主页面和 canvas 决定可用视口。固定的原生窗口大小不是响应式浏览器布局。
- **键盘行为：**Tab 导航、空格滚动等浏览器默认行为可能与 canvas 输入冲突。vgui 提供 `intercept_keyboard_events()`，为浏览器 canvas 安装冒泡阶段的键盘监听器；在原生目标上该函数不执行操作。
- **输入与平台行为：**文本输入、IME、剪贴板、无障碍及其他操作系统集成都需要单独的浏览器验证。

计数器的 WASM `start()` 将 `web_init()`、键盘事件拦截和 `run_embedded` 组合为预览所使用的完整启动路径。

## 验证边界

计数器页面展示维护中的源码，并加载由网站构建生成的预编译 WASM 资源。网站浏览器测试覆盖 canvas 渲染和 Increment 点击交互。这些检查不验证原生窗口显示或原生按钮交互。

请将原生编译、原生交互、WASM 编译、浏览器渲染和浏览器交互视为相互独立的检查。计数器的浏览器覆盖本身不能验证文本输入、IME、剪贴板、无障碍或其他集成。

构建或显示问题请继续阅读[兼容性与故障排查](../compatibility-troubleshooting/)。
