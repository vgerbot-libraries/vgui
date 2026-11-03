---
title: 内置元素与事件
description: 使用 vgui 的类 HTML 元素、GPUI 属性、无障碍元数据、媒体、画布和标准化事件。
---

`view!` 宏识别一组固定的小写标签，并生成 GPUI 元素，而不是浏览器 DOM 节点。采用类 HTML 的拼写并不意味着具备 HTML 的布局、导航、表单或事件行为；下列映射和限制也是 API 的一部分。

输入专用标签请参阅[输入与表单](../inputs-and-forms/)；`ref` 请参阅[引用与上下文](../refs-and-context/)。

## 元素类别

| 类别 | 标签及当前映射 |
| --- | --- |
| 通用容器 | `div`、`span`、`p`、`output`、`option`、`optgroup`、`header`、`footer`、`nav`、`main`、`section`、`article`、`aside`、`address`、`fieldset`、`legend`、`figure`、`figcaption`、`pre`、`blockquote` 和 `q` 均生成 `gpui::div()`。 |
| 标题 | `h1` 至 `h6` 生成具有默认 rem 字号和字重的 div。`h1` 为 2rem/600；`h2` 为 1.5rem/600；`h3` 为 1.25rem/600；`h4` 为 1rem/600；`h5` 为 0.875rem/600；`h6` 为 0.85rem/500。 |
| 文本格式 | `strong`/`b` 为粗体；`em`/`i` 为斜体；`u` 带下划线；`s`/`del`/`strike` 带删除线；`mark` 高亮；`small` 为 0.875rem；`code`/`kbd`/`samp`/`var` 使用等宽字体。`cite`、`abbr`、`dfn`、`bdi`、`bdo` 和 `time` 是普通 div。 |
| 列表 | `ul`、`ol` 和 `dl` 是纵向 flex；`li` 是普通 div；`dt` 为粗体；`dd` 有 16px 左内边距。不会生成项目符号、编号或 HTML 列表语义。 |
| 链接与按钮 | `a` 添加指针光标和蓝色文本；`button` 添加指针光标、居中文本及默认 `tabindex={0}`。 |
| 表格 | `table`、`thead`、`tbody` 和 `tfoot` 是纵向 flex；`caption` 是普通 div；`tr` 是全宽横向 flex；`td` 和 `th` 使用 `flex_1`，其中 `th` 为粗体且文本居中。 |
| 空元素与无操作标签 | `br` 生成一行高度的 div；`hr` 生成全宽 1px 分隔线；`wbr`、`colgroup` 和 `col` 生成 `gpui::Empty`。 |

多数语义标签只是样式别名。尤其是 `output`、`option` 和 `optgroup`，它们本身不具备表单行为。

### 需要注意的 HTML 差异

- `a` 上的 `href` 会被接受但不使用；请附加 `on:click` 等应用操作，不要期待导航。
- 表格使用 flex 布局。`td`/`th` 上的 `colspan` 会转换为相对的 `flex_grow`；`rowspan` 会被接受但不影响布局。等宽 flex 不合适时，请显式设置单元格宽度。
- `br`、`hr` 和 `wbr` 使用上述映射，不具备浏览器的换行和断词语义。
- 未知标签和未知属性会导致编译错误。每个属性都需要 `=value`；布尔属性必须显式书写，例如 `disabled={true}`。

## 面向 GPUI 的属性

在生成的 GPUI 元素具备相应能力时，通用内置元素支持以下属性：

| 属性 | 用途 |
| --- | --- |
| `style={css! { ... }}` | 应用 vgui 的 Rust CSS 样式。 |
| `class="..."` | 应用 Tailwind 风格工具类，包括受支持的状态和响应式变体。 |
| `hover={...}`、`active={...}`、`focus={...}` | 应用状态样式。 |
| `id="..."` | 设置 GPUI 元素 ID。需要有状态交互的功能可以在未提供 ID 时生成一个。 |
| `tabindex={0}` | 非负值设置 Tab 索引；负值使用 GPUI 仅可聚焦的路径。 |
| `ref={node_ref.clone()}` | 在受支持的位置绑定持久的焦点和滚动句柄。 |
| `animate={...}` | 子元素构建完成后应用受支持的动画表达式。 |
| `{..props}` | 向通用元素应用一个 GPUI spread。 |
| `drag={...}`、`drag:preview={...}`、`can_drop={...}` | 配置 GPUI 拖放行为。 |

专用元素有意只接受更小的属性集。输入、`select`、`textarea`、`form`、`canvas`、`dialog`、`portal`、`floating` 和 `uniform_list` 并不继承上表所有属性。

## ARIA 与角色

通用内置元素接受 `role="..."` 和 `aria:name=value`。角色字符串会解析为 GPUI/AccessKit 角色；未知角色会变成 `GenericContainer`。语义化 HTML 标签名不会自动添加对应的 ARIA 角色。

支持的 ARIA 映射包括：

- `aria:label`、`aria:description`、`aria:keyshortcuts`
- `aria:selected`、`aria:expanded`
- `aria:toggled="true"`、`"mixed"`，其他字符串表示 false
- `aria:valuenow` 或 `aria:numeric_value`，以及 `aria:numeric_value_step`
- `aria:value`、`aria:placeholder`

```rust
view! {
    <div role="slider" aria:label="Volume" aria:valuenow={50f64}>
        {"Volume"}
    </div>
}
```

其他 `aria:*` 名称目前会被忽略，而不是继续传递。`input`、`select`、`textarea`、`label` 和 `form` 拒绝直接使用 `role`/`aria:*` 属性；需要时请把无障碍元数据放在合适的通用外层元素上。`img` 的 `alt` 属性会被接受，但不用于无障碍。

## 图像、计量器与展开区域

```rust
view! {
    <img
        src={"logo.png"}
        alt={"Logo"}
        object_fit="contain"
        on:load={move |_cx: &mut gpui::App| {}}
        on:error={move |_cx: &mut gpui::App| {}}
    />
}
```

`img` 必须提供 `src`。`object_fit` 接受字符串值 `fill`、`contain`、`cover`、`scale-down` 和 `none`。`on:load` 与 `on:error` 是图像专用的 `Fn(&mut App)` 回调，没有事件负载；它们观察 GPUI 资源加载状态的转换。当前回调包装器无法观察 `ImageSource::Image` 来源，因此该来源变体无法使用这些回调。`svg` 同样必须提供 `src`，并映射为 `gpui::svg().path(src)`。

`progress` 接受 `value` 和 `max`（默认值分别为 0 和 1）。`meter` 接受 `value`、`min`、`max`、`low`、`high` 和 `optimum`；后三个是可选数值。`details` 接受 `open={bool}`，并通过 vgui 的展开辅助函数渲染子元素；`summary` 只添加指针光标样式。

## 画布绘制

`canvas` 必须提供 `paint={|ctx| ...}`，并且只接受 `paint`、`style` 和 `class`。它不能包含子元素或事件属性。闭包在 GPUI 绘制期间接收 `Context2D`；坐标相对于画布，绘制采用立即模式。

```rust
use vgui::prelude::*;

view! {
    <canvas
        class="w-full h-48"
        paint={move |ctx: &mut Context2D| {
            ctx.set_fill_style(color("#2563eb"));
            ctx.fill_rect(12.0, 12.0, 96.0, 48.0);
        }}
    />
}
```

`Context2D` 是由 GPUI 支撑的绘制 API，不是浏览器 `HTMLCanvasElement`，也不是保留模式画布上下文。其支持的路径、变换、文本和绘制操作请参阅 API 文档。

## 其他专用内置元素

| 标签 | 必需与可选属性 |
| --- | --- |
| `dialog` | `open={bool}` 和 `on:close={Fn(&mut App)}`；不接受其他属性。 |
| `portal` | 可选 `priority={usize}`；较高值绘制在较低值之上。 |
| `floating` | 必需 `position={Point<Pixels>}`，用于窗口坐标定位。 |
| `uniform_list` | 必需 `count={usize}` 和 `render={|range, window, cx| ...}`；可选 `scroll_handle`、`style` 和 `class`；不接受子元素或事件。 |
| `datalist` | 必需 `id`，可选 `options={Vec<String>}`；注册建议项而不渲染节点。 |
| `radiogroup` | 不接受属性；协调后代单选按钮的焦点。 |

输入相关标签见[输入与表单](../inputs-and-forms/)。

## 标准化指针、键盘、滚轮与窗口事件

标准化处理器使用 vgui 事件结构体，并接收 `(&event, &mut Window, &mut App)`：

| 属性 | 负载 | 语义与限制 |
| --- | --- | --- |
| `on:keydown`、`on:keyup` | `&KeyboardEvent` | GPUI 键盘事件，包含 `key`、尽力映射的 `code`、修饰键标志、`key_char` 和 `repeat`。GPUI 没有物理按键代码，因此 `code` 可能为空；keyup 的 `repeat` 始终为 false。 |
| `on:pointerdown`、`on:pointerup`、`on:pointermove` | `&PointerEvent` | 由鼠标支撑的事件，包含窗口坐标系中的像素坐标、按钮/位掩码值、修饰键和点击次数。pointer up 通过 GPUI 的 capture-any-mouse-up 路径注册。 |
| `on:wheel` | `&WheelEvent` | 像素增量和修饰键标志。行增量按每行 16px 转换。 |
| `on:dblclick` | `&PointerEvent` | GPUI 点击次数至少为两次时提供鼠标点击数据。 |
| `on:contextmenu` | `&PointerEvent` | 提供次要按钮点击的鼠标数据。 |
| `on:resize` | `&ResizeEvent` | 注册接收窗口视口新宽高的处理器。它是窗口级事件，而不是元素级 `ResizeObserver`，尽管属性写在元素上。 |

```rust
view! {
    <div
        tabindex={0}
        on:keydown={move |event: &KeyboardEvent, _window, cx| {
            if event.key == "Escape" {
                set_open.set(cx, false);
                event.stop_propagation();
            }
        }}
        on:pointermove={move |event: &PointerEvent, _window, _cx| {
            set_position.set(_cx, (event.client_x, event.client_y));
        }}
    >
        {"Interactive area"}
    </div>
}
```

元素键盘监听器需要经过相应的 GPUI 焦点路径。`KeyboardEvent::stop_propagation()` 会在闭包返回后转换为 GPUI 的传播停止，同时影响 vgui 的全局键盘监听器。这里没有通用 DOM `Event`、`prevent_default`、`current_target`，也没有指针事件的 `stop_propagation` API。

指针事件目前仅支持鼠标：`pointer_type` 为 `Mouse`，`pointer_id` 为 1，`is_primary` 为 true。没有标准化的触摸/笔事件流、pointer-cancel 事件或指针捕获方法。pointer down 的 `button` 用 0/1/2 表示左/中/右键；移动时为 -1。pointer up 的 `buttons` 为 0，因为 GPUI 不提供其余按钮的按下状态。

## GPUI 与特定状态的事件属性

除标准化事件层外，还可使用以下属性：

| 属性 | 处理器形式或行为 |
| --- | --- |
| `on:click` | GPUI `ClickEvent` 监听器；只需要 `&mut App` 时可使用 `click(move |cx| ...)`。 |
| `on:scroll` | 原始 GPUI 滚轮监听器；需要 `WheelEvent` 时使用 `on:wheel`。 |
| `on:mouseenter`、`on:mouseleave` | 由 GPUI 悬停转换驱动的 `Fn(&mut Window, &mut App)`。 |
| `on:hover` | 同时处理进入和离开的 `Fn(bool, &mut Window, &mut App)`。 |
| `on:modifiers_changed` | 原始 GPUI 修饰键监听器。 |
| `on:mouse_down_out`、`on:mouse_up_out`、`on:any_mouse_down` | 原始 GPUI 鼠标监听器；`mouse_up_out` 仅针对左键。 |
| `on:drop`、`on:drag_move` | GPUI 拖放监听器，在适用时与拖放属性配合使用。 |

`on:input`、`on:change`、`on:submit`、`on:reset`、`on:close`、`on:load` 和 `on:error` 属于特定组件，而非所有元素。不支持的事件名称会导致编译错误。
