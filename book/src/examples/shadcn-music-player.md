# Shadcn Music Player Example

## Live Demo

<iframe src="../wasm/shadcn-music-player/" width="100%" height="640" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

A full-screen music player built with `vgui-shadcn` components. Layout matches a streaming player: playlist sidebar, searchable track list, and a persistent bottom bar with transport, seek, and volume.

Playback is simulated with `use_interval` — there is no HTML `<audio>` backend.

## Key Concepts

- **`vgui-shadcn` tokens.** `apply_theme(true)` installs the dark shadcn preset (`oklch()` CSS variables).
- **Components.** `Sidebar`, `Avatar`, `Separator`, `TextField`, `Slider`, `Popover`, `Drawer`, `Button`.
- **Simulated transport.** `playing` gates a 250ms interval that advances `current_time`. End-of-track respects shuffle and repeat.
- **Overlays.** Volume uses `Popover` + `floating_at(NodeRef)`. The menu button opens a `Drawer` for the same nav as the sidebar.

## Running

**Native:**

    cargo run -p vgui-shadcn-music-player

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-shadcn-music-player --release
    wasm-bindgen --target web --out-dir examples/shadcn-music-player/dist \
        --no-typescript target/wasm32-unknown-unknown/release/shadcn-music-player.wasm
    python3 scripts/serve_plain.py 8080 examples/shadcn-music-player

## Source Code

See `examples/shadcn-music-player/src/main.rs`.
