//! `<img>` `on:load` / `on:error` event support.
//!
//! gpui's `Img` element loads images asynchronously through the asset system
//! but exposes no callback for load completion or failure. `__img_with_events`
//! wraps the image source in a tracking closure that fires user callbacks on
//! load-state transitions, then returns a normal `gpui::Img` so all the usual
//! chaining (`.object_fit()`, events, styling) still applies.

use std::cell::Cell;
use std::sync::Arc;

use gpui::{
    img, App, ImageCacheError, ImageSource, Img, ImgResourceLoader, RenderImage, Window,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImgLoadState {
    Loading,
    Loaded,
    Error,
}

/// Build a `gpui::Img` that fires `on_load` / `on_error` when the image source
/// finishes loading or fails.
///
/// Called by the `view!` macro when an `<img>` element carries `on:load` or
/// `on:error`. Both callbacks take `Fn(&mut App)` (no event payload), matching
/// the `on:close` pattern on `<dialog>`. A missing callback defaults to a
/// no-op so the macro never has to construct an `Option`.
pub fn __img_with_events(
    source: impl Into<ImageSource>,
    on_load: impl Fn(&mut App) + 'static,
    on_error: impl Fn(&mut App) + 'static,
) -> Img {
    let source = source.into();
    let on_load = Arc::new(on_load) as Arc<dyn Fn(&mut App)>;
    let on_error = Arc::new(on_error) as Arc<dyn Fn(&mut App)>;
    let state = Arc::new(Cell::new(ImgLoadState::Loading));

    match source {
        // Resource sources (strings, URLs, paths) are the common case. gpui's
        // own `Img` calls `window.use_asset::<ImgResourceLoader>(&resource, cx)`
        // internally; we replicate that call inside a `Custom` closure so we can
        // observe the result and fire callbacks on state transitions.
        ImageSource::Resource(resource) => {
            let source = ImageSource::Custom(Arc::new(move |window: &mut Window, cx: &mut App| {
                let result = window.use_asset::<ImgResourceLoader>(&resource, cx);
                fire_on_transition(&state, &result, &on_load, &on_error, cx);
                result
            }));
            img(source)
        }
        // A user-supplied custom loader. Wrap it to observe its result.
        ImageSource::Custom(orig_fn) => {
            let source = ImageSource::Custom(Arc::new(move |window: &mut Window, cx: &mut App| {
                let result = orig_fn(window, cx);
                fire_on_transition(&state, &result, &on_load, &on_error, cx);
                result
            }));
            img(source)
        }
        // Render images are immediately available — fire `on_load` on first
        // access and return the data unchanged.
        ImageSource::Render(data) => {
            let source = ImageSource::Custom(Arc::new(move |_window: &mut Window, cx: &mut App| {
                let old = state.replace(ImgLoadState::Loaded);
                if old != ImgLoadState::Loaded {
                    on_load(cx);
                }
                Some(Ok(data.clone()))
            }));
            img(source)
        }
        // `Image(Arc<Image>)` sources are decoded by `AssetLogger<ImageDecoder>`,
        // whose `ImageDecoder` type is private to gpui — we cannot name it to
        // call `use_asset` ourselves, so tracking is impossible. Fall back to a
        // plain `gpui::img`; callbacks simply won't fire for this rare source
        // type.
        ImageSource::Image(data) => img(ImageSource::Image(data)),
    }
}

fn fire_on_transition(
    state: &Cell<ImgLoadState>,
    result: &Option<Result<Arc<RenderImage>, ImageCacheError>>,
    on_load: &Arc<dyn Fn(&mut App)>,
    on_error: &Arc<dyn Fn(&mut App)>,
    cx: &mut App,
) {
    let new_state = match result {
        Some(Ok(_)) => ImgLoadState::Loaded,
        Some(Err(_)) => ImgLoadState::Error,
        None => ImgLoadState::Loading,
    };
    let old_state = state.replace(new_state);
    if old_state != new_state {
        match new_state {
            ImgLoadState::Loaded => on_load(cx),
            ImgLoadState::Error => on_error(cx),
            ImgLoadState::Loading => {}
        }
    }
}
