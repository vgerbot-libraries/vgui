---
name: vgui-test
description: Write and run vgui tests — trybuild compile-fail, render-scope widget tests, native gpui window integration tests, dual-target checks. Use when adding tests, fixing compile-fail diagnostics, or verifying a change before claiming it is done.
---

# vgui Testing

Toolchain is **nightly** (`rust-toolchain.toml`). Prefer `cargo +nightly`.

## What exists

| Kind | Location | Notes |
| --- | --- | --- |
| Unit | `crates/*/src/**` `#[cfg(test)]` | Fast; no window |
| Compile-fail | `crates/vgui/tests/ui/*.rs` + `*.stderr` | `trybuild` via `tests/ui.rs` |
| Widget smoke | `crates/vgui/tests/widgets_compile.rs` | Needs render scope |
| Integration | `crates/vgui/tests/*.rs` | Some open a real gpui window; native-only |
| Examples | `examples/<name>/` | Dual-target binaries, not unit tests |

Window / `application()` tests must be:

```rust
#![cfg(not(target_family = "wasm"))]
```

## Render scope for `view!` / signals

`create_signal` and several widgets panic without an active `VguiRoot` scope. Tests that only need to *construct* elements:

```rust
struct RenderScope;
impl RenderScope {
    fn new() -> Self {
        vgui::__test_enter_render_scope();
        RenderScope
    }
}
impl Drop for RenderScope {
    fn drop(&mut self) {
        vgui::__test_exit_render_scope();
    }
}

#[test]
fn constructs() {
    let _scope = RenderScope::new();
    let el = view! { <button>{"Go"}</button> };
    let _ = el.into_any_element();
}
```

Copy this pattern from `crates/vgui/tests/widgets_compile.rs`. `__test_*` are test-only helpers, not public API.

For type-check without constructing slot-backed widgets:

```rust
fn assert_into_any<E: gpui::IntoElement>(_: fn() -> E) {}
assert_into_any(|| view! { <textarea rows={4u32} /> });
```

## trybuild compile-fail

`crates/vgui/tests/ui.rs` runs every `tests/ui/*.rs` as compile-fail.

1. Add `tests/ui/<case>.rs` — smallest snippet that must not compile.
2. Run `cargo +nightly test -p vgui --test ui -- --nocapture` (or full `cargo +nightly test -p vgui`).
3. Commit the generated/updated `tests/ui/<case>.stderr` **exactly**.

`css!` unknown properties error (`unknown CSS property 'florb'`). `view!` rejects illegal structure (input children, unknown input type, bad `<Switch>`). Do not weaken these tests.

When rustc diagnostic text changes, update `.stderr` in the same change. Do not skip trybuild.

## Native window tests

See `crates/vgui/tests/store.rs`: `gpui_platform::application()`, `open_window`, `vgui::mount`, then drive updates on the UI thread. Use `Arc<Mutex<_>>` / atomics to observe renders. Keep them native-only.

Do not spawn extra threads that call into vgui APIs. The runtime is single-threaded.

## Commands

```bash
# Framework tests
cargo +nightly test -p vgui
cargo +nightly test -p vgui --test ui
cargo +nightly test -p vgui --test widgets_compile

# Workspace (native)
cargo +nightly test --workspace

# Dual-target compile — required after source/Cargo.toml changes
cargo +nightly check
cargo +nightly check --target wasm32-unknown-unknown

# One example
cargo +nightly check -p vgui-counter
cargo +nightly check --target wasm32-unknown-unknown -p vgui-counter
```

WASM trybuild / window tests are not the goal; WASM verification is `cargo check --target wasm32-unknown-unknown` plus `scripts/build_wasm.sh <name>` when the book demo must update.

## Authoring rules for new tests

- Assert behavior, not the edit that produced it.
- Prefer one focused test file per subsystem (`store.rs`, `events.rs`, `ref_handle.rs`).
- Numeric/bool literals in `view!` attrs often need suffixes (`0.5f64`, `1usize`, `true`) — match existing tests.
- After tests, both targets must still compile. Native `cargo test` passing does not prove WASM.
