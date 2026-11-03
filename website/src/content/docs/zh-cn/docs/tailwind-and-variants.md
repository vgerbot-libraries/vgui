---
title: Tailwind 风格类与变体
description: 使用 vgui 支持的静态与动态工具类、交互和响应式前缀，以及类型化组件变体。
---

vgui 接受一组经过选择的 Tailwind 风格类名，并将它们应用到 GPUI 样式字段。这些名称看起来熟悉，但没有 Tailwind 构建步骤，也不会输出浏览器 CSS。这里的示例列出了有代表性的受支持形式，不应据此假设所有 Tailwind 工具类都可用。

## 视图中的静态 class

通常直接使用字面量 `class` 属性：

```rust
let element = view! {
    <div class="flex items-center justify-between gap-4 p-4 w-full bg-slate-100 rounded-md">
        <span class="text-sm font-medium text-slate-900">{"Downloads"}</span>
        <span class="text-xs text-slate-500">{"12"}</span>
    </div>
};
```

`view!` 为该属性调用 `tw!`，并接入生成的全部状态、断点、动画和过渡配置。直接调用 `tw!("flex p-4")` 会返回相同的 `TwStyle` 类型，但其独立 `ApplyStyle` 实现只应用基础样式；除非有意直接操作 `TwStyle`，否则应通过 `class` 使用完整样式功能。

编译时和运行时路径都会静默跳过未知或不受支持的类名。某个类看起来没有效果时，应检查渲染后的样式。

## 支持的工具类类别

编译时宏和运行时解释器共享类解析逻辑，覆盖相同的核心类别：

| 类别 | 支持的形式包括 |
| --- | --- |
| 布局 | `flex`、`block`、`grid`、`hidden`；flex 方向、换行、增长和收缩；`justify-*`、`items-*`、`self-*`、`content-*`；相对/绝对定位和 inset 边 |
| 间距与尺寸 | `p-*`、`m-*`、方向与轴向间距、`gap-*`；宽高、最小/最大尺寸、`w-full`、`h-auto`、`m-auto`；`aspect-square`、`aspect-video` 和数值比例 |
| 排版 | 色板和任意文本颜色；`text-xs` 到 `text-9xl`；字重和字体族；对齐、行高、斜体、装饰、空白、截断和行数限制 |
| 视觉样式 | `bg-*`、`text-*` 和 `border-*` 颜色；不透明度；边框宽度、样式和单边宽度；圆角；阴影；光标样式 |
| 网格与溢出 | `grid-cols-N`、`grid-rows-N`；溢出和可见性工具类 |

间距后缀使用 vgui 的 Tailwind 风格比例，例如 `p-2` 为 8 px，`p-4` 为 16 px。颜色包含标准命名色板及 50 到 950 的色阶。这些都映射到 GPUI：同名的浏览器工具类可能具有不同行为。

### 任意值和颜色不透明度

支持的长度工具类接受方括号中的 `px`、`rem`、百分比或纯数字值。支持的颜色工具类接受十六进制、`rgb(...)` 和 `rgba(...)` 值：

```rust
let element = view! {
    <div class="w-[500px] h-[50%] p-[12px] bg-[#505050] text-[rgb(255,255,255)] rounded-[8px]">
        <span class="bg-blue-500/50 text-white">{"Arbitrary values"}</span>
    </div>
};
```

`/NN` 后缀为支持的色板、黑色和白色颜色工具类设置不透明度。任意值仍取决于目标工具类：方括号语法不会开放任意 CSS 属性。

## 动态 class 组合

需要根据条件选择 class 片段时使用 `twc!`：

```rust
let selected = true;
let disabled = false;

let classes = twc!(
    "flex items-center px-4 py-2 rounded",
    selected.then_some("bg-blue-600 text-white"),
    (!selected).then_some("bg-gray-200 text-gray-900"),
    disabled.then_some("opacity-50")
);

let element = view! {
    <button class={classes}>{"Filter"}</button>
};
```

`twc!` 返回 `TwClass`，而不是 `TwStyle`。其来源可以是 `&str`、`String`、`Option<T>` 或另一个 `TwClass`。等价的 builder API 是 `TwClass::new().add(...).add_if(condition, ...).build()`。

每个变体内的类按顺序应用，后写入的 GPUI 字段值生效。例如，`twc!("p-4", "p-2")` 产生 8 px 内边距。这是顺序式样式 refinement，不是生成后的 Tailwind 样式表层叠顺序。

对于在其他位置组装的 class 字符串，可以直接传入字符串，也可以调用运行时函数：

```rust
let runtime_classes = String::from("flex items-center gap-2 p-3 hover:bg-slate-200");
let style = tw_dynamic(runtime_classes.as_str());
let element = view! {
    <div class={style}>{"Runtime classes"}</div>
};
```

`class={some_string}` 和 `tw!(some_string)` 也会通过 `IntoTwStyle` 处理非字面量输入。需要显式取得 `TwStyle` 值时可以使用 `tw_dynamic(&str)`；不存在 `tw_dynamic!` 宏。

## 交互状态

支持的状态前缀是 `hover:`、`focus:` 和 `active:`：

```rust
let element = view! {
    <button class="px-4 py-2 rounded bg-blue-600 text-white hover:bg-blue-700 focus:border-2 focus:border-blue-300 active:bg-blue-800">
        {"Save"}
    </button>
};
```

`view!` 将这些闭包映射到 GPUI 对应的元素状态样式。对于 `css!`，应改用独立的 `hover={...}`、`focus={...}` 和 `active={...}` 属性。

每个工具类只使用一个变体前缀。不要依赖 `md:hover:bg-blue-700` 这类堆叠组合：当前解析器为每个类保存一个变体，而不是状态与断点的组合。

## 响应式断点

响应式前缀采用移动优先的最小宽度：

| 前缀 | 最小视口宽度 |
| --- | --- |
| `sm:` | 640 px |
| `md:` | 768 px |
| `lg:` | 1024 px |
| `xl:` | 1280 px |

```rust
let element = view! {
    <div class="flex flex-col gap-2 p-4 md:flex-row md:gap-6 lg:p-8">
        <span>{"Navigation"}</span>
        <span>{"Content"}</span>
    </div>
};
```

渲染期间，`view!` 检查当前视口宽度，并在基础样式之后按 `sm`、`md`、`lg`、`xl` 顺序应用所有达到阈值的断点。因此，`md:flex-row` 在更宽的视口中仍然生效，除非后续适用的断点写入同一字段。在渲染视口之外，断点解析回退到 base。

这些是 GPUI 中的视口宽度检查，不是浏览器媒体查询。支持的前缀就是上面列出的四个；不要根据 Tailwind 推断任意断点或最大宽度断点也可用。

## 类型化组件变体

对于可复用组件的样式维度，使用 `variants!`，避免维护多组并行的条件 class 选择：

```rust
variants! {
    Button {
        base => css! {
            border-radius: 6px;
            cursor: pointer;
            font-weight: 500;
        },
        variant {
            primary => css! { background: #2563eb; color: #ffffff; },
            danger => css! { background: #dc2626; color: #ffffff; },
        },
        size {
            sm => css! { padding: 4px 8px; font-size: 12px; },
            md => css! { padding: 8px 16px; font-size: 14px; },
        },
    }
}

let element = view! {
    <button
        style={ButtonVariants::default()
            .variant(ButtonVariant::Danger)
            .size(ButtonSize::Md)}
        hover={css! { opacity: 0.85; }}
    >
        {"Delete"}
    </button>
};
```

对于这个声明，宏会生成 `ButtonVariant`、`ButtonSize` 和 `ButtonVariants`。`ButtonVariants::default()` 选择每个维度的第一个选项（此处为 `Primary` 和 `Sm`），builder 方法可以替换这些选择。应用该结构体时，先添加可选的基础样式，再按声明顺序添加每个维度选中的样式。

`css!` 是变体值的清晰选择，并可与主题变量配合。`tw!` 表达式通过这条通用 `ApplyStyle` 路径只能提供基础工具类字段；`variants!` 不会接入其状态、响应式、动画和过渡配置。交互样式应像上例一样放在元素上。

主题和动态效果请继续阅读[主题与动画](../theming-and-animation/)。所有样式入口的对比请参阅[样式概览](../overview/)。
