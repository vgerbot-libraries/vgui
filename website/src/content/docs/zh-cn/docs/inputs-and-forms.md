---
title: 输入与表单
description: 将文本、选择、范围、下拉框、多行文本和文件控件连接到 signal，并遵守 vgui 的表单限制。
---

vgui 输入是通过 `view!` 属性配置的 GPUI 控件。它们不会创建浏览器控件，也不参与浏览器表单提交。请在 signal 或其他应用状态中保存编辑后的值，并在后续渲染时通过控件属性传回这些值。

通用属性和指针/键盘处理器请参阅[内置元素与事件](../built-in-elements-and-events/)；焦点和滚动句柄请参阅[引用与上下文](../refs-and-context/)。

## 输入类型与状态流

`input` 是空元素，默认 `type="text"`。`type` 值必须是字符串字面量。支持的类型包括：

| 类别 | 类型 |
| --- | --- |
| 文本类 | `text`、`password`、`search`、`email`、`url`、`tel`、`number`、`date`、`datetime-local`、`time`、`month`、`week`、`color` |
| 布尔类 | `checkbox`、`radio` |
| 数值类 | `range` |
| 文件类 | `file` |
| 按钮类 | `submit`、`button`、`reset` |
| 隐藏类 | `hidden` |

文本和范围控件将 GPUI entity 状态保存在响应式作用域的 slot 中。它们的 `value` 属性会把 props 同步到该状态；文本输入聚焦时，正在进行的编辑不会被传入的 `value` 覆盖。这不是浏览器 DOM 绑定，因此应用回调仍应显式记录变化。

## 文本类输入

```rust
view! {
    <input
        type="text"
        value={name.get()}
        placeholder="Name"
        on:input={move |value: &str, cx: &mut gpui::App| {
            set_name.set(cx, value.to_string());
        }}
        on:change={move |value: &str, cx: &mut gpui::App| {
            set_committed_name.set(cx, value.to_string());
        }}
    />
}
```

`on:input` 的形式为 `FnMut(&str, &mut App)`，在控件值变化时运行。对于单行输入，`on:change` 在按 Enter 和失去焦点时运行。文本编辑包含 GPUI 支撑的选择、剪贴板和 IME 路径；password 会掩码显示字符。日期/时间/月份和颜色类型包含自定义选择器路径，而 `week` 仍是文本类型，不是浏览器周控件。

有效的文本输入属性包括：

| 属性 | 行为 |
| --- | --- |
| `value`、`placeholder` | 当前字符串值和占位符。未提供占位符时，各类型会提供面向格式的默认占位符。 |
| `disabled`、`readonly` | 禁用交互或阻止编辑。 |
| `min`、`max` | 数值校验使用的可选 `f64` 边界。 |
| `step` | 作为数值元数据接受并存储；当前文本校验不会检查步长匹配。 |
| `required`、`pattern`、`minlength` | 类 HTML 的视觉约束检查。 |
| `maxlength` | 宏会接受，但当前不会同步到持久的文本输入状态中；不要依赖它进行校验。 |
| `list` | `datalist` 建议注册表的 ID。 |
| `style`、`class`、`id`、`tabindex` | 控件样式、标签查找 ID 和焦点配置。 |

直接 `ref`、`role`、`aria:*`、spread、动画和拖放属性没有接入文本输入。需要外层 ref 或 ARIA 元数据时，请将输入放在通用元素中。

### 校验与 datalist

校验会改变输入边框；它不会阻止回调或提交表单。这些检查有意小于 HTML 约束校验：

- `required` 拒绝空字符串。
- `minlength` 统计 Rust 字符，而不是 UTF-8 字节。
- `pattern` 是精确字面量或一个开头/结尾 `*` 通配符（`foo*`、`*bar`），不是正则表达式。
- `email` 要求恰好一个 `@` 且两侧非空；`url` 要求 `http://` 或 `https://` 前缀。
- `number` 解析为 `f64`，并在存在 `min`/`max` 时应用边界。

`datalist` 不渲染任何内容。它在必需的 `id` 下注册 `options={Vec<String>}`；具有匹配 `list="id"` 的文本输入可以显示前缀匹配建议。

```rust
view! {
    <div>
        <datalist id="cities" options={vec!["Paris".to_string(), "London".to_string()]} />
        <input type="text" list="cities" value={city.get()} />
    </div>
}
```

## 复选框与单选按钮

两个控件都使用 `checked={bool}`、`disabled={bool}` 和 `on:change={FnMut(bool, &mut App)}`。复选框报告切换后的值；单选按钮在激活时报告 `true`。应用仍负责更新 checked 状态。

```rust
view! {
    <input
        type="checkbox"
        checked={done.get()}
        on:change={move |checked: bool, cx: &mut gpui::App| set_done.set(cx, checked)}
    />
}
```

要实现互斥选择，请从同一应用状态派生每个单选按钮的 `checked` 值。这里没有基于 HTML `name` 的分组行为。

```rust
view! {
    <radiogroup>
        <input type="radio" checked={choice.get() == 0}
            on:change={move |_checked: bool, cx: &mut gpui::App| set_choice.set(cx, 0)} />
        <input type="radio" checked={choice.get() == 1}
            on:change={move |_checked: bool, cx: &mut gpui::App| set_choice.set(cx, 1)} />
    </radiogroup>
}
```

`radiogroup` 不接受属性。它的方向键通过 roving tab stop 在已注册单选按钮间移动焦点；移动焦点本身不会选择单选按钮，也不会调用 `on:change`。复选框、单选按钮、文件和按钮类输入支持文档所述的外层样式和 `ref`。

## 范围

```rust
view! {
    <input
        type="range"
        min={0.0f64}
        max={100.0f64}
        step={1.0f64}
        value={volume.get()}
        on:change={move |value: f64, cx: &mut gpui::App| set_volume.set(cx, value)}
    />
}
```

默认值为 `min=0`、`max=100`、`step=1` 和 `value=0`。`on:change` 在指针拖动和键盘调整期间接收 `f64`。方向键使用 `step`，Shift 将其乘以十，Home/End 选择边界。支持 `disabled`、`style`、`class`、`id` 和 `tabindex`。直接 `ref` 不会绑定；请按[引用与上下文](../refs-and-context/)所述使用外层 `div`。

## 下拉选择

`select` 通过属性而不是 `option` 子元素接收数据。扁平选项是 `Vec<(String, String)>` 值-标签对。一个可选闭包子元素可以同时自定义已选内容和选项行。

```rust
view! {
    <select
        options={vec![
            ("one".to_string(), "One".to_string()),
            ("two".to_string(), "Two".to_string()),
        ]}
        value={selection.get()}
        on:change={move |value: &str, cx: &mut gpui::App| {
            set_selection.set(cx, value.to_string());
        }}
    />
}
```

分组行使用 `groups={Vec<(String, Vec<(String, String)>)>}`。非空分组驱动弹出层渲染；扁平选项仍参与标签查找。使用 `multiple={true}` 时，`value` 和 `on:change` 参数是逗号分隔的选中项，切换后弹出层保持打开。因此，包含逗号的选项值无法通过这种编码可靠表示。

自定义渲染器的形式为 `Fn(&str, &str) -> impl IntoElement`：

```rust
view! {
    <select options={options} value={selection.get()}>
        {move |value: &str, label: &str| view! {
            <div class="flex gap-2">
                <span>{value.to_string()}</span>
                <span>{label.to_string()}</span>
            </div>
        }}
    </select>
}
```

`select` 支持 `style` 和 `class`；`name`、`id` 和 `tabindex` 会被接受但不使用。直接 `ref`、ARIA 属性、spread 和非 `change` 事件会被拒绝。不要嵌套 `option` 或 `optgroup` 节点并期待它们填充控件。

## 多行文本

`textarea` 同样是空元素，并使用与单行输入相同的文本回调类型。Enter 插入换行；`on:change` 在失去焦点时运行。

```rust
view! {
    <textarea
        rows={4u32}
        value={notes.get()}
        placeholder="Notes"
        on:input={move |value: &str, cx: &mut gpui::App| {
            set_notes.set(cx, value.to_string());
        }}
    />
}
```

它支持 `value`、`placeholder`、`disabled`、`readonly`、`rows`、`on:input`、`on:change`、`style`、`class`、`id` 和 `tabindex`。`name` 会被接受但不使用。`rows` 控制最小高度；未提供时使用控件默认最小值。直接 `ref`、role/ARIA 属性和无关事件会被拒绝；适当时请使用外层元素。

## 文件选择

```rust
view! {
    <input
        type="file"
        value="Choose files"
        multiple={true}
        on:change={move |paths: Vec<std::path::PathBuf>, cx: &mut gpui::App| {
            set_paths.set(cx, paths);
        }}
    />
}
```

对于文件输入，`value` 是按钮标签，而不是所选文件名。GPUI 文件提示完成后，回调接收所选 `PathBuf` 值。`multiple` 控制多选。此控件没有 `accept` 过滤器、文件内容 API 或目录选择模式。虽然通用输入解析会接受 `disabled`，但文件控件当前没有将它接入提示操作。支持 `style`、`class`、状态样式、`id`、`tabindex` 和外层 `ref`。

## 标签

将 `for` 与文本类控件已注册的 `id` 一起使用，或包裹控件以便标签发现第一个标签目标：

```rust
view! {
    <div>
        <label for="username">{"Username"}</label>
        <input id="username" type="text" value={name.get()} />
    </div>
}
```

```rust
view! {
    <label class="flex items-center gap-2">
        <input type="checkbox" checked={done.get()} />
        <span>{"Done"}</span>
    </label>
}
```

显式标签会聚焦其注册目标。包裹式标签会聚焦发现的目标，并对注册了操作的控件调用该操作。这是 vgui 标签作用域行为，不是任意的 HTML 标签激活。

## 表单、按钮、重置与隐藏值

`form` 是具有构建期表单作用域的 GPUI 样式容器。除了支持的样式、ref 和通用事件属性外，它接受 `on:submit={FnMut(&mut App)}` 和 `on:reset={FnMut(&mut App)}`。它没有 HTML `action`、`method`、`FormData`、网络提交、自动控件收集或校验阻止行为。

按钮类输入使用 `value` 作为标签，并默认 Tab 索引为 0。显式 `on:click` 会替换宏的自动 submit/reset 绑定：

```rust
view! {
    <form class="flex gap-2">
        <input type="text" value={name.get()}
            on:input={move |value: &str, cx: &mut gpui::App| set_name.set(cx, value.to_string())} />
        <input type="submit" value="Save"
            on:click={click(move |cx| set_saved.set(cx, true))} />
        <input type="reset" value="Clear"
            on:click={click(move |cx| set_name.set(cx, String::new()))} />
    </form>
}
```

当前隐式分派存在限制：默认 submit/reset 处理器在调用时查找表单栈，但该栈在子元素构建完成后已经弹出。单行文本输入中的 Enter 会提交 `on:change`，但同样不应依赖它进行隐式表单提交。请使用显式点击处理器实现保存/重置行为，并在应用代码中执行校验、序列化和状态清理。

`input type="hidden"` 生成 `gpui::Empty`；它的 `value` 不会被收集到任何位置。`output` 只是通用 div 别名。因此，这些控件不会仅仅因为嵌套在 `form` 中就获得 HTML 表单语义。
