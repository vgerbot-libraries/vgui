//! Headless tests for batch-7 widgets: Cartesian plot, onboarding
//! questionnaire, settings panel, runtime inspector, and touch text-selection.
//!
//! Plot and inspector are canvas/structural widgets whose only interaction
//! (inspector chevrons) is rendered through `for_each`, which the gpui test
//! platform does not hit-test; those are covered by multi-frame draw smoke
//! tests. The settings, questionnaire and touch controls are exercised with
//! real pointer events against coordinates found by throwaway sweeps.

#![cfg(not(target_family = "wasm"))]

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gpui::{
    point, px, IntoElement, Modifiers, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
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

fn drag(cx: &mut VisualTestContext, x0: f32, y0: f32, x1: f32, y1: f32) {
    cx.simulate_event(MouseDownEvent {
        position: point(px(x0), px(y0)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
        first_mouse: false,
    });
    cx.simulate_event(MouseMoveEvent {
        position: point(px(x1), px(y1)),
        pressed_button: Some(MouseButton::Left),
        modifiers: Modifiers::default(),
    });
    cx.simulate_event(MouseUpEvent {
        position: point(px(x1), px(y1)),
        modifiers: Modifiers::default(),
        button: MouseButton::Left,
        click_count: 1,
    });
}

/// Clicking the toggle pill flips the backing store; clicking a choice option
/// records its index.
#[test]
fn settings_toggle_and_choice_update_store() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let handle: Rc<RefCell<Option<vgui::Store<Settings>>>> = Rc::new(RefCell::new(None));
    let (_view, mut cx) = app.add_window_view({
        let handle = handle.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let (el, store) = settings_panel(
                    SettingPanel::new("Prefs", Settings::new())
                        .row(SettingRow::toggle("wifi", "Wi-Fi"))
                        .row(SettingRow::choice(
                            "theme",
                            "Theme",
                            vec!["Light".to_string(), "Dark".to_string()],
                        )),
                );
                *handle.borrow_mut() = Some(store);
                gpui::div().w(px(320.)).child(el).into_any_element()
            })
        }
    });
    draw(&mut cx);
    assert!(!handle.borrow().as_ref().unwrap().get().toggle("wifi"));
    // Toggle pill (verified coordinate).
    click(&mut cx, 274., 56.);
    draw(&mut cx);
    assert!(
        handle.borrow().as_ref().unwrap().get().toggle("wifi"),
        "toggle pill click must flip the wifi preference"
    );
    // "Dark" option (verified coordinate).
    click(&mut cx, 78., 124.);
    draw(&mut cx);
    assert_eq!(
        handle.borrow().as_ref().unwrap().get().choice("theme"),
        1,
        "clicking Dark must select option index 1"
    );
}

/// Pressing Next with an empty required answer never finishes or panics.
#[test]
fn questionnaire_blocks_empty_answer() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let finished = Rc::new(Cell::new(0u8));
    let (_view, mut cx) = app.add_window_view({
        let finished = finished.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let q = questionnaire(
                    vec![Question::new("A"), Question::new("B")],
                    Rc::new({
                        let f = finished.clone();
                        move |_a: &Vec<String>, _c| f.set(1)
                    }),
                );
                gpui::div().w(px(320.)).child(q).into_any_element()
            })
        }
    });
    draw(&mut cx);
    for _ in 0..3 {
        click(&mut cx, 86., 118.);
        draw(&mut cx);
    }
    assert_eq!(finished.get(), 0, "empty answer must never finish");
}

/// With always-valid questions, Next advances and the last press finishes;
/// the post-finish screen keeps hook slots stable across another draw.
#[test]
fn questionnaire_advances_and_finishes() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let finished = Rc::new(Cell::new(0u8));
    let (_view, mut cx) = app.add_window_view({
        let finished = finished.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let q = questionnaire(
                    vec![
                        Question::new("A").accept_when(|_| true),
                        Question::new("B").accept_when(|_| true),
                    ],
                    Rc::new({
                        let f = finished.clone();
                        move |_a: &Vec<String>, _c| f.set(1)
                    }),
                );
                gpui::div().w(px(320.)).child(q).into_any_element()
            })
        }
    });
    draw(&mut cx);
    click(&mut cx, 86., 118.); // Next -> step 2
    draw(&mut cx);
    assert_eq!(finished.get(), 0, "first press advances, does not finish");
    click(&mut cx, 86., 118.); // Finish
    draw(&mut cx);
    assert_eq!(finished.get(), 1, "second press on the last step finishes");
    draw(&mut cx); // post-finish render must not panic on hook order
}

/// Dragging the end caret left clamps and orders the reported selection range.
#[test]
fn touch_selection_drag_reports_ordered_range() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let last = Rc::new(Cell::new((0usize, 0usize)));
    let (_view, mut cx) = app.add_window_view({
        let last = last.clone();
        move |window, cx| {
            vgui::VguiRoot::new(cx, window.window_handle(), move || {
                let t = touch_selection(
                    TouchSelection::new(20),
                    Rc::new({
                        let l = last.clone();
                        move |s, e, _c| l.set((s, e))
                    }),
                );
                gpui::div().w(px(240.)).child(t).into_any_element()
            })
        }
    });
    draw(&mut cx);
    // Grab the right caret and drag it to the middle: end index ~10.
    drag(&mut cx, 180., 24., 90., 24.);
    draw(&mut cx);
    let (s, e) = last.get();
    assert_eq!(s, 0, "start caret stays at 0");
    assert!((8..=12).contains(&e), "end caret lands near index 10, got {e}");
    assert!(s <= e, "range stays ordered");
}

/// A multi-series plot draws across two frames without panicking.
#[test]
fn plot_draws_two_frames() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            let p = plot(
                Plot::new(220., 140.)
                    .series(PlotSeries::new(
                        "s1",
                        vec![(0., 0.), (1., 2.), (2., 1.), (3., 4.)],
                    ))
                    .series(PlotSeries::new("s2", vec![(0., 3.), (1., 1.), (3., 2.)]))
                    .x_label("x")
                    .y_label("y"),
            );
            gpui::div().w(px(240.)).child(p).into_any_element()
        })
    });
    draw(&mut cx);
    draw(&mut cx);
}

/// An inspector with nested nodes and state draws without panicking.
#[test]
fn inspector_draws_tree() {
    apply_theme(false);
    let mut app = TestAppContext::single();
    let (_view, mut cx) = app.add_window_view(|window, cx| {
        vgui::VguiRoot::new(cx, window.window_handle(), || {
            let tree = InspectNode::new("Root", "view")
                .state("open = true")
                .child(
                    InspectNode::new("Button", "component")
                        .state("count = 2")
                        .child(InspectNode::new("label", "text")),
                )
                .child(InspectNode::new("Store", "signal"));
            let panel = inspector(Inspector::new("Inspector", vec![tree]));
            gpui::div().w(px(320.)).child(panel).into_any_element()
        })
    });
    draw(&mut cx);
    draw(&mut cx);
}
