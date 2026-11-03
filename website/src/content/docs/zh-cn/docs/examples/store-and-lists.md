---
title: Store 与列表
description: 使用 Store 选择器管理聚合状态，并使用 For 或按位置划分作用域的 Index 渲染集合。
---

`Store` 将相关字段组织在一个读取句柄和一个写入句柄之后。本页使用当前 store 集成测试中的重点摘录，然后展示当前的 `For` 和 `Index` 列表语法。这些摘录应放在已挂载的响应式视图中，并不是完整应用。

## 建立聚合状态模型

store 测试使用一个可克隆的状态结构体：

```rust
#[derive(Clone, PartialEq, Default)]
struct AppState {
    count: i32,
    name: String,
    flag: bool,
}
```

创建 store，并选择视图需要的状态切片：

```rust
let (store, set_store) = create_store(AppState::default());
let count_sig = store.select(|s| s.count);
let name_sig = store.select(|s| s.name.clone());

view! { <div>{format!("{} {}", count_sig.get(), name_sig.get())}</div> }
```

`create_store` 返回 `Store<AppState>` 和 `SetStore<AppState>`。每次 `select` 调用都会为选中的值返回一个读取 signal。`count` 这类可复制字段可以直接返回；当选择器必须返回拥有所有权的 `String` 时，则应克隆 `name` 这类数据。

## 选择器过滤未变化的值

store 写入即使没有在更新闭包中修改任何内容，也会发出通知。这有别于 signal 跳过相等值通知的行为，是有意设计。每次写入时，选择器都会重新计算并比较选中的切片：

- 执行 `s.count = 42` 后，`count_sig` 改变，而 `name_sig` 仍为空；
- 执行 `s.name = "hello".to_string()` 后，计数仍为 `42`，名称发生变化；
- 只修改 `s.flag` 时，两个选中的切片都不变。

因此，选择器会过滤未变化选中值的后续传播。它们**不会**阻止 store 通知、选择器重新计算或每次根视图渲染。不要将其描述为 keyed DOM 更新，也不要认为它能保证无关行绝不重新渲染。

当前测试还通过 `get` 和 `with` 演示了直接读取：

```rust
let via_get = store.get().count;
let via_with = store.with(|s| s.name.clone());
```

`get` 获取状态值，`with` 则通过闭包借用状态，让你选择要读取或克隆的内容。

## 修改字段或替换状态

使用写入句柄和 GPUI 应用上下文进行局部修改：

```rust
set_store.update(cx, |s| s.count = 42);
set_store.update(cx, |s| s.name = "hello".to_string());
set_store.update(cx, |s| {
    s.count = 99;
    s.name = "world".to_string();
});
set_store.update(cx, |s| s.flag = true);
```

当替换完整值更清晰时，使用 `set`：

```rust
set_store.set(
    cx,
    AppState {
        count: 100,
        name: "replaced".to_string(),
        flag: true,
    },
);
```

即使 `set_store.update(cx, |_s| {})` 也会发出通知。如果下游使用方只应观察特定切片的变化，请使用 `select`。

## 使用 `For` 渲染值

`For` 接受 `each` 迭代器、可选的 `fallback`，以及一个接收 `(item, index)` 的子闭包。下面的重点摘录渲染拥有所有权的字符串和空状态消息：

```rust
let items = vec!["alpha".to_string(), "beta".to_string()];

view! {
    <div>
        <For each={items} fallback={view! { <span>{"No items"}</span> }}>
            {move |item, index| view! {
                <span>{format!("{}: {}", index, item)}</span>
            }}
        </For>
    </div>
}
```

`For` 在渲染期间枚举迭代器并构建各行元素，适合直接根据项目值派生的行。它不会执行 keyed reconciliation，也不提供 `Index` 那种按位置划分的响应式子作用域。父视图重新渲染时，变化后的集合也会重新渲染。

## 使用 `Index` 保留按位置划分的作用域

`Index` 同样具有 `each`、可选的 `fallback` 和 `(item, index)` 闭包：

```rust
view! {
    <Index each={vec![1u32, 2, 3]} fallback={view! { <div>{"empty"}</div> }}>
        {|n, i| view! { <div>{format!("{}: {}", i, n)}</div> }}
    </Index>
}
```

在响应式作用域内，每个位置都会获得一个持久的子作用域。只要项目仍处于该位置，在行闭包中创建的 signal、memo 和 effect 就能跨渲染保留。列表缩短时，多余位置的作用域会被销毁，并运行其清理回调。

这里的标识**基于位置，而不是项目 ID**。插入或重排项目可能让已有位置作用域对应不同的项目值。需要按位置保留行内状态时选择 `Index`；直接渲染值时选择 `For`。两种控件都要求唯一的子内容是闭包，也都支持空列表回退内容。

## 测试覆盖与后续阅读

原生 store 集成测试驱动真实 GPUI 窗口执行计数、名称、组合、无关字段、完整状态和空操作更新，然后断言记录的值。列表编译覆盖包含非空和空 `Index` 场景；一个 compile-fail UI 测试会拒绝非闭包的 `Index` 子内容。这些检查不是指针交互测试或浏览器渲染测试。

在[第一个响应式视图](../../first-reactive-view/)中比较 store 通知与 signal 更新，或返回完整的[计数器](../counter/)查看最小挂载应用。平台验证边界请参阅[原生与 Web](../../native-and-web/)。
