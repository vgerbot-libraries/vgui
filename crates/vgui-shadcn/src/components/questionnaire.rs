//! Onboarding / questionnaire flow.
//!
//! A multi-step wizard: one question per page, Back / Next navigation, and a
//! Finish action on the last page. Answers are accumulated in a reactive
//! store and handed to `on_finish`. Next is disabled while the current
//! question's answer is empty (or fails an optional predicate).

use std::rc::Rc;

use gpui::App;
use vgui::prelude::*;
use vgui::{click, create_signal, create_store};

/// One question in a [`Questionnaire`].
#[derive(Clone)]
pub struct Question {
    /// Short prompt shown above the input.
    pub prompt: String,
    /// Optional validity predicate over the current answer.
    pub accept: Option<Rc<dyn Fn(&str) -> bool>>,
}

impl Question {
    /// A question with a prompt; any non-empty answer is accepted.
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            accept: None,
        }
    }

    /// Require the answer to satisfy `predicate`.
    pub fn accept_when(mut self, predicate: impl Fn(&str) -> bool + 'static) -> Self {
        self.accept = Some(Rc::new(predicate));
        self
    }

    fn valid(&self, answer: &str) -> bool {
        match &self.accept {
            Some(predicate) => predicate(answer),
            None => !answer.trim().is_empty(),
        }
    }
}


/// Render an interactive questionnaire.
///
/// `on_finish` receives the collected answers in question order.
pub fn questionnaire(
    questions: Vec<Question>,
    on_finish: Rc<dyn Fn(&Vec<String>, &mut App)>,
) -> gpui::AnyElement {
    let total = questions.len();
    let (answers, set_answers) = create_store(vec![String::new(); total.max(1)]);
    let (done, set_done) = create_signal(false);
    let (index, set_index) = create_signal(0usize);

    let is_done = done.get() || total == 0;
    let current_idx = index.get();

    let back = {
        let index = index.clone();
        let set_index = set_index.clone();
        click(move |cx| {
            let i = index.get();
            if i > 0 {
                set_index.set(cx, i - 1);
            }
        })
    };
    let next = {
        let index = index.clone();
        let set_index = set_index.clone();
        let set_done = set_done.clone();
        let answers = answers.clone();
        let questions = questions.clone();
        let on_finish = on_finish.clone();
        click(move |cx| {
            let i = index.get();
            if i >= questions.len() || !questions[i].valid(&answers.get()[i]) {
                return;
            }
            if i + 1 < questions.len() {
                set_index.set(cx, i + 1);
            } else {
                set_done.set(cx, true);
                let snapshot = answers.get();
                on_finish(&snapshot, cx);
            }
        })
    };

    let current_idx_clamped = current_idx.min(total.saturating_sub(1));
    let question_prompt = questions.get(current_idx_clamped).map(|q| q.prompt.clone());
    let current_answer = answers.get().get(current_idx).cloned().unwrap_or_default();
    let is_last = total == 0 || current_idx + 1 >= total;
    let progress = format!("Step {} / {}", current_idx + 1, total.max(1));
    let prompt = question_prompt.unwrap_or_default();


    let input_set_answers = set_answers.clone();
    let snapshot = answers.get();
    let form = vgui::view! {
        <div style={vgui::css!{
            display: flex; flex-direction: column; gap: 10px; padding: 12px;
            min-width: 260px;
        }}>
            <span style={vgui::css!{ color: var(--muted-foreground); font-size: 12px; }}>
                {progress}
            </span>
            <label style={vgui::css!{ color: var(--foreground); font-size: 14px; font-weight: 500; }}>
                {prompt}
            </label>
            <input
                value={current_answer}
                style={vgui::css!{
                    height: 30px; padding-left: 8px; padding-right: 8px;
                    border-width: 1px; border-style: solid; border-color: var(--border);
                    border-radius: var(--radius); background: var(--background);
                    color: var(--foreground);
                }}
                on:input={vgui::input_cb(move |raw, cx| {
                    let qi = index.get();
                    input_set_answers.update(cx, |buf| {
                        if qi < buf.len() { buf[qi] = raw.to_string(); }
                    });
                })}
            />
            <div style={vgui::css!{ display: flex; flex-direction: row; gap: 8px; margin-top: 4px; }}>
                <button
                    style={vgui::css!{
                        height: 30px; padding-left: 12px; padding-right: 12px;
                        border-width: 1px; border-style: solid; border-color: var(--border);
                        border-radius: var(--radius); background: var(--background);
                        color: var(--foreground);
                    }}
                    on:click={back}
                >{"Back"}</button>
                <button
                    style={vgui::css!{
                        height: 30px; padding-left: 12px; padding-right: 12px;
                        border-width: 0px; border-radius: var(--radius);
                        background: var(--primary); color: var(--primary-foreground);
                    }}
                    on:click={next}
                >{if is_last { "Finish" } else { "Next" }}</button>
            </div>
        </div>
    };
    let count = snapshot.len();
    let done_view = vgui::view! {
        <div style={vgui::css!{
            display: flex; flex-direction: column; gap: 8px; padding: 12px;
        }}>
            <span style={vgui::css!{ color: var(--foreground); font-size: 14px; }}>
                {"All done"}
            </span>
            <span style={vgui::css!{ color: var(--muted-foreground); font-size: 12px; }}>
                {format!("{} answers submitted", count)}
            </span>
        </div>
    };
    vgui::show(is_done, done_view, form)
}
