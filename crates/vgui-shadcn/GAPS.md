# vgui-shadcn gaps

Components and framework features that cannot be ported because gpui/vgui has no equivalent. These are current limitations, not deferred work inside this crate.

## Skipped components

| Component | Blocker |
| --- | --- |
| apple-dock | Hover scale needs `transform` |
| marquee | Transform-based scroll |
| spinner (rotate) | `animate-spin` / rotate |
| watermark | Rotated overlay |
| chart | JS chart library (recharts); no canvas chart port |
| tanstack-table | JS table engine |
| tabulator-table | JS table engine |
| landing-hero | Decorative `background-image` |
| hoc/data-table | Depends on tanstack-table |

## Missing framework features

| Feature | Impact |
| --- | --- |
| `transform` / `translate` / `rotate` / `scale` | Dock magnification, marquee, watermark, spinner |
| `animate-spin` | Loading spinners |
| `position: fixed` / `sticky` | Sticky player chrome uses flex layout instead |
| `background-image` | Hero/artwork photos; music player uses color blocks |
| HTML `<audio>` | Music player simulates playback with `set_interval` |
| `:focus-visible` / `:focus-within` | Focus rings use `:focus` |
