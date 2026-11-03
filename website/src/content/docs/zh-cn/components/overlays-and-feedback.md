---
title: 浮层与反馈
description: 使用当前 vgui-shadcn API 控制对话框和锚定弹出层，并渲染警告与确定性进度条。
---

本指南介绍 `vgui-shadcn` 已实现的两个浮层组件和两个反馈组件。代码片段用于 vgui 响应式渲染闭包，不是完整应用。它们假定已有以下 import，并已在该闭包中调用 `apply_theme(false)`。

```rust
use gpui::IntoElement;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;
```

## 组件形式

下面的 PascalCase 值由带有 `#[vgui_component]` 的函数生成。示例使用 `Popover { ... }` 这样的结构体表达式；`view!` 也支持相应的 PascalCase 标签形式。对话框包装器名为 `DialogComponent`，因为其源函数是 `dialog_component`；可视化标题和描述辅助组件则名为 `DialogTitle` 和 `DialogDescription`。

## 对话框（Dialog）

`DialogComponent` 由 `open` 控制。其 `on_close` 回调请求状态变更；所属视图必须将 `open` 设置为 `false`。子元素是任意 `AnyElement` 值，因此“取消”等操作需要显式组合。

```rust
let (open, set_open) = create_signal(false);
let trigger = {
    let set_open = set_open.clone();
    view! {
        <button on:click={click(move |cx| set_open.set(cx, true))}>
            {"Delete item"}
        </button>
    }
};
let dialog = DialogComponent {
    open: open.get(),
    on_close: Box::new({
        let set_open = set_open.clone();
        move |cx| set_open.set(cx, false)
    }),
    children: vec![
        DialogTitle {
            children: vec![view! { <span>{"Delete item?"}</span> }.into_any_element()],
        }
        .into_any_element(),
        DialogDescription {
            children: vec![
                view! { <span>{"This action cannot be undone."}</span> }.into_any_element(),
            ],
        }
        .into_any_element(),
        view! {
            <button on:click={click(move |cx| set_open.set(cx, false))}>
                {"Cancel"}
            </button>
        }
        .into_any_element(),
    ],
};

view! { <div>{trigger}{dialog}</div> }
```

底层 vgui 对话框提供全屏背景层，阻止鼠标事件到达背景内容。按 Escape 或在内容外按下鼠标会调用 `on_close`。核心焦点处理会在打开时将焦点移入对话框，让 Tab 和 Shift+Tab 在其中循环，并在关闭后恢复之前的焦点；原生 headless 测试覆盖了这些核心行为。`DialogTitle` 和 `DialogDescription` 是带样式的 span，其本身不会建立完整的 ARIA 标签关联。

## 弹出层（Popover）

`Popover` 在 `NodeRef` 锚点下方的浮动层中渲染子元素。使用 `ref={anchor.clone()}` 将同一个 ref 绑定到触发器，并通过应用状态控制可见性。

```rust
let (open, set_open) = create_signal(false);
let anchor = NodeRef::new();
let trigger = {
    let set_open = set_open.clone();
    view! {
        <button
            ref={anchor.clone()}
            on:click={click(move |cx| {
                set_open.update(cx, |is_open| *is_open = !*is_open)
            })}
        >
            {"Project settings"}
        </button>
    }
};
let popover = Popover {
    open: open.get(),
    anchor,
    children: vec![
        view! { <div>{"Choose the settings to display."}</div> }.into_any_element(),
    ],
};

view! { <div>{trigger}{popover}</div> }
```

定位使用锚点上一帧的边界。必要时，浮动层会向窗口内吸附，并保留 8 px 边距；锚点尚未绑定时，回退位置是窗口原点。此 API 没有位置方向选项。`Popover` 也不会添加点击外部关闭、Escape 处理或焦点陷阱，因此需要根据应用需求自行实现并验证交互。

## 警告（Alert）

`Alert` 是静态容器，提供 `AlertVariant::Default` 和 `AlertVariant::Destructive` 样式。可在 `children` 中组合 `AlertTitle`、`AlertDescription` 或其他元素。

```rust
let alert = Alert {
    variant: AlertVariant::Destructive,
    children: vec![
        AlertTitle {
            children: vec![view! { <span>{"Save failed"}</span> }.into_any_element()],
        }
        .into_any_element(),
        AlertDescription {
            children: vec![
                view! { <span>{"Check available storage and try again."}</span> }
                    .into_any_element(),
            ],
        }
        .into_any_element(),
    ],
};

view! { <div>{alert}</div> }
```

破坏性变体会改变边框和前景颜色，但不会添加图标、关闭操作或自动播报。源代码没有设置 ARIA alert 角色或 live region 行为。应用所需的关闭操作和语义必须另行组合并验证。

## 进度条（Progress）

`Progress` 是由 `value` 和 `max` 定义的确定性进度条，两者均为 `f64`。工作推进时，通过应用状态更新源值。

```rust
let (value, set_value) = create_signal(35.0f64);
let progress = Progress {
    value: value.get(),
    max: 100.0,
};

view! {
    <div>
        {progress}
        <button on:click={click(move |cx| {
            set_value.update(cx, |value| *value = (*value + 10.0).min(100.0))
        })}>
            {"Advance"}
        </button>
    </div>
}
```

填充比例为 `(value / max).clamp(0.0, 1.0)`，因此超出范围的值不会溢出轨道。`max` 小于或等于零时渲染空进度条。此实现没有不确定模式、内置文本标签或自动的 progressbar 数值语义。

## 当前限制

这些组件是拥有独立 API 表面的 vgui 实现，不声称与上游 shadcn 完全对等。焦点行为、关闭方式、播报和键盘交互因组件而异；不暗示任何无障碍认证或尚未验证的浏览器交互支持。

返回[组件概览](../)。
