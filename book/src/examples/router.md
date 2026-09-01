# Router Example

## Live Demo

<iframe src="../wasm/router/" width="100%" height="500" style="border:1px solid #444; border-radius:4px;"></iframe>

## Overview

The router example demonstrates the `vgui` SPA router with parameterized route
matching, programmatic navigation, and a 404 fallback:

- `create_router("/")` — creates a router with an initial path.
- `router.navigate(cx, path)` — updates the current path inside an event handler
  (where `cx: &mut App` is available).
- `router.match_route(pattern)` — reactively reads `router.path()` and returns
  `Option<RouteMatch>`, matching static segments and named params like
  `/users/:id`.
- `RouteMatch::params` — a `HashMap<String, String>` of extracted path
  parameters, used here to pull the `id` from `/users/:id`.
- A cascade of `if let Some(m)` blocks for route dispatch, falling through to a
  404 view when no pattern matches.
- `for_each` for rendering the user list, with each item navigating to its
  detail route on click.

## Source Code

```rust
#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::for_each;
use vgui::prelude::*;
use vgui::router::RouteMatch;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn route_content(router: &Router) -> gpui::AnyElement {
    if router.match_route("/").is_some() {
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Home"}</h2>
                <p class="text-sm text-[#aaa]">{"Welcome to the router example."}</p>
            </div>
        }.into_any_element();
    }
    if router.match_route("/users").is_some() {
        let users = vec![
            ("1", "Alice"),
            ("2", "Bob"),
            ("3", "Charlie"),
        ];
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Users"}</h2>
                <ul class="flex flex-col gap-1 mt-2">
                    {for_each(users, |(id, name), _| {
                        let router = router.clone();
                        view! {
                            <li
                                class="p-2 cursor-pointer hover:bg-[#333] rounded text-sm"
                                on:click={click(move |cx| router.navigate(cx, &format!("/users/{id}")))}
                            >
                                {name.to_string()}
                            </li>
                        }
                    })}
                </ul>
            </div>
        }.into_any_element();
    }
    if let Some(m) = router.match_route("/users/:id") {
        let id = m.params.get("id").cloned().unwrap_or_else(|| "?".to_string());
        let router = router.clone();
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{format!("User #{}", id)}</h2>
                <p class="text-sm text-[#aaa]">{format!("Details for user {}", id)}</p>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm mt-2"
                    on:click={click(move |cx| router.navigate(cx, "/users"))}
                >
                    {"Back to list"}
                </button>
            </div>
        }.into_any_element();
    }
    if router.match_route("/settings").is_some() {
        return view! {
            <div class="p-4">
                <h2 class="text-lg font-bold">{"Settings"}</h2>
                <p class="text-sm text-[#aaa]">{"Settings page content."}</p>
            </div>
        }.into_any_element();
    }
    // Fallback 404
    view! {
        <div class="p-4">
            <h2 class="text-lg font-bold">{"404"}</h2>
            <p class="text-sm text-[#aaa]">{"Page not found."}</p>
        </div>
    }.into_any_element()
}

fn app() -> impl gpui::IntoElement {
    let router = create_router("/");
    let r_home = router.clone();
    let r_users = router.clone();
    let r_settings = router.clone();
    let r_path = router.clone();

    view! {
        <div class="flex flex-col bg-[#1a1a2e] text-white h-full">
            // Nav bar
            <div class="flex flex-row gap-2 p-3 bg-[#2d2d44]">
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_home.navigate(cx, "/"))}
                >
                    {"Home"}
                </button>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_users.navigate(cx, "/users"))}
                >
                    {"Users"}
                </button>
                <button
                    class="px-3 py-1 bg-[#333] hover:bg-[#444] rounded text-white text-sm"
                    on:click={click(move |cx| r_settings.navigate(cx, "/settings"))}
                >
                    {"Settings"}
                </button>
            </div>

            // Current path display
            <div class="text-sm text-gray-400 p-2">
                {format!("Current path: {}", r_path.path())}
            </div>

            // Route content
            <div class="flex-1">
                {route_content(&router)}
            </div>
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(500.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    };

    #[cfg(not(target_family = "wasm"))]
    gpui_app.run(launch);

    #[cfg(target_family = "wasm")]
    std::mem::forget(gpui_app.run_embedded(launch));
}

#[cfg(not(target_family = "wasm"))]
fn main() {
    run();
}

#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() {
    gpui_platform::web_init();
    vgui::intercept_keyboard_events();
    run();
}
```

## Key Concepts

### `create_router()` with initial path

`create_router("/")` constructs a `Router` backed by a signal holding the
current path. The initial path is `"/"`, so the home view renders on first
paint. The router is `Clone` (it is a handle around shared state), so it can be
cheaply cloned into each event handler closure that needs to navigate.

### `match_route()` — reactive read via `path()`

`router.match_route(pattern)` calls `router.path()` internally, which performs
a reactive signal read. Because the read happens inside the `view!` render
scope, any later `navigate()` that changes the path triggers a re-render and
the route content updates automatically — no manual subscription needed.
`match_route` returns `Option<RouteMatch>`: `Some` when the current path
matches the pattern (including named params), `None` otherwise.

### `RouteMatch::params` — `HashMap<String, String>`

For the pattern `/users/:id`, a match against `/users/2` produces a
`RouteMatch` whose `params` map contains `{"id": "2"}`. The detail view
extracts it with `m.params.get("id").cloned().unwrap_or_else(|| "?".into())`
and renders `User #2`. Params are always strings; convert to other types at
the call site if needed.

### `navigate()` in event handlers

`router.navigate(cx, path)` writes the new path into the router's signal. It
requires `cx: &mut App`, which is available inside `click(move |cx| ...)`
closures but **not** inside `app()` itself — this is why the route dispatch
uses `match_route()` (reactive read, no `cx`) rather than `router.render(cx,
...)`. The nav-bar buttons and list-item clicks all call `navigate` to move
between routes.

### Cascade `if let Some` pattern for route dispatch

`route_content(&router) -> gpui::AnyElement` checks each route pattern in turn
with `if router.match_route(...).is_some()` / `if let Some(m) = ...`, returning
the matching view as soon as one hits. If no pattern matches, execution falls
through to the final `view! { ... 404 ... }` expression. Each branch calls
`.into_any_element()` to erase the specific view type into `gpui::AnyElement`
so all branches share a single return type.

### `for_each` for list items

The users list uses `for_each(users, |(id, name), _| view! { <li ...> })` to
render one `<li>` per user. Each item clones the router and wires an
`on:click` handler that navigates to `/users/{id}`, demonstrating how
navigation can originate from dynamically generated list elements. `for_each`
is imported explicitly via `use vgui::for_each;`.

### Running

**Native:**

    cargo run -p vgui-router

**Web (WASM):**

    cargo build --target wasm32-unknown-unknown -p vgui-router --release
    wasm-bindgen --target web --out-dir examples/router/dist \
        --no-typescript target/wasm32-unknown-unknown/release/router.wasm
    python3 scripts/serve_plain.py 8080 examples/router
