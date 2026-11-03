---
title: 数据展示
description: 使用 Card、Badge、DescriptionList 和 DataTable 函数 API 展示结构化内容。
---

以下示例都是可直接用于 vgui 应用的完整渲染函数。请按照[快速开始](../../docs/quick-start/)挂载它们。每个函数都应用浅色主题；主题设置请参阅[组件概览](../)。

## Card 与 Badge

`Card` 接收任意子元素。`CardHeader` 提供标题和描述，`Badge` 使用四种 variant 之一标记内容：

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn project_card() -> impl gpui::IntoElement {
    apply_theme(false);

    Card {
        children: vec![
            CardHeader {
                title: "vgui-shadcn".to_string(),
                description: "Styled components for vgui".to_string(),
            }
            .into_any_element(),
            Badge {
                variant: BadgeVariant::Secondary,
                children: vec!["Rust".to_string().into_any_element()],
            }
            .into_any_element(),
        ],
    }
}
```

`Card` 提供主题化容器、内边距、边框和纵向间距，但没有定义独立的 body 或 footer 组件。`CardHeader` 总会渲染 `title` 和 `description` 两个字符串。`BadgeVariant` 支持 `Default`、`Secondary`、`Outline` 和 `Destructive`，徽章子元素类型为 `Vec<gpui::AnyElement>`。

## DescriptionList

`DescriptionList` 渲染可选标题和 `(term, detail)` 行。每个 detail 都是 `AnyElement`，因此既可以包含文本，也可以包含其他组件：

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn package_details() -> impl gpui::IntoElement {
    apply_theme(false);

    DescriptionList {
        title: Some("Package".to_string()),
        items: vec![
            (
                "Name".to_string(),
                "vgui-shadcn".to_string().into_any_element(),
            ),
            (
                "Kind".to_string(),
                Badge {
                    variant: BadgeVariant::Outline,
                    children: vec!["Component library".to_string().into_any_element()],
                }
                .into_any_element(),
            ),
        ],
    }
}
```

将 `title` 设为 `None` 可省略标题。term 列固定为 160 px；每一行都有底边框，包括最后一行。此组件用于结构化的只读详情，不提供表单双向绑定。

## DataTable

`table` 是函数 API。`DataTable::new` 接收固定宽度的列和字符串单元格行，函数返回 `(gpui::AnyElement, Store<TableState>)`：

```rust
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

fn team_table() -> impl gpui::IntoElement {
    apply_theme(false);

    let (element, state) = table(DataTable::new(
        vec![
            TableColumn::new("Name", 120.0).sortable(),
            TableColumn::new("Role", 160.0),
        ],
        vec![
            vec!["Alice".into(), "Engineer".into()],
            vec!["Bob".into(), "Designer".into()],
            vec!["Carol".into(), "Engineer".into()],
        ],
    ));

    view! {
        <div class="flex flex-col gap-2 p-4">
            <span>{format!("{} row(s) selected", state.get().selected.len())}</span>
            {element}
        </div>
    }
}
```

- `TableColumn::new(title, width)` 创建不可排序的列；链式调用 `.sortable()` 可启用点击表头排序。
- 点击可排序表头会按升序、降序、不排序循环。排序按单元格字符串的字典序比较，不会按数字或日期解析。
- 点击表体行会切换其选中状态。可以同时选中多行；即使排序后，`TableState.selected` 仍保存原始输入顺序中的行索引。
- 通过返回的 store 读取 `sort_col`、`sort_asc` 和 `selected`。列宽是固定像素值，单元格是字符串，而不是任意子元素。

这是 crate 自带的表格实现，不是 TanStack Table 或 Tabulator 集成。返回[组件目录](../)查看其他展示模块。
