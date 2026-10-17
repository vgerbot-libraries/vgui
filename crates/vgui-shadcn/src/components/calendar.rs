//! Month calendar grid for date selection.
//!
//! A self-contained month view: prev/next month navigation, weekday header,
//! and a day grid with leading blanks for the first weekday. The selected day
//! is highlighted and clicking a day fires `on_select(year, month, day)`.
//! Date math (weekday offset, leap-aware month lengths) is local and matches
//! vgui's built-in date input picker.
//!
//! The interactive cells are built with the gpui builder API (the same pattern
//! as the menus and vgui's built-in date picker) so they are laid out and
//! hit-tested reliably.

use std::rc::Rc;

use gpui::{
    div, px, App, ClickEvent, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, Styled, Window,
};
use vgui::{create_signal, next_auto_id};
use vgui::prelude::*;

type OnSelect = Rc<dyn Fn(i32, u32, u32, &mut App)>;

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 30,
    }
}

fn month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

/// Day-of-week (0 = Sunday) of the first day of `month` (Zeller's congruence).
fn weekday_of_first(year: i32, month: u32) -> u32 {
    let (y, m): (i32, i32) = if month < 3 {
        (year - 1, month as i32 + 12)
    } else {
        (year, month as i32)
    };
    let k = y % 100;
    let j = y / 100;
    let h = (1 + 13 * (m + 1) / 5 + k + k / 4 + j / 4 + 5 * j) % 7;
    ((h + 6) % 7) as u32
}

fn nav_button(
    id: &'static str,
    glyph: &'static str,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> gpui::AnyElement {
    let b = div()
        .id((id, next_auto_id()))
        .w(px(24.))
        .h(px(24.))
        .flex()
        .items_center()
        .justify_center()
        .cursor_pointer()
        .rounded_md()
        .on_click(on_click)
        .child(glyph);
    css! { color: var(--muted-foreground); font-size: 10px; }
        .apply(b)
        .into_any_element()
}

/// Render one month calendar.
///
/// - `selected` — currently selected `(year, month, day)`, if any.
/// - `on_select(year, month, day, cx)` — fired when a day cell is clicked.
pub fn calendar(
    selected: Option<(i32, u32, u32)>,
    on_select: OnSelect,
) -> gpui::AnyElement {
    let (init_year, init_month) = selected.map(|(y, m, _)| (y, m)).unwrap_or((2026, 1));
    let (year, set_year) = create_signal(init_year);
    let (month, set_month) = create_signal(init_month);

    let y = year.get();
    let m = month.get();
    let first_weekday = weekday_of_first(y, m);
    let dim = days_in_month(y, m);
    let prev = {
        let year = year.clone();
        let month = month.clone();
        let set_year = set_year.clone();
        let set_month = set_month.clone();
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            let mut yy = year.get();
            let mut mm = month.get();
            if mm == 1 {
                mm = 12;
                yy -= 1;
            } else {
                mm -= 1;
            }
            set_year.set(cx, yy);
            set_month.set(cx, mm);
        }
    };
    let next = {
        let year = year.clone();
        let month = month.clone();
        move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
            let mut yy = year.get();
            let mut mm = month.get();
            if mm == 12 {
                mm = 1;
                yy += 1;
            } else {
                mm += 1;
            }
            set_year.set(cx, yy);
            set_month.set(cx, mm);
        }
    };

    let title = div()
        .text_size(px(13.))
        .child(format!("{} {}", month_name(m), y));
    let title = css! { font-weight: 600; color: var(--foreground); }.apply(title);

    let header = div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .mb_2()
        .child(nav_button("cal-prev", "\u{25C0}", prev))
        .child(title)
        .child(nav_button("cal-next", "\u{25B6}", next));

    // Weekday header row.
    let mut weekday_row = div().flex().flex_row().mb_1();
    for wd in ["Su", "Mo", "Tu", "We", "Th", "Fr", "Sa"] {
        let cell = div()
            .w(px(32.))
            .h(px(20.))
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(10.))
            .child(wd);
        let cell = css! { color: var(--muted-foreground); }.apply(cell);
        weekday_row = weekday_row.child(cell);
    }

    // Day grid: leading blanks, then day buttons wrapped into 7-cell weeks.
    let mut grid = div().flex().flex_col();
    let mut week = div().flex().flex_row();
    let mut col: i32 = 0;
    for _ in 0..first_weekday {
        week = week.child(div().w(px(32.)).h(px(32.)));
        col += 1;
    }
    for day in 1..=dim {
        let is_selected = selected
            .map(|(sy, sm, sd)| sy == y && sm == m && sd == day)
            .unwrap_or(false);
        let on_day = on_select.clone();
        let cell = div()
            .id(("cal-day", next_auto_id()))
            .w(px(32.))
            .h(px(32.))
            .flex()
            .items_center()
            .justify_center()
            .rounded_md()
            .cursor_pointer()
            .text_size(px(12.))
            .on_click(move |_: &ClickEvent, _: &mut Window, cx: &mut App| {
                on_day(y, m, day, cx);
            })
            .child(format!("{}", day));
        let cell = if is_selected {
            css! {
                background: var(--primary);
                color: var(--primary-foreground);
            }
            .apply(cell)
        } else {
            css! {
                background: var(--background);
                color: var(--foreground);
            }
            .apply(cell)
        };
        week = week.child(cell);
        col += 1;
        if col == 7 {
            grid = grid.child(week);
            week = div().flex().flex_row();
            col = 0;
        }
    }
    if col > 0 {
        // Trailing blanks to complete the final week.
        for _ in col..7 {
            week = week.child(div().w(px(32.)).h(px(32.)));
        }
        grid = grid.child(week);
    }

    let root = div()
        .flex()
        .flex_col()
        .child(header)
        .child(weekday_row)
        .child(grid);
    css! {
        background: var(--popover);
        color: var(--popover-foreground);
        border-width: 1px;
        border-style: solid;
        border-color: var(--border);
        border-radius: var(--radius);
        padding: 12px;
    }
    .apply(root)
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn february_respects_leap_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        // Century rule: divisible by 100 but not 400 is not a leap year.
        assert_eq!(days_in_month(1900, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29);
    }

    #[test]
    fn month_lengths_match_gregorian_calendar() {
        for m in [1u32, 3, 5, 7, 8, 10, 12] {
            assert_eq!(days_in_month(2025, m), 31);
        }
        for m in [4u32, 6, 9, 11] {
            assert_eq!(days_in_month(2025, m), 30);
        }
    }

    #[test]
    fn weekday_of_known_dates() {
        // 2024-03-01 was a Friday (index 5, Sunday = 0).
        assert_eq!(weekday_of_first(2024, 3), 5);
        // 2026-01-01 is a Thursday (index 4).
        assert_eq!(weekday_of_first(2026, 1), 4);
        // January is treated as month 13 of the previous year internally.
        assert_eq!(weekday_of_first(2025, 1), 3); // 2025-01-01 Wednesday
    }
}
