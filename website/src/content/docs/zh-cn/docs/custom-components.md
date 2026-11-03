---
title: 自定义组件
description: 使用属性、子节点、回调和属性展开创建类型化、可复用的 vgui 组件。
---

小型渲染辅助逻辑可以使用普通 Rust 函数；需要具有命名属性的大写标签时，使用 `#[vgui_component]`。该属性宏会生成类型化属性结构体及其 GPUI `IntoElement` 实现。

## 定义具有命名属性的组件

以下示例采用与 vgui 当前组件实现相同的模式：

```rust
#[vgui_component]
pub fn card_header(
    title: String,
    description: String,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-1">
            <span class="font-bold">{title}</span>
            <span>{description}</span>
        </div>
    }
}
```

使用函数名的 PascalCase 形式调用它：

```rust
view! {
    <CardHeader title="Tasks" description="Three remaining" />
}
```

对于 `card_header`，宏会生成公开的 `CardHeader` 结构体，其中包含函数参数对应的字段，并生成调用原始渲染函数体的 `IntoElement` 实现。正是这个生成的结构体使大写属性语法可以工作。

参数必须使用简单标识符模式，不支持带有 `self` 的方法，函数必须具有 `impl gpui::IntoElement` 等非 unit 返回类型。`String` 类型参数会成为接受 `Into<String>` 的泛型字段，因此示例可以接收字符串字面量。`Option<String>` 的内部值也会获得相应的 `Into<String>` 支持。

## 接收子节点

组件需要包含任意视图子节点时，声明 `children: Vec<gpui::AnyElement>` 参数。当前 `Card` 模式使用 `for_each` 添加这些子节点：

```rust
#[vgui_component]
pub fn card(
    children: Vec<gpui::AnyElement>,
) -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col gap-2 rounded-lg p-4">
            {for_each(children, |child, _| child)}
        </div>
    }
}
```

宏会将提供的每个子节点转换为 `gpui::AnyElement`，并填充 `children` 字段：

```rust
view! {
    <Card>
        <span>"Panel title"</span>
        <span>"Panel body"</span>
    </Card>
}
```

只有标签中存在子节点时才会提供 `children`；宏不会为必需的 `children` 字段自动构造空 vector。其他字段同样必须由调用方或展开的基础值提供。需要表示缺省值时，请使用可选属性并显式传入，或通过展开的基础值提供。

## 接收事件回调

`on:event` 属性会映射到 `on_event` 字段，并由组件展开代码装箱。请声明匹配的装箱回调类型：

```rust
#[vgui_component]
pub fn action_button(
    label: String,
    on_click: Box<dyn Fn(&mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    view! {
        <button on:click={click(move |cx| on_click(cx))}>
            {label}
        </button>
    }
}
```

调用方直接传入闭包，无需手动构造 `Box`：

```rust
view! {
    <ActionButton
        label="Save"
        on:click={move |cx: &mut gpui::App| {
            set_saved.set(cx, true);
        }}
    />
}
```

内部内置 `<button>` 仍使用正常的事件 API，这里通过 `click` 适配器处理。组件回调类型是由你定义的接口：将捕获的写入句柄克隆到 `move` 闭包中，并选择与组件所暴露信息匹配的参数类型。

其他属性名称会直接映射到字段。需要转义 Rust 关键字时使用原始字段标识符：`type=` 映射到 `r#type`，`for=` 映射到 `r#for`，`ref=` 映射到 `r#ref`。

## 转发并覆盖属性

组件展开使用 Rust 结构体更新语法。给定兼容的 `props` 值，以下代码会提供其余字段并覆盖 `title`：

```rust
view! {
    <CardHeader {..props} title={"Updated title"} />
}
```

每个组件只允许一个 `{..props}` 或 `{...props}` 展开。显式具名字段优先于基础值。这是结构体组合，不是不相关属性对象的自动合并，也不会为缺失字段生成默认值。

内置元素上的展开则不同：它会针对具体元素类型调用 `vgui::Spread` trait。相关区别请参阅 [`view!` 宏](../view-macro/)。

## 理解展开和作用域

对于任何大写标签，当前展开方式取决于其内容：

| 标签形式 | 展开结构 |
| --- | --- |
| `<Greeting />` | `Greeting()` |
| `<Greeting name={"world"} />` | `Greeting { name: "world" }` |
| `<Greeting>"world"</Greeting>` | `Greeting { children: vec![...] }` |

结构体字面值形式使用 `#[vgui_component]`。对于不需要命名属性的简单辅助逻辑，通过插值调用普通 Rust 函数通常更清晰：

```rust
fn badge(label: String) -> impl gpui::IntoElement {
    view! { <span class="rounded px-2">{label}</span> }
}

view! { <div>{badge("New".to_string())}</div> }
```

组件不会仅因为被调用就创建新的响应式作用域。在组件函数体中创建的 signal、store、memo、effect 和清理回调会占用调用方的位置式 slot。请无条件且按顺序创建它们；分支局部状态使用 `<Match>`，位置局部状态使用 `<Index>`，并避免在长度可变的 `<For>` 行中创建响应式原语。

slot 规则请参阅[响应式](../reactivity/)，具有作用域的生命周期请参阅[控制流](../control-flow/)。组件渲染仍参与[架构](../architecture/)所述的根视图重新渲染，而不是独立的虚拟 DOM 子树。
