pub struct Css {
    apply: Box<dyn FnOnce(&mut gpui::StyleRefinement) + 'static>,
}

impl Css {
    pub fn new(apply: impl FnOnce(&mut gpui::StyleRefinement) + 'static) -> Self {
        Self {
            apply: Box::new(apply),
        }
    }

    pub fn apply<E: gpui::Styled>(self, mut el: E) -> E {
        (self.apply)(el.style());
        el
    }

    pub fn refine(self, mut style: gpui::StyleRefinement) -> gpui::StyleRefinement {
        (self.apply)(&mut style);
        style
    }
}

pub trait ApplyStyle<E> {
    fn apply_to(self, el: E) -> E;
}

impl<E: gpui::Styled> ApplyStyle<E> for Css {
    fn apply_to(self, el: E) -> E {
        self.apply(el)
    }
}

pub struct TwStyle {
    pub base: Box<dyn FnOnce(&mut gpui::StyleRefinement) + 'static>,
    pub hover: Option<Box<dyn Fn(&mut gpui::StyleRefinement) + 'static>>,
    pub focus: Option<Box<dyn Fn(&mut gpui::StyleRefinement) + 'static>>,
    pub active: Option<Box<dyn Fn(&mut gpui::StyleRefinement) + 'static>>,
    pub animation: Option<crate::animation::TwAnimation>,
    pub transition: Option<crate::animation::TwTransition>,
}

impl TwStyle {
    pub fn refine(self, mut style: gpui::StyleRefinement) -> gpui::StyleRefinement {
        (self.base)(&mut style);
        style
    }
}

impl<E: gpui::Styled> ApplyStyle<E> for TwStyle {
    fn apply_to(self, mut el: E) -> E {
        (self.base)(el.style());
        el
    }
}

// ---------------------------------------------------------------------------
// Dynamic class composition — TwClass, IntoTwStyle, TwClassSource, twc!
// ---------------------------------------------------------------------------

/// A builder for composing multiple class sources with conditional inclusion
/// and last-write-wins conflict resolution.
///
/// Use [`TwClass::new`] / [`TwClass::add`] / [`TwClass::add_if`] or the
/// [`twc!`](crate::twc) macro, then call [`TwClass::build`] (or
/// [`IntoTwStyle::into_tw_style`]) to produce a [`TwStyle`].
#[derive(Debug, Default)]
pub struct TwClass {
    parts: Vec<String>,
}

impl TwClass {
    pub fn new() -> Self {
        Self {
            parts: Vec::new(),
        }
    }

    /// Append a class source. Accepts `&str`, `String`, `Option<&str>`,
    /// `Option<String>`, nested `TwClass`, or any `TwClassSource`.
    pub fn add(mut self, source: impl TwClassSource) -> Self {
        source.append_to(&mut self);
        self
    }

    /// Conditionally append a class string.
    pub fn add_if(mut self, cond: bool, classes: &str) -> Self {
        if cond {
            self.parts.push(classes.to_string());
        }
        self
    }

    /// Build a [`TwStyle`] by joining all parts and parsing at runtime.
    ///
    /// Classes are applied sequentially to one `StyleRefinement` per variant;
    /// last-write-wins at the field level (e.g. `"p-4 p-2"` → padding=8).
    pub fn build(self) -> TwStyle {
        crate::tw_dynamic::tw_dynamic(&self.parts.join(" "))
    }
}

/// Polymorphic input for [`TwClass::add`].
pub trait TwClassSource {
    fn append_to(self, class: &mut TwClass);
}

impl TwClassSource for &str {
    fn append_to(self, class: &mut TwClass) {
        class.parts.push(self.to_string());
    }
}

impl TwClassSource for String {
    fn append_to(self, class: &mut TwClass) {
        class.parts.push(self);
    }
}

impl<T: TwClassSource> TwClassSource for Option<T> {
    fn append_to(self, class: &mut TwClass) {
        if let Some(v) = self {
            v.append_to(class);
        }
    }
}

impl TwClassSource for TwClass {
    fn append_to(self, class: &mut TwClass) {
        class.parts.extend(self.parts);
    }
}

/// Unified conversion to [`TwStyle`].
///
/// Implemented for `&str`, `String`, [`TwClass`], and [`TwStyle`] itself.
/// The `tw!` macro delegates here for non-literal inputs.
pub trait IntoTwStyle {
    fn into_tw_style(self) -> TwStyle;
}

impl IntoTwStyle for &str {
    fn into_tw_style(self) -> TwStyle {
        crate::tw_dynamic::tw_dynamic(self)
    }
}

impl IntoTwStyle for String {
    fn into_tw_style(self) -> TwStyle {
        crate::tw_dynamic::tw_dynamic(&self)
    }
}

impl IntoTwStyle for TwClass {
    fn into_tw_style(self) -> TwStyle {
        self.build()
    }
}

impl IntoTwStyle for TwStyle {
    fn into_tw_style(self) -> TwStyle {
        self
    }
}

/// Ergonomic class composition macro.
///
/// ```ignore
/// twc!("flex p-4", cond.then_some("bg-blue-500"), "text-white")
/// ```
///
/// Each argument is passed to [`TwClass::add`], which dispatches via
/// [`TwClassSource`] (`&str`, `String`, `Option<&str>`, nested `TwClass`).
#[macro_export]
macro_rules! twc {
    ($($expr:expr),* $(,)?) => {{
        let mut __class = ::vgui::TwClass::new();
        $(__class = __class.add($expr);)*
        __class
    }};
}
