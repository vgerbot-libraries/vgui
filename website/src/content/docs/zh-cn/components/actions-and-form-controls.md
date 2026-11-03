---
title: 操作与表单控件
description: 使用 Button、TextField、Switch 和 Select builder API 构建受控控件。
---

以下示例都是可直接用于 vgui 应用的完整渲染函数。请按照[快速开始](../../docs/quick-start/)挂载它们。每个函数都应用浅色主题；信号驱动的主题切换请参阅[组件概览](../)。

## Button

`Button` 接收 variant、size、`on:click` 回调和子元素。下面的示例持有点击次数，并通过信号更新它：

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn click_button() -> impl gpui::IntoElement {
    apply_theme(false);
    let (clicks, set_clicks) = create_signal(0_u32);

    view! {
        <Button
            variant={ButtonVariant::Default}
            size={ButtonSize::Default}
            on:click={move |cx| set_clicks.update(cx, |count| *count += 1)}
        >
            {format!("Clicked {} times", clicks.get())}
        </Button>
    }
}
```

- `ButtonVariant`：`Default`、`Destructive`、`Outline`、`Secondary`、`Ghost` 和 `Link`。
- `ButtonSize`：`Default`、`Sm`、`Lg` 和 `Icon`。
- 回调接收 `&mut gpui::App`，而不是点击事件。在结构体表达式中，对应字段为 `on_click: Box<dyn Fn(&mut gpui::App) + 'static>`。
- 当前 API 没有 `disabled` 属性。`IconButton` 是独立的字形便捷组件，固定使用 ghost variant 和 icon size。

## TextField 与 Switch

公开模块名为 `input`，但其中的宏组件是 `TextField`。下面两个控件都是受控组件：回调将新值写回信号，渲染时读取这些信号。

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn profile_form() -> impl gpui::IntoElement {
    apply_theme(false);
    let (name, set_name) = create_signal(String::new());
    let (notifications, set_notifications) = create_signal(false);

    view! {
        <div class="flex flex-col gap-3 p-4">
            {TextField {
                value: name.get(),
                placeholder: "Display name".to_string(),
                on_input: Box::new(move |value, cx| set_name.set(cx, value)),
                class: None,
            }}
            <div class="flex flex-row items-center gap-2">
                {Switch {
                    checked: notifications.get(),
                    on_change: Box::new(move |checked, cx| {
                        set_notifications.set(cx, checked)
                    }),
                }}
                <span>
                    {if notifications.get() { "Notifications on" } else { "Notifications off" }}
                </span>
            </div>
        </div>
    }
}
```

`TextField` 要求提供 `value`、`placeholder` 和 `on_input`；`class` 是外层容器的可选附加类。其输入类型固定为 text，`on_input` 接收新的 `String` 和 `&mut gpui::App`。

`Switch` 要求提供 `checked` 和 `on_change`。点击会请求 `!checked`，但调用方仍负责保存新值。请使用上面的结构体表达式，因为 `<Switch>` 已保留为 vgui 的控制流标签。当前 switch API 没有 label 或 disabled 属性。

## Select

Select 使用函数加 builder，而不是 PascalCase 标签。`ShadcnSelectProps::new` 接收选项、受控的 `Option<V>` 值和 `Rc` 变化回调。值类型必须实现 `Clone + PartialEq + 'static`。

```rust
use std::rc::Rc;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn language_select() -> gpui::AnyElement {
    apply_theme(false);
    let (language, set_language) = create_signal(Some("rust".to_string()));

    let props = ShadcnSelectProps::new(
        vec![
            SelectItem::new("Rust", "rust".to_string()),
            SelectItem::new("TypeScript", "typescript".to_string()),
            SelectItem::new("Python", "python".to_string()).disabled(true),
        ],
        language.get(),
        Rc::new(move |value, cx| set_language.set(cx, value)),
    )
    .placeholder("Choose a language")
    .searchable(true)
    .clearable(true)
    .accessibility_label("Programming language")
    .size(SelectSize::Md);

    select(props)
}
```

### Builder 选项

| API | 行为 |
| --- | --- |
| `SelectItem::new(title, value)` | 创建一个选项。链式调用 `.disabled(bool)` 可禁止确认，调用 `.display_title(...)` 可自定义触发器中的已选内容。 |
| `SelectGroup::new(title, items)` | 创建带标题的分组。可直接传入分组，或使用 `SelectItems::grouped(groups)`；扁平选项可使用 `SelectItems::flat(items)`。 |
| `.placeholder(...)`、`.title_prefix(...)`、`.icon(...)` | 配置触发器的空值文本、已选标题前缀和自定义图标。 |
| `.searchable(bool)`、`.search_placeholder(...)` | 启用搜索并设置搜索占位文本。默认匹配器对标题执行不区分大小写的子串匹配。 |
| `.matcher(...)`、`.render_item(...)`、`.empty(...)` | 覆盖匹配逻辑、自定义选项行内容，并在筛选无结果时渲染内容。行事件和尺寸仍由 Select 管理。 |
| `.clearable(bool)`、`.disabled(bool)` | 允许清除已选值，或禁用整个控件。清除时发布 `None`。 |
| `.size(SelectSize::...)` | 选择 `Xs`、`Sm`、`Md` 或 `Lg`；默认为 `Md`。 |
| `.menu_width(px(...))`、`.menu_max_h(px(...))` | 设置弹出菜单宽度和最大高度。宽度默认跟随触发器，最大高度默认为 320 px。 |
| `.accessibility_label(...)`、`.id(...)`、`.focus_ring(bool)`、`.appearance(bool)` | 设置触发器标签和元素 ID，或开关焦点环与 shadcn 外观。 |
| `.on_dismiss(...)` | 在弹出菜单关闭时运行回调。确认选项时会先调用 `on_change`，然后关闭菜单。 |

选择器是受控单选组件。确认可用选项会发布 `Some(value)`；未确认便关闭不会改变已提交的值。禁用选项可以获得键盘高亮，但不能被确认。此 API 没有多选模式。

继续阅读[数据展示](../data-display/)，了解面向内容的组件。
