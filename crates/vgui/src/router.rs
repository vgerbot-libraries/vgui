//! SPA Router — signal-driven, declarative route matching.
//!
//! Provides a minimal single-page-app router built on vgui's reactive
//! system. The current path is stored in a `Signal<String>`, and route
//! patterns like `/users/:id` are matched against it to extract params.
//!
//! ## Usage
//!
//! ```ignore
//! use vgui::prelude::*;
//! use vgui::router::{create_router, Router, RouteMatch};
//! use vgui::view;
//!
//! fn app() -> impl IntoElement {
//!     let router = create_router("/");
//!     // navigate with router.navigate(cx, "/users/42");
//!     // match routes with router.match_route("/users/:id")
//!     view! {
//!         <div>
//!             {router.render(cx, &[
//!                 ("/", |_| view! { <div>{"Home"}</div> }.into_any_element()),
//!                 ("/users/:id", |params| {
//!                     let id = params.get("id").unwrap_or("?");
//!                     view! { <div>{"User "}{id}</div> }.into_any_element()
//!                 }),
//!             ])}
//!         </div>
//!     }
//! }
//! ```

use std::collections::HashMap;

use gpui::{App, AppContext};

use crate::reactive::{create_signal, ReadSignal, WriteSignal};

/// The current router state, backed by a signal.
///
/// Created with [`create_router`], which returns a `Router` that can be
/// cloned cheaply (inner data is reference-counted).
#[derive(Clone)]
pub struct Router {
    path_read: ReadSignal<String>,
    path_write: WriteSignal<String>,
}

impl Router {
    /// Navigate to a new path. Triggers a re-render via the signal.
    pub fn navigate<C: AppContext>(&self, cx: &mut C, path: &str) {
        self.path_write.set(cx, path.to_string());
    }

    /// Get the current path (reactive — tracks as a dependency in memos/effects).
    pub fn path(&self) -> String {
        self.path_read.get()
    }

    /// Get the current path via the gpui context (non-tracking read).
    pub fn path_with(&self, cx: &App) -> String {
        self.path_read.get_with(cx)
    }

    /// The underlying read signal for the current path.
    pub fn path_signal(&self) -> ReadSignal<String> {
        self.path_read.clone()
    }

    /// Match a pattern against the current path and return extracted params.
    ///
    /// Pattern segments starting with `:` are treated as param placeholders.
    /// Returns `None` if the path doesn't match the pattern.
    pub fn match_route(&self, pattern: &str) -> Option<RouteMatch> {
        let path = self.path();
        match_pattern(pattern, &path)
    }

    /// Render the first matching route from a list of `(pattern, handler)` pairs.
    ///
    /// Each handler receives a `&HashMap<String, String>` of extracted params.
    /// If no route matches, the fallback is called.
    pub fn render<F, E>(
        &self,
        cx: &App,
        routes: &[(&str, F)],
        fallback: impl Fn() -> E,
    ) -> E
    where
        F: Fn(&HashMap<String, String>) -> E,
    {
        let path = self.path_with(cx);
        for (pattern, handler) in routes {
            if let Some(m) = match_pattern(pattern, &path) {
                return handler(&m.params);
            }
        }
        fallback()
    }
}

/// Result of a successful route match.
#[derive(Debug, Clone)]
pub struct RouteMatch {
    /// The pattern that matched (e.g. `/users/:id`).
    pub pattern: String,
    /// The actual path that was matched (e.g. `/users/42`).
    pub path: String,
    /// Extracted parameters (e.g. `{"id": "42"}`).
    pub params: HashMap<String, String>,
}

/// Create a router with the given initial path.
///
/// Must be called inside a `VguiRoot` render scope (same as `create_signal`).
pub fn create_router(initial: &str) -> Router {
    let (path_read, path_write) = create_signal(initial.to_string());
    Router {
        path_read,
        path_write,
    }
}

/// Match a route pattern against a path.
///
/// Patterns support `:param` placeholders for single path segments.
/// Trailing slashes are normalized (treated as no trailing slash).
/// A `*` at the end of a pattern acts as a wildcard match for the rest of the path.
///
/// # Examples
///
/// ```
/// use vgui::router::match_pattern;
/// use std::collections::HashMap;
///
/// let m = match_pattern("/users/:id", "/users/42").unwrap();
/// assert_eq!(m.params.get("id"), Some(&"42".to_string()));
///
/// assert!(match_pattern("/users/:id", "/posts/42").is_none());
///
/// let m = match_pattern("/files/*", "/files/a/b/c").unwrap();
/// assert_eq!(m.params.get("*"), Some(&"a/b/c".to_string()));
/// ```
pub fn match_pattern(pattern: &str, path: &str) -> Option<RouteMatch> {
    let pattern = pattern.trim_end_matches('/');
    let path = path.trim_end_matches('/');

    let pattern_segs: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let path_segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

    let mut params = HashMap::new();

    // Wildcard: last pattern segment is `*`
    if pattern_segs.last() == Some(&"*") {
        let prefix_len = pattern_segs.len() - 1;
        if path_segs.len() < prefix_len {
            return None;
        }
        for (i, pseg) in pattern_segs[..prefix_len].iter().enumerate() {
            if path_segs.get(i) != Some(pseg) {
                return None;
            }
        }
        let rest = path_segs[prefix_len..].join("/");
        params.insert("*".to_string(), rest);
        return Some(RouteMatch {
            pattern: pattern.to_string(),
            path: path.to_string(),
            params,
        });
    }

    if pattern_segs.len() != path_segs.len() {
        return None;
    }

    for (pseg, vseg) in pattern_segs.iter().zip(path_segs.iter()) {
        if let Some(name) = pseg.strip_prefix(':') {
            params.insert(name.to_string(), (*vseg).to_string());
        } else if pseg != vseg {
            return None;
        }
    }

    Some(RouteMatch {
        pattern: pattern.to_string(),
        path: path.to_string(),
        params,
    })
}

/// Build a path from a pattern and params.
///
/// Replaces `:param` placeholders with values from `params`.
/// Missing params are left as-is.
///
/// # Examples
///
/// ```
/// use vgui::router::build_path;
/// use std::collections::HashMap;
///
/// let mut params = HashMap::new();
/// params.insert("id".to_string(), "42".to_string());
/// assert_eq!(build_path("/users/:id", &params), "/users/42");
/// ```
pub fn build_path(pattern: &str, params: &HashMap<String, String>) -> String {
    let segs: Vec<String> = pattern
        .split('/')
        .filter(|s| !s.is_empty())
        .map(|seg| {
            if let Some(name) = seg.strip_prefix(':') {
                params.get(name).cloned().unwrap_or_else(|| seg.to_string())
            } else {
                seg.to_string()
            }
        })
        .collect();
    format!("/{}", segs.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_match() {
        let m = match_pattern("/", "/").unwrap();
        assert!(m.params.is_empty());

        let m = match_pattern("/users", "/users").unwrap();
        assert!(m.params.is_empty());
    }

    #[test]
    fn param_extraction() {
        let m = match_pattern("/users/:id", "/users/42").unwrap();
        assert_eq!(m.params.get("id"), Some(&"42".to_string()));

        let m = match_pattern("/users/:id/posts/:pid", "/users/42/posts/7").unwrap();
        assert_eq!(m.params.get("id"), Some(&"42".to_string()));
        assert_eq!(m.params.get("pid"), Some(&"7".to_string()));
    }

    #[test]
    fn no_match_different_segments() {
        assert!(match_pattern("/users/:id", "/posts/42").is_none());
        assert!(match_pattern("/users/:id", "/users").is_none());
        assert!(match_pattern("/users", "/users/42").is_none());
    }

    #[test]
    fn trailing_slash_normalized() {
        assert!(match_pattern("/users", "/users/").is_some());
        assert!(match_pattern("/users/", "/users").is_some());
    }

    #[test]
    fn wildcard_match() {
        let m = match_pattern("/files/*", "/files/a/b/c").unwrap();
        assert_eq!(m.params.get("*"), Some(&"a/b/c".to_string()));

        let m = match_pattern("/files/*", "/files").unwrap();
        assert_eq!(m.params.get("*"), Some(&"".to_string()));

        assert!(match_pattern("/files/*", "/other/a").is_none());
    }

    #[test]
    fn build_path_replaces_params() {
        let mut params = HashMap::new();
        params.insert("id".to_string(), "42".to_string());
        params.insert("tab".to_string(), "settings".to_string());
        assert_eq!(build_path("/users/:id/:tab", &params), "/users/42/settings");
    }

    #[test]
    fn build_path_missing_param_kept_as_is() {
        let params = HashMap::new();
        assert_eq!(build_path("/users/:id", &params), "/users/:id");
    }
}
