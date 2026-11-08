---
name: writing-component-docs
description: Conventions for live, bilingual examples in the Astro component documentation. Applies when editing component guides, their Rust examples, or website preview infrastructure.
globs:
  - "crates/vgui-shadcn/examples/**"
  - "website/src/content/docs/components/**"
  - "website/src/content/docs/zh-cn/components/**"
  - "website/src/components/LiveRustExample.astro"
  - "website/public/wasm/**"
  - "website/scripts/build-wasm.mjs"
paths:
  - "crates/vgui-shadcn/examples/**"
  - "website/src/content/docs/components/**"
  - "website/src/content/docs/zh-cn/components/**"
  - "website/src/components/LiveRustExample.astro"
  - "website/public/wasm/**"
  - "website/scripts/build-wasm.mjs"
trigger: auto
tags: [docs, components, examples, wasm, i18n]
---

# Rule: Writing Component Documentation

Every example that demonstrates component structure, appearance, or interaction
in the website component guides must have a real, precompiled WASM preview.

## 1. Live previews are the default

- Put each runnable example in `crates/vgui-shadcn/examples/<name>.rs` and
  register it in `website/scripts/build-wasm.mjs`.
- Render its source and preview with `LiveRustExample`. The displayed source
  must come from the same Rust file through a raw import; never maintain a
  second copy in a Markdown fence.
- A static example still needs a visual preview. Lack of interaction is not an
  exemption.
- Screenshots, recordings, static UI code blocks, and manually duplicated
  source are not substitutes for a live preview.
- Examples must support native and WASM targets and use a root layout that
  adapts to the preview canvas.

## 2. Genuine code snippets are exempt

Imports, dependency or tool configuration, shell commands, isolated API
signatures or types, and partial mechanisms that do not render UI may remain
fenced code snippets. Explain any required context or omitted setup.

A code block that constructs a component, calls `view!`, demonstrates visual
styling, or binds UI interaction is an example, not an exempt snippet. Do not
avoid the preview requirement by calling such an example an excerpt.

## 3. Keep locales and behavior synchronized

- English and `zh-cn` pages must contain the same examples in the same order,
  with equivalent API explanations and limitations.
- Both locales must reference the same example name and Rust source. Localize
  prose, preview labels, and iframe titles rather than maintaining localized
  Rust copies.
- Preserve the same relative documentation route in both locales.

## 4. Verify the preview, not just the page

When adding or changing an example:

1. Check its native and `wasm32-unknown-unknown` compilation.
2. Build the website so its registered JS and WASM artifacts are generated.
3. Verify that the preview initializes and displays a visible canvas.
4. Exercise the example's core interactions when it has interactive behavior.
5. Check responsive layouts and ensure generated WASM artifacts are not
   committed.

A successful `cargo check`, HTTP 200 response, or iframe element alone does not
establish that a preview works. Follow `cross-platform.md` for dual-target
requirements and report any runtime verification that could not be performed.
