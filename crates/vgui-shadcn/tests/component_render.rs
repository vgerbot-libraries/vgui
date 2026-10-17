use gpui::IntoElement;
use vgui_shadcn::prelude::*;

#[test]
fn render_all_new_components() {
    vgui::__test_enter_render_scope();
    apply_theme(false);
    // tag: closable variant exercises the close-button branch
    Tag {
        variant: TagVariant::Default,
        label: String::from("new"),
        closable: true,
        on_close: Box::new(|_: &mut gpui::App| {}),
    }
    .into_any_element();

    // breadcrumb: first entry is a link, last is current page
    Breadcrumb {
        items: vec![
            (
                String::from("Home"),
                Some(
                    Box::new(|_: &mut gpui::App| {}) as Box<dyn Fn(&mut gpui::App)>,
                ),
            ),
            (String::from("Docs"), None),
        ],
    }
    .into_any_element();

    // pagination at a middle page exercises the ellipsis window
    Pagination {
        page: 5,
        page_count: 20,
        on_change: Box::new(|_: usize, _: &mut gpui::App| {}),
    }
    .into_any_element();

    // rating: interactive and read-only
    Rating {
        value: 3.0,
        count: 5,
        on_change: Some(Box::new(|_: usize, _: &mut gpui::App| {})),
    }
    .into_any_element();
    Rating {
        value: 2.4,
        count: 5,
        on_change: None,
    }
    .into_any_element();

    // toolbar + separator
    Toolbar { children: vec![] }.into_any_element();
    ToolbarSeparator {}.into_any_element();

    // group box
    GroupBox {
        title: String::from("Group"),
        children: vec![],
    }
    .into_any_element();

    // description list
    DescriptionList {
        title: Some(String::from("Details")),
        items: vec![(String::from("Name"), "Ada".to_string().into_any_element())],
    }
    .into_any_element();

    // button group
    ButtonGroup { children: vec![] }.into_any_element();

    // toggle pressed
    Toggle {
        pressed: true,
        on_toggle: Box::new(|_: bool, _: &mut gpui::App| {}),
        children: vec![],
    }
    .into_any_element();
}
