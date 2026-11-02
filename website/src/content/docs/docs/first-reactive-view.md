---
title: Your First Reactive View
description: Understand signal reads, writes, dependency tracking, and rendering in the vgui counter.
---

The [Quick Start](../quick-start/) counter uses a small reactive state graph. This page examines excerpts from that complete program rather than introducing a second example.

## Create and read state

```rust
let (count, set_count) = create_signal(0i32);
```

`create_signal` returns two handles:

- `count` is the read handle. Calling `count.get()` clones the current value and records the read when dependency tracking is active.
- `set_count` is the write handle. It provides operations such as `set` and `update` for changing the value through the GPUI application context.

The view reads the signal when constructing its text:

```rust
<span>{format!("count = {}", count.get())}</span>
```

Signals are created inside a vgui reactive render scope. Their state is cached in positional slots so repeated renders reuse the existing value instead of resetting it. Keep reactive primitive creation order and types stable across renders; do not conditionally reorder signal, memo, or effect calls.

## Update state from an event

```rust
on:click={click(move |cx| set_count.update(cx, |n| *n += 1))}
```

The `click` helper adapts the closure to a GPUI click listener. When the button is clicked, `update` mutates the integer through `cx`. If the value changed, vgui notifies dependents and requests a root view render. Setting or writing an equal `PartialEq` value does not trigger that notification.

This is not a virtual-DOM diff. Memos and effects track their signal dependencies, while signal notifications ultimately call `cx.notify()` on the mounted root view.

## Derive values and run effects

`create_memo` computes and caches a derived value from tracked signal reads. It recomputes when a dependency changes and avoids publishing an equal result.

`create_effect` also tracks signal reads, but it runs a side effect instead of returning a value. Its closure runs once when registered during rendering and reruns when a tracked dependency changes. Reactive resources belong to their scope; scope cleanup is important when you move on to timers or subscriptions.

For this first counter, no memo or effect is necessary: the view can derive the display string directly from `count`.

## What to check when state does not update

1. Read the signal in the view, memo, or effect that should depend on it.
2. Change it with the write handle and the appropriate application context.
3. Keep reactive primitive calls in a stable order across renders.
4. Do not expect browser DOM nodes, a virtual DOM, or per-text-node updates.

See the [vgui API documentation](../../api/vgui/) for complete signatures, then continue to [Native & Web](../native-and-web/) for platform boundaries.
