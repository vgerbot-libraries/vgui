# Contributing

Contributions to `vgui` are welcome! This guide covers the basics of getting
set up for development.

## Development Setup

### Clone and build

```bash
git clone https://github.com/vgerbot-libraries/vgui.git
cd vgui
cargo build
```

The first build compiles `gpui` and its graphics backends, so expect a longer
initial compile. See [Installation](./getting-started/installation.md) for
system library prerequisites.

### Run the examples

The examples are the fastest way to verify your changes:

```bash
cargo run -p vgui-counter
cargo run -p vgui-todolist
cargo run -p vgui-inputs
cargo run -p vgui-tags-demo
```

### Run tests

```bash
cargo test --workspace
```

Integration tests live in `crates/vgui/tests/`. The `element_id` test suite
verifies that auto-generated element ids are stable across re-renders.

## Code Style

- Follow standard `rustfmt` formatting. Run `cargo fmt` before committing.
- Run `cargo clippy --workspace` and address warnings.
- Keep public API items documented with `///` doc comments — these appear in
  rustdoc.
- Prefer the existing patterns in the codebase:
  - Proc-macro crates use hand-rolled token-tree parsing (no `syn`-based JSX
    parser).
  - Built-in elements map to `gpui::div()` with builder chains.
  - Reactivity follows the slot-index model (like React hooks).
- When adding a new HTML element, add it to the `emit_builtin` match in
  `crates/vgui-view/src/builtin.rs`.
- When adding a new CSS property, add it to the appropriate category module in
  `crates/vgui-css/src/` (`layout.rs`, `box_model.rs`, `visual.rs`, or
  `text.rs`).
- When adding a new Tailwind utility, add it to `emit_exact` or `emit_prefixed`
  in `crates/vgui-tailwind/src/lib.rs`.

## Adding Documentation

The mdBook lives in `book/src/`. To preview changes:

```bash
cargo install mdbook
cd book
mdbook serve --open
```

When adding a new page, update `book/src/SUMMARY.md` to include it in the
table of contents.

The book content should be grounded in the actual source code — verify API
signatures, attribute lists, and behavior against the implementation before
documenting them.

## Reporting Issues

Report bugs and request features on the
[GitHub issue tracker](https://github.com/vgerbot-libraries/vgui/issues).

When reporting a bug, please include:

1. The Rust toolchain version (`rustc --version`).
2. The OS and window system (e.g., Linux/Wayland, macOS, Windows).
3. A minimal reproduction — the smallest `view!` snippet that triggers the
   issue.
4. The expected behavior vs. actual behavior.
5. Any relevant compiler output or runtime panics.

Since `vgui` is early-stage, breaking changes are expected between releases.
If you are building against a specific commit, pin your dependency with a
`rev` or `tag` specifier in `Cargo.toml`.
