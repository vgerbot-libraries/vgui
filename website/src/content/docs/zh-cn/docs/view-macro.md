---
title: view! 宏
description: 使用 vgui 元素、属性、插值、组件和控制流构建 GPUI 元素树。
---

`view!` 在编译期将类似 JSX 的 Rust token 转换为 GPUI 元素构建器表达式。它提供声明式编写语法，不是 HTML 运行时或浏览器 DOM 抽象。

## 从单个根节点开始

一次 `view!` 调用接受一个根元素、片段或插值表达式。以下[快速开始](../quick-start/)摘录组合了文本、属性和事件：

```rust
view! {
    <div class="flex flex-col gap-3 p-4 text-white">
        <span>{format!("count = {}", count.get())}</span>
        <button
            class="p-2 bg-[#0000ff] hover:bg-[#000088] rounded"
            on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
        >
            {"Increment"}
        </button>
    </div>
}
```

宏会生成普通 Rust 表达式。视图中使用的 `count`、`set_count` 等值遵循正常的 Rust 捕获和所有权规则。

## 插入文本和 Rust 表达式

固定文本使用带引号的字面量，表达式使用大括号：

```rust
view! {
    <div>
        "Plain text"
        <span>{format!("Hello, {}", name)}</span>
        {badge("New".to_string())}
    </div>
}
```

插值表达式通过 `vgui::into_child` 转换。它们可以生成文本、元素、组件或其他受支持的视图子节点。`{|| { /* Rust code */ view! { <div>"result"</div> } }}` 这类代码块只有在调用闭包后才是结果表达式；单独的 `{|| ...}` 仍然是闭包值。

片段可以在不使用具名元素的情况下组合多个节点：

```rust
view! {
    <>
        <span>"First"</span>
        <span>"Second"</span>
    </>
}
```

当片段或控制流是元素的直接子节点时，其子节点会直接添加到父元素，不会经过额外的控制流包装层。

## 选择内置元素或组件

标签拼写决定展开方式：

- `<div>`、`<button>`、`<input>` 等小写标签选择 vgui 内置元素。大多数通用容器会成为具有标签特定默认行为的 GPUI `div` 构建器；输入框等专用元素会构建组件。
- 大写标签选择组件。属性和子节点会成为结构体字段，而两者都没有的标签会成为函数调用。
- `<Show>`、`<For>`、`<Index>`、`<Switch>` 和 `<Match>` 会被识别为控制流，而不是普通组件。

这些类似 HTML 的名称并不表示完整的浏览器 HTML 行为。请使用 vgui 实现的属性和元素行为。组件调用请参阅[自定义组件](../custom-components/)，特殊元素请参阅[控制流](../control-flow/)。

## 传递属性

每个属性都需要 `=`。可以使用字面量、大括号中的 Rust 表达式、裸标识符表达式或布尔字面量：

```rust
view! {
    <div
        id="summary"
        class="flex flex-col gap-2"
        style={css! { display: flex; gap: 8px; }}
        tabindex={0}
        role="region"
        aria:label="Summary"
    >
        {content}
    </div>
}
```

常用的内置属性包括：

| 形式 | 用途 |
| --- | --- |
| `class="..."` | 应用 vgui 支持的 Tailwind 风格工具类，包括受支持的状态和断点变体。 |
| `style={css! { ... }}` | 应用 Rust CSS 样式值。 |
| `hover={...}`、`active={...}`、`focus={...}` | 应用交互状态样式。 |
| `id=...`、`tabindex=...` | 设置元素标识和焦点行为。 |
| `role=...`、`aria:name=...` | 设置受支持的无障碍元数据。 |
| `ref={node_ref}` | 在支持的位置绑定 `NodeRef`。 |

class 和 CSS 支持是具有 GPUI 行为的 vgui 子集，不是浏览器样式表。不支持的属性或事件名称会产生编译错误，而不会作为 HTML 静默转发。

## 处理事件

内置事件使用 `on:event={handler}`。传入事件所需的闭包形式，或使用 vgui 提供的适配器：

```rust
on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
```

这里的 `click` 适配接收 `&mut App` 的闭包。直接使用 GPUI 点击监听器时，闭包会接收点击事件、窗口和应用上下文。组件事件可能有不同签名，例如文本输入回调接收变化后的文本和应用上下文。添加新事件时，请查看元素的 Rust API。

事件闭包在创建它的渲染表达式之外运行。`move` 闭包需要的 signal 或写入句柄应先克隆。

## 使用展开语法转发属性

`{..props}` 和 `{...props}` 都可以表示一个展开值。在组件上，展开会成为 Rust 结构体更新语法：

```rust
view! { <CardHeader {..props} title={"Override"} /> }
```

显式组件字段会生成在 `..props` 之前，因此具名字段会覆盖基础结构体提供的值。只允许一个展开值。在受支持的内置元素上，值通过 `vgui::Spread` 应用；属性类型必须为具体元素类型实现该 trait。展开不会绕过正常的类型和属性检查。

## 注意语法约束

- 只提供一个根节点，并用匹配标签关闭每个非空元素。
- `<input>`、`<br>`、`<hr>` 和 `<wbr>` 可以省略自闭合斜杠；其他元素需要 `/>` 或关闭标签。
- 动态子节点和属性使用 `{expr}`，不要使用 JSX 字符串或 JavaScript 值。
- 控制流元素有各自的必需属性和子节点结构；请参阅[控制流](../control-flow/)。
- 计算视图时进行的响应式调用仍遵循位置式作用域规则；请参阅[响应式](../reactivity/)和[架构](../architecture/)。

生成的值实现 GPUI 的元素转换 trait，可以从渲染函数返回或作为子节点传递。根渲染如何使用该值，请参阅[架构](../architecture/)。
