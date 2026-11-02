---
title: 第一个响应式视图
description: 理解 vgui 计数器中的 signal 读取、写入、依赖跟踪和渲染。
---

[快速开始](../quick-start/)中的计数器使用了一个小型响应式状态图。本页分析该完整程序的摘录，不引入第二个示例。

## 创建并读取状态

```rust
let (count, set_count) = create_signal(0i32);
```

`create_signal` 返回两个句柄：

- `count` 是读取句柄。调用 `count.get()` 会克隆当前值，并在依赖跟踪处于活动状态时记录这次读取。
- `set_count` 是写入句柄。它提供 `set`、`update` 等操作，通过 GPUI 应用上下文修改值。

视图在构造文本时读取 signal：

```rust
<span>{format!("count = {}", count.get())}</span>
```

signal 在 vgui 的响应式渲染作用域中创建。它们的状态缓存在位置式 slot 中，因此重复渲染会复用现有值，而不会重置。请在多次渲染之间保持响应式原语的创建顺序和类型稳定，不要根据条件改变 signal、memo 或 effect 的调用顺序。

## 从事件更新状态

```rust
on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
```

`click` 辅助函数将闭包适配为 GPUI 点击监听器。点击按钮时，`update` 通过 `cx` 修改整数。如果值发生变化，vgui 会通知依赖方并请求重新渲染根视图。通过 `PartialEq` 比较后相等的设置或写入不会触发该通知。

这不是虚拟 DOM diff。memo 和 effect 跟踪各自的 signal 依赖，而 signal 通知最终会调用已挂载根视图的 `cx.notify()`。

## 派生值与副作用

`create_memo` 根据被跟踪的 signal 读取计算并缓存派生值。依赖变化时它会重新计算，并避免发布相等的结果。

`create_effect` 同样跟踪 signal 读取，但它执行副作用而不是返回一个值。它的闭包在渲染期间注册时运行一次，并在被跟踪的依赖变化时重新运行。响应式资源属于各自的作用域；开始使用定时器或订阅时，需要注意作用域清理。

第一个计数器不需要 memo 或 effect：视图可以直接根据 `count` 派生显示字符串。

## 状态没有更新时检查什么

1. 在应当依赖该 signal 的视图、memo 或 effect 中读取它。
2. 使用写入句柄和适当的应用上下文修改它。
3. 在多次渲染之间保持响应式原语调用顺序稳定。
4. 不要期待浏览器 DOM 节点、虚拟 DOM 或按文本节点的更新。

完整签名请参阅 [vgui API 文档](../../../api/vgui/)，然后继续阅读[原生与 Web](../native-and-web/)，了解平台边界。
