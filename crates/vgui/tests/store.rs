//! Integration tests for `create_store` / `Store` / `SetStore`.
//!
//! Drives a real gpui window through multiple store updates and verifies
//! that `Store::select` provides fine-grained reactivity: a selector only
//! changes value when its slice actually changes, even though every store
//! write triggers a re-render.

#![cfg(not(target_family = "wasm"))]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use parking_lot::Mutex;
use vgui::prelude::*;

use gpui_platform::application;

#[derive(Clone, PartialEq, Default)]
struct AppState {
    count: i32,
    name: String,
    flag: bool,
}

struct Harness {
    set_store: Option<SetStore<AppState>>,
    last_count: i32,
    last_name: String,
}


#[test]
fn store_select_filters_unchanged_slices() {
    // This test can't use the helper above easily because we need
    // interleaved snapshots between steps. Inline the logic instead.
    let gpui_app = application();

    let harness = Arc::new(Mutex::new(Harness {
        set_store: None,
        last_count: 0,
        last_name: String::new(),
    }));
    let armed = Arc::new(AtomicBool::new(false));
    let results: Arc<Mutex<Vec<(i32, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let h_render = harness.clone();
    let h_spawn = harness.clone();
    let armed_r = armed.clone();
    let armed_s = armed.clone();
    let res = results.clone();

    let launch = move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(200.), px(100.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                vgui::mount(cx, move || {
                    let (store, set_store) = create_store(AppState::default());
                    let count_sig = store.select(|s| s.count);
                    let name_sig = store.select(|s| s.name.clone());

                    {
                        let mut h = h_render.lock();
                        if h.set_store.is_none() {
                            h.set_store = Some(set_store.clone());
                        }
                    }

                    if armed_r.load(Ordering::SeqCst) {
                        let mut h = h_render.lock();
                        h.last_count = count_sig.get();
                        h.last_name = name_sig.get();
                    }

                    view! { <div>{format!("{} {}", count_sig.get(), name_sig.get())}</div> }
                })
            },
        )
        .unwrap();

        cx.spawn(async move |cx| {
            // Wait for window-setup renders to settle.
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            armed_s.store(true, Ordering::SeqCst);

            // Step 1: update count only.
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.update(cx, |s| s.count = 42);
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            // Step 2: update name only — count_sig must stay 42.
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.update(cx, |s| s.name = "hello".to_string());
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            // Step 3: update both.
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.update(cx, |s| {
                        s.count = 99;
                        s.name = "world".to_string();
                    });
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            // Step 4: update unrelated field (flag) — neither sig changes.
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.update(cx, |s| s.flag = true);
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            cx.update(|cx| cx.quit());
        })
        .detach();
    };

    gpui_app.run(launch);

    let r = results.lock();

    // 4 snapshots, one per step.
    assert_eq!(r.len(), 4, "expected 4 snapshots, got {r:?}");

    // Step 1: count=42, name="" (name unchanged).
    assert_eq!(r[0].0, 42, "count should be 42 after step 1");
    assert_eq!(r[0].1, "", "name should still be empty after step 1");

    // Step 2: count=42 (unchanged!), name="hello".
    assert_eq!(
        r[1].0, 42,
        "count should still be 42 after name-only update (fine-grained)"
    );
    assert_eq!(r[1].1, "hello", "name should be 'hello' after step 2");

    // Step 3: count=99, name="world".
    assert_eq!(r[2].0, 99, "count should be 99 after step 3");
    assert_eq!(r[2].1, "world", "name should be 'world' after step 3");

    // Step 4: count=99 (unchanged!), name="world" (unchanged!).
    assert_eq!(
        r[3].0, 99,
        "count should still be 99 after flag-only update"
    );
    assert_eq!(
        r[3].1, "world",
        "name should still be 'world' after flag-only update"
    );
}

#[test]
fn store_get_and_with_read_state() {
    let gpui_app = application();

    let harness = Arc::new(Mutex::new(Harness {
        set_store: None,
        last_count: 0,
        last_name: String::new(),
    }));
    let armed = Arc::new(AtomicBool::new(false));
    let results: Arc<Mutex<Vec<(i32, String)>>> = Arc::new(Mutex::new(Vec::new()));

    let h_render = harness.clone();
    let h_spawn = harness.clone();
    let armed_r = armed.clone();
    let armed_s = armed.clone();
    let res = results.clone();

    let launch = move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(200.), px(100.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                vgui::mount(cx, move || {
                    let (store, set_store) = create_store(AppState {
                        count: 7,
                        name: "init".to_string(),
                        flag: false,
                    });

                    {
                        let mut h = h_render.lock();
                        if h.set_store.is_none() {
                            h.set_store = Some(set_store.clone());
                        }
                    }

                    // Exercise both get() and with().
                    let via_get = store.get().count;
                    let via_with = store.with(|s| s.name.clone());

                    if armed_r.load(Ordering::SeqCst) {
                        let mut h = h_render.lock();
                        h.last_count = via_get;
                        h.last_name = via_with.clone();
                    }

                    view! { <div>{format!("{} {}", via_get, via_with)}</div> }
                })
            },
        )
        .unwrap();

        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(Duration::from_millis(100))
                .await;
            armed_s.store(true, Ordering::SeqCst);

            // Trigger a re-render so the armed render closure records the
            // current state. A no-op update still notifies (store semantics).
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.update(cx, |_s| {});
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;

            // Snapshot initial state (armed, before the real update).
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            // Replace entire state via set().
            cx.update(|cx| {
                if let Some(ss) = h_spawn.lock().set_store.as_ref() {
                    ss.set(
                        cx,
                        AppState {
                            count: 100,
                            name: "replaced".to_string(),
                            flag: true,
                        },
                    );
                }
            });
            cx.background_executor()
                .timer(Duration::from_millis(80))
                .await;
            {
                let h = h_spawn.lock();
                res.lock().push((h.last_count, h.last_name.clone()));
            }

            cx.update(|cx| cx.quit());
        })
        .detach();
    };

    gpui_app.run(launch);

    let r = results.lock();
    assert_eq!(r.len(), 2, "expected 2 snapshots, got {r:?}");
    // Initial state via get()/with().
    assert_eq!(r[0].0, 7, "initial count via get() should be 7");
    assert_eq!(r[0].1, "init", "initial name via with() should be 'init'");
    // After set().
    assert_eq!(r[1].0, 100, "count should be 100 after set()");
    assert_eq!(r[1].1, "replaced", "name should be 'replaced' after set()");
}
