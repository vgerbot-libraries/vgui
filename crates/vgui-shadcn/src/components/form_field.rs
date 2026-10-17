//! Validation-aware form and field wrappers.
//!
//! [`form`] creates a [`FormController`], provides it to descendants through
//! context during construction, and exposes [`form_submit_button`] which runs
//! every registered validator and only then invokes the submit callback.
//! [`form_field`] registers a validator against the controller and shows
//! inline error text. [`validate_form`] imperatively validates the active
//! form. Built-in validators ([`required`], [`min_len`], [`max_len`],
//! [`email`], [`pattern`]) cover the common cases.

use std::cell::RefCell;
use std::rc::Rc;

use gpui::{App, Div};
use vgui::prelude::*;
use vgui::{
    create_signal, form_scope, provide_context, use_context, Context, ReadSignal, WriteSignal,
};

struct FieldEntry {
    validate: Rc<dyn Fn() -> Option<String>>,
    set_error: WriteSignal<Option<String>>,
}

/// Shared validation registry and submit action for one [`form`].
///
/// Fields repopulate the registry on each render; submit evaluates the
/// current set.
#[derive(Clone)]
pub struct FormController {
    fields: Rc<RefCell<Vec<FieldEntry>>>,
    on_submit: Rc<dyn Fn(&mut App)>,
}

thread_local! {
    // The most recently constructed form, backing the imperative
    // [`validate_form`].
    static ACTIVE_FORM: RefCell<Option<FormController>> = const { RefCell::new(None) };
}

impl FormController {
    fn new(on_submit: Rc<dyn Fn(&mut App)>) -> Self {
        Self {
            fields: Rc::new(RefCell::new(Vec::new())),
            on_submit,
        }
    }

    /// Run every validator, push errors into the fields, and report validity.
    fn validate(&self, cx: &mut App) -> bool {
        let mut valid = true;
        for entry in self.fields.borrow().iter() {
            let error = (entry.validate)();
            if error.is_some() {
                valid = false;
            }
            entry.set_error.set(cx, error);
        }
        valid
    }
}

static FORM_CONTROLLER: Context<FormController> = Context::new();

/// Imperatively validate the active form. Returns `true` when all fields pass
/// and pushes error messages into any that fail.
pub fn validate_form(cx: &mut App) -> bool {
    ACTIVE_FORM.with(|active| {
        active
            .borrow()
            .as_ref()
            .map(|controller| controller.validate(cx))
            .unwrap_or(true)
    })
}

/// A validation form. `on_submit` runs only when every field is valid, when
/// the form is submitted via [`form_submit_button`] (or the Enter key on a
/// single-line input). Build the [`form_field`]s and a submit button inside
/// `content`.
pub fn form(on_submit: Rc<dyn Fn(&mut App)>, content: impl Fn() -> Div + 'static) -> Div {
    let controller = FormController::new(on_submit);
    ACTIVE_FORM.with(|active| *active.borrow_mut() = Some(controller.clone()));
    let key_controller = controller.clone();

    form_scope(
        Some(Box::new(move |cx: &mut App| {
            if key_controller.validate(cx) {
                (key_controller.on_submit)(cx);
            }
        })),
        None,
        move || {
            controller.fields.borrow_mut().clear();
            let _guard = provide_context(&FORM_CONTROLLER, controller.clone());
            content()
        },
    )
}

/// One labelled, validated field. `control` is the input element; `value` is
/// the current text the validator inspects. Shows inline error text below.
pub fn form_field(
    label: impl Into<String>,
    control: gpui::AnyElement,
    value: String,
    validate: Rc<dyn Fn(&str) -> Option<String>>,
) -> gpui::AnyElement {
    let label = label.into();
    let (error, set_error) = create_signal(Option::<String>::None);
    if let Some(controller) = use_context(&FORM_CONTROLLER) {
        // Re-registered on every render with the field's latest value.
        controller.fields.borrow_mut().push(FieldEntry {
            validate: Rc::new(move || validate(&value)),
            set_error: set_error.clone(),
        });
    }
    let current: ReadSignal<Option<String>> = error;
    let message = current.get();
    vgui::view! {
        <div style={vgui::css! {
            display: flex;
            flex-direction: column;
            gap: 4px;
        }}>
            <label style={vgui::css! {
                font-size: 13px;
                font-weight: 500;
                color: var(--foreground);
            }}>{label}</label>
            {control}
            {match message {
                Some(message) => vgui::view! {
                    <span style={vgui::css! {
                        font-size: 12px;
                        color: var(--destructive);
                    }}>{message}</span>
                },
                None => vgui::view! { <span></span> },
            }}
        </div>
    }
    .into_any_element()
}

/// A button that validates the enclosing [`form`] and submits when valid.
/// Must be constructed inside `form`'s content (it snapshots the controller
/// through context).
pub fn form_submit_button(label: impl Into<String>) -> gpui::AnyElement {
    let label = label.into();
    let controller = use_context(&FORM_CONTROLLER);
    vgui::view! {
        <button
            style={vgui::css! {
                background: var(--primary);
                color: var(--primary-foreground);
                border-radius: var(--radius);
                padding-left: 14px;
                padding-right: 14px;
                padding-top: 7px;
                padding-bottom: 7px;
                font-size: 13px;
                border-width: 0px;
            }}
            on:click={vgui::click(move |cx| {
                if let Some(controller) = controller.clone() {
                    if controller.validate(cx) {
                        (controller.on_submit)(cx);
                    }
                }
            })}
        >
            {label}
        </button>
    }
    .into_any_element()
}

// ── Built-in validators ─────────────────────────────────────────────────

/// Error when the trimmed value is empty.
pub fn required(message: impl Into<String>) -> Rc<dyn Fn(&str) -> Option<String>> {
    let message = message.into();
    Rc::new(move |v: &str| {
        if v.trim().is_empty() {
            Some(message.clone())
        } else {
            None
        }
    })
}

/// Error when shorter than `n` characters.
pub fn min_len(n: usize, message: impl Into<String>) -> Rc<dyn Fn(&str) -> Option<String>> {
    let message = message.into();
    Rc::new(move |v: &str| {
        if v.chars().count() < n {
            Some(message.clone())
        } else {
            None
        }
    })
}

/// Error when longer than `n` characters.
pub fn max_len(n: usize, message: impl Into<String>) -> Rc<dyn Fn(&str) -> Option<String>> {
    let message = message.into();
    Rc::new(move |v: &str| {
        if v.chars().count() > n {
            Some(message.clone())
        } else {
            None
        }
    })
}

/// Error when the value is not a simple email shape.
pub fn email(message: impl Into<String>) -> Rc<dyn Fn(&str) -> Option<String>> {
    let message = message.into();
    Rc::new(move |v: &str| {
        let valid = {
            let mut parts = v.split('@');
            match (parts.next(), parts.next(), parts.next()) {
                (Some(user), Some(domain), None) => {
                    !user.is_empty() && domain.contains('.') && !domain.starts_with('.')
                }
                _ => false,
            }
        };
        if valid {
            None
        } else {
            Some(message.clone())
        }
    })
}

/// Error when the predicate rejects the value.
pub fn pattern(
    matches: Rc<dyn Fn(&str) -> bool>,
    message: impl Into<String>,
) -> Rc<dyn Fn(&str) -> Option<String>> {
    let message = message.into();
    Rc::new(move |v: &str| {
        if matches(v) {
            None
        } else {
            Some(message.clone())
        }
    })
}
