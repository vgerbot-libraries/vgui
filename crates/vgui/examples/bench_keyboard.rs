//! Keyboard event system benchmark.
//!
//! Measures the throughput of the three core code paths added by the
//! keyboard event system:
//!
//! 1. `KeyboardEvent` construction from a `Keystroke` (the hot path in
//!    every `on:keydown` / `on:keyup` dispatch).
//! 2. `stop_propagation` flag check — the per-handler branch executed in
//!    the global dispatch loop.
//! 3. Handler registration + dispatch — simulates N `use_key_down`
//!    handlers registered into a scope and dispatched in order, with
//!    propagation stopping at a configurable handler index.
//!
//! The benchmark is deterministic: fixed seed, no I/O, no timers.

use std::hint::black_box;
use std::time::Instant;

use gpui::{Keystroke, Modifiers};
use vgui::KeyboardEvent;

const ITERATIONS: usize = 100_000;
const NUM_HANDLERS: usize = 64;

fn make_keystroke(key: &str, key_char: Option<&str>, mods: Modifiers) -> Keystroke {
    Keystroke {
        modifiers: mods,
        key: key.to_string(),
        key_char: key_char.map(str::to_string),
    }
}

fn bench_construction() -> u128 {
    let strokes: Vec<Keystroke> = (0..256)
        .map(|i| {
            let key = match i % 10 {
                0 => "enter",
                1 => "space",
                2 => "escape",
                3 => "arrowup",
                4 => "arrowdown",
                5 => "tab",
                6 => "backspace",
                7 => "a",
                8 => "z",
                _ => "f1",
            };
            let mods = Modifiers {
                shift: i & 1 != 0,
                control: i & 2 != 0,
                alt: i & 4 != 0,
                platform: i & 8 != 0,
                function: false,
            };
            let kc = if i & 16 != 0 {
                Some("x")
            } else {
                None
            };
            make_keystroke(key, kc, mods)
        })
        .collect();

    let start = Instant::now();
    let mut acc: u64 = 0;
    for _ in 0..ITERATIONS {
        for ks in &strokes {
            let ev = KeyboardEvent::from_keystroke(ks, true);
            acc = acc.wrapping_add(ev.key.len() as u64);
            acc = acc.wrapping_add(ev.code.len() as u64);
        }
    }
    let elapsed = start.elapsed();
    black_box(acc);
    elapsed.as_nanos()
}

fn bench_stop_propagation() -> u128 {
    let ks = make_keystroke("a", Some("a"), Modifiers::default());
    let start = Instant::now();
    let mut acc: u64 = 0;
    for i in 0..ITERATIONS {
        let ev = KeyboardEvent::from_keystroke(&ks, false);
        if i & 3 == 0 {
            ev.stop_propagation();
        }
        acc = acc.wrapping_add(ev.is_propagation_stopped() as u64);
    }
    let elapsed = start.elapsed();
    black_box(acc);
    elapsed.as_nanos()
}

fn bench_handler_dispatch() -> u128 {
    use std::cell::RefCell;
    use std::rc::Rc;

    let counter: Rc<RefCell<u64>> = Rc::new(RefCell::new(0));

    // Build a vector of handlers mimicking what
    // `collect_all_key_down_handlers` produces.
    let handlers: Vec<Box<dyn Fn(&KeyboardEvent)>> = (0..NUM_HANDLERS)
        .map(|idx| {
            let c = counter.clone();
            Box::new(move |ev: &KeyboardEvent| {
                *c.borrow_mut() += 1;
                // Handler at index 16 stops propagation.
                if idx == 16 {
                    ev.stop_propagation();
                }
            }) as Box<dyn Fn(&KeyboardEvent)>
        })
        .collect();

    let ks = make_keystroke("enter", None, Modifiers::default());

    let start = Instant::now();
    for _ in 0..ITERATIONS {
        let ev = KeyboardEvent::from_keystroke(&ks, true);
        for h in &handlers {
            h(&ev);
            if ev.is_propagation_stopped() {
                break;
            }
        }
    }
    let elapsed = start.elapsed();
    black_box(*counter.borrow());
    elapsed.as_nanos()
}

fn bench_new_constructor() -> u128 {
    let start = Instant::now();
    let mut acc: u64 = 0;
    for i in 0..ITERATIONS {
        let ev = KeyboardEvent::new(
            "ArrowUp",
            "ArrowUp",
            i & 1 != 0,
            i & 2 != 0,
            i & 4 != 0,
            i & 8 != 0,
            i & 16 != 0,
            if i & 32 != 0 { Some("A".to_string()) } else { None },
        );
        acc = acc.wrapping_add(ev.key.len() as u64);
        acc = acc.wrapping_add(ev.is_propagation_stopped() as u64);
    }
    let elapsed = start.elapsed();
    black_box(acc);
    elapsed.as_nanos()
}

fn main() {
    let t_construct = bench_construction();
    let t_stop_prop = bench_stop_propagation();
    let t_dispatch = bench_handler_dispatch();
    let t_new = bench_new_constructor();

    // Primary metric: total nanoseconds across all benchmarks (lower = better).
    let total = t_construct + t_stop_prop + t_dispatch + t_new;

    println!("METRIC total_ns={}", total);
    println!("METRIC construction_ns={}", t_construct);
    println!("METRIC stop_propagation_ns={}", t_stop_prop);
    println!("METRIC handler_dispatch_ns={}", t_dispatch);
    println!("METRIC new_constructor_ns={}", t_new);
    println!("METRIC iterations={}", ITERATIONS);
    println!("METRIC num_handlers={}", NUM_HANDLERS);
}
