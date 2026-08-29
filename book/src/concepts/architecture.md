# Architecture

## Workspace Layout

`vgui` is organized as a Cargo workspace with four crates:

| Crate              | Kind        | Description                                                         |
| ------------------ | ----------- | ------------------------------------------------------------------- |
| `vgui`             | lib         | The main crate: reactivity, root mounting, styling traits, widgets. |
| `vgui-view`        | proc-macro  | The `view!` macro — JSX-like syntax parser and code generator.      |
| `vgui-css`         | proc-macro  | The `css!` macro — CSS declaration parser.                          |
| `vgui-tailwind`    | proc-macro  | The `tw!` macro and the Tailwind class registry.                    |

In addition, four example binaries live under `examples/`:

| Example           | Package name     | Demonstrates                                        |
| ----------------- | ---------------- | --------------------------------------------------- |
| Counter           | `vgui-counter`   | Signals, `create_memo`, `<Show>` control flow.      |
| Todo List         | `vgui-todolist`  | `<For>` with fallback, `css!` styling, filtering.   |
| Inputs Demo       | `vgui-inputs`    | All `<input>` types with live echo.                 |
| Tags Demo         | `vgui-tags-demo` | HTML tag coverage, tables, progress, details, etc.  |

## Crate Dependencies

The dependency graph is straightforward:

```
vgui-view ──┐
vgui-css  ──┼──► vgui ──► gpui
vgui-tailwind ┘
```

- `vgui` depends on `gpui` and re-exports the three proc-macro crates (`view`,
  `css`, `tw`) via `pub use`.
- The proc-macro crates are completely independent of `vgui` at the macro
  level — they emit `::vgui::*` and `::gpui::*` qualified paths, so they work
  in any crate that depends on `vgui`.
- Application crates depend on both `vgui` and `gpui`.

The `vgui` crate's internal module structure:

| Module            | Responsibility                                                    |
| ----------------- | ----------------------------------------------------------------- |
| `reactive`        | `create_signal`, `create_memo`, `create_effect`, `ReadSignal`, `WriteSignal`, scope management, dependency tracking. |
| `root`            | `VguiRoot` entity, `Scope` (the reactive owner), `mount()`.       |
| `control`         | `show`, `show_when`, `for_each`, `for_each_or`, `progress`, `details`, `dialog`. |
| `input_text`      | `TextInput` widget (text fields, text areas), `TextInputProps`, `TextAreaProps`, `TextKind`. |
| `input_widgets`   | `checkbox`, `radio`, `range_input`, `file_input`, `select` and their props. |
| `label`           | Label-to-input focus association via `for=` / wrapping.          |
| `style`           | `Css`, `TwStyle`, `ApplyStyle` trait.                             |
| `child`           | `IntoViewChild` trait, `into_child`, `click` helper.              |
| `prelude`         | Re-exports the common API surface + `gpui::prelude::*`.           |

## How Rendering Works

### The render cycle

1. **`vgui::mount(cx, app)`** creates a `VguiRoot` gpui entity. `VguiRoot`
   holds a `Scope` (the reactive owner) and a `Box<dyn FnMut() -> AnyElement>`
   closure wrapping the user's `app()` function.

2. **On every render**, `VguiRoot::render`:
   - Resets `scope.index` to 0 — this is the per-render slot counter that gives
     `create_signal`/`create_memo`/`create_effect` their stable identity across
     re-renders (like React's hooks rules).
   - Calls `enter_scope` to set the current scope and `gpui::Context` in a
     thread-local.
   - Calls `(self.render)()` — the user's `app()` function — which calls
     `create_signal`, `view!`, etc.
   - Calls `exit_scope` to clear the thread-local.
   - Wraps the result in a `gpui::div()` with a `Tab`/`Shift+Tab` key handler
     for focus cycling.

3. **Inside `app()`**, calls to `create_signal`/`create_memo`/`create_effect`
   are resolved by slot index. On the first render, a new signal/memo/effect is
   created and stored in `scope.slots[index]`. On subsequent renders, the
   existing slot is reused — the signal's value persists across re-renders.

4. **`view!` expansion** produces `gpui` element builder expressions. Each
   element is a `gpui::div()` (or `gpui::img()`, `gpui::svg()`, or a vgui
   widget constructor) with `.child()` chains, `.style()` mutations, and event
   handlers attached.

5. **When a signal changes** via `WriteSignal::set` or `WriteSignal::update`,
   `gpui` notifies the `VguiRoot` entity, which calls `notify_dep`. This
   re-evaluates only the memos and effects whose tracked dependencies include
   the changed signal, then calls `cx.notify()` to trigger a re-render of the
   `VguiRoot` entity — which re-runs `app()` from the top.

### Why re-run from the top?

Unlike SolidJS, which compiles components into fine-grained effect graphs,
`vgui` re-runs the entire `app()` closure on each render. The slot model
ensures that `create_signal` returns the *same* signal (not a new one) on every
render, so state is preserved. The cost of re-running the closure is low
because `view!` produces lightweight builder expressions, and `gpui`'s
virtual-DOM-less rendering only repaints what actually changed.

### Stateful widget persistence

Input widgets (`TextInput`, `RangeInput`) are `gpui` entities cached in
reactive scope slots via `get_or_create_view`. On the first render, a new
entity is created and stored. On subsequent renders, the same entity handle is
returned, so cursor position, selection, drag state, and IME composition
persist across re-renders — even though `app()` runs from scratch each time.
