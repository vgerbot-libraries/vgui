use gpui::{IntoElement, ParentElement};

pub fn show(
    when: bool,
    then: impl IntoElement,
    fallback: impl IntoElement,
) -> gpui::AnyElement {
    if when {
        then.into_any_element()
    } else {
        fallback.into_any_element()
    }
}

pub fn show_when(when: bool, then: impl IntoElement) -> gpui::AnyElement {
    if when {
        then.into_any_element()
    } else {
        gpui::Empty.into_any_element()
    }
}

pub fn for_each<T, E: IntoElement>(
    items: impl IntoIterator<Item = T>,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let mut parent = gpui::div();
    for (i, item) in items.into_iter().enumerate() {
        parent = parent.child(child(item, i));
    }
    parent.into_any_element()
}

pub fn for_each_or<T, E: IntoElement, F: IntoElement>(
    items: impl IntoIterator<Item = T>,
    fallback: F,
    mut child: impl FnMut(T, usize) -> E,
) -> gpui::AnyElement {
    let mut parent = gpui::div();
    let mut n = 0;
    for (i, item) in items.into_iter().enumerate() {
        n += 1;
        parent = parent.child(child(item, i));
    }
    if n == 0 {
        fallback.into_any_element()
    } else {
        parent.into_any_element()
    }
}
