---
title: 事件与输入
description: 处理规范化 vgui 事件和类型化输入回调，同时明确原生与浏览器的测试边界。
---

vgui 在 `view!` 处理器中提供 Web 风格的事件数据，但元素和事件仍来自 GPUI，而不是浏览器 DOM 节点。下面的规范化事件摘录遵循 `crates/vgui/tests/events.rs`；控件摘录遵循 `crates/vgui/tests/widgets_compile.rs` 和当前输入回调类型。

## 添加规范化事件处理器

键盘、指针、尺寸变化、双击、上下文菜单和滚轮处理器接收借用的事件、窗口以及应用上下文。以下重点摘录在一个元素上构建全部九个规范化事件属性：

```rust
let el = view! {
    <div
        on:keydown={move |_e: &KeyboardEvent, _w, _cx| {}}
        on:keyup={move |_e: &KeyboardEvent, _w, _cx| {}}
        on:pointerdown={move |_e: &PointerEvent, _w, _cx| {}}
        on:pointerup={move |_e: &PointerEvent, _w, _cx| {}}
        on:pointermove={move |_e: &PointerEvent, _w, _cx| {}}
        on:resize={move |_e: &ResizeEvent, _w, _cx| {}}
        on:dblclick={move |_e: &PointerEvent, _w, _cx| {}}
        on:contextmenu={move |_e: &PointerEvent, _w, _cx| {}}
        on:wheel={move |_e: &WheelEvent, _w, _cx| {}}
    >
        {"events"}
    </div>
};
let _any = el.into_any_element();
```

使用这些事件字段，而不必依赖 GPUI 原始事件结构体：

- `KeyboardEvent` 提供 `key`、尽力映射的 `code`、`repeat`、`key_char`，以及 `shift_key`、`ctrl_key`、`alt_key`、`meta_key`。未知物理键码会得到空的 `code`。需要停止传播时调用 `e.stop_propagation()`。
- `PointerEvent` 提供窗口坐标系中的 `client_x`、`client_y`，以及 `button`、`buttons`、修饰键标志、`click_count` 和指针元数据。当前指针规范化只填充鼠标数据；存在 `Pen` 和 `Touch` 变体并不表示已经支持触笔或触摸事件。
- `ResizeEvent` 包含窗口视口新的 `width` 和 `height`。它不是逐元素的尺寸观察器，也不会强制元素获得 ID。
- `WheelEvent` 暴露 `delta_x`、`delta_y` 和修饰键标志。滚动增量会规范化为像素；按行计算的增量使用每行 16 像素。

这些名称类似浏览器 API，是为了让事件处理代码更熟悉，并不表示原生视图已经变成 DOM 树。

## 根据文本输入更新状态

文本类 input 和 textarea 使用接收 `(&str, &mut App)` 的 `on:input`。受控文本字段可以把每个新字符串复制到 signal 中：

```rust
let (text, set_text) = create_signal(String::new());

view! {
    <input
        type="text"
        value={text.get()}
        on:input={move |value: &str, cx: &mut App| {
            set_text.set(cx, value.to_string());
        }}
    />
}
```

输入回调接收当前字符串，因此在存储前要将其转换为拥有所有权的 `String`。一般的 signal 更新与渲染行为与[第一个响应式视图](../../first-reactive-view/)相同。

## 处理类型化的选择变化

select 的 `on:change` 回调接收 `&str` 和 `&mut App`。当前编译测试覆盖普通选项、分组选项和多选。以下是其中的多选摘录：

```rust
view! {
    <select
        multiple={true}
        options={vec![
            ("a".to_string(), "Alpha".to_string()),
            ("b".to_string(), "Beta".to_string()),
        ]}
        value={"a,b".to_string()}
        on:change={move |_v: &str, _cx: &mut App| {}}
    />
}
```

多选值是逗号分隔的字符串。其他控件有意使用不同的回调值类型：checkbox 和 radio 的变化回调使用 `bool`，range 使用 `f64`，file 使用 `Vec<std::path::PathBuf>`。请让闭包匹配具体控件，不要假设所有事件都传递字符串。

## 表单与文本区域

表单的 submit 和 reset 回调接收应用上下文。控件编译测试还检查文本输入的验证属性以及 submit/reset 输入类型：

```rust
view! {
    <form on:submit={move |_cx| {}} on:reset={move |_cx| {}}>
        <input type="text" required={true} pattern={"abc"} minlength={1usize} />
        <input type="submit" value="Go" />
        <input type="reset" value="Clear" />
    </form>
}
```

textarea 接受 `u32` 行数，并支持与文本类 input 相同的 `&str` 输入回调形式：

```rust
view! {
    <textarea
        rows={4u32}
        on:input={move |_value: &str, _cx: &mut App| {}}
    />
}
```

## 当前测试能证明什么

事件集成测试在测试渲染作用域内构造带规范化事件的元素，并将其转换为 `AnyElement`；它还检查仅带 resize 的 `div` 没有元素 ID。控件测试对上面展示的控件和回调进行编译检查或构造。尤其是其中的 `textarea` 检查只对视图做类型检查而不构造控件，因为 textarea 需要已挂载根视图中的 slot。

这些属于编译和元素构造检查。它们不会分发原生键盘或指针输入、展开 select、提交表单、编辑 textarea，也不会验证浏览器焦点、IME、canvas 输入或无障碍。原生交互与浏览器交互都需要独立验证；请参阅[原生与 Web](../../native-and-web/)。完整的原生挂载和点击驱动状态更新请参阅[计数器](../counter/)。
