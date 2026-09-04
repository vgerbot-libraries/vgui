#![cfg_attr(target_family = "wasm", no_main)]

use std::cell::RefCell;
use std::time::Duration;

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    // ── use_interval demo ───────────────────────────────────────────
    // A counter that ticks every `delay` milliseconds. The delay is
    // reactively controlled — changing it re-runs the effect inside
    // `use_interval`, cancelling the old timer and starting a new one.
    // Setting delay to 0 pauses the interval entirely.
    let (count, set_count) = create_signal(0i32);
    let (delay, set_delay) = create_signal(1000u64);
    let (running, set_running) = create_signal(true);

    // When `running` is false, we feed 0 into the delay signal so the
    // interval pauses. When true, we use the user-selected delay.
    let effective_delay = create_memo({
        let delay = delay.clone();
        let running = running.clone();
        move || if running.get() { delay.get() } else { 0 }
    });

    let set_count_reset = set_count.clone();
    use_interval(
        move |cx| set_count.update(cx, |n| *n += 1),
        effective_delay,
    );
    let set_running_start = set_running.clone();
    let set_running_pause = set_running.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] w-[460px] h-[420px] text-white justify-center items-center">
            <h2 class="text-lg font-bold">{"useInterval Demo"}</h2>

            // ── Timer display ────────────────────────────────────────
            <div class="text-4xl font-mono tabular-nums">
                {format!("{}s", count.get() / 10)}
            </div>
            <div class="text-sm text-[#aaa]">
                {format!("ticks: {}", count.get())}
            </div>

            // ── Start / Pause ────────────────────────────────────────
            <div class="flex gap-2">
                <button
                    class={twc!(
                        "px-4 py-2 rounded font-medium",
                        running.get().then_some("bg-[#3b82f6] hover:bg-[#2563eb]"),
                        (!running.get()).then_some("bg-[#666] hover:bg-[#555]")
                    )}
                    on:click={click(move |cx| set_running_start.set(cx, true))}
                >
                    {"Start"}
                </button>
                <button
                    class={twc!(
                        "px-4 py-2 rounded font-medium",
                        (!running.get()).then_some("bg-[#ef4444] hover:bg-[#dc2626]"),
                        running.get().then_some("bg-[#666] hover:bg-[#555]")
                    )}
                    on:click={click(move |cx| set_running_pause.set(cx, false))}
                >
                    {"Pause"}
                </button>
                <button
                    class="px-4 py-2 rounded font-medium bg-[#6366f1] hover:bg-[#4f46e5]"
                    on:click={click(move |cx| {
                        set_count_reset.set(cx, 0);
                    })}
                >
                    {"Reset"}
                </button>
            </div>

            // ── Delay selector ───────────────────────────────────────
            <div class="flex flex-col gap-2 items-center">
                <span class="text-sm text-[#aaa]">{format!("Delay: {} ms", delay.get())}</span>
                <div class="flex gap-2">
                    {vgui::for_each([100u64, 500, 1000, 2000], move |d, _| {
                        let set_delay = set_delay.clone();
                        let delay = delay.clone();
                        view! {
                            <button
                                class={twc!(
                                    "px-3 py-1.5 rounded text-sm",
                                    (delay.get() == d).then_some("bg-[#10b981] text-white"),
                                    (delay.get() != d).then_some("bg-[#333] hover:bg-[#444] text-[#ccc]")
                                )}
                                on:click={click(move |cx| set_delay.set(cx, d))}
                            >
                                {format!("{}ms", d)}
                            </button>
                        }
                    })}
                </div>
            </div>

            // ── set_interval demo (independent) ──────────────────────
            <Blinker />
        </div>
    }
}

/// A self-contained component that uses the low-level `set_interval`
/// directly (without `use_interval`). The interval is created once on
/// first render and cancelled when the scope is disposed via
/// `on_cleanup`.
#[allow(non_snake_case)]
fn Blinker() -> impl gpui::IntoElement {
    let (on, set_on) = create_signal(false);

    // `set_interval` returns an `IntervalHandle` that owns the gpui task.
    // Dropping the handle cancels the interval (like JS `clearInterval`).
    // Here we keep the handle alive by moving it into `on_cleanup`, which
    // runs — and drops it — when this scope is disposed.
    let handle = set_interval(
        move |cx| set_on.update(cx, |v| *v = !*v),
        Duration::from_millis(600),
    );
    let handle = RefCell::new(Some(handle));
    on_cleanup(move || {
        *handle.borrow_mut() = None;
    });

    view! {
        <div class="flex items-center gap-2 mt-2">
            <span class="text-sm text-[#aaa]">{"set_interval blink:"}</span>
            <div class={twc!(
                "w-4 h-4 rounded-full transition-colors",
                on.get().then_some("bg-[#f59e0b]"),
                (!on.get()).then_some("bg-[#333]")
            )} />
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(460.), px(420.0)), cx);
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
