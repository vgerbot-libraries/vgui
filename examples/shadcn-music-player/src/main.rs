#![cfg_attr(target_family = "wasm", no_main)]

use gpui::{px, size, App, Bounds, WindowBounds, WindowOptions};
use vgui::for_each;
use vgui::prelude::*;
use vgui_shadcn::prelude::*;

#[cfg(not(target_family = "wasm"))]
use gpui_platform::application;
#[cfg(target_family = "wasm")]
use gpui_platform::single_threaded_web;

#[derive(Clone, PartialEq)]
struct Track {
    title: String,
    artist: String,
    album: String,
    duration: f64,
}

fn catalog() -> Vec<Track> {
    vec![
        Track { title: "Night Drive".into(), artist: "Neon Harbor".into(), album: "After Hours".into(), duration: 214.0 },
        Track { title: "Glasshouse".into(), artist: "Lumen".into(), album: "Clear".into(), duration: 187.0 },
        Track { title: "Low Tide".into(), artist: "Marlow".into(), album: "Coast".into(), duration: 241.0 },
        Track { title: "Paper Planes".into(), artist: "Kite Club".into(), album: "Fold".into(), duration: 198.0 },
        Track { title: "Copper Sky".into(), artist: "Saffron".into(), album: "Dust".into(), duration: 226.0 },
        Track { title: "Static Bloom".into(), artist: "Violet Grid".into(), album: "Signal".into(), duration: 173.0 },
        Track { title: "Second Wind".into(), artist: "Northline".into(), album: "Relay".into(), duration: 205.0 },
        Track { title: "Quiet Markets".into(), artist: "Elm & Oak".into(), album: "Sunday".into(), duration: 232.0 },
    ]
}

fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0) as u32;
    format!("{}:{:02}", s / 60, s % 60)
}

fn initials(name: &str) -> String {
    name.split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

fn app() -> impl gpui::IntoElement {
    apply_theme(true);

    let tracks = catalog();
    let n = tracks.len();
    let (index, set_index) = create_signal(0usize);
    let (playing, set_playing) = create_signal(false);
    let (time, set_time) = create_signal(0.0f64);
    let (volume, set_volume) = create_signal(0.8f64);
    let (muted, set_muted) = create_signal(false);
    let (shuffle, set_shuffle) = create_signal(false);
    let (repeat, set_repeat) = create_signal(false);
    let (search, set_search) = create_signal(String::new());
    let (liked, set_liked) = create_signal(Vec::<usize>::new());
    let (vol_open, set_vol_open) = create_signal(false);
    let (drawer_open, set_drawer_open) = create_signal(false);
    let vol_ref = NodeRef::new();

    let delay = create_memo({
        let playing = playing.clone();
        move || if playing.get() { 250u64 } else { 0 }
    });

    use_interval(
        {
            let tracks = tracks.clone();
            let index = index.clone();
            let set_index = set_index.clone();
            let time = time.clone();
            let set_time = set_time.clone();
            let set_playing = set_playing.clone();
            let shuffle = shuffle.clone();
            let repeat = repeat.clone();
            move |cx| {
                let i = index.get();
                let dur = tracks.get(i).map(|t| t.duration).unwrap_or(1.0);
                set_time.update(cx, |t| *t += 0.25);
                let t = time.get();
                if t >= dur {
                    if repeat.get() {
                        set_time.set(cx, 0.0);
                    } else if shuffle.get() {
                        let next = (i + 3) % n;
                        set_index.set(cx, next);
                        set_time.set(cx, 0.0);
                    } else if i + 1 < n {
                        set_index.set(cx, i + 1);
                        set_time.set(cx, 0.0);
                    } else {
                        set_playing.set(cx, false);
                        set_time.set(cx, 0.0);
                    }
                }
            }
        },
        delay,
    );

    let i = index.get();
    let track = tracks.get(i).cloned().unwrap_or_else(|| Track {
        title: String::new(),
        artist: String::new(),
        album: String::new(),
        duration: 1.0,
    });
    let q = search.get().to_lowercase();
    let filtered: Vec<(usize, Track)> = tracks
        .iter()
        .enumerate()
        .filter(|(_, t)| {
            q.is_empty()
                || t.title.to_lowercase().contains(&q)
                || t.artist.to_lowercase().contains(&q)
        })
        .map(|(idx, t)| (idx, t.clone()))
        .collect();

    let liked_now = liked.get();
    let playing_now = playing.get();
    let muted_now = muted.get();
    let shuffle_now = shuffle.get();
    let repeat_now = repeat.get();
    let time_now = time.get();
    let volume_now = volume.get();

    let set_play = set_playing.clone();
    let set_idx_prev = set_index.clone();
    let set_time_prev = set_time.clone();
    let set_idx_next = set_index.clone();
    let set_time_next = set_time.clone();
    let set_shuffle_btn = set_shuffle.clone();
    let set_repeat_btn = set_repeat.clone();
    let set_mute = set_muted.clone();
    let set_vol_open_btn = set_vol_open.clone();
    let set_drawer = set_drawer_open.clone();
    let set_drawer_close = set_drawer_open.clone();
    let set_search_input = set_search.clone();
    let set_time_seek = set_time.clone();
    let set_volume_slider = set_volume.clone();

    let nav = |label: &'static str, icon: &'static str, active: bool| {
        sidebar_item(icon, label, active, |_| {})
    };

    view! {
        <div class="w-full h-full flex flex-col" style={css! {
            background: var(--background);
            color: var(--foreground);
        }}>
            <div class="flex flex-row flex-1 overflow-hidden">
                {Sidebar {
                    children: vec![
                        view! {
                            <div class="flex flex-row items-center gap-2 pt-2 pr-1 pb-4 pl-1">
                                <span class="text-base font-bold">{icons::MUSIC.to_string()}</span>
                                <span class="text-base font-bold">{"Vgui Music"}</span>
                            </div>
                        }.into_any_element(),
                        nav("Discover", icons::HOME, true).into_any_element(),
                        nav("Library", icons::LIBRARY, false).into_any_element(),
                        nav("Playlists", icons::LIST, false).into_any_element(),
                        Separator { vertical: false }.into_any_element(),
                        view! {
                            <span class="text-xs py-2 px-3" style={css! { color: var(--muted-foreground); }}>
                                {"PLAYLISTS"}
                            </span>
                        }.into_any_element(),
                        nav("Night Shift", icons::DISC, false).into_any_element(),
                        nav("Focus Mix", icons::DISC, false).into_any_element(),
                        nav("Liked Songs", icons::HEART, false).into_any_element(),
                    ],
                }}

                <div class="flex-1 flex flex-col pt-4 pr-6 pb-4 pl-6 gap-4 overflow-hidden">
                    <div class="flex flex-row items-center gap-3">
                        {icon_button(icons::MENU, move |cx| set_drawer.update(cx, |v| *v = !*v))}
                        {TextField {
                            value: search.get(),
                            placeholder: "Search tracks".into(),
                            on_input: Box::new(move |v, cx| set_search_input.set(cx, v)),
                            class: Some("flex-1".into()),
                        }}
                    </div>
                    <span class="text-xl font-bold">{"Listen Now"}</span>
                    <div class="flex-1 overflow-hidden flex flex-col gap-1">
                        {for_each(filtered, {
                            let index = index.clone();
                            let set_index = set_index.clone();
                            let set_playing = set_playing.clone();
                            let set_time = set_time.clone();
                            let liked = liked.clone();
                            let set_liked = set_liked.clone();
                            move |(idx, t): (usize, Track), _| {
                                let active = index.get() == idx;
                                let is_liked = liked.get().contains(&idx);
                                let set_index = set_index.clone();
                                let set_playing = set_playing.clone();
                                let set_time = set_time.clone();
                                let set_liked = set_liked.clone();
                                let bg = if active {
                                    css! { border-radius: var(--radius); background: var(--accent); }
                                } else {
                                    css! { border-radius: var(--radius); background: var(--background); }
                                };
                                view! {
                                    <div
                                        class="flex flex-row items-center gap-3 py-2 px-3 cursor-pointer"
                                        style={bg}
                                        hover={css! { background: var(--muted); }}
                                    >
                                        <div
                                            class="flex flex-row items-center gap-3 p-1 w-full"
                                            on:click={click(move |cx| {
                                                set_index.set(cx, idx);
                                                set_time.set(cx, 0.0);
                                                set_playing.set(cx, true);
                                            })}
                                        >
                                            {Avatar { initials: initials(&t.artist), size: 36.0 }}
                                            <div class="flex-1 flex flex-col">
                                                <span class="text-sm font-medium">{t.title.clone()}</span>
                                                <span class="text-xs" style={css! { color: var(--muted-foreground); }}>{format!("{} · {}", t.artist, t.album)}</span>
                                            </div>
                                            <span class="text-xs" style={css! { color: var(--muted-foreground); }}>{fmt_time(t.duration)}</span>
                                            {icon_button(if is_liked { icons::HEART } else { icons::HEART_OUTLINE }, move |cx| {
                                                set_liked.update(cx, |ids| {
                                                    if let Some(pos) = ids.iter().position(|x| *x == idx) {
                                                        ids.remove(pos);
                                                    } else {
                                                        ids.push(idx);
                                                    }
                                                });
                                            })}
                                        </div>
                                    </div>
                                }
                            }
                        })}
                    </div>
                </div>
            </div>

            <div class="flex flex-col border-t border-solid pt-2 px-4 pb-3 gap-2" style={css! {
                background: var(--card);
                border-color: var(--border);
            }}>
                {slider(time_now, 0.0, track.duration, move |v, cx| set_time_seek.set(cx, v))}
                <div class="flex flex-row items-center gap-4">
                    <div class="flex flex-row items-center gap-3 flex-1">
                        {Avatar { initials: initials(&track.artist), size: 44.0 }}
                        <div class="flex flex-col">
                            <span class="text-sm font-semibold">{track.title.clone()}</span>
                            <span class="text-xs" style={css! { color: var(--muted-foreground); }}>{track.artist.clone()}</span>
                        </div>
                    </div>
                    <div class="flex flex-row items-center gap-2">
                        {icon_button(icons::SHUFFLE, move |cx| set_shuffle_btn.update(cx, |v| *v = !*v))}
                        {icon_button(icons::SKIP_BACK, move |cx| {
                            set_idx_prev.update(cx, |i| *i = if *i == 0 { n - 1 } else { *i - 1 });
                            set_time_prev.set(cx, 0.0);
                        })}
                        {icon_button(if playing_now { icons::PAUSE } else { icons::PLAY }, move |cx| {
                            set_play.update(cx, |v| *v = !*v);
                        })}
                        {icon_button(icons::SKIP_FORWARD, move |cx| {
                            set_idx_next.update(cx, |i| *i = (*i + 1) % n);
                            set_time_next.set(cx, 0.0);
                        })}
                        {icon_button(icons::REPEAT, move |cx| set_repeat_btn.update(cx, |v| *v = !*v))}
                    </div>
                    <div class="flex flex-row items-center gap-2 flex-1 justify-end">
                        <span class="text-xs" style={css! { color: var(--muted-foreground); }}>
                            {format!("{} / {}", fmt_time(time_now), fmt_time(track.duration))}
                        </span>
                        <div ref={vol_ref.clone()}>
                            {icon_button(if muted_now { icons::VOLUME_MUTE } else { icons::VOLUME }, move |cx| {
                                set_mute.update(cx, |v| *v = !*v);
                                set_vol_open_btn.update(cx, |v| *v = !*v);
                            })}
                        </div>
                        {Popover {
                            open: vol_open.get(),
                            anchor: vol_ref.clone(),
                            children: vec![
                                Slider {
                                    value: if muted_now { 0.0 } else { volume_now },
                                    min: 0.0,
                                    max: 1.0,
                                    step: 0.01,
                                    on_change: Box::new(move |v, cx| set_volume_slider.set(cx, v)),
                                }
                                .into_any_element(),
                            ],
                        }}
                    </div>
                </div>
                <div class="flex flex-row gap-3 text-xs" style={css! { color: var(--muted-foreground); }}>
                    <span>{if shuffle_now { "shuffle on" } else { "shuffle off" }}</span>
                    <span>{if repeat_now { "repeat on" } else { "repeat off" }}</span>
                    <span>{format!("{} liked", liked_now.len())}</span>
                </div>
            </div>

            {Drawer {
                open: drawer_open.get(),
                on_close: Box::new(move |cx| set_drawer_close.set(cx, false)),
                children: vec![
                    view! { <span class="font-bold">{"Browse"}</span> }.into_any_element(),
                    sidebar_item(icons::HOME, "Discover", true, |_| {}).into_any_element(),
                    sidebar_item(icons::LIBRARY, "Library", false, |_| {}).into_any_element(),
                    sidebar_item(icons::LIST, "Playlists", false, |_| {}).into_any_element(),
                ],
            }}
        </div>
    }
}

fn run() {
    #[cfg(not(target_family = "wasm"))]
    let gpui_app = application();
    #[cfg(target_family = "wasm")]
    let gpui_app = single_threaded_web();

    let launch = |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(960.), px(640.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| vgui::mount(window, cx, app),
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
