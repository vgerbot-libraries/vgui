//! Numeric entry with increment/decrement steppers.
//!
//! A text field paired with ▲ / ▼ stepper buttons. Typing is free-form while
//! focused; the buffer is parsed on every change and `on_change` fires only for
//! in-range finite numbers. Steppers snap to the nearest whole `step` and clamp
//! to `min`/`max`.

use vgui::prelude::*;

fn format_value(v: f64) -> String {
    if v.fract().abs() < f64::EPSILON && v.is_finite() {
        format!("{:.0}", v)
    } else {
        let s = format!("{:.4}", v);
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

fn clamp_to(v: f64, min: Option<f64>, max: Option<f64>) -> f64 {
    let mut v = v;
    if let Some(lo) = min {
        v = v.max(lo);
    }
    if let Some(hi) = max {
        v = v.min(hi);
    }
    v
}

/// Controlled numeric field with steppers.
///
/// - `value` — current numeric value (also the initial edit buffer).
/// - `step` — increment applied by the stepper buttons.
/// - `min`/`max` — optional inclusive bounds.
/// - `on_change` — called with the new value after a valid edit or step.
pub fn number_input(
    value: f64,
    step: f64,
    min: Option<f64>,
    max: Option<f64>,
    on_change: std::rc::Rc<dyn Fn(f64, &mut gpui::App)>,
) -> gpui::AnyElement {
    let (text, set_text) = create_signal(format_value(value));

    let commit = {
        let set_text = set_text.clone();
        let on_change = on_change.clone();
        move |next: f64, cx: &mut gpui::App| {
            let next = clamp_to(next, min, max);
            set_text.set(cx, format_value(next));
            on_change(next, cx);
        }
    };

    let step_up = {
        let text = text.clone();
        let commit = commit.clone();
        click(move |cx| {
            let current: f64 = text.get().trim().parse().unwrap_or(0.0);
            let stepped = ((current / step).round() + 1.0) * step;
            commit(stepped, cx);
        })
    };
    let step_down = {
        let text = text.clone();
        let commit = commit.clone();
        click(move |cx| {
            let current: f64 = text.get().trim().parse().unwrap_or(0.0);
            let stepped = ((current / step).round() - 1.0) * step;
            commit(stepped, cx);
        })
    };

    view! {
        <div style={css! {
            display: flex;
            flex-direction: row;
            align-items: stretch;
            background: var(--background);
            border-width: 1px;
            border-style: solid;
            border-color: var(--input);
            border-radius: var(--radius);
            height: 36px;
            width: 140px;
        }}>
            <input
                type="text"
                value={text.get()}
                class="flex-1 w-full"
                style={css! {
                    border-width: 0;
                    background: var(--background);
                    color: var(--foreground);
                    padding: 4px 8px;
                    min-width: 0;
                    min-height: 0;
                }}
                on:input={input_cb({
                    let set_text = set_text.clone();
                    let on_change = on_change.clone();
                    move |v, cx| {
                        let raw = v.to_string();
                        let trimmed = raw.trim();
                        // Keep the buffer so the user can type "-", ".", "1." etc.,
                        // but only publish finite, in-range numbers.
                        if let Ok(n) = trimmed.parse::<f64>() {
                            if n.is_finite() {
                                let in_range = (min.map(|lo| n >= lo).unwrap_or(true))
                                    && (max.map(|hi| n <= hi).unwrap_or(true));
                                if in_range {
                                    on_change(n, cx);
                                }
                            }
                        }
                        set_text.set(cx, raw);
                    }
                })}
            />
            <div style={css! {
                display: flex;
                flex-direction: column;
                border-left-width: 1px;
                border-style: solid;
                border-color: var(--border);
                width: 26px;
            }}>
                <button
                    style={css! {
                        flex: 1;
                        border-width: 0;
                        border-bottom-width: 1px;
                        border-style: solid;
                        border-color: var(--border);
                        background: var(--background);
                        color: var(--muted-foreground);
                        cursor: pointer;
                        font-size: 9px;
                        padding: 0px;
                    }}
                    hover={css! { background: var(--accent); color: var(--foreground); }}
                    on:click={step_up}
                >{"\u{25B2}"}</button>
                <button
                    style={css! {
                        flex: 1;
                        border-width: 0;
                        background: var(--background);
                        color: var(--muted-foreground);
                        cursor: pointer;
                        font-size: 9px;
                        padding: 0px;
                    }}
                    hover={css! { background: var(--accent); color: var(--foreground); }}
                    on:click={step_down}
                >{"\u{25BC}"}</button>
            </div>
        </div>
    }
    .into_any_element()
}
