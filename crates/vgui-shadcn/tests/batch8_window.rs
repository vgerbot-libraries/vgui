//! Headless tests for the batch-8 sortable/selectable data table.
//!
//! Header click cycles ascending -> descending -> unsorted; body-row clicks
//! toggle selection keyed by the *original* row index, so selection stays
//! correct while the visible order changes. Coordinates were found with a
//! throwaway sweep (single 120px sortable column, 30px rows).

#![cfg(not(target_family = "wasm"))]

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{
    point, px, InteractiveElement, IntoElement, Modifiers, MouseButton, MouseDownEvent, MouseUpEvent,
    ParentElement, Styled, TestAppContext, VisualTestContext,
};
use vgui_shadcn::prelude::*;

fn draw(cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
}

fn click(cx: &mut VisualTestContext, x: f32, y: f32) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseUpEvent {
        position: point(px(x), px(y)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
}

type TableHandle = Rc<RefCell<Option<vgui::Store<TableState>>>>;

/// Header clicks cycle sort state asc -> desc -> unsorted.
#[test]
fn table_header_cycles_sort() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let handle: TableHandle = Rc::new(RefCell::new(None));
    let (_view, mut cx) = app.add_window_view({
        let handle = handle.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let (el, store) = table(DataTable::new(
                    vec![TableColumn::new("Name", 120.).sortable()],
                    vec![
                        vec!["Carol".to_string()],
                        vec!["Alice".to_string()],
                        vec!["Bob".to_string()],
                    ],
                ));
                *handle.borrow_mut() = Some(store);
                gpui::div().w(px(140.)).child(el).into_any_element()
            })
        }
    });
    draw(&mut cx);
    let state = || handle.borrow().as_ref().unwrap().get();

    click(&mut cx, 40., 12.);
    draw(&mut cx);
    assert_eq!(state().sort_col, Some(0));
    assert!(state().sort_asc, "first header click sorts ascending");

    click(&mut cx, 40., 12.);
    draw(&mut cx);
    assert_eq!(state().sort_col, Some(0));
    assert!(!state().sort_asc, "second click sorts descending");

    click(&mut cx, 40., 12.);
    draw(&mut cx);
    assert_eq!(state().sort_col, None, "third click clears the sort");
}

/// Selection is keyed by original index and stays correct through sorting.
#[test]
fn table_selection_tracks_original_index_through_sort() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let handle: TableHandle = Rc::new(RefCell::new(None));
    let (_view, mut cx) = app.add_window_view({
        let handle = handle.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let (el, store) = table(DataTable::new(
                    vec![TableColumn::new("Name", 120.).sortable()],
                    vec![
                        vec!["Carol".to_string()],
                        vec!["Alice".to_string()],
                        vec!["Bob".to_string()],
                    ],
                ));
                *handle.borrow_mut() = Some(store);
                gpui::div().w(px(140.)).child(el).into_any_element()
            })
        }
    });
    draw(&mut cx);
    let selected = || handle.borrow().as_ref().unwrap().get().selected;

    click(&mut cx, 40., 12.); // sort asc -> Alice(1), Bob(2), Carol(0)
    draw(&mut cx);

    click(&mut cx, 40., 47.); // first visible body row = Alice
    draw(&mut cx);
    assert_eq!(selected(), vec![1], "first visible ascending row is original 1");

    click(&mut cx, 40., 77.); // second visible row = Bob
    draw(&mut cx);
    assert_eq!(selected(), vec![1, 2], "toggling another row accumulates");

    click(&mut cx, 40., 47.); // toggle Alice back off
    draw(&mut cx);
    assert_eq!(selected(), vec![2], "clicking a selected row deselects it");
}

/// A multi-column table draws across two frames without panicking.
#[test]
fn table_draws_multi_column() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            let (el, _store) = table(DataTable::new(
                vec![
                    TableColumn::new("Name", 120.).sortable(),
                    TableColumn::new("Age", 60.),
                    TableColumn::new("City", 140.),
                ],
                vec![
                    vec!["Alice".into(), "25".into(), "NYC".into()],
                    vec!["Bob".into(), "40".into(), "SFO".into()],
                ],
            ));
            gpui::div().w(px(340.)).child(el).into_any_element()
        })
    });
    draw(&mut cx);
    draw(&mut cx);
}
