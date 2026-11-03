---
title: 引用与上下文
description: 使用 NodeRef 绑定 GPUI 焦点和滚动句柄，并通过 Context 与 Provider 传递同步构建期的值。
---

`NodeRef` 和 `Context` 解决不同问题。ref 提供对 GPUI 焦点、几何信息和滚动状态的命令式访问。Context 则在 Provider 后代构建期间提供克隆的 Rust 值。两者都不是 DOM 引用、通用事件生命周期存储或响应式状态的替代品。

## NodeRef 是 GPUI 句柄

```rust
let node_ref = NodeRef::new();

let element = view! {
    <div ref={node_ref.clone()}>{"Content"}</div>
};
```

`NodeRef::new()` 和 `NodeRef::default()` 创建未绑定外壳。克隆共享同一绑定，因此可以将一个克隆放入标记，并在事件处理器中保留另一个。绑定会安装 GPUI `FocusHandle` 和 `ScrollHandle`；它不会暴露 HTML 元素或任意 DOM 操作。

`is_bound()` 可以在附加前安全调用，并返回是否存在绑定。其他 ref 方法在未绑定时会 panic。在较早事件或条件渲染可能使用 ref 时，请先检查该标志。

## 焦点与滚动

```rust
let list_ref = NodeRef::new();
let list_for_click = list_ref.clone();

view! {
    <div>
        <div ref={list_ref.clone()} class="h-48 overflow-y-scroll">
            {"Items"}
        </div>
        <button on:click={move |_event, window, cx| {
            if list_for_click.is_bound() {
                list_for_click.focus(window, cx);
                list_for_click.scroll_to_bottom();
            }
        }}>
            {"Scroll to bottom"}
        </button>
    </div>
}
```

`focus` 请求 GPUI 焦点。焦点查询按方法签名需要 GPUI window/app 上下文，而滚动和几何操作使用已存储的句柄。

| 方法 | 用途 |
| --- | --- |
| `is_bound()` | 测试 ref 是否已附加。 |
| `focus(window, cx)` | 聚焦附加元素。 |
| `is_focused(window)` | 测试其焦点句柄是否聚焦。 |
| `contains_focused(window, cx)` | 测试它是否包含已聚焦元素。 |
| `bounds()` | 返回附加元素记录的边界。 |
| `scroll_offset()` | 读取当前滚动偏移。 |
| `scroll_to(ix)` | 将 GPUI 列表句柄滚动到项目索引。 |
| `scroll_to_top(ix)` | 滚动以使索引项目位于顶部。 |
| `scroll_to_bottom()` | 滚动到底部。 |
| `set_scroll_offset(point)` | 设置像素滚动偏移。 |
| `child_bounds(ix)` | 读取索引子元素记录的边界。 |
| `child_count()` | 读取记录的子元素数量。 |
| `focus_handle()` | 访问底层 GPUI 焦点句柄。 |
| `scroll_handle()` | 访问底层 GPUI 滚动句柄。 |

边界和子元素测量来自上一绘制帧，而不是新的同步布局过程。请据此处理初始或刚变化的几何信息。滚动操作还取决于附加的 GPUI 元素及其滚动/列表行为；`NodeRef` 不会把每个容器都变成虚拟列表。

## 内置元素的 ref 支持

接受 `ref` 的通用元素，以及 `form`、`label` 和按钮/复选框/单选按钮/文件输入外层，都可以绑定 `NodeRef`。entity 支撑的文本和范围实现不会绑定直接 `ref`；`select` 和 `textarea` 会拒绝该属性。需要这些控件的外层句柄时，请使用受支持的外层元素。

```rust
let input_wrapper = NodeRef::new();

view! {
    <div ref={input_wrapper.clone()}>
        <input type="text" value={name.get()} />
    </div>
}
```

外层 ref 指向外层元素，而不是文本编辑器内部的 GPUI entity。值处理请按[输入与表单](../inputs-and-forms/)所述使用输入回调和应用状态。

## Context 以 Rust 类型为键

```rust
#[derive(Clone, Default)]
struct Palette(&'static str);

static PALETTE: Context<Palette> = Context::new();
```

`Context<T>` 是零大小、可 const 构造的键。上下文查找使用 `TypeId`，因此每个 `T` 有一个逻辑上下文，而不是每个 static 变量一个。两个 `Context<String>` static 仍指向相同类型键。如果不同值本来具有相同 Rust 类型，请使用 `Palette` 这样的 newtype。

值必须可克隆，因为查找会返回拥有的克隆。Context 不携带隐式 signal 跟踪或更新订阅。

## Provider 与 use_context

`Provider` 只接受 `context` 和 `value`。它在布局上透明：不添加 div、样式、ref 或事件目标。构建子元素时，值被压入线程局部 Provider 栈；构建完成后弹出。

```rust
#[derive(Clone, Default)]
struct Palette(&'static str);

static PALETTE: Context<Palette> = Context::new();

fn label() -> impl gpui::IntoElement {
    let palette = use_context_or(&PALETTE, Palette::default);
    view! { <span>{palette.0}</span> }
}

fn app() -> impl gpui::IntoElement {
    view! {
        <Provider context={PALETTE} value={Palette("dark")}>
            <div>{label()}</div>
        </Provider>
    }
}
```

调用 `use_context(&PALETTE)` 可获取 `Option<Palette>`，即最近匹配值的克隆。没有值时，`use_context_or` 提供回退值。消费函数必须像上面的 `label()` 一样在后代构建期间运行；把工作留到以后的点击、异步任务或无关渲染中不会保留 Provider 栈。

独立 Provider 支持零个或一个直接子元素。请像示例一样把多个兄弟元素包在一个显式容器中，以保持构建边界清晰。

## 嵌套 Provider 与遮蔽

相同 `T` 的 Provider 会为其后代遮蔽外层值：

```rust
view! {
    <Provider context={PALETTE} value={Palette("outer")}>
        <div>
            <span>{use_context(&PALETTE).unwrap().0}</span>
            <Provider context={PALETTE} value={Palette("inner")}>
                <span>{use_context(&PALETTE).unwrap().0}</span>
            </Provider>
        </div>
    </Provider>
}
```

外层查找克隆 `"outer"`，内层查找克隆 `"inner"`。每个 Provider 完成子元素构建后，其条目会被移除；边界外的查找看不到它。

## 手动 provide_context 作用域

命令式 Rust 构建可以不使用 `Provider` 元素而采用相同机制：

```rust
let subtree = {
    let _guard = provide_context(&PALETTE, Palette("high-contrast"));
    label()
};
```

`provide_context` 返回 `ProviderGuard`。构建所有预期后代时保持 guard 存活，然后 drop 它以弹出值。上面的显式作用域使该生命周期清晰可见。不要将 guard 移入事件处理器或异步生命周期，以此让上下文持久存在。

## Context 与响应式状态

Context 回答的是“同步构建期间最近的值是什么？”它并不回答“这个值如何随时间更新？”替换 Provider 值本身不会通知消费者，克隆的上下文值也不会自动订阅其来源。

使用 Context 传递后代需要的构建期配置、服务句柄或 signal 句柄。必须发布变化的值请使用 [signal、memo 和 effect](../reactivity/)。Provider 应留在构建线程和 Provider/guard 生命周期内；不要把它当作全局、异步或跨线程存储。

完整方法签名请参阅 [vgui API 文档](../../../api/vgui/)。
