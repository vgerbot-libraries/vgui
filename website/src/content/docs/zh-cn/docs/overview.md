---
title: 样式概览
description: 在 vgui 工具类、CSS 声明、动态 class 组合、主题和类型化组件变体之间进行选择。
---

vgui 在 Rust 中为 GPUI 元素设置样式。工具类和类 CSS 声明会转换为对 `gpui::StyleRefinement` 的修改；vgui 不会创建浏览器样式表、DOM 节点或 CSS 层叠。Tailwind 风格的命名只是一种编写方式，并不保证完整兼容 Tailwind 或浏览器 CSS。

这些页面使用的样式宏和类型都可以从 `vgui::prelude::*` 导入。

## 选择样式工具

| 工具 | 适用场景 | 求值方式 |
| --- | --- | --- |
| `class="..."` | 在 `view!` 中使用字面量工具类字符串，并需要交互、响应式、动画或过渡处理。 | `view!` 在编译时通过 `tw!` 处理字面量。 |
| `style={css! { ... }}` | 更适合 CSS 声明，需要受支持的 Rust 插值，或使用 `var()` 主题值。 | `css!` 宏在编译时检查声明和值。 |
| `tw!("...")` | 需要显式构造 `TwStyle`，而不是让 `class` 自动处理。 | 字面量走编译时路径；非字面量表达式委托给 `IntoTwStyle`。 |
| `class={twc!(...)}` | 组合带条件的 class 片段。 | `twc!` 构建 `TwClass`；`class` 在运行时将其转换为 `TwStyle`。 |
| `class={classes}` 或 `tw_dynamic(classes)` | 完整 class 字符串只能在运行时确定。 | 运行时解释器在构建样式时解析受支持的工具类。`tw_dynamic` 是函数，不是宏。 |
| `variants!` | 组件具有颜色、尺寸等类型化维度，并共享基础样式。 | 宏生成枚举、变体结构体、builder 和 `ApplyStyle`。 |

对于普通元素，应优先使用 `class`，而不是通过 `style` 应用 `tw!`。`class` 集成负责将 hover/focus/active 闭包、断点和动画配置接入元素；`TwStyle` 的通用 `ApplyStyle` 实现只应用 base 闭包。

## 从静态工具类开始

```rust
use vgui::prelude::*;

fn card() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-3 p-4 w-[320px] rounded-lg bg-white shadow-md">
            <span class="text-lg font-semibold text-gray-900">{"Account"}</span>
            <button class="px-4 py-2 rounded bg-blue-600 hover:bg-blue-700 text-white">
                {"Open"}
            </button>
        </div>
    }
}
```

不需要安装 Tailwind，也不需要生成 CSS 文件。受支持的类名会直接转换为 GPUI 样式字段。未知工具类会被静默忽略，因此拼写错误可能表现为样式缺失，而不是编译错误。

工具类类别、运行时组合、断点和 `variants!` 请参阅 [Tailwind 风格类与变体](../tailwind-and-variants/)。

## 声明更清晰时使用类 CSS 语法

```rust
let accent = gpui::rgb(0x2563eb);

let element = view! {
    <button
        style={css! {
            padding: 8px 16px;
            border-radius: 6px;
            background: {accent};
            color: #ffffff;
        }}
        hover={css! { opacity: 0.85; }}
        focus={css! { border-width: 2px; border-color: #111827; }}
    >
        {"Save"}
    </button>
};
```

`css!` 支持一组由 GPUI 支撑的属性和值，包括布局、间距、尺寸、颜色、排版、边框和部分视觉效果。受支持的 `{expr}` 插值可为目标属性传入 Rust 值。与未知工具类不同，未知 CSS 属性和不支持的值会产生编译错误。

伪选择器不写在 `css!` 内。应像上例一样使用元素的 `hover`、`focus` 和 `active` 属性，或在 `class` 中使用对应的工具类前缀。

主题变量使用相同的声明语法：

```rust
let themed = css! {
    --accent: #2563eb;
    color: var(--accent);
    background: var(--surface, #ffffff);
};
```

局部自定义属性为当前块提供默认值；已安装的 `Theme` 可以覆盖它。[主题与动画](../theming-and-animation/)介绍解析、作用域主题、响应式切换以及当前动画限制。

## 组合运行时 class

```rust
let selected = true;
let classes = twc!(
    "px-4 py-2 rounded",
    selected.then_some("bg-blue-600 text-white"),
    (!selected).then_some("bg-gray-200 text-gray-900")
);

let button = view! {
    <button class={classes}>{"Filter"}</button>
};
```

`twc!` 接受 class 字符串、可选 class 来源和嵌套的 `TwClass` 值。每个变体内的类按顺序应用；两个类写入同一个 GPUI 字段时，后写入的值生效。普通 `String` 或 `&str` 表达式也可以传给 `class`，`tw_dynamic("flex p-4")` 则直接公开运行时解释器。

## 牢记 GPUI 边界

- 样式支持是针对 GPUI 选择的子集，不是浏览器兼容层。
- class 和声明不会形成外部浏览器样式表规则那样的全局层叠。
- 静态和动态工具类共享受支持的类解析器，但动态字符串也不能使用 vgui 未实现的工具类。
- 动画和过渡名称映射到 GPUI 的命令式动画 API，行为范围比 CSS 更窄。

继续阅读 [Tailwind 风格类与变体](../tailwind-and-variants/)或[主题与动画](../theming-and-animation/)。准确的 Rust 签名请参阅 [vgui API 文档](../../../api/vgui/)。
