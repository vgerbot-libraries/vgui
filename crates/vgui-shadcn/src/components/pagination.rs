use vgui::for_each;
use vgui::prelude::*;

use crate::icons;

/// Build the collapsed page window: `(label, target page)`. A `None` target
/// renders a non-interactive ellipsis. Mirrors shadcn pagination collapsing.
fn page_window(page: usize, page_count: usize) -> Vec<(String, Option<usize>)> {
    if page_count <= 7 {
        return (1..=page_count).map(|p| (p.to_string(), Some(p))).collect();
    }
    let mut entries = vec![("1".to_string(), Some(1usize))];
    let start = page.saturating_sub(1).max(2);
    let end = (page + 1).min(page_count - 1);
    if start > 2 {
        entries.push(("…".to_string(), None));
    }
    for p in start..=end {
        entries.push((p.to_string(), Some(p)));
    }
    if end < page_count - 1 {
        entries.push(("…".to_string(), None));
    }
    entries.push((page_count.to_string(), Some(page_count)));
    entries
}

fn page_button_style(active: bool) -> vgui::Css {
    if active {
        css! {
            display: flex;
            align-items: center;
            justify-content: center;
            height: 32px;
            min-width: 32px;
            padding: 0px 6px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--primary);
            background: var(--primary);
            color: var(--primary-foreground);
            cursor: pointer;
            font-size: 14px;
        }
    } else {
        css! {
            display: flex;
            align-items: center;
            justify-content: center;
            height: 32px;
            min-width: 32px;
            padding: 0px 6px;
            border-radius: var(--radius);
            border-width: 1px;
            border-style: solid;
            border-color: var(--border);
            background: var(--background);
            color: var(--foreground);
            cursor: pointer;
            font-size: 14px;
        }
    }
}

#[vgui_component]
pub fn pagination(
    page: usize,
    page_count: usize,
    on_change: Box<dyn Fn(usize, &mut gpui::App) + 'static>,
) -> impl gpui::IntoElement {
    let on_change = std::rc::Rc::new(on_change);
    let page = page.max(1);
    let page_count = page_count.max(1);
    let entries = page_window(page, page_count);
    let on_change_tail = on_change.clone();
    let at_start = page <= 1;
    let at_end = page >= page_count;

    view! {
        <nav style={css! {
            display: flex;
            flex-direction: row;
            align-items: center;
            gap: 4px;
        }}>
            <button
                style={page_button_style(false)}
                hover={if at_start { css! { opacity: 1; } } else { css! { background: var(--accent); } }}
                on:click={click({
                    let on_change = on_change.clone();
                    move |cx| if !at_start { on_change(page - 1, cx) }
                })}
            >
                {icons::CHEVRON_LEFT}
            </button>
            {for_each(entries.into_iter().enumerate(), move |(_, (label, target)), _| {
                let on_change = on_change.clone();
                if let Some(target) = target {
                    let active = target == page;
                    view! {
                        <button
                            style={page_button_style(active)}
                            on:click={click(move |cx| on_change(target, cx))}
                        >
                            {label}
                        </button>
                    }.into_any_element()
                } else {
                    view! {
                        <span style={css! {
                            display: flex;
                            align-items: center;
                            justify-content: center;
                            height: 32px;
                            min-width: 32px;
                            color: var(--muted-foreground);
                        }}>
                            {label}
                        </span>
                    }.into_any_element()
                }
            })}
            <button
                style={page_button_style(false)}
                hover={if at_end { css! { opacity: 1; } } else { css! { background: var(--accent); } }}
                on:click={click(move |cx| if !at_end { on_change_tail(page + 1, cx) })}
            >
                {icons::CHEVRON_RIGHT}
            </button>
        </nav>
    }
}

#[cfg(test)]
mod tests {
    use super::page_window;

    fn targets(page: usize, count: usize) -> Vec<Option<usize>> {
        page_window(page, count)
            .into_iter()
            .map(|(_, t)| t)
            .collect()
    }

    #[test]
    fn small_counts_show_every_page_without_ellipsis() {
        assert_eq!(
            targets(1, 5),
            vec![Some(1), Some(2), Some(3), Some(4), Some(5)]
        );
        assert_eq!(targets(3, 7).iter().filter(|t| t.is_none()).count(), 0);
    }

    #[test]
    fn middle_page_collapses_both_sides() {
        // Page 5 of 20 -> 1 … 4 5 6 … 20
        assert_eq!(
            targets(5, 20),
            vec![
                Some(1),
                None,
                Some(4),
                Some(5),
                Some(6),
                None,
                Some(20)
            ]
        );
    }

    #[test]
    fn first_page_has_no_leading_ellipsis() {
        assert_eq!(targets(1, 20), vec![Some(1), Some(2), None, Some(20)]);
    }

    #[test]
    fn last_page_has_no_trailing_ellipsis() {
        assert_eq!(targets(20, 20), vec![Some(1), None, Some(19), Some(20)]);
    }
}
