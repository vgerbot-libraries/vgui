---
name: vgui-extend
description: Extend the vgui framework itself — view!/css!/tw! proc-macros, built-in HTML tags, CSS properties, Tailwind utilities, reactivity, widgets, WASM helpers. Use when changing crates/vgui*, adding language features, or updating book API pages to match source.
---

# vgui Framework Extension

This skill is for **framework work** in `crates/`, not application UI. For app/example code use `vgui-authoring`.

Follow `.agents/rules/cross-platform.md` and `final-state-only.md`. Docs describe current behavior only — no "previously / now" narration.

## Workspace map

| Crate | Role | Touch when |
| --- | --- | --- |
| `crates/vgui` | Runtime: reactivity, mount, widgets, theme, router, overlays | Signals, inputs, prelude exports |
| `crates/vgui-view` | `view!` parser + emit | Tags, attributes, control flow |
| `crates/vgui-css` | `css!`, `theme!`, `variants!` | CSS properties, variants syntax |
| `crates/vgui-tailwind` | `tw!` emit | Compile-time class → gpui |
| `crates/vgui-tailwind-core` | Shared parse/tables (no gpui) | Class names, spacing, colors — keep in sync with `tw_dynamic` |
| `crates/shortcuts` | Keyboard shortcut crate | Shortcut parsing / keymap |

Proc-macros are independent of the `vgui` crate at compile time. They **emit** `::vgui::*` and `::gpui::*` paths. Applications depend on `vgui` + `gpui`.

Nightly + `wasm32-unknown-unknown` (`rust-toolchain.toml`). vgui is **single-threaded** (`Rc` / `RefCell` / `thread_local`). Do not add `Send`/`Sync` bounds to public APIs.

## Where to add what

| Feature | File |
| --- | --- |
| Built-in HTML tag | `crates/vgui-view/src/builtin.rs` (`emit_builtin` match) |
| `view!` attribute kind | `crates/vgui-view/src/lib.rs` `AttrKind` + parse/emit |
| Control-flow tag | `crates/vgui-view/src/control.rs` + special-case in `emit_element` |
| `<Provider>` | `crates/vgui-view/src/provider.rs` |
| Custom component emit | `crates/vgui-view/src/component.rs` |
| CSS property | `crates/vgui-css/src/{layout,box_model,visual,text}.rs` + parse/keywords |
| `variants!` / `theme!` | `crates/vgui-css/src/variants.rs`, `lib.rs` |
| Tailwind utility | `crates/vgui-tailwind/src/lib.rs` (`emit_exact` / `emit_prefixed`) **and** `vgui-tailwind-core` tables so `tw_dynamic` matches |
| Runtime widget | `crates/vgui/src/input_*.rs` / `control.rs` / `overlay.rs` … then `lib.rs` + `prelude.rs` |
| Reactivity | `crates/vgui/src/reactive.rs`, `root.rs` |

Parser style: **hand-rolled token-tree parsing** (`TokenTree` walks). Do not add a JSX parser crate. Use `syn` for Rust literals/errors, `quote` for emit.

## Invariants

1. **Slot identity.** `create_signal` and friends are indexed by call order per scope. Conditional `create_*` is a bug. Per-branch state belongs in `<Switch>` / `<Index>` child scopes.
2. **Re-render is top-down.** `app()` re-runs; slots reuse state. gpui is immediate-mode — no persistent DOM. Persist widget entities with scope slots (`get_or_create_view`), not by storing elements.
3. **WASM + native.** Gate platform code with `cfg(target_family = "wasm")`. WASM-only deps go in `[target.'cfg(target_family = "wasm")'.dependencies]`. No `std::fs` / threads / `application()` on WASM.
4. **Prelude is the public surface.** New app-facing items: `pub use` in `crates/vgui/src/lib.rs` and `prelude.rs` if they belong in everyday authoring.
5. **Hidden emit helpers** stay `__`-prefixed (`__bind_ref`, `__apply_breakpoint_styles`, …). Do not document them as public API.
6. **`css!` is strict; `tw!` is lenient.** Unknown CSS properties → compile error. Unknown Tailwind classes → skip. Keep that split.
7. **gpui mapping.** Most tags are `gpui::div()` plus default styles. Inputs/img/svg/canvas are the exceptions. Do not pretend gpui has CSSOM, list-style markers, or navigable `<a href>`.

## Docs and examples

Book lives in `book/src/`. After an API change:

1. Update the matching page (concepts / styling / elements / api-reference).
2. Ground every signature in source — do not document hoped-for CSS/HTML.
3. If the feature needs a demo, **extend an existing example** unless it is a new theme. Registration for a new example: root `Cargo.toml` members, `scripts/build_docs.sh` `EXAMPLES`, `book/src/SUMMARY.md`.

## Tests to add with the change

| Change | Test |
| --- | --- |
| New `css!` error / `view!` reject | `crates/vgui/tests/ui/*.rs` + `*.stderr` (`trybuild` in `tests/ui.rs`) |
| Widget / tag still constructs | `crates/vgui/tests/widgets_compile.rs` (use `RenderScope` + `__test_enter_render_scope`) |
| Reactivity / store | `crates/vgui/tests/*.rs`; window tests `#![cfg(not(target_family = "wasm"))]` |
| Tailwind parse tables | unit tests next to `vgui-tailwind-core` / `tw_dynamic` |

`trybuild` stderr must match rustc output exactly. Update the `.stderr` file when the diagnostic text changes intentionally.

## Verification (mandatory)

```bash
cargo +nightly check
cargo +nightly check --target wasm32-unknown-unknown
cargo +nightly test -p vgui
```

For example-facing changes also check the example crate on both targets.

A native-only green check is not done.
