use gpui::IntoElement;

pub fn into_child(value: impl IntoViewChild) -> gpui::AnyElement {
    value.into_view_child()
}

pub trait IntoViewChild {
    fn into_view_child(self) -> gpui::AnyElement;
}

impl<T: IntoElement> IntoViewChild for T {
    fn into_view_child(self) -> gpui::AnyElement {
        self.into_any_element()
    }
}

pub fn click(
    f: impl Fn(&mut gpui::App) + 'static,
) -> impl Fn(&gpui::ClickEvent, &mut gpui::Window, &mut gpui::App) + 'static {
    move |_, _, cx| f(cx)
}

#[cfg(test)]
mod tests {
    use super::into_child;

    #[test]
    fn into_child_i32_does_not_panic() {
        let _ = into_child(42i32.to_string());
    }
}
