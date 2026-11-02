---
title: Introduction
description: Learn what vgui is, how it relates to GPUI, and where to start.
---

vgui is an experimental, declarative GUI framework for Rust built on [GPUI](https://github.com/zed-industries/zed), the GPU-accelerated UI toolkit used by Zed. It combines GPUI windows and rendering with a web-style authoring experience.

## What vgui provides

- **Declarative views:** the `view!` macro supports elements, attributes, interpolation, fragments, components, and control flow such as `Show` and `For`.
- **Reactive state:** signals, memos, and effects track dependencies and update application state without a browser DOM.
- **Rust styling tools:** use `css!`, Tailwind-style classes, dynamic classes, themes, and component variants where supported.
- **Desktop and input primitives:** build on GPUI windows, events, text input, focus, overlays, and accessibility integrations.

vgui is not a web framework or a browser DOM compatibility layer. Its markup compiles to GPUI elements, and its supported HTML-like syntax, CSS properties, and Tailwind utilities are subsets with their own behavior.

## Is vgui a good fit?

vgui is useful if you want to explore declarative Rust interfaces on GPUI, prototype native applications, or experiment with signal-driven UI architecture.

It is currently **early-stage, experimental software**. APIs can change between revisions, and the project uses Git dependencies rather than stable crates.io releases. Do not expect complete web API compatibility or stable behavior across undocumented platforms.

## Start here

Follow these pages in order to build a small counter application:

1. [Installation](./installation/) prepares Rust and the native build dependencies.
2. [Quick Start](./quick-start/) creates a complete application from an empty directory.
3. [Your First Reactive View](./first-reactive-view/) explains the counter's state and update flow.
4. [Native & Web](./native-and-web/) describes the two targets and their current verification boundaries.
5. [Compatibility & Troubleshooting](./compatibility-troubleshooting/) helps diagnose toolchain, dependency, and window problems.

For complete Rust types and function signatures, use the [API documentation](../api/vgui/). Guides explain tasks; Rustdoc remains the API reference.
