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
