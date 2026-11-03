---
title: 响应式
description: 使用 signal、memo、effect、store 和作用域清理管理 vgui 状态。
---

vgui 将响应式值保存在 GPUI entity 中，并把其句柄缓存在位置式作用域 slot 中。signal 保存独立值，memo 派生缓存值，effect 同步副作用，store 组织聚合状态。

## 创建并更新 signal

```rust
let (count, set_count) = create_signal(0i32);
let doubled = create_memo({
    let count = count.clone();
    move || count.get() * 2
});

view! {
    <div>
        <span>{format!("count = {}", count.get())}</span>
        <span>{format!("doubled = {}", doubled.get())}</span>
        <button on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}>
            {"Increment"}
        </button>
    </div>
}
```

`create_signal(initial)` 返回 `ReadSignal<T>` 和 `WriteSignal<T>`。两个句柄都可以低成本克隆到派生计算和事件闭包中。`T` 必须实现 `Clone + PartialEq + 'static`。

使用 `set_count.set(cx, value)` 替换值，或使用 `set_count.update(cx, |value| ...)` 原地修改。signal 写入会比较新旧值：相等值不会触发通知。`update` 还会返回闭包的结果。

## 选择 get 或 get_with

`ReadSignal::get()` 从响应式缓存中克隆值，并在依赖跟踪处于活动状态时记录该 signal：

```rust
let value = count.get();
```

`ReadSignal::get_with(cx)` 则直接从 GPUI entity 读取当前值、刷新缓存，并且不记录依赖：

```rust
let value = count.get_with(cx);
```

在视图、memo 和 effect 中使用 `get()`。当渲染作用域之外的代码（例如稍后运行的事件或任务）明确需要基于 entity 读取且不建立响应式依赖时，使用 `get_with(cx)`。`Store` 也提供相同的区别。

## 使用 memo 派生缓存值

`create_memo` 在首次创建 memo 时运行闭包，记录初始计算期间发生的 signal 和 store 读取，并缓存结果。后续根渲染会复用 memo slot，而不重新计算。已记录的依赖变化时会再次运行计算；相等的新结果不会发布给 memo 自己的依赖方。

```rust
let remaining = create_memo({
    let todos = todos.clone();
    move || todos.get().iter().filter(|todo| !todo.done).count()
});
```

memo 的结果也是 `ReadSignal<T>`，其中 `T: Clone + PartialEq + 'static`。memo 适用于希望缓存其计算的派生值，不适用于副作用。

## 运行 effect 并注册清理

`create_effect` 在首次运行时记录依赖，并在渲染期间立即同步执行闭包：

```rust
create_effect({
    let count = count.clone();
    move || {
        eprintln!("count changed to: {}", count.get());
    }
});
```

已记录的依赖变化时，effect 会重新运行；仅仅发生根渲染并不会使其重新运行。effect 不会延迟到绘制之后。请避免在没有终止条件的情况下写入会递归触发同一 effect 的值。

`on_cleanup` 与 effect 的重新运行相互独立。它注册一个位置式清理回调，该回调在外层作用域释放时运行，而不是在每次 effect 执行前运行：

```rust
on_cleanup(move || {
    eprintln!("scope disposed");
});
```

清理回调适用于 `<Match>` 分支或 `<Index>` 位置拥有的资源。释放时会先运行子作用域清理再运行父作用域清理，并清除作用域的 slot 和订阅。这些生命周期请参阅[控制流](../control-flow/)。

## 在 store 中组织聚合状态

当多个相关字段属于同一个状态值时，使用 `create_store`：

```rust
#[derive(Clone, Default)]
struct AppState {
    count: i32,
    name: String,
    flag: bool,
}

let (store, set_store) = create_store(AppState::default());
let count = store.select(|s| s.count);
let name = store.select(|s| s.name.clone());
```

store 要求 `T: Clone + 'static`，但不要求 `PartialEq`。其读取和写入 API 如下：

| API | 行为 |
| --- | --- |
| `store.get()` | 克隆整个状态，并在依赖跟踪活动时跟踪整个 store。 |
| `store.with(|state| ...)` | 借用整个状态而不克隆，同样跟踪整个 store。 |
| `store.get_with(cx)` | 直接从 GPUI entity 读取并刷新缓存，不进行跟踪。 |
| `store.select(|state| slice)` | 为选中值创建由 memo 支持的 `ReadSignal`。 |
| `set_store.set(cx, state)` | 替换整个状态并始终通知。 |
| `set_store.update(cx, |state| ...)` | 修改状态并始终通知，即使修改为空操作。 |

选择器会在 store 写入后重新计算，但只有选中的 `Clone + PartialEq` 值发生变化时才发布。这会过滤下游 memo 和 effect 通知，但**不会**阻止每次 store 写入所请求的根视图重新渲染。通过 `get` 或 `with` 读取会依赖整个 store，而不是某个字段。

单个独立值使用 signal 更简单；具有有用派生切片的聚合状态则使用 store 和选择器。当前 store 集成测试同时覆盖了整体状态读取和未变化切片过滤。

## 保持位置式身份稳定

所有这些创建 API 都参与当前作用域的 slot 序列。请在多次根渲染之间无条件地以稳定顺序和稳定类型创建它们。组件不会自动创建新作用域，因此其函数体中创建的原语也会占用调用方的 slot。

不要在按条件渲染的 `<Show>` 主体或长度可变的 `<For>` 闭包中创建响应式原语。分支局部状态使用 `<Match>` 分支，位置局部列表项状态使用 `<Index>`，并注意 `<Index>` 的位置不是列表项 key。

[架构](../architecture/)介绍通知和根视图重新渲染周期。完整的小型状态示例请参阅[第一个响应式视图](../first-reactive-view/)。
