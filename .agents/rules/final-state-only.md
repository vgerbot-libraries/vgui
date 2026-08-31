---
name: final-state-only
description: Code and documentation MUST reflect only the final state of a change. Never record the change process — edit history, before/after diffs, "previously was", migration notes, or commented-out old versions — inside source files or docs. Applies to all source, docs, comments, and book pages.
globs:
  - "**/*.rs"
  - "**/*.toml"
  - "**/*.md"
  - "**/*.html"
  - "**/*.js"
  - "**/*.ts"
paths:
  - "crates/**"
  - "examples/**"
  - "book/**"
  - "scripts/**"
  - "*.md"
  - "*.toml"
trigger: auto
tags: [final-state, no-edit-history, code-hygiene, documentation]
---

# Rule: Final State Only

This project is under active development. Every file MUST read as though it
was written once, correctly, in its final form. The **process** of getting
there — intermediate versions, refactor steps, what something used to be —
belongs in git history and commit messages, **never** in the files
themselves.

## 1. What is forbidden inside files

Never leave any trace of the change process in source or documentation:

- **No before/after narration.** No "previously X, now Y", "changed from A
  to B", "was renamed", "used to be", "old behavior", "before this change".
- **No edit-history comments.** No `// changed 2024-...`, `// updated to
  use...`, `// refactored: ...`, `// fixed bug where ...`. Comments explain
  *what the code does* and *why*, never *how it got there*.
- **No commented-out old code.** Delete the old version. Git remembers it.
  Keeping `// old_impl();` next to `new_impl();` is noise, not safety.
- **No migration/transition scaffolding.** No "kept for compatibility",
  "temporary until X", "TODO: remove once migration complete", "legacy
  path". If it is no longer the intended behavior, remove it. If it is still
  needed, write it as a normal, first-class part of the code.
- **No diff-style markers in docs.** No `~~old~~ new`, no `- removed` /
  `+ added` blocks inside prose, no "In v0.3 this was …". Documentation
  describes the current system as it stands today.
- **No "WIP" / "draft" / "placeholder" / "to be replaced" markers.** A file
  is either complete and correct, or the work is not done (see §3).

## 2. What is allowed

- **Comments that explain intent or non-obvious invariants.** "We use
  `Rc` here because vgui is single-threaded" — good. "Switched from `Arc`
  to `Rc` because vgui is single-threaded" — bad; drop the "switched from".
- **`TODO`/`FIXME` for genuinely open future work**, written as a present
  statement of a known gap: "TODO: handle empty input". These describe a
  real current limitation, not the history of a change. Do NOT use them to
  record what was just changed or what a refactor left behind.
- **Changelog files** (if the project adopts one) are the one place change
  history is appropriate. As of now this project has no changelog; do not
  create one without explicit request.
- **Commit messages and PR descriptions** carry the change process. That is
  their job, not the files'.

## 3. "Final" means complete, not frozen

"Final state" does not mean the code can never change again. It means each
commit should leave every touched file in a coherent, self-consistent state
with no leftover scaffolding from the edit that produced it. When you make
a change:

1. Apply the change.
2. Remove every trace of the old version and of the editing act itself
   (commented code, transition comments, before/after notes).
3. Re-read the file top to bottom. It must read as if written fresh.

If a change is genuinely incomplete and cannot be finished in this pass,
state that to the user explicitly — do **not** bury a "WIP"/"placeholder"
marker in the file as a silent confession. An unfinished file with a
`TODO: implement` stub is acceptable **only** when the task itself is
explicitly scoped to leave a stub; otherwise finish the work.

## 4. Examples

### Bad — edit history in a comment

```rust
// Previously we stored colors as hex strings, but now we use the Color
// enum for type safety.
pub color: Color,
```

### Good — comment explains the current design only

```rust
// Stored as a typed enum rather than a raw string so invalid colors
// are caught at compile time.
pub color: Color,
```

### Bad — commented-out old code kept "just in case"

```rust
fn render(&self, cx: &mut App) {
    // old_render_path(self, cx);
    new_render_path(self, cx)
}
```

### Good — old code deleted

```rust
fn render(&self, cx: &mut App) {
    new_render_path(self, cx)
}
```

### Bad — doc narrates the change

> The `view!` macro was updated in 0.4 to accept spread attributes. Before
> that you had to call `.spread()` manually.

### Good — doc states current capability

> The `view!` macro accepts spread attributes via the `..` syntax.

## 5. Verification

Before considering any file edit done:

- Grep the touched files for forbidden markers: `previously`, `used to`,
  `was renamed`, `changed from`, `before this`, `old_`, `legacy`,
  `kept for`, `temporary`, `WIP`, `placeholder`, `to be replaced`,
  `// TODO: remove`, and commented-out code blocks.
- Re-read each touched file end to end. If any sentence or comment only
  makes sense if you know the prior version, delete or rewrite it.
