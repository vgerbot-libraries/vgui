use gpui::{px, size, App, Application, Bounds, WindowBounds, WindowOptions};
use vgui::prelude::*;

fn app() -> impl gpui::IntoElement {
    let (text, set_text) = create_signal(String::new());
    let (password, set_password) = create_signal(String::new());
    let (checked, set_checked) = create_signal(false);
    let (radio_val, set_radio) = create_signal(0i32);
    let (slider, set_slider) = create_signal(50.0f64);
    let (number_val, set_number) = create_signal(String::new());
    let (date_val, set_date) = create_signal(String::new());
    let sr0 = set_radio.clone();
    let sr1 = set_radio.clone();
    let sr2 = set_radio.clone();

    view! {
        <div class="flex flex-col gap-4 p-6 bg-[#1a1a2e] w-[600px] h-[700px] text-white overflow-y-auto">
            <span class="text-lg font-bold">{"vgui <input> demo"}</span>

            // Text input with live mirror
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Text (on:input)"}</span>
                <input
                    type="text"
                    placeholder="Type here..."
                    on:input={move |v: &str, cx: &mut App| set_text.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("echo: \"{}\"", text.get())}</span>
            </div>

            // Password
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Password"}</span>
                <input
                    type="password"
                    placeholder="secret"
                    on:input={move |v: &str, cx: &mut App| set_password.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("len: {} chars", password.get().chars().count())}</span>
            </div>

            // Checkbox
            <div class="flex flex-row gap-2 items-center">
                <input
                    type="checkbox"
                    checked={checked.get()}
                    on:change={move |v: bool, cx: &mut App| set_checked.set(cx, v)}
                    tabindex={-1}
                />
                <span class="text-sm">{format!("checkbox: {}", if checked.get() { "on" } else { "off" })}</span>
            </div>

            // Radio group
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Radio group"}</span>
                <div class="flex flex-row gap-4 items-center">
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 0} on:change={move |_v: bool, cx: &mut App| sr0.set(cx, 0)} />
                        <span class="text-sm">{"A"}</span>
                    </div>
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 1} on:change={move |_v: bool, cx: &mut App| sr1.set(cx, 1)} />
                        <span class="text-sm">{"B"}</span>
                    </div>
                    <div class="flex flex-row gap-1 items-center">
                        <input type="radio" checked={radio_val.get() == 2} on:change={move |_v: bool, cx: &mut App| sr2.set(cx, 2)} />
                        <span class="text-sm">{"C"}</span>
                    </div>
                </div>
                <span class="text-sm text-[#0f0]">{format!("selected: {}", radio_val.get())}</span>
            </div>

            // Range slider
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Range slider"}</span>
                <input
                    type="range"
                    min={0.0f64}
                    max={100.0f64}
                    step={1.0f64}
                    value={slider.get()}
                    on:change={move |v: f64, cx: &mut App| set_slider.set(cx, v)}
                    tabindex={1}
                />
                <span class="text-sm text-[#0f0]">{format!("value: {:.1}", slider.get())}</span>
            </div>

            // Number input
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Number (min 0, max 100)"}</span>
                <input
                    type="number"
                    min={0.0f64}
                    max={100.0f64}
                    placeholder="42"
                    on:input={move |v: &str, cx: &mut App| set_number.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("number: \"{}\"", number_val.get())}</span>
            </div>

            // Date input (text-entry v1)
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"Date (YYYY-MM-DD)"}</span>
                <input
                    type="date"
                    placeholder="2026-01-15"
                    on:input={move |v: &str, cx: &mut App| set_date.set(cx, v.to_string())}
                    tabindex={0}
                />
                <span class="text-sm text-[#0f0]">{format!("date: \"{}\"", date_val.get())}</span>
            </div>

            // File input
            <div class="flex flex-col gap-1">
                <span class="text-sm text-[#888]">{"File picker"}</span>
                <input
                    type="file"
                    value="Choose file..."
                    on:change={move |paths: Vec<std::path::PathBuf>, _cx: &mut App| {
                        if let Some(p) = paths.first() {
                            eprintln!("file selected: {:?}", p);
                        }
                    }}
                />
            </div>

            // Submit button
            <input type="submit" value="Submit" on:click={click(move |_cx| eprintln!("submit clicked"))} />

            // Hidden input (renders nothing)
            <input type="hidden" value="invisible" />
        </div>
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(600.), px(700.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| vgui::mount(cx, app),
        )
        .unwrap();
    });
}
