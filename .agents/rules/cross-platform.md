---
name: cross-platform
description: This project is multi-platform — native (Linux) and web (WASM). Every change MUST compile and run correctly on ALL supported targets. Applies to all source, dependency, and build changes.
globs:
  - "**/*.rs"
  - "**/Cargo.toml"
  - ".cargo/config.toml"
  - "rust-toolchain.toml"
paths:
  - "crates/**"
  - "examples/**"
  - "Cargo.toml"
  - ".cargo/config.toml"
  - "rust-toolchain.toml"
trigger: auto
tags: [cross-platform, wasm, native, compilation, compatibility]
---

# Rule: Cross-Platform Compatibility

This project targets **multiple platforms**. Every modification — new code,
dependency changes, build config, or refactors — MUST keep **all** targets
compiling and running correctly.

## 1. Supported targets

| Target | Triple | Entry | Notes |
|--------|--------|-------|-------|
| Native (Linux) | host triple (e.g. `x86_64-unknown-linux-gnu`) | `fn main()` | Wayland + X11 via `gpui_platform` features |
| Web | `wasm32-unknown-unknown` | `#[wasm_bindgen(start)]` | Single-threaded; runs in the browser |

The toolchain is **nightly** with `wasm32-unknown-unknown` pre-installed
(see `rust-toolchain.toml`). Always use `cargo +nightly`.

## 2. Mandatory verification before considering work done

After ANY source or dependency change, verify **both** targets compile:

```bash
# Native
cargo +nightly check

# WASM
cargo +nightly check --target wasm32-unknown-unknown
```

For examples and binaries that have a runtime path, also verify they run:

```bash
# Native
cargo +nightly run -p <package>

# WASM (build + bindgen + serve, or use scripts/build_wasm.sh <name>)
cargo +nightly build --target wasm32-unknown-unknown -p <package> --release
wasm-bindgen --target web --out-dir examples/<name>/dist \
    --no-typescript target/wasm32-unknown-unknown/release/<name>.wasm
```

**Do NOT claim a change is complete after checking only one target.** A native
`cargo check` passing does NOT imply WASM works, and vice versa.

## 3. Platform-specific code patterns

### 3.1 Use `cfg(target_family = "wasm")` for conditional code

Gate platform-specific logic with `cfg`, never with runtime checks:

```rust
// Correct — compile-time gating
#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;
```

### 3.2 Platform-specific dependencies

Put WASM-only dependencies under a target-specific section in `Cargo.toml`:

```toml
[target.'cfg(target_family = "wasm")'.dependencies]
wasm-bindgen = "0.2"
web-sys = { version = "0.3", features = ["Window", "Event"] }
```

Never add a WASM-only crate (e.g. `web-sys`, `js-sys`, `wasm-bindgen`) to the
generic `[dependencies]` section — it will break native compilation.

Conversely, never add a native-only crate (anything that links to a C library,
uses threads, filesystem APIs, or OS-specific bindings) without gating it
behind `cfg(not(target_family = "wasm"))`.

### 3.3 Single-threaded constraint

vgui uses `Rc`, `RefCell`, and `thread_local` internally — it is
**single-threaded**. This has two consequences:

1. **On WASM**, use `gpui_platform::single_threaded_web()`, NOT
   `application()`. The latter enables multi-threading via web workers, which
   causes `Send`/`Sync` violations.

2. **Do not introduce `Send`/`Sync` requirements** in public APIs. Code that
   compiles natively may fail on WASM if it requires a type to be `Send`.

### 3.4 No unsupported std APIs on WASM

WASM has no filesystem, no native threads, no `std::time::Instant`
(on some targets), and no process spawning. Avoid:

- `std::fs`, `std::process`, `std::thread`
- `std::time::SystemTime` (use `web-sys` `performance.now()` on WASM)
- Any crate that transitively requires these without a WASM fallback

If a feature is genuinely unavailable on one platform, gate it with `cfg` and
provide a fallback or omit it — do not let it silently break compilation.

## 4. Adding new dependencies

Before adding any dependency:

1. Confirm it compiles for `wasm32-unknown-unknown` (check its docs/Cargo
   features for a `wasm`/`web` feature gate).
2. If it is platform-specific, gate it under the appropriate
   `[target.'cfg(...)'.dependencies]` section.
3. Re-run both `cargo +nightly check` and
   `cargo +nightly check --target wasm32-unknown-unknown`.

## 5. CI / build expectations

Both targets are expected to build cleanly. A change that breaks either target
is a regression, even if it was not the target you were working on. When in
doubt, run the checks in §2 before yielding.
