# Router

vgui includes a minimal single-page-app (SPA) router built on the reactive
system. The current path is stored in a `Signal<String>`, and route patterns
like `/users/:id` are matched against it to extract parameters.

## Creating a router

`create_router(initial)` creates a `Router` backed by a signal. The `Router`
is `Clone` (inner data is reference-counted), so it can be freely cloned into
closures.

```rust
use vgui::prelude::*;
use vgui::router::create_router;

fn app() -> impl gpui::IntoElement {
    let router = create_router("/");
    // ...
}
```

## Navigation

`router.navigate(cx, path)` updates the underlying signal, which triggers a
re-render of any scope that read `router.path()`. This is the only way to
change the current route.

```rust
router.navigate(cx, "/users/42");
```

## Reading the path

| Method | Description |
|--------|-------------|
| `router.path()` | Reactive read — tracks as a dependency in memos/effects. |
| `router.path_with(cx)` | Non-tracking read via the gpui context. |
| `router.path_signal()` | Returns the underlying `ReadSignal<String>`. |

## Pattern matching

`match_pattern(pattern, path)` matches a route pattern against a path string
and returns `Option<RouteMatch>`:

- `:param` segments capture a single path segment into `params`.
- A trailing `*` wildcard captures the rest of the path.
- Trailing slashes are normalized (treated as no trailing slash).

```rust
use vgui::router::match_pattern;

let m = match_pattern("/users/:id", "/users/42").unwrap();
assert_eq!(m.params.get("id"), Some(&"42".to_string()));

let m = match_pattern("/files/*", "/files/a/b/c").unwrap();
assert_eq!(m.params.get("*"), Some(&"a/b/c".to_string()));
```

`RouteMatch` has three fields:

| Field | Type | Description |
|-------|------|-------------|
| `pattern` | `String` | The pattern that matched (e.g. `/users/:id`). |
| `path` | `String` | The actual path that was matched (e.g. `/users/42`). |
| `params` | `HashMap<String, String>` | Extracted parameters. |

### Building paths

`build_path(pattern, params)` substitutes `:param` placeholders with values
from the params map. Missing params are left as-is.

```rust
use vgui::router::build_path;
use std::collections::HashMap;

let mut params = HashMap::new();
params.insert("id".to_string(), "42".to_string());
assert_eq!(build_path("/users/:id", &params), "/users/42");
```

## Route dispatch

`router.render(cx, routes, fallback)` iterates a slice of
`(&str, handler)` pairs and renders the first matching route's element. Each
handler receives `&HashMap<String, String>` of extracted params. If no route
matches, the fallback closure is called.

```rust
use vgui::prelude::*;
use vgui::router::create_router;
use vgui::view;

fn app() -> impl gpui::IntoElement {
    let router = create_router("/");

    view! {
        <div>
            {router.render(cx, &[
                ("/", |_| view! { <div>{"Home"}</div> }.into_any_element()),
                ("/users/:id", |params| {
                    let id = params.get("id").unwrap_or("?");
                    view! { <div>{"User "}{id}</div> }.into_any_element()
                }),
                ("/*", |_| view! { <div>{"Not found"}</div> }.into_any_element()),
            ])}
        </div>
    }
}
```

`router.match_route(pattern)` is a convenience that matches a single pattern
against the current path and returns `Option<RouteMatch>`.

## Signal-driven re-render

Navigation is signal-driven: calling `navigate` updates the path signal, which
triggers a re-render of any scope that read `router.path()` or
`router.path_signal()`. No manual re-render call is needed — the reactive
system handles propagation automatically.
