use std::cell::RefCell;
use std::rc::Rc;

use vgui::prelude::*;

static GREETING: Context<String> = Context::new();

#[test]
fn provider_macro_provides_to_child_during_construction() {
    let observed = Rc::new(RefCell::new(String::new()));
    let obs = observed.clone();
    let _el = view! {
        <Provider context={GREETING} value={"hello".to_string()}>
            { { let o = obs.clone(); *o.borrow_mut() = use_context(&GREETING).unwrap(); gpui::Empty } }
        </Provider>
    };
    assert_eq!(&*observed.borrow(), "hello");
    assert!(use_context(&GREETING).is_none()); // stack popped after <Provider>
}

#[test]
fn provider_macro_nested_inner_shadows_outer() {
    let outer_seen = Rc::new(RefCell::new(false));
    let inner_val = Rc::new(RefCell::new(String::new()));
    let o = outer_seen.clone();
    let iv = inner_val.clone();
    let _el = view! {
        <Provider context={GREETING} value={"outer".to_string()}>
            { { *o.borrow_mut() = use_context(&GREETING).unwrap() == "outer"; gpui::Empty } }
            <Provider context={GREETING} value={"inner".to_string()}>
                { { *iv.borrow_mut() = use_context(&GREETING).unwrap(); gpui::Empty } }
            </Provider>
        </Provider>
    };
    assert!(*outer_seen.borrow());
    assert_eq!(&*inner_val.borrow(), "inner");
    assert!(use_context(&GREETING).is_none());
}
