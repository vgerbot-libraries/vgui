---
title: 导航与布局
description: 使用当前 vgui-shadcn Rust API 组合面包屑、分页、标签页和可拖拽调整大小的分栏。
---

本指南介绍四个已实现的 `vgui-shadcn` 组件。代码片段用于 vgui 响应式渲染闭包，不是完整应用。它们假定已有以下 import，并已在该闭包中调用 `apply_theme(false)`。

```rust
use gpui::IntoElement;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;
```

## 组件形式

带有 `#[vgui_component]` 的函数会生成 PascalCase 组件，例如 `Breadcrumb { ... }` 和 `Pagination { ... }`。普通函数保留 snake_case 名称：可调整分栏使用 `h_resizable(...)` 或 `v_resizable(...)`。这些示例使用 Rust 结构体表达式；`view!` 也支持相应的 PascalCase 标签形式。两种形式都不是上游 shadcn 组件的直接替代品。

## 面包屑（Breadcrumb）

`Breadcrumb` 接受 `Vec<BreadcrumbItem>`。每一项都是 `(String, Option<Box<dyn Fn(&mut gpui::App)>>)` 二元组。最后一项始终渲染为不可交互的当前页文本；前面的项只有带回调时才会渲染为按钮。

```rust
let home: BreadcrumbItem = (
    "Home".into(),
    Some(Box::new(|_cx| {})),
);
let breadcrumb = Breadcrumb {
    items: vec![
        home,
        ("Components".into(), None),
        ("Navigation".into(), None),
    ],
};

view! { <div>{breadcrumb}</div> }
```

空回调展示了所需的类型形式；请将其替换为所属视图的导航操作。前面的项如果为 `None`，会保留为纯文本，因此缺少回调并不表示禁用链接。分隔符会自动插入，空间不足时整行会换行。

## 分页（Pagination）

`Pagination` 是受控组件：传入当前 `page`、`page_count` 和 `on_change` 回调。请在应用状态中保存页码，并在回调中更新它。

```rust
let (page, set_page) = create_signal(5usize);
let pagination = Pagination {
    page: page.get(),
    page_count: 20,
    on_change: Box::new(move |next, cx| set_page.set(cx, next)),
};

view! { <div>{pagination}</div> }
```

页数不超过 7 时完整显示，不进行折叠。更大的范围保留第一页和最后一页，显示当前页附近的窗口，并插入不可交互的省略号。例如，第 5 页、共 20 页时显示 `1 … 4 5 6 … 20`。到达相应边界时，上一页和下一页回调不会执行。内部会将 `page` 和 `page_count` 分别钳制到至少 1；超过范围的大页码不会再规范化到 `page_count`。

## 标签页（Tabs）

标签页组件族提供布局和可点击触发器，而不是自动管理选中状态的状态机。`Tabs` 包含 `TabsList` 以及你提供的任意面板内容。每个 `TabsTrigger` 分别接收 `active` 标志和回调。

```rust
let (active, set_active) = create_signal("account");
let account = TabsTrigger {
    active: active.get() == "account",
    on_click: Box::new({
        let set_active = set_active.clone();
        move |cx| set_active.set(cx, "account")
    }),
    children: vec![view! { <span>{"Account"}</span> }.into_any_element()],
};
let password = TabsTrigger {
    active: active.get() == "password",
    on_click: Box::new(move |cx| set_active.set(cx, "password")),
    children: vec![view! { <span>{"Password"}</span> }.into_any_element()],
};
let panel = if active.get() == "account" {
    view! { <span>{"Account settings"}</span> }.into_any_element()
} else {
    view! { <span>{"Password settings"}</span> }.into_any_element()
};
let tabs = Tabs {
    children: vec![
        TabsList {
            children: vec![account.into_any_element(), password.into_any_element()],
        }
        .into_any_element(),
        panel,
    ],
};

view! { <div>{tabs}</div> }
```

条件面板属于应用代码。此实现没有 `TabsContent` 组件、基于值的触发器注册或内置方向键导航；不要根据上游标签页组件推断这些行为。

## 可调整大小的分栏（Resizable）

`h_resizable(left, right, initial_left, total_width)` 创建带可拖拽分隔条的水平分栏。`v_resizable(top, bottom, initial_top, total_height)` 提供对应的垂直分栏。两者都接受 `AnyElement` 子元素和像素尺寸。

```rust
let split = h_resizable(
    view! { <div>{"Navigator"}</div> }.into_any_element(),
    view! { <div>{"Main content"}</div> }.into_any_element(),
    240.0,
    800.0,
);

view! {
    <div style={css! { height: 320px; }}>
        {split}
    </div>
}
```

第一个面板初始为 240 px，第二个面板填充剩余空间。拖拽 6 px 分隔条会更新内部 signal；实现使用 48 px 最小尺寸常量钳制拖拽位置，并隐藏两个面板的溢出内容。总宽度或总高度是显式的固定像素值，而不是对响应式父容器的测量。原生 headless 交互测试覆盖了水平分隔条拖拽；这并不能证明浏览器或所有平台上的指针行为。

## 当前限制

这些 API 实现了 shadcn 风格行为的一个子集，不声称具备完整的上游功能对等性、无障碍认证或尚未验证的浏览器交互支持。特别是，面包屑导航、标签页选择和分页数据仍由应用负责。

返回[组件概览](../)。
