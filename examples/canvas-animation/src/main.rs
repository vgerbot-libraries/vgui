#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

// Canvas dimensions (logical pixels).
const W: f32 = 480.0;
const H: f32 = 360.0;

const PALETTE: [&str; 7] = [
    "#ef4444", "#3b82f6", "#10b981", "#f59e0b",
    "#8b5cf6", "#ec4899", "#06b6d4",
];

#[derive(Clone, PartialEq)]
struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    radius: f32,
    color_idx: usize,
}

fn init_particles(n: usize) -> Vec<Particle> {
    (0..n)
        .map(|i| {
            let angle = (i as f32) * 2.399;
            let speed = 1.2 + (i as f32 % 3.0) * 0.6;
            Particle {
                x: W * 0.5 + ((i as f32 * 41.0) % (W * 0.5)),
                y: H * 0.5 + ((i as f32 * 67.0) % (H * 0.5)),
                vx: angle.cos() * speed,
                vy: angle.sin() * speed,
                radius: 5.0 + (i as f32 % 3.0) * 2.0,
                color_idx: i % PALETTE.len(),
            }
        })
        .collect()
}

fn app() -> impl gpui::IntoElement {
    let (particles, set_particles) = create_signal(init_particles(12));
    let (running, set_running) = create_signal(true);
    let (speed, set_speed) = create_signal(2u64);
    let (count, set_count) = create_signal(12u32);

    // Derive the effective interval delay from `running` and `speed`.
    // 0 pauses the interval; otherwise ~50ms / speed gives 20–100 fps.
    let delay = create_memo({
        let running = running.clone();
        let speed = speed.clone();
        move || if running.get() { 50u64 / speed.get().max(1) } else { 0 }
    });

    // Clone before moving into use_interval — needed by Reset and
    // particle-count buttons later.
    let set_particles_reset = set_particles.clone();

    // Animation loop: update particle positions on every tick.
    use_interval(
        move |cx| {
            set_particles.update(cx, |ps| {
                for p in ps.iter_mut() {
                    p.x += p.vx;
                    p.y += p.vy;
                    if p.x < p.radius {
                        p.x = p.radius;
                        p.vx = p.vx.abs();
                    }
                    if p.x > W - p.radius {
                        p.x = W - p.radius;
                        p.vx = -p.vx.abs();
                    }
                    if p.y < p.radius {
                        p.y = p.radius;
                        p.vy = p.vy.abs();
                    }
                    if p.y > H - p.radius {
                        p.y = H - p.radius;
                        p.vy = -p.vy.abs();
                    }
                }
            });
        },
        delay,
    );

    // Read signal values during render so the canvas paint closure
    // captures the current snapshot.  The signal reads are tracked,
    // so any change triggers a re-render and a fresh canvas paint.
    let snapshot = particles.get();
    let is_running = running.get();
    let cur_count = count.get();

    // Clones for closures that outlive this render call.
    let set_running_play = set_running.clone();
    let set_running_pause = set_running.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#0f0f1e] w-[540px] h-[600px] text-white rounded">
            <h2 class="text-lg font-bold">{"Particle Animation"}</h2>
            <span class="text-sm text-[#888] -mt-2">
                {"use_interval + canvas — reactive timer-driven rendering"}
            </span>

            // ── Canvas ──────────────────────────────────────────────
            <canvas
                class="w-[480px] h-[360px] bg-[#16162a] rounded"
                paint={move |ctx: &mut Context2D| {
                    // Constellation lines: connect nearby particles.
                    for i in 0..snapshot.len() {
                        for j in (i + 1)..snapshot.len() {
                            let dx = snapshot[i].x - snapshot[j].x;
                            let dy = snapshot[i].y - snapshot[j].y;
                            let dist = (dx * dx + dy * dy).sqrt();
                            if dist < 110.0 {
                                let alpha = (1.0 - dist / 110.0) * 0.4;
                                ctx.set_global_alpha(alpha);
                                ctx.set_stroke_style(color("#5588cc"));
                                ctx.set_line_width(1.0);
                                ctx.begin_path();
                                ctx.move_to(snapshot[i].x, snapshot[i].y);
                                ctx.line_to(snapshot[j].x, snapshot[j].y);
                                ctx.stroke();
                            }
                        }
                    }
                    ctx.set_global_alpha(1.0);

                    // Particles.
                    for p in &snapshot {
                        ctx.set_fill_style(color(PALETTE[p.color_idx]));
                        ctx.begin_path();
                        ctx.arc(p.x, p.y, p.radius, 0.0, std::f32::consts::TAU, false);
                        ctx.fill();
                    }
                }}
            />

            // ── Play / Pause / Reset ────────────────────────────────
            <div class="flex gap-2">
                <button
                    class={twc!(
                        "px-4 py-2 rounded font-medium",
                        is_running.then_some("bg-[#3b82f6] hover:bg-[#2563eb]"),
                        (!is_running).then_some("bg-[#444] hover:bg-[#555]")
                    )}
                    on:click={click(move |cx| set_running_play.set(cx, true))}
                >
                    {"Play"}
                </button>
                <button
                    class={twc!(
                        "px-4 py-2 rounded font-medium",
                        (!is_running).then_some("bg-[#ef4444] hover:bg-[#dc2626]"),
                        is_running.then_some("bg-[#444] hover:bg-[#555]")
                    )}
                    on:click={click(move |cx| set_running_pause.set(cx, false))}
                >
                    {"Pause"}
                </button>
                <button
                    class="px-4 py-2 rounded font-medium bg-[#6366f1] hover:bg-[#4f46e5]"
                    on:click={click({
                        let sp = set_particles_reset.clone();
                        move |cx| sp.set(cx, init_particles(cur_count as usize))
                    })}
                >
                    {"Reset"}
                </button>
            </div>

            // ── Speed selector ──────────────────────────────────────
            <div class="flex gap-2 items-center">
                <span class="text-sm text-[#888] w-16">{"Speed"}</span>
                {vgui::for_each([1u64, 2, 3, 5], move |s, _| {
                    let set_speed = set_speed.clone();
                    let speed = speed.clone();
                    view! {
                        <button
                            class={twc!(
                                "px-3 py-1.5 rounded text-sm",
                                (speed.get() == s).then_some("bg-[#10b981] text-white"),
                                (speed.get() != s).then_some("bg-[#333] hover:bg-[#444] text-[#ccc]")
                            )}
                            on:click={click(move |cx| set_speed.set(cx, s))}
                        >
                            {format!("{}x", s)}
                        </button>
                    }
                })}
            </div>

            // ── Particle count selector ─────────────────────────────
            <div class="flex gap-2 items-center">
                <span class="text-sm text-[#888] w-16">{"Particles"}</span>
                {vgui::for_each([6u32, 12, 20, 30], move |n, _| {
                    let set_count = set_count.clone();
                    let count = count.clone();
                    let set_particles = set_particles_reset.clone();
                    view! {
                        <button
                            class={twc!(
                                "px-3 py-1.5 rounded text-sm",
                                (count.get() == n).then_some("bg-[#10b981] text-white"),
                                (count.get() != n).then_some("bg-[#333] hover:bg-[#444] text-[#ccc]")
                            )}
                            on:click={click(move |cx| {
                                set_count.set(cx, n);
                                set_particles.set(cx, init_particles(n as usize));
                            })}
                        >
                            {format!("{}", n)}
                        </button>
                    }
                })}
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
        let bounds = Bounds::centered(None, size(px(540.), px(600.0)), cx);
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
