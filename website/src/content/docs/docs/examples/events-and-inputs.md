---
title: Events and Inputs
description: Handle normalized vgui events and typed input callbacks while respecting native and browser test boundaries.
---

vgui exposes web-aligned event data in `view!` handlers, but the elements and events still come from GPUI rather than browser DOM nodes. The normalized event excerpt below follows `crates/vgui/tests/events.rs`; the widget excerpts follow `crates/vgui/tests/widgets_compile.rs` and the current input callback types.

## Attach normalized event handlers

Keyboard, pointer, resize, double-click, context-menu, and wheel handlers receive a borrowed event, a window, and an application context. This focused excerpt builds one element with all nine normalized event attributes:

```rust
let el = view! {
    <div
        on:keydown={move |_e: &KeyboardEvent, _w, _cx| {}}
        on:keyup={move |_e: &KeyboardEvent, _w, _cx| {}}
        on:pointerdown={move |_e: &PointerEvent, _w, _cx| {}}
        on:pointerup={move |_e: &PointerEvent, _w, _cx| {}}
        on:pointermove={move |_e: &PointerEvent, _w, _cx| {}}
        on:resize={move |_e: &ResizeEvent, _w, _cx| {}}
        on:dblclick={move |_e: &PointerEvent, _w, _cx| {}}
        on:contextmenu={move |_e: &PointerEvent, _w, _cx| {}}
        on:wheel={move |_e: &WheelEvent, _w, _cx| {}}
    >
        {"events"}
    </div>
};
let _any = el.into_any_element();
```

Use the event fields instead of depending on GPUI's original event structs:

- `KeyboardEvent` provides `key`, best-effort `code`, `repeat`, `key_char`, and `shift_key`, `ctrl_key`, `alt_key`, `meta_key`. Unknown physical key codes produce an empty `code`. Call `e.stop_propagation()` when propagation should stop.
- `PointerEvent` provides window-space `client_x` and `client_y`, `button`, `buttons`, modifier flags, `click_count`, and pointer metadata. Pointer normalization currently populates mouse data; the presence of `Pen` and `Touch` variants does not establish touch or pen event support.
- `ResizeEvent` contains the window viewport's new `width` and `height`. It is not a per-element resize observer and does not force an element ID.
- `WheelEvent` exposes `delta_x`, `delta_y`, and modifier flags. Scroll deltas are normalized to pixels; line deltas use a 16-pixel line height.

These names resemble browser APIs so event-handling code is familiar, not because a native view has become a DOM tree.

## Update state from text input

Text-based inputs and text areas use `on:input` with `(&str, &mut App)`. A controlled text field can copy each new string into a signal:

```rust
let (text, set_text) = create_signal(String::new());

view! {
    <input
        type="text"
        value={text.get()}
        on:input={move |value: &str, cx: &mut App| {
            set_text.set(cx, value.to_string());
        }}
    />
}
```

The input callback receives the current string, so convert it to an owned `String` before storing it. The general signal update and rendering behavior is the same as in [Your First Reactive View](../../first-reactive-view/).

## Handle typed selection changes

A select's `on:change` callback receives `&str` and `&mut App`. The current compile test exercises flat options, grouped options, and multiple selection. This is its multiple-selection excerpt:

```rust
view! {
    <select
        multiple={true}
        options={vec![
            ("a".to_string(), "Alpha".to_string()),
            ("b".to_string(), "Beta".to_string()),
        ]}
        value={"a,b".to_string()}
        on:change={move |_v: &str, _cx: &mut App| {}}
    />
}
```

For multiple selection, the value is a comma-separated string. Other controls deliberately use different callback value types: checkbox and radio changes use `bool`, range changes use `f64`, and file changes use `Vec<std::path::PathBuf>`. Match the closure to the control rather than assuming every event delivers a string.

## Forms and text areas

Form submit and reset callbacks receive the application context. The widget compile test also checks text-input validation attributes and submit/reset input types:

```rust
view! {
    <form on:submit={move |_cx| {}} on:reset={move |_cx| {}}>
        <input type="text" required={true} pattern={"abc"} minlength={1usize} />
        <input type="submit" value="Go" />
        <input type="reset" value="Clear" />
    </form>
}
```

A text area accepts a `u32` row count and supports the same `&str` input callback shape as text-based inputs:

```rust
view! {
    <textarea
        rows={4u32}
        on:input={move |_value: &str, _cx: &mut App| {}}
    />
}
```

## What the current tests establish

The event integration test constructs the normalized-event element inside a test render scope and converts it into an `AnyElement`; it also checks that a resize-only `div` has no element ID. The widget test compile-checks or constructs the controls and callbacks shown above. In particular, its `textarea` check type-checks the view without constructing the widget because a text area needs a mounted root slot.

These are compile and element-construction checks. They do not dispatch native keyboard or pointer input, open a select, submit a form, edit a text area, or exercise browser focus, IME, canvas input, or accessibility. Native interaction and browser interaction each require separate validation; see [Native & Web](../../native-and-web/). For a complete native mount and a click-driven state update, use the [Counter](../counter/).
