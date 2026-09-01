#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;

#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

fn app() -> impl gpui::IntoElement {
    view! {
        <div class="flex flex-col items-center justify-center bg-[#1e1e2e] w-full h-full">
            <canvas
                class="w-[400px] h-[300px] bg-[#2d2d44]"
                paint={move |ctx: &mut Context2D| {
                    // Red filled rectangle
                    ctx.set_fill_style(color("#ff0000"));
                    ctx.fill_rect(10.0, 10.0, 80.0, 50.0);

                    // Blue stroked rectangle
                    ctx.set_stroke_style(color("#0000ff"));
                    ctx.set_line_width(3.0);
                    ctx.stroke_rect(110.0, 10.0, 80.0, 50.0);

                    // Green filled circle (arc path)
                    ctx.set_fill_style(color("#00cc44"));
                    ctx.begin_path();
                    ctx.arc(60.0, 130.0, 30.0, 0.0, std::f32::consts::TAU, false);
                    ctx.fill();

                    // Yellow stroked triangle (line path)
                    ctx.set_stroke_style(color("#ffdd00"));
                    ctx.set_line_width(2.0);
                    ctx.begin_path();
                    ctx.move_to(150.0, 100.0);
                    ctx.line_to(120.0, 160.0);
                    ctx.line_to(180.0, 160.0);
                    ctx.close_path();
                    ctx.stroke();

                    // "Hello Canvas" text in white
                    ctx.set_fill_style(color("#ffffff"));
                    ctx.set_font("16px sans-serif");
                    ctx.fill_text("Hello Canvas", 10.0, 200.0);

                    // Rotated purple square (save/restore/rotate)
                    ctx.save();
                    ctx.translate(300.0, 100.0);
                    ctx.rotate(std::f32::consts::FRAC_PI_4);
                    ctx.set_fill_style(color("#9933ff"));
                    ctx.fill_rect(-25.0, -25.0, 50.0, 50.0);
                    ctx.restore();

                    // Text alignment demo
                    ctx.set_font("12px sans-serif");
                    ctx.set_fill_style(color("#aaaaaa"));
                    ctx.set_text_align(CanvasTextAlign::Center);
                    ctx.fill_text("centered text", 200.0, 260.0);
                }}
            />
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();

    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(440.), px(360.0)), cx);
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
