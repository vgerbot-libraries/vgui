# Animations & Transitions

vgui provides Tailwind-compatible animation and transition utilities that map onto
gpui's animation engine. All configuration is parsed at compile time by the `tw!`
macro (or at runtime via `tw_dynamic`), so there is zero per-frame parsing cost.

## Keyframe Animations

Built-in `animate-*` classes run a repeating keyframe loop on the element:

| Class            | Effect                                          |
|------------------|-------------------------------------------------|
| `animate-pulse`  | Opacity fades between 1.0 and 0.5 (2 s loop).   |
| `animate-bounce` | Vertical margin offset ±8 px (1 s loop).        |
| `animate-ping`   | Opacity 1.0 → 0 with slight scale (1 s loop).   |
| `animate-spin`   | No-op (gpui has no transform/rotation for divs).|

```rust
view! {
    <div class="bg-blue-500 rounded p-4 animate-pulse">
        {"Loading…"}
    </div>
}
```

### Custom animation via `animate={...}`

For full control, pass a closure to the `animate` attribute. The closure receives
the element and must return an `AnimationElement` (via `with_animation`):

```rust
use std::time::Duration;
use gpui::AnimationExt;

view! {
    <div
        class="bg-amber-500 rounded p-4"
        animate={|el| el.with_animation(
            "breath",
            gpui::Animation::new(Duration::from_millis(1500))
                .repeat()
                .with_easing(gpui::ease_in_out),
            |mut el, delta| {
                el = el.opacity(0.4 + 0.6 * (delta * std::f32::consts::PI).sin());
                el
            },
        )}
    >
        {"Custom breathing"}
    </div>
}
```

## Transitions

Transitions animate style changes when an element enters or leaves the `hover`
state. Declare a `transition-*` class alongside a `hover:` variant:

| Class                | Properties interpolated |
|----------------------|-------------------------|
| `transition`         | opacity, color, background, margin, padding |
| `transition-opacity` | opacity only            |
| `transition-colors`  | text color, background  |
| `transition-all`     | all animatable properties |

```rust
view! {
    <button class="bg-indigo-500 hover:opacity-50 rounded p-3 transition-opacity duration-300">
        {"Fade on hover"}
    </button>
}
```

### Timing Modifiers

| Modifier        | Example         | Effect                              |
|-----------------|-----------------|-------------------------------------|
| `duration-*`    | `duration-300`  | Transition duration in ms.          |
| `ease-*`        | `ease-in-out`   | Easing function (`linear`, `ease-in`, `ease-out`, `ease-in-out`, `ease-bounce`). |
| `delay-*`       | `delay-100`     | Stored but not applied (gpui has no native delay). |

```rust
view! {
    <button class="bg-indigo-500 hover:bg-blue-600 rounded p-3 transition-colors duration-300 ease-in-out">
        {"Color on hover"}
    </button>
}
```

## How It Works

1. **Compile time** — the `tw!` proc-macro parses `animate-*`, `transition-*`,
   `duration-*`, `ease-*`, and `delay-*` classes and emits `TwAnimation` /
   `TwTransition` structs inside `TwStyle`.
2. **`view!` macro** — when a `class` attribute is present, the macro destructures
   `TwStyle` and calls `apply_animation` / `apply_transition` after children are
   attached.
3. **`apply_transition`** — creates a hover signal, registers `on_hover`, and uses
   gpui's `with_animation` to interpolate between the base and hover
   `StyleRefinement` snapshots.

### Limitations

- **`animate-spin`** is a no-op — gpui does not support CSS transforms/rotation
  on `div` elements.
- **Animation + transition on the same element** — when both are present,
  animation takes priority. gpui's `AnimationElement` does not implement `Styled`,
  so a transition (whose animator needs `Styled`) cannot wrap it.
- **`delay-*`** is parsed and stored for API completeness but has no effect;
  gpui's animation API has no native delay support.
