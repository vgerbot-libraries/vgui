---
title: 主题与动画
description: 使用 Theme 解析 css! 变量，通过 signal 驱动的渲染切换主题，并使用 vgui 基于 GPUI 的动画与悬停过渡。
---

vgui 主题为 `css!` 中的 `var()` 引用提供类型化值。动画和过渡工具类使用 GPUI 的命令式动画 API。它们都不是浏览器 CSS：主题变量不形成 DOM 层叠，动态效果也只支持这里描述的属性和行为。

## CSS 变量和默认值

`css!` 内的自定义属性为同一块中的引用定义编译时回退值。也可以在使用处通过 `var(--name, fallback)` 提供回退值：

```rust
let style = css! {
    --accent: #2563eb;
    --gap: 12px;
    color: var(--accent);
    gap: var(--gap);
    background: var(--surface, #ffffff);
};
```

应用样式时，vgui 在当前线程局部的 `Theme` 中查找每个变量。主题值优先于局部或内联回退值。如果主题和回退值都没有提供变量，应用样式时会 panic。值的类型与目标属性不匹配时，也会 panic 并给出描述性错误。

在接收方 `css!` 属性支持 `var()` 的前提下，变量支持类型化的颜色、长度、数字和关键字。例如，关键字可以驱动 `flex-direction`，长度可以驱动 `gap`，数字可以驱动 `opacity`。`tw!` 和运行时工具类字符串不是主题变量的使用路径；`var()` 应通过 `css!` 使用。

## 安装主题和限定作用域

使用 `theme!` 构建主题，再通过 `set_theme` 安装：

```rust
let light = theme! {
    --accent: #2563eb;
    --surface: #ffffff;
    --foreground: #111827;
};

set_theme(light);

let element = view! {
    <span style={css! {
        padding: 8px;
        color: var(--foreground);
        background: var(--surface);
    }}>
        {"Themed text"}
    </span>
};
```

`theme!` 只接受自定义属性声明。对于运行时计算的值，使用类型化的 `Theme` builder；名称不包含开头的 `--`：

```rust
let mut custom = Theme::new();
custom.set_color("accent", gpui::rgb(0x2563eb));
custom.set_number("muted-opacity", 0.5);
set_theme(custom);
```

`set_theme` 替换当前线程的主题。它不会附着到 GPUI 子树，也不会向子元素提供浏览器式继承。

使用 `with_theme` 可以在元素构造过程周围建立动态作用域：

```rust
let contrast = theme! {
    --accent: #ffff00;
};

let element = with_theme(&contrast, || {
    view! {
        <span style={css! { color: var(--accent); }}>
            {"Scoped theme"}
        </span>
    }
});
```

`with_theme` 为闭包安装一个克隆，并在闭包返回后恢复之前的主题。它限定的是构造样式时的变量解析作用域，不是持久存在的主题 provider 组件。

## 通过 signal 切换主题

**`set_theme` 不会自动响应式更新。**它不会通知 GPUI，也不会请求重新渲染。当前源码支持的模式是在渲染闭包内读取 signal，并在构造带样式元素之前调用 `set_theme`：

```rust
fn themed_app() -> impl gpui::IntoElement {
    let (dark, set_dark) = create_signal(false);

    set_theme(if dark.get() {
        theme! {
            --surface: #111827;
            --foreground: #f9fafb;
        }
    } else {
        theme! {
            --surface: #ffffff;
            --foreground: #111827;
        }
    });

    view! {
        <button
            style={css! {
                padding: 8px 16px;
                border-radius: 6px;
                color: var(--foreground);
                background: var(--surface);
            }}
            on:click={click(move |cx| {
                set_dark.update(cx, |value| *value = !*value);
            })}
        >
            {"Toggle theme"}
        </button>
    }
}
```

`dark.get()` 读取会注册渲染依赖。更新 `dark` 会请求另一次渲染；这次渲染安装选定的主题，并使用新的变量值重新构建元素。如果只在事件处理器中调用 `set_theme`，虽然线程局部主题会改变，却不会触发上述重新渲染。

## 内置动画

`animate-*` 工具类创建循环的 GPUI 动画：

| 类 | 当前行为 | 默认周期 |
| --- | --- | --- |
| `animate-pulse` | 不透明度从 1 变为 0.5，再恢复。 | 2 秒 |
| `animate-bounce` | 上外边距从 0 变为 -10 px，再恢复。 | 1 秒 |
| `animate-ping` | 不透明度在周期的前 75% 降到 0，并保持到下一周期。不会缩放元素。 | 1 秒 |
| `animate-spin` | 没有视觉效果；GPUI `div` 元素不提供所需的旋转支持。 | 1 秒 |

```rust
let element = view! {
    <div class="p-4 rounded bg-blue-500 text-white animate-pulse duration-1000 ease-out">
        {"Loading"}
    </div>
};
```

`duration-N` 以毫秒设置时长。支持的缓动类是 `ease-linear`、`ease-in`、`ease-out` 和 `ease-in-out`；内置动画默认使用 `ease-in-out`。

## 自定义 GPUI 动画

需要内置动画之外的行为时，使用 `animate` 属性并直接调用 GPUI 的 `AnimationExt::with_animation`：

```rust
use std::time::Duration;
use gpui::{Animation, AnimationExt};

let element = view! {
    <div
        class="p-4 bg-blue-500 text-white"
        animate={|el| el.with_animation(
            "fade",
            Animation::new(Duration::from_secs(1)).repeat(),
            |el, delta| el.opacity(1.0 - delta),
        )}
    >
        {"Custom animation"}
    </div>
};
```

闭包接收元素并返回其 `AnimationElement`。这是 GPUI 动画，不是 CSS keyframe 声明。元素上显式的 `animate` 属性优先于来自 `class` 的动画或过渡配置。

## 悬停过渡

`transition-*` 工具类在指针进入或离开元素时，在基础样式与 `hover:` 样式之间执行动画：

| 类 | 插值属性 |
| --- | --- |
| `transition-opacity` | 不透明度 |
| `transition-colors` | 文本颜色、边框颜色和纯色背景颜色 |
| `transition`、`transition-all` | 不透明度加上上述支持的颜色 |

```rust
let element = view! {
    <button class="px-4 py-2 rounded bg-indigo-500 text-white hover:bg-blue-600 transition-colors duration-300 ease-in-out">
        {"Hover me"}
    </button>
};
```

`view!` 创建悬停 signal，并截取基础样式和“基础加悬停”样式的快照。过渡默认使用 150 ms 和 `ease-in-out`，除非另行覆盖。未选择的悬停属性会立即变化。

该机制仅限悬停过渡，不会为 focus/active/响应式变化或任意 signal 驱动的样式变化执行动画。`transition-shadow` 和 `transition-transform` 不提供过渡，`transition-all` 也不会插值外边距、内边距等布局属性。

## 时序与组合限制

- `delay-N` 会被解析和存储，但当前没有效果；GPUI 不提供此配置所需的原生延迟能力。
- `duration-N` 和 `ease-*` 配置基础动画/过渡类，而不是带变体前缀的工具类。
- 同一元素同时具有 class 动画和 class 过渡时，动画优先。
- 在当前实现中，不应把 `animate-spin`、阴影过渡和变换过渡当作可见动态效果使用。

class 支持和状态前缀请参阅 [Tailwind 风格类与变体](../tailwind-and-variants/)。有关选择 `css!`、`class` 或动态组合的说明，请返回[样式概览](../overview/)。
