---
title: 控制流
description: 在 vgui 中使用 Show、For、Index、Switch 和 Match 渲染条件、列表与分支。
---

`view!` 宏为控制流元素提供专门的展开规则。选择时既要考虑 UI 结构，也要考虑所需的响应式作用域；它们的状态生命周期有意设计为不同。

## 使用 Show 按条件渲染

`<Show>` 适用于单个布尔条件。`when` 为 true 时渲染其子节点，否则渲染可选的 `fallback`：

```rust
view! {
    <Show
        when={count.get() > 0}
        fallback={view! { <span>"No items"</span> }}
    >
        <span>{format!("{} items", count.get())}</span>
    </Show>
}
```

没有 `fallback` 时，false 条件会生成 `gpui::Empty`。`when` 必须计算为 `bool`，`<Show>` 只接受 `when` 和 `fallback`。

`<Show>` **不会**创建子响应式作用域。条件为 false 时会跳过其主体，因此如果只在该主体内创建 signal、store、memo、effect 或清理回调，就会改变父作用域的位置式 slot 序列。请将状态放在条件之外，或在状态属于某个分支时使用具有作用域的 `<Match>` 分支。

## 使用 For 迭代简单列表

`<For>` 适用于直接渲染 `IntoIterator`。它唯一的子节点必须是插值闭包，接收 `(item, index)` 并返回元素：

```rust
view! {
    <For
        each={todos.get()}
        fallback={view! { <div>"No todos here."</div> }}
    >
        {move |todo: Todo, index: usize| view! {
            <div>{format!("{}: {}", index + 1, todo.title)}</div>
        }}
    </For>
}
```

每次根渲染都会为每个列表项运行闭包。迭代器为空时渲染 fallback；没有 fallback 则不渲染任何内容。`<For>` 只接受 `each` 和 `fallback`。

`<For>` 没有基于 key 的协调，也没有列表项级子作用域。所有闭包调用共享父作用域的位置式 slot 序列。请保持行无状态，并从外部传入状态或回调；不要在长度可变的 `<For>` 行中创建响应式原语，包括通过有状态组件间接创建。每个位置需要独立响应式状态时，请选择 `<Index>`。

## 使用 Index 为每个位置提供作用域

`<Index>` 同样使用 `each`、可选 `fallback` 和 `(item, index)` 闭包，但会为每个位置进入一个持久子作用域：

```rust
view! {
    <Index
        each={items.get()}
        fallback={view! { <div>"No items."</div> }}
    >
        {move |item: String, index: usize| view! {
            <div>{format!("Item {}: {}", index, item)}</div>
        }}
    </Index>
}
```

行内创建的 signal、memo、effect、store 和清理回调属于该位置的作用域。位置仍然存在时，它们会跨根渲染保留。列表缩短时，超出新长度的作用域会被释放，并运行其 `on_cleanup` 回调。

这是**按位置确定身份**，不是按列表项 key 确定身份。插入、删除或重新排序列表项，可能使已有位置局部状态对应不同的列表项值。当位置就是预期身份时使用 `<Index>`；不要将其视为基于 key 的列表 diff 机制。

## 使用 Switch 和 Match 选择第一个分支

当多个条件中只应渲染一个时，使用 `<Switch>`。`<Match>` 条件按源码顺序计算，第一个为 true 的分支生效：

```rust
view! {
    <Switch fallback={view! { <div>"Unknown status"</div> }}>
        <Match when={status.get() == "loading"}>
            <div>"Loading..."</div>
        </Match>
        <Match when={status.get() == "error"}>
            <div>"Error"</div>
        </Match>
        <Match when={status.get() == "ready"}>
            <div>"Ready"</div>
        </Match>
    </Switch>
}
```

没有条件匹配时渲染 fallback；未提供 fallback 则结果为 `gpui::Empty`。`<Switch>` 只接受 `fallback`，其子节点必须是 `<Match>` 元素，每个 `<Match>` 都必须以 `when` 作为唯一属性。

每个 `<Match>` 分支都有独立的子作用域和 slot 序列。分支保持活跃时，活跃分支中创建的状态会跨根渲染保留。切换分支会释放不再活跃的分支作用域、运行其清理回调；以后再次进入该分支时会重新创建状态。

## 比较作用域和布局行为

| 结构 | 响应式作用域 | 空或不活跃时的结果 | 主要用途 |
| --- | --- | --- | --- |
| `<Show>` | 共享父作用域 | fallback 或 `gpui::Empty` | 单个条件，状态在条件外管理 |
| `<For>` | 所有行共享父作用域 | fallback 或无内容 | 简单、无状态迭代 |
| `<Index>` | 每个位置一个持久子作用域 | fallback 或 `gpui::Empty` | 位置局部列表项状态和清理 |
| `<Switch>` / `<Match>` | 每个分支一个子作用域 | fallback 或 `gpui::Empty` | 首个匹配分支和分支局部状态 |

当这些元素是另一个元素的直接子节点时，`view!` 会将选中或迭代产生的子节点直接加入父元素，保留父元素的 flex 或 grid 布局，不增加控制流包装层。作为独立表达式时，非空 `For` 和 `Index` 辅助函数会返回包含各行的 `div`。

响应式创建和释放规则请参阅[响应式](../reactivity/)。底层解析和直接子节点展开请参阅 [`view!` 宏](../view-macro/)；向组件传递行状态请参阅[自定义组件](../custom-components/)。
