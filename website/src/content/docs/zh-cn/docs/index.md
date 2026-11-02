---
title: 简介
description: 了解 vgui 是什么、它与 GPUI 的关系，以及从哪里开始学习。
---

vgui 是一个基于 [GPUI](https://github.com/zed-industries/zed) 的实验性 Rust 声明式 GUI 框架。GPUI 是 Zed 使用的 GPU 加速 UI 工具包。vgui 将 GPUI 的窗口与渲染能力和接近 Web 的界面编写方式结合起来。

## vgui 提供什么

- **声明式视图：**`view!` 宏支持元素、属性、插值、片段、组件，以及 `Show`、`For` 等控制流。
- **响应式状态：**signal、memo 和 effect 跟踪依赖并更新应用状态，不依赖浏览器 DOM。
- **Rust 样式工具：**在支持的范围内使用 `css!`、Tailwind 风格 class、动态 class、主题和组件变体。
- **桌面与输入基础设施：**基于 GPUI 的窗口、事件、文本输入、焦点、浮层和无障碍集成构建界面。

vgui 不是 Web 框架，也不是浏览器 DOM 兼容层。它的标记会编译为 GPUI 元素；所支持的类 HTML 语法、CSS 属性和 Tailwind 工具类都是有特定行为的子集。

## vgui 适合吗

如果你想探索 GPUI 上的声明式 Rust 界面、为原生应用制作原型，或尝试 signal 驱动的 UI 架构，vgui 值得尝试。

它目前仍是**早期实验性软件**。API 可能在不同版本之间变化，项目使用 Git 依赖而非稳定的 crates.io 发布。不要期待完整的 Web API 兼容性，也不要把未验证平台上的行为视为稳定保证。

## 从这里开始

按顺序阅读以下页面，构建一个小型计数器应用：

1. [安装](./installation/)：准备 Rust 和原生构建依赖。
2. [快速开始](./quick-start/)：从空目录创建完整应用。
3. [第一个响应式视图](./first-reactive-view/)：理解计数器的状态和更新流程。
4. [原生与 Web](./native-and-web/)：了解两种目标及当前验证边界。
5. [兼容性与故障排查](./compatibility-troubleshooting/)：诊断工具链、依赖和窗口问题。

完整的 Rust 类型与函数签名请查阅 [API 文档](../../api/vgui/)。指南负责讲解任务，Rustdoc 仍是 API 参考。
